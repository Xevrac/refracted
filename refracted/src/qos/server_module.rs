//! Local QoS coordinator -- BlazeSDK `QosManager` / DirtySDK `QosClient` probe endpoints (BWPS/LTPS).

use crate::common::error::{io_is_expected_peer_close, BlazeResult};
use crate::core::inspector::inspector_module::{capture_grpc, CapturedGrpc, GrpcDirection};
use crate::grpc::{grpc_body_decode_capture, parse_grpc_frame};
use bytes::Bytes;
use h2::server::{self, SendResponse};
use http::{Request, Response};
use rustls::ServerConfig;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, ReadBuf};
use tokio::net::{TcpStream, UdpSocket};
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, warn};

const QOS_TAG: &str = "\x1b[38;2;80;200;120m[QoS]\x1b[0m";

/// DirtySDK latency probes use requestid < 2; bandwidth uses >= 2 (see QosApi recv path).
const QOS_REQUEST_ID_LATENCY: u32 = 1;
const QOS_REQUEST_ID_BANDWIDTH: u32 = 2;
const QOS_DEFAULT_NUM_PROBES: u32 = 10;
const QOS_DEFAULT_PROBE_SIZE: u32 = 120;
/// Minimum UDP reply size so the client takes the response path (not the 20-byte peer-echo path).
const QOS_UDP_REPLY_MIN: usize = 30;

static QOS_REQ_SECRET: AtomicU32 = AtomicU32::new(1);

/// Port roles matching Blaze preAuth QOSS (`BWPS` / `LTPS` PSA+PSP).
#[derive(Clone, Copy)]
struct QosBind {
    port: u16,
    /// Short role for logs: `bwps` (coordinator), `ltps` (latency/data), `alt`.
    role: &'static str,
}

fn looks_like_http_request(first_line: &str) -> bool {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return false;
    }
    matches!(
        parts[0],
        "GET" | "POST" | "HEAD" | "OPTIONS" | "PUT" | "DELETE" | "PATCH"
    ) && (parts[1].starts_with('/') || parts[1].starts_with("http://") || parts[1].starts_with("https://"))
}

fn looks_like_http_prefix(buf: &[u8]) -> bool {
    const METHODS: &[&[u8]] = &[
        b"GET ", b"POST ", b"HEAD ", b"OPTIONS ", b"PUT ", b"DELETE ", b"PATCH ",
    ];
    METHODS.iter().any(|m| buf.len() >= m.len() && buf.starts_with(m))
        || METHODS.iter().any(|m| m.starts_with(buf) && !buf.is_empty())
}

fn http_request_complete(buf: &[u8]) -> bool {
    // Headers terminated by blank line.
    memchr_crlf_crlf(buf).is_some()
}

fn memchr_crlf_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}

fn classify_http_probe(path_lc: &str) -> &'static str {
    if path_lc == "/qos/firewall" {
        "firewall"
    } else if path_lc == "/qos/firetype" {
        "firetype"
    } else if path_lc == "/qos/qos" {
        "bandwidth"
    } else if path_lc.contains("clientcall") {
        "coordinator"
    } else if path_lc.contains("health") || path_lc.contains("check") {
        "health"
    } else {
        "http"
    }
}

fn next_req_secret() -> u32 {
    let v = QOS_REQ_SECRET.fetch_add(1, Ordering::Relaxed);
    if v == 0 {
        QOS_REQ_SECRET.fetch_add(1, Ordering::Relaxed)
    } else {
        v
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    for pair in query.split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next()?;
        if k == key {
            return Some(it.next().unwrap_or(""));
        }
    }
    None
}

fn parse_u32_param(query: &str, key: &str) -> Option<u32> {
    query_param(query, key)?.parse().ok()
}

/// Host-order style IPv4 integer DirtySDK XML parsers expect (`10.0.0.230` → `0x0A0000E6`).
fn ipv4_to_qos_u32(ip: Ipv4Addr) -> u32 {
    u32::from_be_bytes(ip.octets())
}

fn guess_lan_ipv4() -> Option<Ipv4Addr> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    match sock.local_addr().ok()?.ip() {
        IpAddr::V4(v4) if !v4.is_unspecified() && !v4.is_loopback() => Some(v4),
        _ => None,
    }
}

/// IP/port to reflect in firewall XML and UDP probe replies (EXIP).
fn reflect_endpoint(peer: SocketAddr) -> (Ipv4Addr, u16) {
    let port = peer.port();
    let ip = match peer.ip() {
        IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_unspecified() => v4,
        _ => {
            if let Some(bits) = crate::session::peek_qos_observed_exip_ip() {
                Ipv4Addr::from(bits)
            } else if let Some(lan) = guess_lan_ipv4() {
                lan
            } else {
                Ipv4Addr::new(127, 0, 0, 1)
            }
        }
    };
    (ip, port)
}

fn build_qos_xml(qos_port: u16, qtyp: u32) -> String {
    let secret = next_req_secret();
    let request_id = if qtyp >= 2 {
        QOS_REQUEST_ID_BANDWIDTH
    } else {
        QOS_REQUEST_ID_LATENCY
    };
    format!(
        "<qos>\
         <numprobes>{QOS_DEFAULT_NUM_PROBES}</numprobes>\
         <probesize>{QOS_DEFAULT_PROBE_SIZE}</probesize>\
         <qosport>{qos_port}</qosport>\
         <requestid>{request_id}</requestid>\
         <reqsecret>{secret}</reqsecret>\
         </qos>"
    )
}

fn build_firewall_xml(peer: SocketAddr, secret_hint: Option<(u32, u32)>) -> String {
    let (ip, port) = reflect_endpoint(peer);
    let ip_u = ipv4_to_qos_u32(ip);
    let (request_id, secret) = secret_hint.unwrap_or_else(|| (1, next_req_secret()));
    format!(
        "<firewall>\
         <numinterfaces>1</numinterfaces>\
         <ips><ips>{ip_u}</ips></ips>\
         <ports><ports>{port}</ports></ports>\
         <requestid>{request_id}</requestid>\
         <reqsecret>{secret}</reqsecret>\
         </firewall>"
    )
}

fn build_firetype_xml() -> String {
    // Values other than 5 trigger the firetype status callback in DirtySDK.
    "<firetype><firetype>1</firetype></firetype>".to_string()
}

fn http_ok_xml(body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/xml\r\n\
         Content-Length: {}\r\n\
         Connection: keep-alive\r\n\
         \r\n\
         {}",
        body.len(),
        body
    )
    .into_bytes()
}

/// DirtySDK UDP probe reply: echo client header (incl. send tick at +16), stamp EXIP/port at +20/+24.
fn build_udp_probe_reply(request: &[u8], peer: SocketAddr) -> Vec<u8> {
    let (ip, port) = reflect_endpoint(peer);
    let mut resp = vec![0u8; request.len().max(QOS_UDP_REPLY_MIN)];
    let copy_len = request.len().min(resp.len());
    resp[..copy_len].copy_from_slice(&request[..copy_len]);
    resp[20..24].copy_from_slice(&ipv4_to_qos_u32(ip).to_be_bytes());
    resp[24..26].copy_from_slice(&port.to_be_bytes());
    if resp.len() >= 30 {
        resp[26..30].copy_from_slice(&0u32.to_be_bytes());
    }
    resp
}

fn proto_write_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn proto_write_key(out: &mut Vec<u8>, field_number: u32, wire_type: u8) {
    proto_write_varint(out, ((field_number as u64) << 3) | (wire_type as u64));
}

fn proto_write_len_delimited(out: &mut Vec<u8>, field_number: u32, data: &[u8]) {
    proto_write_key(out, field_number, 2);
    proto_write_varint(out, data.len() as u64);
    out.extend_from_slice(data);
}

fn proto_write_string(out: &mut Vec<u8>, field_number: u32, value: &str) {
    proto_write_len_delimited(out, field_number, value.as_bytes());
}

fn proto_write_sint32(out: &mut Vec<u8>, field_number: u32, value: i32) {
    let zz = ((value << 1) ^ (value >> 31)) as u32;
    proto_write_key(out, field_number, 0);
    proto_write_varint(out, zz as u64);
}

fn wrap_grpc_message_frame(protobuf_payload: &[u8]) -> Vec<u8> {
    let mut framed = Vec::with_capacity(5 + protobuf_payload.len());
    framed.push(0);
    framed.extend_from_slice(&(protobuf_payload.len() as u32).to_be_bytes());
    framed.extend_from_slice(protobuf_payload);
    framed
}

fn build_qos_clientcall_response_payload(is_followup_call: bool) -> Vec<u8> {
    let mut out = Vec::new();
    proto_write_sint32(&mut out, 2, -1033725037);
    proto_write_string(&mut out, 3, "v4[159.196.128.63]");

    if !is_followup_call {
        let field7_hex = [
            "0a076177732d736a63120d35342e3135312e33312e3134311894a401221056a74f9896d52b1e113786c20e84be80288004",
            "0a076177732d696164120e31332e3232332e3234352e3137311898a4012210df07e8b866e3a1f4bb6e9ad2cb491063288004",
            "0a076177732d6c6872120a31362e36302e382e3634189ca40122109f13fceaa4312c75127673832d56a2c8288004",
            "0a076177732d667261120c33352e3135382e35312e39381893a40122106ea78025cc837accb0611b34db2f7920288004",
            "0a076177732d737964120e31352e3133342e3230392e3133331899a401221023bdd8c9ead82ae5fe973a12f7073b80288004",
            "0a076177732d6e7274120d31332e3131342e3130352e35371893a4012210a061280bd88a143a7e8f0a24132827d0288004",
            "0a076177732d686b67120d39352e34302e3130332e31333118a0a4012210b7cf472719f2891d21504f201cd5c0ba288004",
            "0a076177732d62727a120e35342e3233332e3133342e3138391899a40122101ab0175f6ee051efa0bea507e276d35b288004",
            "0a076177732d73696e120c34372e3132392e3231312e3318a0a4012210da5429f6b1e47364b9c4546f1eba6bd2288004",
            "0a076177732d637074120d31332e3234362e3233352e33301888a4012210db97100682588a4af06d76880300f8c5288004",
            "0a076177732d706478120d33342e3232302e3233392e3535189fa4012210a55d59a73cabcd56107d6610d15605f9288004",
            "0a076177732d69636e120c34332e3230322e322e323239188fa4012210e21a52744cae29efc5e9c76655d4bcac288004",
            "0a076177732d647562120d332e3235352e3138322e3134361888a4012210c7978a661cc54c0ebbd5a635db73ec53288004",
            "0a076177732d636d68120d332e3134352e3231322e323136188fa40122100856a4faf2d2cc6b4493f52586f0569f288004",
        ];
        for hex_blob in field7_hex {
            if let Ok(decoded) = hex::decode(hex_blob) {
                proto_write_len_delimited(&mut out, 7, &decoded);
            }
        }

        let mut field9 = Vec::new();
        proto_write_string(&mut field9, 1, "rtt");
        proto_write_string(&mut field9, 2, "ALL");
        proto_write_sint32(&mut field9, 3, 4);
        proto_write_sint32(&mut field9, 7, 5000);
        proto_write_sint32(&mut field9, 8, 100);
        proto_write_sint32(&mut field9, 9, 1750);
        proto_write_sint32(&mut field9, 10, 1);
        proto_write_sint32(&mut field9, 11, 2);
        proto_write_sint32(&mut field9, 12, 1000);
        proto_write_len_delimited(&mut out, 9, &field9);
    } else {
        proto_write_sint32(&mut out, 4, -2);
        proto_write_string(&mut out, 6, "v4[123.456.789.10]:56204");
        let regions: [(&str, i32); 14] = [
            ("aws-syd", 6),
            ("aws-sin", 54),
            ("aws-hkg", 70),
            ("aws-sjc", -76),
            ("aws-pdx", 86),
            ("aws-icn", -87),
            ("aws-nrt", -87),
            ("aws-iad", 104),
            ("aws-cmh", -106),
            ("aws-fra", -129),
            ("aws-lhr", -137),
            ("aws-dub", 138),
            ("aws-brz", -162),
            ("aws-cpt", -207),
        ];
        for (name, rtt) in regions {
            let mut item = Vec::new();
            proto_write_string(&mut item, 1, name);
            proto_write_sint32(&mut item, 2, rtt);
            proto_write_sint32(&mut item, 5, 272728568);
            proto_write_len_delimited(&mut out, 10, &item);
        }
    }

    proto_write_sint32(&mut out, 11, 3);
    out
}

fn capture_qos_grpc_record(
    direction: GrpcDirection,
    method: &str,
    path: &str,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    grpc_status: Option<String>,
) {
    let decoded = grpc_body_decode_capture(&body);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    capture_grpc(CapturedGrpc {
        capture_seq: 0,
        timestamp,
        direction,
        method: method.to_string(),
        path: path.to_string(),
        host: "qoscoordinator.gameservices.ea.com".to_string(),
        headers,
        body_size: body.len(),
        body,
        protobuf_data: decoded.protobuf_chunks.first().cloned(),
        protobuf_chunks: decoded.protobuf_chunks,
        is_compressed: decoded.any_frame_was_compressed,
        grpc_status,
    });
}

struct PrependStream<S> {
    head: Vec<u8>,
    head_off: usize,
    inner: S,
}

impl<S> PrependStream<S> {
    fn new(inner: S, first: Vec<u8>) -> Self {
        Self {
            head: first,
            head_off: 0,
            inner,
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for PrependStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut filled_from_head = false;
        if self.head_off < self.head.len() {
            let rem = &self.head[self.head_off..];
            let n = rem.len().min(buf.remaining());
            buf.put_slice(&rem[..n]);
            self.head_off += n;
            filled_from_head = n > 0;
            if self.head_off < self.head.len() || buf.remaining() == 0 {
                return Poll::Ready(Ok(()));
            }
        }
        match Pin::new(&mut self.inner).poll_read(cx, buf) {
            Poll::Pending if filled_from_head => Poll::Ready(Ok(())),
            other => other,
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for PrependStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        b: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, b)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Session counter / probe kinds -- INFO only at begin + end (BlazeSDK span: connected → qos completed).
struct QosSessionLog {
    peer: SocketAddr,
    bind: QosBind,
    transport: &'static str,
    started: Instant,
    probes: u32,
    kinds: Vec<&'static str>,
    outcome: &'static str,
}

impl QosSessionLog {
    fn begin(peer: SocketAddr, bind: QosBind, transport: &'static str) -> Self {
        // DirtySDK QosClient dials preAuth PSA (BWPS/LTPS) -- peer is the Blaze client NetConn IP.
        info!(
            "{QOS_TAG} begin {peer} → {}:{} ({}) via {}",
            bind.role, bind.port, target_hint(bind.role), transport
        );
        Self {
            peer,
            bind,
            transport,
            started: Instant::now(),
            probes: 0,
            kinds: Vec::new(),
            outcome: "ok",
        }
    }

    fn note_probe(&mut self, kind: &'static str) {
        self.probes = self.probes.saturating_add(1);
        if !self.kinds.iter().any(|k| *k == kind) {
            self.kinds.push(kind);
        }
    }

    fn fail(&mut self, outcome: &'static str) {
        self.outcome = outcome;
    }

    fn end(self) {
        let ms = self.started.elapsed().as_millis();
        let kinds = if self.kinds.is_empty() {
            "-".to_string()
        } else {
            self.kinds.join("+")
        };
        info!(
            "{QOS_TAG} end {} → {}:{} {} {} probe(s) [{}] via {} {}ms",
            self.peer,
            self.bind.role,
            self.bind.port,
            self.outcome,
            self.probes,
            kinds,
            self.transport,
            ms
        );
    }
}

fn target_hint(role: &str) -> &'static str {
    match role {
        "bwps" => "coordinator/BWPS",
        "ltps" => "ping-site/LTPS",
        "alt" => "coordinator-alt",
        _ => "qos",
    }
}

pub struct QosProtocolServer {
    host: String,
    ssl_context: Option<Arc<ServerConfig>>,
}

impl QosProtocolServer {
    pub fn new(host: String, ssl_context: Option<Arc<ServerConfig>>) -> Self {
        Self { host, ssl_context }
    }

    pub fn ports_from_config(p: &crate::common::game::ServicePorts) -> Vec<(u16, String)> {
        vec![
            (p.qos_coordinator, "QoS Coordinator".into()),
            (p.qos_data, "QoS Data Port".into()),
            (p.qos_alt, "QoS Coordinator Alt".into()),
        ]
    }

    pub async fn start_qos_server(
        &self,
        ports: &crate::common::game::ServicePorts,
    ) -> BlazeResult<()> {
        let binds = [
            QosBind {
                port: ports.qos_coordinator,
                role: "bwps",
            },
            QosBind {
                port: ports.qos_data,
                role: "ltps",
            },
            QosBind {
                port: ports.qos_alt,
                role: "alt",
            },
        ];
        for bind in binds {
            let host_tcp = self.host.clone();
            let host_udp = self.host.clone();
            let tls = self.ssl_context.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::run_qos_server(host_tcp, bind, tls).await {
                    error!("QoS {} :{} error: {}", bind.role, bind.port, e);
                }
            });
            tokio::spawn(async move {
                if let Err(e) = Self::run_qos_udp(host_udp, bind).await {
                    error!("QoS {} UDP :{} error: {}", bind.role, bind.port, e);
                }
            });
        }
        Ok(())
    }

    async fn run_qos_server(
        host: String,
        bind: QosBind,
        ssl_context: Option<Arc<ServerConfig>>,
    ) -> BlazeResult<()> {
        let addr = format!("{}:{}", host, bind.port);
        let socket = tokio::net::TcpSocket::new_v4()
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        #[cfg(windows)]
        socket
            .set_reuseaddr(true)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        socket
            .bind(addr.parse().map_err(|e| {
                crate::common::error::BlazeError::InvalidPacket(format!("Invalid address: {}", e))
            })?)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        let listener = socket
            .listen(128)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        if !crate::common::startup_progress::is_startup_in_progress() {
            info!(
                "{QOS_TAG} listening {} ({}) on {}",
                bind.role,
                target_hint(bind.role),
                addr
            );
        }

        loop {
            let (stream, peer) = listener.accept().await?;
            let tls = ssl_context.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_qos_connection(stream, peer, bind, tls).await {
                    warn!(
                        "{QOS_TAG} {} → {}:{} handler error: {}",
                        peer, bind.role, bind.port, e
                    );
                }
            });
        }
    }

    /// DirtySDK latency/bandwidth probes are SOCK_DGRAM to `.qosport` from `/qos/qos` XML.
    async fn run_qos_udp(host: String, bind: QosBind) -> BlazeResult<()> {
        let addr = format!("{}:{}", host, bind.port);
        let socket = UdpSocket::bind(&addr)
            .await
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        if !crate::common::startup_progress::is_startup_in_progress() {
            info!(
                "{QOS_TAG} listening {} UDP ({}) on {}",
                bind.role,
                target_hint(bind.role),
                addr
            );
        }

        let mut buf = vec![0u8; 2048];
        loop {
            let (n, peer) = match socket.recv_from(&mut buf).await {
                Ok(v) => v,
                Err(e) => {
                    warn!("{QOS_TAG} {} UDP :{} recv error: {e}", bind.role, bind.port);
                    continue;
                }
            };
            if n < 4 {
                continue;
            }
            // Client discards probes whose first dword ntohl == 0.
            let id = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            if id == 0 {
                continue;
            }

            crate::session::record_qos_observed_client_endpoint(peer);
            let reply = build_udp_probe_reply(&buf[..n], peer);
            if let Err(e) = socket.send_to(&reply, peer).await {
                debug!(
                    "{QOS_TAG} {peer} → {}:{} UDP reply failed: {e}",
                    bind.role, bind.port
                );
                continue;
            }
            debug!(
                "{QOS_TAG} {peer} → {}:{} UDP probe {n}B→{}B",
                bind.role,
                bind.port,
                reply.len()
            );
        }
    }

    async fn handle_qos_connection(
        mut stream: TcpStream,
        peer: SocketAddr,
        bind: QosBind,
        ssl_context: Option<Arc<ServerConfig>>,
    ) -> BlazeResult<()> {
        crate::session::record_qos_observed_client_endpoint(peer);

        // Read a real first chunk (not 1 byte). A 1-byte peek splits "GET…" into
        // "G" + "ET…" and both fail HTTP classification → [ignored] probes.
        let mut first_buf = vec![0u8; 4096];
        let n = match stream.read(&mut first_buf).await {
            Ok(0) => {
                debug!("{QOS_TAG} {peer} → {}:{} no data", bind.role, bind.port);
                return Ok(());
            }
            Ok(n) => n,
            Err(e) => {
                debug!("{QOS_TAG} {peer} → {}:{} no data: {e}", bind.role, bind.port);
                return Ok(());
            }
        };
        first_buf.truncate(n);
        let first = first_buf[0];
        let prep = PrependStream::new(stream, first_buf);

        if first == 0x16 {
            if let Some(cfg) = ssl_context {
                debug!("{QOS_TAG} {peer} → {}:{} TLS handshake", bind.role, bind.port);
                let acceptor = TlsAcceptor::from(cfg);
                match acceptor.accept(prep).await {
                    Ok(tls_stream) => {
                        return Self::handle_qos_h2_connection(tls_stream, peer, bind, "tls-h2")
                            .await;
                    }
                    Err(e) => {
                        debug!(
                            "{QOS_TAG} {peer} → {}:{} TLS failed: {e}",
                            bind.role, bind.port
                        );
                        return Ok(());
                    }
                }
            } else {
                debug!(
                    "{QOS_TAG} {peer} → {}:{} TLS without cert -- drop",
                    bind.role, bind.port
                );
                return Ok(());
            }
        }

        if first == 0x50 {
            return Self::handle_qos_h2_connection(prep, peer, bind, "h2").await;
        }

        debug!(
            "{QOS_TAG} {peer} → {}:{} cleartext first=0x{:02x} ({}B)",
            bind.role, bind.port, first, n
        );
        Self::handle_qos_io_loop(prep, peer, bind).await
    }

    async fn handle_qos_h2_connection<S>(
        stream: S,
        peer: SocketAddr,
        bind: QosBind,
        transport: &'static str,
    ) -> BlazeResult<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let mut session = QosSessionLog::begin(peer, bind, transport);
        let mut conn = server::handshake(stream).await?;
        while let Some(next) = conn.accept().await {
            match next {
                Ok((request, respond)) => {
                    match Self::process_qos_h2_request(request, respond, peer, bind).await {
                        Ok(kind) => session.note_probe(kind),
                        Err(e) => {
                            debug!(
                                "{QOS_TAG} {peer} → {}:{} h2 stream error: {e}",
                                bind.role, bind.port
                            );
                            session.fail("h2-error");
                        }
                    }
                }
                Err(e) => {
                    debug!(
                        "{QOS_TAG} {peer} → {}:{} h2 accept error: {e}",
                        bind.role, bind.port
                    );
                    if session.probes == 0 {
                        session.fail("h2-accept");
                    }
                    break;
                }
            }
        }
        session.end();
        Ok(())
    }

    async fn process_qos_h2_request(
        mut request: Request<h2::RecvStream>,
        mut respond: SendResponse<Bytes>,
        peer: SocketAddr,
        bind: QosBind,
    ) -> BlazeResult<&'static str> {
        let method = request.method().as_str().to_string();
        let path = request.uri().path().to_string();
        let path_lc = path.to_lowercase();
        let content_type_lc = request
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();
        let is_grpc = content_type_lc.contains("application/grpc")
            || path_lc.contains("/grpc.")
            || path_lc.contains("grpc")
            || path_lc.contains("health")
            || path_lc.contains("check");

        let request_headers: Vec<(String, String)> = request
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_string(),
                    v.to_str().unwrap_or("<binary>").to_string(),
                )
            })
            .collect();
        let mut request_body = Vec::new();
        while let Some(chunk) = request.body_mut().data().await {
            let c = chunk?;
            request_body.extend_from_slice(&c);
        }
        if is_grpc {
            Self::capture_qos_grpc(
                GrpcDirection::ClientToServer,
                &method,
                &path,
                request_headers,
                request_body.clone(),
                None,
            );
        }

        if is_grpc {
            let request_proto = parse_grpc_frame(&request_body).ok().map(|(_, data)| data);
            let request_proto_len = request_proto.as_ref().map(|b| b.len()).unwrap_or(0);
            let is_followup_call =
                path == "/eadp.qoscoordinator.QOSCoordinator/ClientCall" && request_proto_len > 250;
            let response_body = if path == "/eadp.qoscoordinator.QOSCoordinator/ClientCall" {
                wrap_grpc_message_frame(&build_qos_clientcall_response_payload(is_followup_call))
            } else {
                vec![0, 0, 0, 0, 0]
            };
            let response = Response::builder()
                .status(200)
                .header("content-type", "application/grpc")
                .body(())
                .map_err(|e| crate::common::error::BlazeError::Http2(e.to_string()))?;
            let mut send = respond.send_response(response, false)?;
            send.send_data(Bytes::copy_from_slice(&response_body), false)?;

            let mut trailers = http::HeaderMap::new();
            trailers.insert("grpc-status", http::HeaderValue::from_static("0"));
            trailers.insert("grpc-message", http::HeaderValue::from_static(""));
            send.send_trailers(trailers)?;
            Self::capture_qos_grpc(
                GrpcDirection::ServerToClient,
                &method,
                &path,
                vec![
                    ("content-type".to_string(), "application/grpc".to_string()),
                    ("grpc-status".to_string(), "0".to_string()),
                    ("grpc-message".to_string(), "".to_string()),
                ],
                response_body.clone(),
                Some("0".to_string()),
            );

            let kind = classify_http_probe(&path_lc);
            debug!(
                "{QOS_TAG} {peer} → {}:{} gRPC {method} {path} ({}B) followup={}",
                bind.role,
                bind.port,
                response_body.len(),
                is_followup_call
            );
            return Ok(kind);
        }

        let path_and_query = request.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(path.as_str());
        let query = path_and_query.split_once('?').map(|(_, q)| q).unwrap_or("");
        let body = Self::qos_http_body(&path_lc, query, peer, bind.port);
        let kind = classify_http_probe(&path_lc);

        let body_bytes = Bytes::copy_from_slice(body.as_bytes());
        let response = Response::builder()
            .status(200)
            .header("content-type", "text/xml")
            .header("content-length", body.len().to_string())
            .body(())
            .map_err(|e| crate::common::error::BlazeError::Http2(e.to_string()))?;

        let mut send = respond.send_response(response, false)?;
        send.send_data(body_bytes, true)?;

        debug!(
            "{QOS_TAG} {peer} → {}:{} h2 200 {method} {path} ({}B {kind})",
            bind.role,
            bind.port,
            body.len()
        );
        Ok(kind)
    }

    fn capture_qos_grpc(
        direction: GrpcDirection,
        method: &str,
        path: &str,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        grpc_status: Option<String>,
    ) {
        capture_qos_grpc_record(direction, method, path, headers, body, grpc_status);
    }

    async fn handle_qos_io_loop<S: AsyncRead + AsyncWrite + Unpin>(
        mut stream: S,
        peer: SocketAddr,
        bind: QosBind,
    ) -> BlazeResult<()> {
        let mut session = QosSessionLog::begin(peer, bind, "cleartext");
        let mut pending = Vec::new();
        loop {
            let mut read_buf = vec![0u8; 4096];
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                stream.read(&mut read_buf),
            )
            .await
            {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    pending.extend_from_slice(&read_buf[..n]);

                    if looks_like_http_prefix(&pending) && !http_request_complete(&pending) {
                        if pending.len() < 16_384 {
                            continue;
                        }
                    }

                    let chunk = std::mem::take(&mut pending);
                    let (response, kind, detail) = if let Ok(s) = std::str::from_utf8(&chunk) {
                        let first_line = s.lines().next().unwrap_or("").trim_end();
                        if looks_like_http_request(first_line) {
                            let (bytes, path_tag) =
                                Self::handle_http_qos_request(s, peer, bind.port);
                            let kind = if path_tag.contains("firewall") {
                                "firewall"
                            } else if path_tag.contains("firetype") {
                                "firetype"
                            } else if path_tag.contains("/qos/qos") {
                                "bandwidth"
                            } else {
                                "http"
                            };
                            (bytes, kind, format!("{first_line} → {path_tag}"))
                        } else {
                            (
                                Vec::new(),
                                "ignored",
                                format!("non-http {first_line:?}"),
                            )
                        }
                    } else {
                        if chunk.first() == Some(&0x16) {
                            debug!(
                                "{QOS_TAG} {peer} → {}:{} TLS on cleartext socket",
                                bind.role, bind.port
                            );
                        }
                        // Latency/bandwidth probes are UDP; TCP binary stubs are ignored.
                        (Vec::new(), "ignored", "binary-on-tcp".to_string())
                    };

                    session.note_probe(kind);
                    if kind == "ignored" {
                        debug!(
                            "{QOS_TAG} {peer} → {}:{} #{} {kind} {}B | {detail}",
                            bind.role,
                            bind.port,
                            session.probes,
                            chunk.len()
                        );
                    } else {
                        info!(
                            "{QOS_TAG} {peer} → {}:{} #{} {kind} {}B→{}B | {detail}",
                            bind.role,
                            bind.port,
                            session.probes,
                            chunk.len(),
                            response.len()
                        );
                    }

                    if !response.is_empty() {
                        if let Err(e) = stream.write_all(&response).await {
                            if io_is_expected_peer_close(&e) {
                                break;
                            }
                            error!(
                                "{QOS_TAG} {peer} → {}:{} write failed: {e}",
                                bind.role, bind.port
                            );
                            session.fail("write-error");
                            break;
                        }
                        let _ = stream.flush().await;
                    }
                }
                Ok(Err(e)) => {
                    if !io_is_expected_peer_close(&e) {
                        error!(
                            "{QOS_TAG} {peer} → {}:{} read error: {e}",
                            bind.role, bind.port
                        );
                        session.fail("read-error");
                    }
                    break;
                }
                Err(_) => {
                    debug!(
                        "{QOS_TAG} {peer} → {}:{} keepalive wait",
                        bind.role, bind.port
                    );
                    continue;
                }
            }
        }

        session.end();
        Ok(())
    }

    fn qos_http_body(path_lc: &str, query: &str, peer: SocketAddr, bind_port: u16) -> String {
        if path_lc == "/qos/qos" {
            let qtyp = parse_u32_param(query, "qtyp").unwrap_or(1);
            let qos_port = parse_u32_param(query, "prpt")
                .map(|p| p as u16)
                .filter(|p| *p != 0)
                .unwrap_or(bind_port);
            build_qos_xml(qos_port, qtyp)
        } else if path_lc == "/qos/firewall" {
            let rqid = parse_u32_param(query, "rqid").unwrap_or(1);
            let rqsc = parse_u32_param(query, "rqsc").unwrap_or_else(next_req_secret);
            build_firewall_xml(peer, Some((rqid.max(1), rqsc.max(1))))
        } else if path_lc == "/qos/firetype" {
            build_firetype_xml()
        } else {
            build_qos_xml(bind_port, 1)
        }
    }

    fn handle_http_qos_request(
        request: &str,
        peer: SocketAddr,
        bind_port: u16,
    ) -> (Vec<u8>, &'static str) {
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return (Self::http_error_response(400, "Bad Request"), "400 empty");
        }

        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return (
                Self::http_error_response(400, "Bad Request"),
                "400 bad_request_line",
            );
        }

        let path_query = parts[1];
        let (path, query) = match path_query.split_once('?') {
            Some((p, q)) => (p, q),
            None => (path_query, ""),
        };
        let path_lc = path.to_lowercase();
        let body = Self::qos_http_body(&path_lc, query, peer, bind_port);
        let tag: &'static str = if path_lc == "/qos/qos" {
            "200 /qos/qos"
        } else if path_lc == "/qos/firewall" {
            "200 /qos/firewall"
        } else if path_lc == "/qos/firetype" {
            "200 /qos/firetype"
        } else {
            "200 default"
        };
        (http_ok_xml(&body), tag)
    }

    fn http_error_response(status_code: u16, reason: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            status_code,
            reason,
            reason.len(),
            reason
        )
        .into_bytes()
    }
}
