//! Local QoS coordinator — replaces EA’s `qoscoordinator` + regional probes for offline emulation.
//! Binds 3659 / 4001 / 10010 (TLS + HTTP) so the client can use advertised QOSS endpoints locally.
//!
//! Blaze advertises these endpoints via preAuth **QOSS** (e.g. QCNF QCA/QCP) and **LTPS** (PSA/PSP);
//! this module is the TCP side (TLS or cleartext HTTP/binary probes).
//!
//! HTTP/2 on coordinator ports uses the **`h2` crate** (same as the main HTTP/2 stack). The previous
//! manual frame parser mis-handled HEADERS frames (0x01) as DATA and never sent HTTP/2 responses,
//! which left client streams incomplete and caused Blaze disconnect/reconnect churn.

use crate::common::error::{io_is_expected_peer_close, BlazeResult};
use crate::core::inspector::inspector_module::{capture_grpc, CapturedGrpc, GrpcDirection};
use crate::grpc::{grpc_body_decode_capture, parse_grpc_frame};
use bytes::Bytes;
use h2::server::{self, SendResponse};
use http::{Request, Response};
use rustls::ServerConfig;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, warn};

fn bytes_preview(data: &[u8], max: usize) -> String {
    let n = data.len().min(max);
    if n == 0 {
        return "(empty)".to_string();
    }
    let hex = data[..n]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    if data.len() > max {
        format!("{} … (+{} bytes)", hex, data.len() - max)
    } else {
        hex
    }
}

fn looks_like_http_request(first_line: &str) -> bool {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return false;
    }
    matches!(
        parts[0],
        "GET" | "POST" | "HEAD" | "OPTIONS" | "PUT" | "DELETE" | "PATCH"
    ) && parts[1].starts_with('/')
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
    framed.push(0); // compression flag: uncompressed
    framed.extend_from_slice(&(protobuf_payload.len() as u32).to_be_bytes());
    framed.extend_from_slice(protobuf_payload);
    framed
}

fn build_qos_clientcall_response_payload(is_followup_call: bool) -> Vec<u8> {
    let mut out = Vec::new();
    proto_write_sint32(&mut out, 2, -1033725037);
    proto_write_string(&mut out, 3, "v4[159.196.128.63]");

    if !is_followup_call {
        // First response shape from live capture.
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
        // Follow-up response shape from live capture.
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

/// Prepends bytes already read from the socket (used to peek TLS vs plain HTTP).
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
        if self.head_off < self.head.len() {
            let rem = &self.head[self.head_off..];
            let n = rem.len().min(buf.remaining());
            buf.put_slice(&rem[..n]);
            self.head_off += n;
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
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

/// QoS Protocol Server - Handles Quality of Service coordination
/// The QoS coordinator is used by the client to measure network quality and latency
pub struct QosProtocolServer {
    host: String,
    ssl_context: Option<Arc<ServerConfig>>,
}

impl QosProtocolServer {
    /// Create new QoS protocol server (TLS optional — same cert as Web/Blaze for coordinator HTTPS).
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

    /// Start QoS protocol server
    pub async fn start_qos_server(
        &self,
        ports: &crate::common::game::ServicePorts,
    ) -> BlazeResult<()> {
        let c = ports.qos_coordinator;
        let host = self.host.clone();
        let tls = self.ssl_context.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_qos_server(host.clone(), c, tls.clone()).await {
                error!("QoS server error: {}", e);
            }
        });

        let d = ports.qos_data;
        let host = self.host.clone();
        let tls = self.ssl_context.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_qos_server(host.clone(), d, tls.clone()).await {
                error!("QoS data port server error: {}", e);
            }
        });

        let a = ports.qos_alt;
        let host = self.host.clone();
        let tls = self.ssl_context.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_qos_server(host, a, tls).await {
                error!("QoS coordinator alt port server error: {}", e);
            }
        });

        Ok(())
    }

    /// Run QoS server
    async fn run_qos_server(
        host: String,
        port: u16,
        ssl_context: Option<Arc<ServerConfig>>,
    ) -> BlazeResult<()> {
        let addr = format!("{}:{}", host, port);
        let socket = tokio::net::TcpSocket::new_v4()
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        #[cfg(windows)]
        socket.set_reuseaddr(true)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        socket.bind(addr.parse()
            .map_err(|e| crate::common::error::BlazeError::InvalidPacket(format!("Invalid address: {}", e)))?)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        let listener = socket.listen(128)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        if !crate::common::startup_progress::is_startup_in_progress() {
            info!("[QoS] listening on {}", addr);
            debug!(
                "[QoS] advertised in Blaze preAuth QOSS (QCNF QCA/QCP) + LTPS (PSA/PSP)"
            );
        }

        loop {
            let (stream, peer) = listener.accept().await?;
            let tls = ssl_context.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_qos_connection(stream, peer, port, tls).await {
                    warn!("[QoS] peer={} port={} | handler error: {}", peer, port, e);
                }
            });
        }
    }

    /// Handle QoS connection (optional TLS — second request is often TLS ClientHello on port 10010).
    async fn handle_qos_connection(
        mut stream: TcpStream,
        peer: SocketAddr,
        port: u16,
        ssl_context: Option<Arc<ServerConfig>>,
    ) -> BlazeResult<()> {
        crate::session::record_qos_observed_client_endpoint(peer);
        info!("[QoS] peer={} port={} | connected", peer, port);
        debug!(
            "[QoS] peer={} port={} | first byte 0x16 ⇒ TLS; else cleartext HTTP or binary echo",
            peer, port
        );

        let mut first = [0u8; 1];
        if let Err(e) = stream.read_exact(&mut first).await {
            warn!("[QoS] peer={} port={} | no first byte: {}", peer, port, e);
            return Ok(());
        }

        let prep = PrependStream::new(stream, vec![first[0]]);

        if first[0] == 0x16 {
            if let Some(cfg) = ssl_context {
                debug!(
                    "[QoS] peer={} port={} | TLS ClientHello (0x16), handshaking",
                    peer, port
                );
                let acceptor = TlsAcceptor::from(cfg);
                match acceptor.accept(prep).await {
                    Ok(tls_stream) => {
                        info!("[QoS] peer={} port={} | TLS session ready", peer, port);
                        return Self::handle_qos_h2_connection(tls_stream, peer, port).await;
                    }
                    Err(e) => {
                        warn!("[QoS] peer={} port={} | TLS handshake failed: {}", peer, port, e);
                        return Ok(());
                    }
                }
            } else {
                warn!(
                    "[QoS] peer={} port={} | TLS ClientHello but no server TLS config — drop",
                    peer, port
                );
                return Ok(());
            }
        }

        if first[0] == 0x50 {
            debug!(
                "[QoS] peer={} port={} | HTTP/2 cleartext (PRI), h2 handshake",
                peer, port
            );
            return Self::handle_qos_h2_connection(prep, peer, port).await;
        }

        info!("[QoS] peer={} port={} | cleartext session", peer, port);
        debug!(
            "[QoS] peer={} port={} | first_byte=0x{:02x}",
            peer, port, first[0]
        );
        Self::handle_qos_io_loop(prep, peer, port).await
    }

    /// HTTP/2 coordinator (TLS or `PRI * HTTP/2.0` cleartext) — proper HEADERS/DATA responses per stream.
    async fn handle_qos_h2_connection<S>(
        stream: S,
        peer: SocketAddr,
        port: u16,
    ) -> BlazeResult<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let mut conn = server::handshake(stream).await?;
        while let Some(next) = conn.accept().await {
            match next {
                Ok((request, respond)) => {
                    tokio::spawn(async move {
                        if let Err(e) =
                            Self::process_qos_h2_request(request, respond, peer, port).await
                        {
                            warn!("[QoS] peer={} port={} | h2 stream error: {}", peer, port, e);
                        }
                    });
                }
                Err(e) => {
                    warn!("[QoS] peer={} port={} | h2 accept error: {}", peer, port, e);
                    break;
                }
            }
        }
        debug!("[QoS] peer={} port={} | h2 connection finished", peer, port);
        Ok(())
    }

    async fn process_qos_h2_request(
        mut request: Request<h2::RecvStream>,
        mut respond: SendResponse<Bytes>,
        peer: SocketAddr,
        port: u16,
    ) -> BlazeResult<()> {
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

            // gRPC wire frame: compressed-flag(0) + message-length(0), no protobuf payload.
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

            info!(
                "[QoS] peer={} port={} | h2 200 gRPC {} {} ({}B)",
                peer, port, method, path, response_body.len()
            );
            return Ok(());
        }

        let (body, tag): (&str, &'static str) = if path_lc == "/qos/qos" {
            ("OK", "200 /qos/qos")
        } else if path_lc == "/qos/firewall" {
            ("1", "200 /qos/firewall")
        } else {
            ("OK", "200 default")
        };

        let body_bytes = Bytes::copy_from_slice(body.as_bytes());
        let response = Response::builder()
            .status(200)
            .header("content-type", "text/plain")
            .header("content-length", body.len().to_string())
            .body(())
            .map_err(|e| crate::common::error::BlazeError::Http2(e.to_string()))?;

        let mut send = respond.send_response(response, false)?;
        send.send_data(body_bytes, true)?;

        info!(
            "[QoS] peer={} port={} | h2 {} {} {} → {}B",
            peer, port, tag, method, path, body.len()
        );
        Ok(())
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
        port: u16,
    ) -> BlazeResult<()> {
        let mut request_count = 0u32;
        loop {
            let mut read_buf = vec![0u8; 4096];
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                stream.read(&mut read_buf),
            )
            .await
            {
                Ok(Ok(0)) => {
                    info!(
                        "[QoS] peer={} port={} | closed ({} request(s))",
                        peer, port, request_count
                    );
                    break;
                }
                Ok(Ok(n)) => {
                    request_count += 1;
                    let chunk = &read_buf[..n];

                    let (response, kind, detail) =
                        if let Ok(s) = std::str::from_utf8(chunk) {
                            let first_line = s.lines().next().unwrap_or("").trim_end();
                            if looks_like_http_request(first_line) {
                                let (bytes, tag) = Self::handle_http_qos_request(s);
                                (
                                    bytes,
                                    "http",
                                    format!("{} → {}", first_line, tag),
                                )
                            } else {
                                let echo = Self::generate_binary_qos_response(chunk);
                                let prev = bytes_preview(chunk, 24);
                                (
                                    echo,
                                    "binary",
                                    format!("utf8 non-HTTP first_line={:?} preview {}", first_line, prev),
                                )
                            }
                        } else {
                            if chunk.first() == Some(&0x16) {
                                warn!(
                                    "[QoS] peer={} port={} | req #{} | TLS 0x16 on cleartext (unexpected)",
                                    peer, port, request_count
                                );
                            }
                            let echo = Self::generate_binary_qos_response(chunk);
                            let prev = bytes_preview(chunk, 24);
                            (
                                echo,
                                "binary",
                                format!("non-utf8 preview {}", prev),
                            )
                        };

                    info!(
                        "[QoS] peer={} port={} | req {} | {} | {}B → {}B",
                        peer, port, request_count, kind, n, response.len()
                    );
                    debug!("[QoS] peer={} port={} | req {} detail: {}", peer, port, request_count, detail);

                    if !response.is_empty() {
                        if let Err(e) = stream.write_all(&response).await {
                            if io_is_expected_peer_close(&e) {
                                debug!(
                                    "[QoS] peer={} port={} | write stopped (peer closed): {}",
                                    peer, port, e
                                );
                                info!(
                                    "[QoS] peer={} port={} | closed ({} request(s))",
                                    peer, port, request_count
                                );
                            } else {
                                error!("[QoS] peer={} port={} | write failed: {}", peer, port, e);
                            }
                            break;
                        }
                        let _ = stream.flush().await;
                    }
                }
                Ok(Err(e)) => {
                    if io_is_expected_peer_close(&e) {
                        debug!(
                            "[QoS] peer={} port={} | read ended after {} req(s) (peer closed): {}",
                            peer, port, request_count, e
                        );
                        info!(
                            "[QoS] peer={} port={} | closed ({} request(s))",
                            peer, port, request_count
                        );
                    } else {
                        error!("[QoS] peer={} port={} | read error: {}", peer, port, e);
                    }
                    break;
                }
                Err(_) => {
                    debug!(
                        "[QoS] peer={} port={} | read timeout 30s (keepalive), still waiting",
                        peer, port
                    );
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Handle HTTP QoS requests. Second element is a short outcome tag for logs.
    fn handle_http_qos_request(request: &str) -> (Vec<u8>, &'static str) {
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return (Self::http_error_response(400, "Bad Request"), "400 empty");
        }

        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return (Self::http_error_response(400, "Bad Request"), "400 bad_request_line");
        }

        let path_query = parts[1];

        let (path, _query) = if let Some(pos) = path_query.find('?') {
            (&path_query[..pos], &path_query[pos + 1..])
        } else {
            (path_query, "")
        };
        let path_lc = path.to_lowercase();

        let (body, tag): (&str, &'static str) = if path_lc == "/qos/qos" {
            ("OK", "200 /qos/qos")
        } else if path_lc == "/qos/firewall" {
            ("1", "200 /qos/firewall NAT=open")
        } else {
            ("OK", "200 default OK (unknown path)")
        };

        let bytes = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: keep-alive\r\n\
             \r\n\
             {}",
            body.len(),
            body
        )
        .into_bytes();

        (bytes, tag)
    }

    /// Generate HTTP error response
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

    /// Generate binary QoS response (for non-HTTP protocols)
    fn generate_binary_qos_response(_request: &[u8]) -> Vec<u8> {
        // Format: [status_byte][latency_ms: u32][packet_loss: u8][bandwidth: u32]
        let mut response = Vec::new();
        response.push(0x00); // Status: OK
        response.extend_from_slice(&(10u32.to_le_bytes())); // Latency: 10ms
        response.push(0x00); // Packet loss: 0%
        response.extend_from_slice(&(1000000u32.to_le_bytes())); // Bandwidth: 1Mbps
        response
    }
}
