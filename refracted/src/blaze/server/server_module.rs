use crate::crypto::SessionState;
use crate::common::error::{io_is_expected_peer_close, BlazeError, BlazeResult};
use crate::blaze::protocol::{
    build_get_server_instance_reply, find_get_server_instance, Fire2FrameHeader, Fire2FramePacket,
    MessageType, RedirectorWire,
};
use crate::blaze::handlers::{
    handle_packet, handle_packet_fields, handle_user_session_extended_data_update,
    handle_user_session_extended_data_update_first,
    handle_user_session_extended_data_update_second,
    handle_user_session_extended_data_update_third,
    handle_user_sessions_added, handle_user_sessions_authenticated,
};
use crate::blaze::server::connection_info_coalesce::{
    key_b2c_idle_user_session_ex, key_b2c_keepalive_reply, key_b2c_notif, key_b2c_reply,
    key_b2c_toolkit_inject, key_c2b, key_c2b_keepalive, key_fire_b2c, key_fire_c2b, CoalescedBlazeInfo,
    PingBurstCoalescer,
};
use crate::core::inspector::{capture_packet, CapturedPacket, PacketDirection};
use bytes::{Bytes, BytesMut};
use rustls::ServerConfig;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Duration};
use tokio_rustls::{server::TlsStream as ServerTlsStream, TlsAcceptor};
use tracing::{debug, error, info, warn};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};

static AUTH_COMPLETE_MILESTONE_LOGGED: AtomicBool = AtomicBool::new(false);
static MATCH_JOIN_MILESTONE_LOGGED: AtomicBool = AtomicBool::new(false);

/// A wrapper that buffers initial bytes and can "unread" them
#[allow(dead_code)]
struct BufferedStream {
    buffer: Vec<u8>,
    buffer_pos: usize,
    stream: TcpStream,
}

impl BufferedStream {
    #[allow(dead_code)]
    fn new(stream: TcpStream, initial_bytes: Vec<u8>) -> Self {
        Self {
            buffer: initial_bytes,
            buffer_pos: 0,
            stream,
        }
    }
}

impl AsyncRead for BufferedStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        // First, drain the buffer
        if self.buffer_pos < self.buffer.len() {
            let available = self.buffer.len() - self.buffer_pos;
            let to_copy = available.min(buf.remaining());
            buf.put_slice(&self.buffer[self.buffer_pos..self.buffer_pos + to_copy]);
            self.buffer_pos += to_copy;
            return Poll::Ready(Ok(()));
        }
        
        // Then read from the underlying stream
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for BufferedStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

async fn blaze_send(
    stream: &mut (impl AsyncWriteExt + Unpin),
    data: &[u8],
    addr: SocketAddr,
    listener: &str,
    label: &str,
) -> Result<bool, BlazeError> {
    if let Err(e) = stream.write_all(data).await {
        if io_is_expected_peer_close(&e) {
            info!(
                "[Blaze] [{}] peer {} closed on write ({}): {}",
                listener, addr, label, e
            );
            return Ok(false);
        }
        return Err(BlazeError::Io(e));
    }
    if let Err(e) = stream.flush().await {
        if io_is_expected_peer_close(&e) {
            info!(
                "[Blaze] [{}] peer {} closed on flush ({}): {}",
                listener, addr, label, e
            );
            return Ok(false);
        }
        return Err(BlazeError::Io(e));
    }
    Ok(true)
}

async fn blaze_write_only(
    stream: &mut (impl AsyncWriteExt + Unpin),
    data: &[u8],
    addr: SocketAddr,
    listener: &str,
    label: &str,
) -> Result<bool, BlazeError> {
    if let Err(e) = stream.write_all(data).await {
        if io_is_expected_peer_close(&e) {
            info!(
                "[Blaze] [{}] peer {} closed on write ({}): {}",
                listener, addr, label, e
            );
            return Ok(false);
        }
        return Err(BlazeError::Io(e));
    }
    Ok(true)
}

fn capture_outgoing_packet(packet: &Fire2FramePacket, raw_data: &[u8]) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    
    let cmd_name = crate::blaze::components::get_command_name(packet.header.component, packet.header.command)
        .map(|s| s.to_string());
    
    capture_packet(CapturedPacket {
        timestamp,
        direction: PacketDirection::BlazeToClient,
        component: packet.header.component,
        command: packet.header.command,
        msg_num: packet.header.msg_num,
        msg_type: packet.header.msg_type.to_string().to_string(),
        payload_size: packet.payload.len(),
        payload: packet.payload.to_vec(),
        raw_packet: raw_data.to_vec(),
        command_name: cmd_name,
        metadata_size: packet.header.metadata_size,
    });
}

/// Blaze Protocol Server - Handles all Blaze protocol communications
pub struct BlazeProtocolServer {
    host: String,
    ssl_context: Option<Arc<ServerConfig>>,
    running: Arc<AtomicBool>,
}

impl BlazeProtocolServer {
    /// Create new Blaze protocol server
    pub fn new(
        host: String,
        ssl_context: Option<Arc<ServerConfig>>,
        running: Arc<AtomicBool>,
    ) -> Self {
        Self {
            host,
            ssl_context,
            running,
        }
    }

    fn ports_with_tls(
        p: &crate::common::game::ServicePorts,
    ) -> Vec<(u16, &'static str, bool)> {
        let redirector_tls = crate::common::game::get_current_game()
            .map(|g| g.redirector_tls)
            .unwrap_or(true);
        vec![
            (p.blaze_gosredirector, "gosredirector", redirector_tls),
            (p.blaze_gosca, "gosca", true),
            (p.blaze_main, "blaze-main", true),
            (p.blaze_alt, "blaze-alt", true),
            (p.blaze_sec, "blaze-sec", true),
        ]
    }

    pub fn ports_from_config(p: &crate::common::game::ServicePorts) -> Vec<(u16, String)> {
        Self::ports_with_tls(p)
            .into_iter()
            .map(|(port, name, _)| (port, name.to_string()))
            .collect()
    }

    /// Start all Blaze protocol servers
    pub async fn start_blaze_servers(
        &self,
        ports: &crate::common::game::ServicePorts,
    ) -> BlazeResult<()> {
        let blaze_ports = Self::ports_with_tls(ports);

        for (port, name, use_tls) in blaze_ports {
            let host = self.host.clone();
            let ssl_context = if use_tls {
                self.ssl_context.clone()
            } else {
                None
            };
            let running = self.running.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    Self::run_blaze_server(host, port, name, ssl_context, running)
                        .await
                {
                    error!("{} server error: {}", name, e);
                }
            });
            // Blaze server started (logged by startup progress)
        }

        Ok(())
    }

    /// Run a Blaze protocol server
    async fn run_blaze_server(
        host: String,
        port: u16,
        name: &str,
        ssl_context: Option<Arc<ServerConfig>>,
        running: Arc<AtomicBool>,
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
            info!("{} server listening on {}", name, addr);
        }

        loop {
            let (stream, addr) = listener.accept().await?;
            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m New connection accepted on {} from {}", name, addr);
            let ssl_context = ssl_context.clone();
            let name = name.to_string();
            let running = running.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_blaze_connection(stream, addr, &name, ssl_context, running).await {
                    error!("{} connection error: {}", name, e);
                }
            });
        }
    }

    /// Handle Blaze connection
    async fn handle_blaze_connection(
        stream: TcpStream,
        addr: SocketAddr,
        name: &str,
        ssl_context: Option<Arc<ServerConfig>>,
        _running: Arc<AtomicBool>,
    ) -> BlazeResult<()> {
        if name == "gosredirector" {
            info!(
                "\x1b[38;2;150;150;255m[GOS]\x1b[0m connection from {}",
                addr
            );
        } else {
            info!("{} connection from {}", name, addr);
        }
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Processing {} connection from {}", name, addr);

        if let Some(config) = ssl_context {
            let looks_like_tls = match timeout(Duration::from_millis(300), stream.peek(&mut [0u8; 1])).await {
                Ok(Ok(1)) => {
                    let mut b = [0u8; 1];
                    // Peek again to retrieve byte value for routing decision.
                    if let Ok(1) = stream.peek(&mut b).await {
                        // TLS record types seen at start of connection are usually 0x16 (handshake),
                        // but allow common record types to be robust with resumed/error paths.
                        matches!(b[0], 0x14 | 0x15 | 0x16 | 0x17)
                    } else {
                        true
                    }
                }
                Ok(Ok(_)) => false,
                Ok(Err(_)) => true,
                Err(_) => true,
            };

            if !looks_like_tls {
                info!(
                    "[Blaze] [{}] {} does not look like TLS; handling as plain Blaze",
                    name, addr
                );
                if name == "gosredirector" {
                    let protocol = crate::common::game::get_current_game()
                        .map(|g| g.protocol)
                        .unwrap_or_else(|| "Fire2Frame".to_string());
                    Self::handle_gosredirector_blaze(stream, addr, &protocol).await?;
                } else {
                    Self::handle_blaze_plain(stream, addr, name).await?;
                }
                return Ok(());
            }

            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Initiating TLS handshake for {} from {}", name, addr);
            let acceptor = TlsAcceptor::from(config);
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    // Log TLS connection details
                    let (_, session) = tls_stream.get_ref();
                    let protocol = session.protocol_version();
                    let negotiated_cipher = session.negotiated_cipher_suite();
                    info!("TLS handshake completed for {}", addr);
                    debug!(
                        "TLS handshake for {}: protocol {:?}, cipher {:?}",
                        addr,
                        protocol,
                        negotiated_cipher.map(|c| c.suite())
                    );

                    // Special handling for gosredirector - it uses Blaze-framed redirector packets.
                    if name == "gosredirector" {
                        let protocol = crate::common::game::get_current_game()
                            .map(|g| g.protocol)
                            .unwrap_or_else(|| "Fire2Frame".to_string());
                        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Routing {} connection to redirector Blaze handler ({})", name, protocol);
                        Self::handle_gosredirector_blaze(tls_stream, addr, &protocol).await?;
                    } else {
                        // Handle Blaze over TLS
                        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Routing {} connection to Blaze protocol handler", name);
                        Self::handle_blaze_over_tls(
                            tls_stream,
                            addr,
                            name,
                        )
                        .await?;
                    }
                }
                Err(e) => {
                    error!("TLS handshake failed for {} ({}): {}", name, addr, e);
                    crate::debug_println!("\x1b[38;2;255;150;150m[Blaze]\x1b[0m TLS handshake failed for {} ({}): {}", name, addr, e);
                    let error_str = format!("{}", e);
                    let error_debug = format!("{:?}", e);
                    error!("[TLS] Error details: {}", error_debug);
                    
                    if error_str.contains("AlertReceived") {
                        error!("Client sent TLS alert - certificate validation likely failed");
                    } else if error_str.contains("CorruptMessage") {
                        error!("TLS message corruption - possible protocol version mismatch");
                    } else if error_str.contains("InappropriateMessage") {
                        error!("TLS protocol error - version or cipher suite mismatch?");
                    } else if error_str.contains("InappropriateHandshakeMessage") {
                        error!("TLS handshake message error - possible version mismatch (client may require TLS 1.0/1.1)");
                    } else if error_str.contains("PeerIncompatibleError") {
                        error!("TLS peer incompatible - client/server protocol version mismatch");
                    } else if error_str.contains("forcibly closed") || error_str.contains("10054") {
                        error!("[TLS] Client closed connection during handshake - likely certificate rejection");
                        error!("[TLS] Check if CertVerifyCertificateChainPolicy hook is being called");
                    }
                    
                    // Check if this might be a TLS version issue
                    if error_str.contains("Inappropriate") || error_str.contains("Incompatible") {
                        error!("NOTE: rustls only supports TLS 1.2 and 1.3");
                        error!("If client requires TLS 1.0/1.1, consider using native-tls instead");
                    }
                    
                    // Convert boxed error to IO error (acceptor.accept returns Box<dyn Error>)
                    return Err(BlazeError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("TLS handshake failed: {}", e),
                    )));
                }
            }
        } else {
            if name == "gosredirector" {
                let protocol = crate::common::game::get_current_game()
                    .map(|g| g.protocol)
                    .unwrap_or_else(|| "Fire2Frame".to_string());
                Self::handle_gosredirector_blaze(stream, addr, &protocol).await?;
            } else {
                // Handle plain Blaze connection
                crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Handling plain Blaze connection for {} from {}", name, addr);
                Self::handle_blaze_plain(stream, addr, name).await?;
            }
        }

        Ok(())
    }

    /// Handle Blaze over TLS
    async fn handle_blaze_over_tls<S: AsyncReadExt + AsyncWriteExt + Unpin>(
        stream: ServerTlsStream<S>,
        addr: SocketAddr,
        name: &str,
    ) -> BlazeResult<()> {
        info!("Handling Blaze over TLS for {}", addr);
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m handle_blaze_over_tls entered for {} from {}", name, addr);

        // Create session state
        let mut state = SessionState::new();
        let sid = crate::session::blaze_sessions::register(addr, name);
        state.blaze_session_id = Some(sid);
        let _blaze_sess = crate::session::blaze_sessions::BlazeSessionGuard::new(sid);
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Created new session state for {}", addr);

        let protocol = crate::common::game::get_current_game()
            .map(|g| g.protocol)
            .unwrap_or_else(|| "Fire2Frame".to_string());
        if protocol.eq_ignore_ascii_case("fireframe") {
            match Self::handle_blaze_protocol_fireframe(stream, addr, name, &mut state).await {
                Ok(()) => {}
                Err(BlazeError::InvalidPacket(msg))
                    if msg == "fireframe parser mismatch with incoming stream" =>
                {
                    warn!(
                        "[Blaze] {} {} appears Fire2Frame despite FireFrame profile, retrying with Fire2Frame parser",
                        name, addr
                    );
                    // Reconnect path handles retry on next connection; return cleanly here.
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        } else {
            Self::handle_blaze_protocol(stream, addr, name, &mut state).await?;
        }

        Ok(())
    }

    /// Handle plain Blaze
    async fn handle_blaze_plain(
        stream: TcpStream,
        addr: SocketAddr,
        name: &str,
    ) -> BlazeResult<()> {
        info!("Handling plain Blaze for {}", addr);
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m handle_blaze_plain entered for {} from {}", name, addr);

        // Create session state
        let mut state = SessionState::new();
        let sid = crate::session::blaze_sessions::register(addr, name);
        state.blaze_session_id = Some(sid);
        let _blaze_sess = crate::session::blaze_sessions::BlazeSessionGuard::new(sid);
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Created new session state for {}", addr);

        let protocol = crate::common::game::get_current_game()
            .map(|g| g.protocol)
            .unwrap_or_else(|| "Fire2Frame".to_string());
        if protocol.eq_ignore_ascii_case("fireframe") {
            match Self::handle_blaze_protocol_fireframe(stream, addr, name, &mut state).await {
                Ok(()) => {}
                Err(BlazeError::InvalidPacket(msg))
                    if msg == "fireframe parser mismatch with incoming stream" =>
                {
                    warn!(
                        "[Blaze] {} {} appears Fire2Frame despite FireFrame profile, retrying with Fire2Frame parser",
                        name, addr
                    );
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        } else {
            Self::handle_blaze_protocol(stream, addr, name, &mut state).await?;
        }

        Ok(())
    }

    /// Handle Blaze protocol communication
    async fn handle_blaze_protocol(
        mut stream: impl AsyncReadExt + AsyncWriteExt + Unpin,
        addr: SocketAddr,
        name: &str,
        state: &mut SessionState,
    ) -> BlazeResult<()> {
        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m handle_blaze_protocol entered for {} ({})", addr, name);
        let mut buffer = BytesMut::new();
        let scoped_key = format!("BLAZE_MAIN|{}", addr);
        let mut info_coalesce = CoalescedBlazeInfo::new_scoped(&scoped_key);
        let mut ping_burst = PingBurstCoalescer::new_scoped(&scoped_key);

        let mut chunk = vec![0u8; 4096];
        let mut inject_rx = crate::blaze::server::toolkit_inject::subscribe_toolkit_blaze_wire();

        loop {
            tokio::select! {
                recv_inj = inject_rx.recv() => {
                    match recv_inj {
                        Ok(wire_plain) => {
                            if wire_plain.is_empty() {
                                continue;
                            }
                            match Fire2FramePacket::from_bytes(&wire_plain) {
                                Ok(pkt) => {
                                    let mut data = wire_plain;
                                    if state.crypto_enabled && !data.is_empty() {
                                        match state.c_out.encrypt_copy(&data) {
                                            Ok(enc) => data = enc,
                                            Err(e) => {
                                                warn!(
                                                    "[Blaze] toolkit inject encrypt failed for {}: {}",
                                                    addr, e
                                                );
                                                continue;
                                            }
                                        }
                                    }
                                    capture_outgoing_packet(&pkt, &data);
                                    let mt = pkt.header.msg_type.to_string();
                                    let wl = data.len();
                                    let inj_key = key_b2c_toolkit_inject(
                                        pkt.header.component,
                                        pkt.header.command,
                                        pkt.header.msg_num,
                                        &mt,
                                        wl,
                                    );
                                    let inj_line =
                                        match crate::blaze::components::get_command_name(
                                            pkt.header.component,
                                            pkt.header.command,
                                        ) {
                                            Some(nm) => format!(
                                                "[Blaze→Client] {} toolkit inject Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                                                nm,
                                                pkt.header.component,
                                                pkt.header.command,
                                                wl,
                                                mt,
                                                pkt.header.msg_num,
                                            ),
                                            None => format!(
                                                "[Blaze→Client] toolkit inject Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                                                pkt.header.component,
                                                pkt.header.command,
                                                wl,
                                                mt,
                                                pkt.header.msg_num,
                                            ),
                                        };
                                    info_coalesce.log(&inj_key, inj_line);
                                    if !blaze_send(
                                        &mut stream,
                                        &data,
                                        addr,
                                        name,
                                        "toolkit inject",
                                    )
                                    .await?
                                    {
                                        return Ok(());
                                    }
                                }
                                Err(e) => warn!(
                                    "[Blaze] toolkit inject dropped for {} (not valid): {}",
                                    addr, e
                                ),
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_skipped)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {}
                    }
                }

                read_res = timeout(Duration::from_secs(15), stream.read(&mut chunk)) => {
            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m [{}] Waiting to read from {}", name, addr);
            let n = match read_res {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => {
                    if io_is_expected_peer_close(&e) {
                        info!(
                            "[Blaze] peer {} closed connection while reading ({}): {}",
                            addr,
                            e.kind(),
                            e
                        );
                        return Ok(());
                    }
                    error!("[Blaze] Read error from {}: {}", addr, e);
                    return Err(BlazeError::Io(e));
                }
                Err(_) => {
                    // Perriodic user-session update while idle.
                    if let Ok(payload) = handle_user_session_extended_data_update_first(&[]) {
                        if !payload.is_empty() {
                            let packet = Fire2FramePacket::new_send(
                                0x7802,
                                0x01,
                                0,
                                MessageType::Notification,
                                payload,
                            );
                            let mut data = packet.to_bytes().to_vec();
                            if state.crypto_enabled && !data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut data) {
                                    error!("Encryption error for idle UserSessionExtendedDataUpdate: {}", e);
                                }
                            }
                            capture_outgoing_packet(&packet, &data);
                            let pl = packet.payload.len();
                            let hb_line = format!(
                                "[Blaze→Client] UserSessionExtendedDataUpdate Component=30722, Command=1, Size={}, MsgType=NOTIFICATION, MsgNum=0 (idle heartbeat)",
                                pl
                            );
                            info_coalesce.log(&key_b2c_idle_user_session_ex(pl), hb_line);
                            if !blaze_send(
                                &mut stream,
                                &data,
                                addr,
                                name,
                                "idle UserSessionExtendedDataUpdate heartbeat",
                            )
                            .await?
                            {
                                return Ok(());
                            }
                        }
                    }
                    continue;
                }
            };

            if n == 0 {
                info!("Client {} disconnected", addr);
                crate::debug_println!("\x1b[38;2;255;150;150m[Blaze]\x1b[0m Client {} disconnected", addr);
                break;
            }

            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Read {} bytes from {}", n, addr);
            buffer.extend_from_slice(&chunk[..n]);
            if let Some(sid) = state.blaze_session_id {
                if state.last_registry_crypto != Some(state.crypto_enabled) {
                    if crate::session::blaze_sessions::set_crypto_enabled(sid, state.crypto_enabled)
                    {
                        state.last_registry_crypto = Some(state.crypto_enabled);
                    }
                }
            }
            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Buffer now has {} bytes", buffer.len());

            // Process complete packets
            while buffer.len() >= Fire2FrameHeader::HEADER_SIZE {
                // Try to detect if this is a Fire2Frame packet
                // Fire2Frame packets should have a reasonable size (not too large)
                let potential_size =
                    u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Skip if size is unreasonably large (likely not a Fire2Frame)
                if potential_size > 10000 {
                    info!("[Blaze] Skipping packet: size too large ({})", potential_size);
                    let _ = buffer.split_to(1);
                    continue;
                }

                // Parse header
                crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Attempting to parse Fire2Frame header from {} (buffer size: {})", addr, buffer.len());
                let header =
                    match Fire2FrameHeader::from_bytes(&buffer[..Fire2FrameHeader::HEADER_SIZE]) {
                        Ok(h) => {
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Header parsed successfully - Component={}, Command={}, Size={}, MsgType={:?}, MsgNum={}", 
                                h.component, h.command, h.get_total_size(), h.msg_type, h.msg_num);
                            h
                        },
                        Err(e) => {
                            // Log the error for debugging
                            error!("Failed to parse header from {}: {:?}, buffer len: {}, first 16 bytes: {:?}", 
                                addr, e, buffer.len(), 
                                if buffer.len() >= 16 { 
                                    format!("{:02x?}", &buffer[..16])
                                } else { 
                                    format!("{:02x?}", &buffer[..])
                                });
                            crate::debug_println!("\x1b[38;2;255;150;150m[Blaze]\x1b[0m Header parse failed from {}: {:?}, buffer len: {}", addr, e, buffer.len());
                            // Try to find the start of a valid header by removing one byte
                            let _ = buffer.split_to(1);
                            continue;
                        }
                    };

                let total_packet_size = header.get_total_size();
                crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Total packet size: {}, buffer has: {}", total_packet_size, buffer.len());

                if buffer.len() < total_packet_size {
                    // Need more data
                    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Need more data: have {} bytes, need {} bytes", buffer.len(), total_packet_size);
                    break;
                }


                // Extract payload (skip metadata)
                let payload_start = Fire2FrameHeader::HEADER_SIZE + header.metadata_size as usize;
                
                // Check bounds before slicing
                if payload_start > total_packet_size {
                    error!("[Blaze] Invalid payload_start {} > total_packet_size {}", payload_start, total_packet_size);
                    let _ = buffer.split_to(1);
                    continue;
                }
                
                let payload = buffer[payload_start..total_packet_size].to_vec();

                // Decrypt payload if crypto is enabled
                let mut decrypted_payload = payload;
                if state.crypto_enabled && !decrypted_payload.is_empty() {
                    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Decrypting payload (size: {})", decrypted_payload.len());
                    if let Err(e) = state.c_in.decrypt(&mut decrypted_payload) {
                        error!("Decryption error: {}", e);
                        crate::debug_println!("\x1b[38;2;255;150;150m[Blaze]\x1b[0m Decryption error: {}", e);
                        // Continue without decryption
                    } else {
                        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Payload decrypted successfully");
                    }
                } else {
                    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Crypto not enabled or empty payload, skipping decryption");
                }
                
                // Handle Component=0, Command=0 as keepalive reply.
                // C# reference responds with an empty REPLY and keeps the session alive.
                if header.component == 0 && header.command == 0 {
                    let ka_line = format!(
                        "[Client→Blaze] Component=0, Command=0, Size={}, MsgType={:?}, MsgNum={} - Keepalive",
                        total_packet_size, header.msg_type, header.msg_num
                    );
                    info_coalesce.log(
                        &key_c2b_keepalive(total_packet_size, &format!("{:?}", header.msg_type)),
                        ka_line,
                    );
                    let keepalive_reply = Fire2FramePacket::new_send(
                        0,
                        0,
                        header.msg_num,
                        MessageType::Reply,
                        Bytes::from(Vec::new()),
                    );
                    let mut keepalive_data = keepalive_reply.to_bytes().to_vec();
                    if state.crypto_enabled && !keepalive_data.is_empty() {
                        if let Err(e) = state.c_out.encrypt(&mut keepalive_data) {
                            error!("Encryption error for keepalive reply: {}", e);
                        }
                    }
                    if !blaze_send(&mut stream, &keepalive_data, addr, name, "KEEPALIVE REPLY").await? {
                        return Ok(());
                    }
                    capture_outgoing_packet(&keepalive_reply, &keepalive_data);
                    let klr = format!(
                        "[Blaze→Client] Component=0, Command=0, Size=16, MsgType=REPLY, MsgNum={}",
                        header.msg_num
                    );
                    info_coalesce.log(&key_b2c_keepalive_reply(), klr);
                    let _ = buffer.split_to(total_packet_size);
                    continue;
                }

                // Log incoming packet - use plain text (no ANSI codes) so logs appear
                // CustomFormatter will detect [Client→] marker and skip [Console] prefix
                let msg_type_str = header.msg_type.to_string().to_string();
                let is_ping_req = header.component == 9
                    && header.command == 2
                    && msg_type_str == "REQUEST";
                if is_ping_req {
                    if let Some(cmd_name) = crate::blaze::components::get_command_name(header.component, header.command) {
                        let line = format!(
                            "[Client→Blaze] {} Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                            cmd_name,
                            header.component,
                            header.command,
                            total_packet_size,
                            msg_type_str,
                            header.msg_num
                        );
                        ping_burst.log_request(line);
                    } else {
                        let line = format!(
                            "[Client→Blaze] Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                            header.component,
                            header.command,
                            total_packet_size,
                            msg_type_str,
                            header.msg_num
                        );
                        ping_burst.log_request(line);
                    }
                } else {
                    ping_burst.flush();
                    let c2b_key = key_c2b(
                        header.component,
                        header.command,
                        total_packet_size,
                        &msg_type_str,
                    );
                    if let Some(cmd_name) = crate::blaze::components::get_command_name(header.component, header.command) {
                        let line = format!(
                            "[Client→Blaze] {} Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                            cmd_name,
                            header.component,
                            header.command,
                            total_packet_size,
                            msg_type_str,
                            header.msg_num
                        );
                        info_coalesce.log(&c2b_key, line);
                    } else {
                        let line = format!(
                            "[Client→Blaze] Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                            header.component,
                            header.command,
                            total_packet_size,
                            msg_type_str,
                            header.msg_num
                        );
                        info_coalesce.log(&c2b_key, line);
                    }
                }

                // Capture packet for inspection (before removing from buffer)
                let raw_packet = buffer[..total_packet_size].to_vec();
                let cmd_name = crate::blaze::components::get_command_name(header.component, header.command)
                    .map(|s| s.to_string());
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                
                capture_packet(CapturedPacket {
                    timestamp,
                    direction: PacketDirection::ClientToBlaze,
                    component: header.component,
                    command: header.command,
                    msg_num: header.msg_num,
                    msg_type: msg_type_str,
                    payload_size: decrypted_payload.len(),
                    payload: decrypted_payload.clone(),
                    raw_packet,
                    command_name: cmd_name,
                    metadata_size: header.metadata_size,
                });

                // Create packet
                let packet = Fire2FramePacket {
                    header,
                    payload: bytes::Bytes::from(decrypted_payload),
                };
                if packet.header.component == 0x0004 && packet.header.command == 0x0009 {
                    if !MATCH_JOIN_MILESTONE_LOGGED.swap(true, Ordering::Relaxed) {
                        crate::console_println!(
                            "\x1b[38;2;255;215;0m[MILESTONE]\x1b[0m Match join requested (GameManager.joinGame)"
                        );
                    }
                }
                crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Created packet, calling handle_packet for Component={}, Command={}", 
                    packet.header.component, packet.header.command);

                let packet_for_handler = packet.clone();
                let handle_result = tokio::task::spawn_blocking(move || handle_packet(&packet_for_handler)).await;

                match handle_result {
                    Ok(Ok(response_payload)) => {
                        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m handle_packet succeeded, response payload size: {}", response_payload.len());
                        
                        // Special handling for createAccount: send ALL THREE packets together without flushing between them
                        // Working server sends: UserAuthenticated, login2, UserAdded all at the same time
                        let is_create_account = packet.header.component == 0x0001 && packet.header.command == 0x0a;
                        
                        if is_create_account {
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m createAccount detected, sending notifications in natural flow");
                            
                            // 1. Send UserAuthenticated notification FIRST
                            let user_auth_payload = handle_user_sessions_authenticated(&[])?;
                            let user_auth_packet = Fire2FramePacket::new_send_with_options(
                                0x7802, // UserSessions component (30722)
                                0x08,   // authenticated command
                                0,      // PacketID=0 for notifications
                                MessageType::Notification,
                                user_auth_payload,
                                1,
                            );

                            let mut user_auth_data = user_auth_packet.to_bytes().to_vec();
                            if state.crypto_enabled && !user_auth_data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut user_auth_data) {
                                    error!("Encryption error for UserAuthenticated: {}", e);
                                }
                            }

                            capture_outgoing_packet(&user_auth_packet, &user_auth_data);
                            if !blaze_send(&mut stream, &user_auth_data, addr, name, "UserAuthenticated notification").await? {
                                return Ok(());
                            }
                            
                            {
                                let pl = user_auth_packet.payload.len();
                                let line = format!(
                                    "[Blaze→Client] UserSessions.UserAuthenticated Component=30722, Command=8, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                                    pl
                                );
                                info_coalesce.log(&key_b2c_notif(0x7802, 0x08, pl), line);
                            }

                            // 2. Send login2 REPLY
                            let response_packet = Fire2FramePacket::new_send(
                                packet.header.component,
                                packet.header.command,
                                packet.header.msg_num, // Use original msg_num
                                MessageType::Reply,
                                response_payload,
                            );
                            let mut response_data = response_packet.to_bytes().to_vec();
                            if state.crypto_enabled && !response_data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut response_data) {
                                    error!("Encryption error: {}", e);
                                    crate::debug_println!("\x1b[38;2;255;150;150m[Blaze]\x1b[0m Encryption error: {}", e);
                                } else {
                                    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Response encrypted successfully (size: {})", response_data.len());
                                }
                            }
                            
                            capture_outgoing_packet(&response_packet, &response_data);
                            if !blaze_send(&mut stream, &response_data, addr, name, "createAccount REPLY").await? {
                                return Ok(());
                            }
                            
                            if let Some(cmd_name) = crate::blaze::components::get_command_name(packet.header.component, packet.header.command) {
                                let k = key_b2c_reply(
                                    packet.header.component,
                                    packet.header.command,
                                    response_packet.payload.len(),
                                );
                                let line = format!(
                                    "[Blaze→Client] {} Component={}, Command={}, Size={}, MsgType=REPLY, MsgNum={}",
                                    cmd_name,
                                    packet.header.component,
                                    packet.header.command,
                                    response_packet.payload.len(),
                                    packet.header.msg_num
                                );
                                info_coalesce.log(&k, line);
                            } else {
                                let k = key_b2c_reply(1, 0x0a, response_packet.payload.len());
                                let line = format!(
                                    "[Blaze→Client] AuthenticationComponent.createAccount Component=1, Command=10, Size={}, MsgType=REPLY, MsgNum={}",
                                    response_packet.payload.len(),
                                    packet.header.msg_num
                                );
                                info_coalesce.log(&k, line);
                            }
                            
                            let user_added_payload = handle_user_sessions_added(&[])?;
                            let user_added_packet = Fire2FramePacket::new_send(
                                0x7802, // UserSessions component (30722)
                                0x02,   // UserAdded command
                                0,      // PacketID=0 for notifications
                                MessageType::Notification,
                                user_added_payload,
                            );

                            let mut user_added_data = user_added_packet.to_bytes().to_vec();
                            if state.crypto_enabled && !user_added_data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut user_added_data) {
                                    error!("Encryption error for UserAdded: {}", e);
                                }
                            }

                            capture_outgoing_packet(&user_added_packet, &user_added_data);
                            if !blaze_send(&mut stream, &user_added_data, addr, name, "UserAdded notification").await? {
                                return Ok(());
                            }
                            
                            {
                                let pl = user_added_packet.payload.len();
                                if let Some(cmd_name) = crate::blaze::components::get_command_name(0x7802, 0x02) {
                                    let line = format!(
                                        "[Blaze→Client] {} Component=30722, Command=2, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                                        cmd_name,
                                        pl
                                    );
                                    info_coalesce.log(&key_b2c_notif(0x7802, 0x02, pl), line);
                                } else {
                                    let line = format!(
                                        "[Blaze→Client] UserSessions.UserAdded Component=30722, Command=2, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                                        pl
                                    );
                                    info_coalesce.log(&key_b2c_notif(0x7802, 0x02, pl), line);
                                }
                            }
                            if !AUTH_COMPLETE_MILESTONE_LOGGED.swap(true, Ordering::Relaxed) {
                                crate::console_println!(
                                    "\x1b[38;2;255;215;0m[MILESTONE]\x1b[0m Auth complete (Blaze user session established)"
                                );
                            }

                            if let Some(sid) = state.blaze_session_id {
                                crate::session::blaze_sessions::mark_authenticated(sid);
                            } else {
                                warn!(
                                    "[Blaze] createAccount complete but blaze_session_id missing — Sessions UI will not show authenticated"
                                );
                            }

                            // Skip normal response sending for createAccount since we already sent it
                            let _ = buffer.split_to(total_packet_size);
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Packet processed and removed from buffer, remaining buffer size: {}", buffer.len());
                            continue; // Continue loop to wait for next packet
                        }

                        // Send response - use Reply message type (for non-createAccount commands)
                        let response_packet = Fire2FramePacket::new_send(
                            packet.header.component,
                            packet.header.command,
                            packet.header.msg_num, // Use original msg_num
                            MessageType::Reply,
                            response_payload,
                        );
                        // Encrypt response if crypto enabled
                        let mut response_data = response_packet.to_bytes().to_vec();
                        if state.crypto_enabled && !response_data.is_empty() {
                            if let Err(e) = state.c_out.encrypt(&mut response_data) {
                                error!("Encryption error: {}", e);
                                crate::debug_println!("\x1b[38;2;255;150;150m[Blaze]\x1b[0m Encryption error: {}", e);
                                // Continue without encryption
                            } else {
                                crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Response encrypted successfully (size: {})", response_data.len());
                            }
                        } else {
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Crypto not enabled or empty response, skipping encryption");
                        }

                        // updateNetworkInfo: send REPLY and extended-data notification together.
                        if packet.header.component == 0x7802 && packet.header.command == 0x14 {
                            state.update_network_info_count += 1;
                            let call_count = state.update_network_info_count;
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m updateNetworkInfo detected (call #{}), sending REPLY + UserSessionExtendedDataUpdate", call_count);
                            let user_session_update_payload = match if call_count == 1 {
                                handle_user_session_extended_data_update_first(&[])
                            } else if call_count == 2 {
                                handle_user_session_extended_data_update_second(&[])
                            } else {
                                handle_user_session_extended_data_update_third(&[])
                            } {
                                Ok(payload) => payload,
                                Err(e) => {
                                    error!("UserSessionExtendedDataUpdate build failed: {}", e);
                                    Bytes::new()
                                }
                            };
                            if user_session_update_payload.is_empty() {
                                error!(
                                    "[Blaze] updateNetworkInfo call #{}: UserSessionExtendedDataUpdate payload empty; client may stall",
                                    call_count
                                );
                            }

                            let mut notif_payload_len: Option<usize> = None;
                            let optional_notification = if !user_session_update_payload.is_empty() {
                                let user_session_update_packet = Fire2FramePacket::new_send(
                                    0x7802,
                                    0x01,
                                    0,
                                    MessageType::Notification,
                                    user_session_update_payload,
                                );
                                notif_payload_len = Some(user_session_update_packet.payload.len());
                                let mut user_session_update_data =
                                    user_session_update_packet.to_bytes().to_vec();
                                if state.crypto_enabled && !user_session_update_data.is_empty() {
                                    if let Err(e) = state.c_out.encrypt(&mut user_session_update_data) {
                                        error!("Encryption error for UserSessionExtendedDataUpdate: {}", e);
                                    }
                                }
                                Some((user_session_update_packet, user_session_update_data))
                            } else {
                                None
                            };

                            capture_outgoing_packet(&response_packet, &response_data);
                            if let Some((ref p, ref d)) = optional_notification {
                                capture_outgoing_packet(p, d);
                            }

                            // Log before each write.
                            if let Some(cmd_name) = crate::blaze::components::get_command_name(packet.header.component, packet.header.command) {
                                let k = key_b2c_reply(
                                    packet.header.component,
                                    packet.header.command,
                                    response_packet.payload.len(),
                                );
                                let line = format!(
                                    "[Blaze→Client] {} Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                                    cmd_name,
                                    packet.header.component,
                                    packet.header.command,
                                    response_packet.payload.len(),
                                    "REPLY",
                                    packet.header.msg_num
                                );
                                info_coalesce.log(&k, line);
                            } else {
                                let k = key_b2c_reply(0x7802, 0x14, response_packet.payload.len());
                                let line = format!(
                                    "[Blaze→Client] UserSessions.updateNetworkInfo Component=30722, Command=20, Size={}, MsgType=REPLY, MsgNum={}",
                                    response_packet.payload.len(),
                                    packet.header.msg_num
                                );
                                info_coalesce.log(&k, line);
                            }
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m updateNetworkInfo REPLY (payload: {} bytes, total packet: {} bytes, MsgNum: {}, call #{})",
                                response_packet.payload.len(), response_data.len(), packet.header.msg_num, call_count);

                            if let Some((_, ref d)) = optional_notification {
                                if let Some(n) = notif_payload_len {
                                    let k = key_b2c_notif(0x7802, 0x01, n);
                                    let line = format!(
                                        "[Blaze→Client] UserSessionExtendedDataUpdate Component=30722, Command=1, Size={}, MsgType=NOTIFICATION, MsgNum=0 (call #{})",
                                        n,
                                        call_count
                                    );
                                    info_coalesce.log(&k, line);
                                }
                                if !blaze_send(
                                    &mut stream,
                                    &response_data,
                                    addr,
                                    name,
                                    "updateNetworkInfo REPLY",
                                )
                                .await?
                                {
                                    warn!(
                                        "[Blaze] [{}] peer {} | updateNetworkInfo REPLY send failed (peer closed)",
                                        name, addr
                                    );
                                    return Ok(());
                                }
                                if !blaze_send(
                                    &mut stream,
                                    d,
                                    addr,
                                    name,
                                    "UserSessionExtendedDataUpdate after updateNetworkInfo",
                                )
                                .await?
                                {
                                    warn!(
                                        "[Blaze] [{}] peer {} | UserSessionExtendedDataUpdate send failed (peer closed)",
                                        name, addr
                                    );
                                    return Ok(());
                                }
                            } else if !blaze_send(
                                &mut stream,
                                &response_data,
                                addr,
                                name,
                                "updateNetworkInfo REPLY (no notification)",
                            )
                            .await?
                            {
                                return Ok(());
                            }

                            let _ = buffer.split_to(total_packet_size);
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Packet processed and removed from buffer, remaining buffer size: {}", buffer.len());
                            
                            // Check if there are more packets in the buffer (client might have sent updateHardwareFlags already)
                            if buffer.len() >= Fire2FrameHeader::HEADER_SIZE {
                                // Try to peek at the next packet header to see what's coming
                                if let Ok(next_header) = Fire2FrameHeader::from_bytes(&buffer[..Fire2FrameHeader::HEADER_SIZE]) {
                                    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Additional data in buffer ({} bytes), next packet: Component={}, Command={}, MsgNum={}, continuing to process...", 
                                        buffer.len(), next_header.component, next_header.command, next_header.msg_num);
                                } else {
                                    crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Additional data in buffer ({} bytes), but couldn't parse next header, continuing to process...", buffer.len());
                                }
                                // Continue the while loop to process the next packet
                                continue;
                            }
                            
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m No more data in buffer, waiting for next packet from client...");
                            // Break out of the while loop to read more data from the stream
                            break;
                        } else {
                            // Normal response: write to client before inspector capture (avoid blocking on packet_buffer lock).
                            if !blaze_send(&mut stream, &response_data, addr, name, "REPLY").await? {
                                return Ok(());
                            }
                            capture_outgoing_packet(&response_packet, &response_data);
                            let log_size = response_packet.total_size();
                            let is_ping_reply = packet.header.component == 9 && packet.header.command == 2;
                            if is_ping_reply {
                                if let Some(cmd_name) = crate::blaze::components::get_command_name(packet.header.component, packet.header.command) {
                                    let line = format!(
                                        "[Blaze→Client] {} Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                                        cmd_name,
                                        packet.header.component,
                                        packet.header.command,
                                        log_size,
                                        "REPLY",
                                        packet.header.msg_num
                                    );
                                    ping_burst.log_reply(line);
                                } else {
                                    let line = format!(
                                        "[Blaze→Client] Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                                        packet.header.component,
                                        packet.header.command,
                                        log_size,
                                        "REPLY",
                                        packet.header.msg_num
                                    );
                                    ping_burst.log_reply(line);
                                }
                            } else {
                                ping_burst.flush();
                                if let Some(cmd_name) = crate::blaze::components::get_command_name(packet.header.component, packet.header.command) {
                                    let k = key_b2c_reply(
                                        packet.header.component,
                                        packet.header.command,
                                        log_size,
                                    );
                                    let line = format!(
                                        "[Blaze→Client] {} Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                                        cmd_name,
                                        packet.header.component,
                                        packet.header.command,
                                        log_size,
                                        "REPLY",
                                        packet.header.msg_num
                                    );
                                    info_coalesce.log(&k, line);
                                } else {
                                    let k = key_b2c_reply(
                                        packet.header.component,
                                        packet.header.command,
                                        log_size,
                                    );
                                    let line = format!(
                                        "[Blaze→Client] Component={}, Command={}, Size={}, MsgType={}, MsgNum={}",
                                        packet.header.component,
                                        packet.header.command,
                                        log_size,
                                        "REPLY",
                                        packet.header.msg_num
                                    );
                                    info_coalesce.log(&k, line);
                                }
                            }
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Normal response sent successfully (size: {})", response_data.len());
                        }

                        // Send SECOND UserSessionExtendedDataUpdate notification AFTER GameManager Command=3
                        if packet.header.component == 0x0004 && packet.header.command == 0x03 {
                            crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m GameManager Command=3 detected, sending second UserSessionExtendedDataUpdate notification");

                            // Send SECOND UserSessionExtendedDataUpdate notification (Component=30722, Command=1)
                            let user_session_update_payload =
                                handle_user_session_extended_data_update(&[])?;
                            let user_session_update_packet = Fire2FramePacket::new_send(
                                0x7802, // UserSessions component (30722)
                                0x01,   // UserSessionExtendedDataUpdate command
                                0,      // PacketID=0 for notifications
                                MessageType::Notification,
                                user_session_update_payload,
                            );

                            let mut user_session_update_data =
                                user_session_update_packet.to_bytes().to_vec();
                            if state.crypto_enabled && !user_session_update_data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut user_session_update_data) {
                                    error!("Encryption error for SECOND UserSessionExtendedDataUpdate: {}", e);
                                }
                            }

                            {
                                let pl = user_session_update_packet.payload.len();
                                let line = format!(
                                    "[Blaze→Client] UserSessionExtendedDataUpdate Component=30722, Command=1, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                                    pl
                                );
                                info_coalesce.log(&key_b2c_notif(0x7802, 0x01, pl), line);
                            }

                            capture_outgoing_packet(&user_session_update_packet, &user_session_update_data);
                            if !blaze_write_only(&mut stream, &user_session_update_data, addr, name, "UserSessionExtendedDataUpdate after GameManager cmd 3").await? {
                                return Ok(());
                            }

                            // UserSessions Command=8 is a client-initiated request, not a server-initiated response
                        }

                        // CNC GameManager async notifications after `joinGame` and `resetDedicatedServer`.
                        if packet.header.component == 0x0004
                            && (packet.header.command == 0x0009
                                || packet.header.command == 0x0019
                                || packet.header.command == 0x0016)
                        {
                            let active_game = crate::common::game::get_current_game_id();
                            crate::debug_println!(
                                "\x1b[38;2;255;215;0m[CNC]\x1b[0m GameManager post-reply notify dispatch hit (game={}, cmd=0x{:04X})",
                                active_game,
                                packet.header.command
                            );
                            if active_game != "cnc" {
                                crate::debug_println!(
                                    "\x1b[38;2;255;200;100m[CNC]\x1b[0m skip post-reply notifications: current game is '{}', not 'cnc'",
                                    active_game
                                );
                            }
                        }

                        if crate::common::game::get_current_game_id() == "cnc"
                            && packet.header.component == 0x0004
                            && (packet.header.command == 0x0009
                                || packet.header.command == 0x0019
                                || packet.header.command == 0x0016)
                        {
                            let is_join = packet.header.command == 0x0009;
                            let gid = if is_join {
                                crate::client::cnc::cnc_extract_join_game_id(&packet.payload)
                            } else {
                                crate::client::cnc::cnc_extract_reset_game_id(&packet.payload)
                            };
                            let flow_label = if is_join { "joinGame" } else { "resetDedicatedServer" };
                            crate::debug_println!(
                                "\x1b[38;2;255;215;0m[CNC]\x1b[0m pushing NotifyGameStateChange + NotifyGameSetup + NotifyPlatformHostInitialized after {} (gid={})",
                                flow_label, gid
                            );

                            let gstate_payload = match crate::client::cnc::build_game_manager_notify_game_state_change(
                                gid,
                                crate::client::cnc::GSTA_RESETABLE,
                            ) {
                                Ok(p) => p,
                                Err(e) => {
                                    crate::debug_println!(
                                        "\x1b[38;2;255;100;100m[CNC]\x1b[0m NotifyGameStateChange encode failed: {:?}",
                                        e
                                    );
                                    return Err(e);
                                }
                            };
                            let gstate_packet = Fire2FramePacket::new_send(
                                0x0004,
                                0x64,
                                0,
                                MessageType::Notification,
                                gstate_payload.clone(),
                            );
                            let mut gstate_data = gstate_packet.to_bytes().to_vec();
                            if state.crypto_enabled && !gstate_data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut gstate_data) {
                                    error!(
                                        "Encryption error for NotifyGameStateChange: {}",
                                        e
                                    );
                                }
                            }
                            {
                                let pl = gstate_packet.payload.len();
                                let line = format!(
                                    "[Blaze→Client] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                                    pl
                                );
                                info_coalesce.log(&key_b2c_notif(0x0004, 0x64, pl), line);
                            }
                            capture_outgoing_packet(&gstate_packet, &gstate_data);

                            let setup_payload = match if is_join {
                                crate::client::cnc::build_game_manager_notify_game_setup_join(gid)
                            } else {
                                crate::client::cnc::build_game_manager_notify_game_setup(
                                    &packet.payload,
                                    gid,
                                )
                            } {
                                Ok(p) => p,
                                Err(e) => {
                                    crate::debug_println!(
                                        "\x1b[38;2;255;100;100m[CNC]\x1b[0m NotifyGameSetup encode failed: {:?}",
                                        e
                                    );
                                    return Err(e);
                                }
                            };
                            let setup_packet = Fire2FramePacket::new_send(
                                0x0004,
                                0x14,
                                0,
                                MessageType::Notification,
                                setup_payload.clone(),
                            );
                            let mut setup_data = setup_packet.to_bytes().to_vec();
                            if state.crypto_enabled && !setup_data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut setup_data) {
                                    error!(
                                        "Encryption error for NotifyGameSetup: {}",
                                        e
                                    );
                                }
                            }
                            {
                                let pl = setup_packet.payload.len();
                                let line = format!(
                                    "[Blaze→Client] GameManager.NotifyGameSetup Component=4, Command=20, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                                    pl
                                );
                                info_coalesce.log(&key_b2c_notif(0x0004, 0x14, pl), line);
                            }
                            capture_outgoing_packet(&setup_packet, &setup_data);
                            if !blaze_write_only(
                                &mut stream,
                                &setup_data,
                                addr,
                                name,
                                if is_join {
                                    "NotifyGameSetup after joinGame"
                                } else {
                                    "NotifyGameSetup after resetDedicatedServer"
                                },
                            )
                            .await?
                            {
                                return Ok(());
                            }

                            // For join flow, create local game via setup before state-change notify.
                            sleep(Duration::from_millis(15)).await;

                            if !blaze_write_only(
                                &mut stream,
                                &gstate_data,
                                addr,
                                name,
                                if is_join {
                                    "NotifyGameStateChange after joinGame"
                                } else {
                                    "NotifyGameStateChange after resetDedicatedServer"
                                },
                            )
                            .await?
                            {
                                return Ok(());
                            }

                            // `Game` is inserted in `onNotifyGameSetup` (`createLocalGame`); defer platform host until then.
                            sleep(Duration::from_millis(15)).await;

                            let phost_payload = match crate::client::cnc::build_game_manager_notify_platform_host_initialized(
                                gid,
                            ) {
                                Ok(p) => p,
                                Err(e) => {
                                    crate::debug_println!(
                                        "\x1b[38;2;255;100;100m[CNC]\x1b[0m NotifyPlatformHostInitialized encode failed: {:?}",
                                        e
                                    );
                                    return Err(e);
                                }
                            };
                            let phost_packet = Fire2FramePacket::new_send(
                                0x0004,
                                0x47,
                                0,
                                MessageType::Notification,
                                phost_payload,
                            );
                            let mut phost_data = phost_packet.to_bytes().to_vec();
                            if state.crypto_enabled && !phost_data.is_empty() {
                                if let Err(e) = state.c_out.encrypt(&mut phost_data) {
                                    error!(
                                        "Encryption error for NotifyPlatformHostInitialized: {}",
                                        e
                                    );
                                }
                            }
                            {
                                let pl = phost_packet.payload.len();
                                let line = format!(
                                    "[Blaze→Client] GameManager.NotifyPlatformHostInitialized Component=4, Command=71, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                                    pl
                                );
                                info_coalesce.log(&key_b2c_notif(0x0004, 0x47, pl), line);
                            }
                            capture_outgoing_packet(&phost_packet, &phost_data);
                            if !blaze_write_only(
                                &mut stream,
                                &phost_data,
                                addr,
                                name,
                                if is_join {
                                    "NotifyPlatformHostInitialized after joinGame"
                                } else {
                                    "NotifyPlatformHostInitialized after resetDedicatedServer"
                                },
                            )
                            .await?
                            {
                                return Ok(());
                            }
                        }
                    }
                    Ok(Err(BlazeError::ConnectionClosed)) => {
                        info!("Client {} sent disconnect signal", addr);
                        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Connection closed signal received from {}", addr);
                        return Ok(());
                    }
                    Ok(Err(e)) => {
                        crate::debug_println!("\x1b[38;2;255;150;150m[Blaze]\x1b[0m handle_packet returned error: {:?}", e);
                        // Use comprehensive error handling system
                        use crate::blaze::errors::{create_error_response, get_error_name, log_error};
                        
                        // Get error code from BlazeError
                        let error_code = e.to_error_code();
                        
                        // Log the error with component/command context
                        log_error(packet.header.component, error_code, packet.header.command);
                        
                        // Create error response payload (TDF format: CNTX + ERRC)
                        let error_payload = create_error_response(packet.header.component, error_code);
                        
                        // Create ERROR_REPLY packet
                        let error_packet = Fire2FramePacket::new_send(
                            packet.header.component,
                            packet.header.command,
                            packet.header.msg_num,
                            MessageType::ErrorReply,
                            Bytes::from(error_payload),
                        );
                        
                        // Encrypt error response if crypto enabled
                        let mut error_data = error_packet.to_bytes().to_vec();
                        if state.crypto_enabled && !error_data.is_empty() {
                            if let Err(enc_err) = state.c_out.encrypt(&mut error_data) {
                                error!("Encryption error for error reply: {}", enc_err);
                            }
                        }
                        
                        // Log error reply with error name
                        let error_name_opt = get_error_name(packet.header.component, error_code);
                        let cmd_name = crate::blaze::components::get_command_name(packet.header.component, packet.header.command)
                            .unwrap_or_else(|| format!("{}.Command({})", 
                                crate::blaze::components::get_component_name(packet.header.component), 
                                packet.header.command));
                        
                        if let Some(error_name) = error_name_opt {
                            // Known error - log normally
                            crate::console_println!(
                                "\x1b[38;2;255;100;100m[Blaze\x1b[0m→\x1b[38;2;255;100;100mClient]\x1b[0m ERROR_REPLY {} -> {} (Code: {}), Size={}, MsgNum={}",
                                cmd_name,
                                error_name,
                                error_code,
                                error_packet.total_size(),
                                packet.header.msg_num
                            );
                        } else {
                            // Unknown error - log with special marker
                            let component_name = crate::blaze::components::get_component_name(packet.header.component);
                            crate::console_println!(
                                "\x1b[38;2;255;165;0m[⚠️  UNKNOWN ERROR_REPLY]\x1b[0m {} -> UNKNOWN_ERROR Component={} ({}), Command={}, ErrorCode={} (⚠️  Please investigate and add to blaze_errors.rs), Size={}, MsgNum={}",
                                cmd_name,
                                packet.header.component,
                                component_name,
                                packet.header.command,
                                error_code,
                                error_packet.total_size(),
                                packet.header.msg_num
                            );
                        }
                        
                        // Capture error reply for inspection
                        capture_outgoing_packet(&error_packet, &error_data);
                        
                        // Send error reply
                        if !blaze_write_only(&mut stream, &error_data, addr, name, "ERROR_REPLY").await? {
                            return Ok(());
                        }
                    }
                    Err(join_err) => {
                        error!("[Blaze] handle_packet task join error: {}", join_err);
                        return Err(BlazeError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            join_err.to_string(),
                        )));
                    }
                }

                        // Remove processed packet from buffer
                        let _ = buffer.split_to(total_packet_size);
                        crate::debug_println!("\x1b[38;2;150;150;255m[Blaze]\x1b[0m Packet processed and removed from buffer, remaining buffer size: {}", buffer.len());
            }
                }
            }
        }

        Ok(())
    }

    async fn handle_blaze_protocol_fireframe(
        mut stream: impl AsyncReadExt + AsyncWriteExt + Unpin,
        addr: SocketAddr,
        name: &str,
        state: &mut SessionState,
    ) -> BlazeResult<()> {
        let mut buffer = BytesMut::new();
        let scoped_key = format!("BLAZE_FIRE|{}", addr);
        let mut info_coalesce = CoalescedBlazeInfo::new_scoped(&scoped_key);
        let mut ping_burst = PingBurstCoalescer::new_scoped(&scoped_key);
        loop {
            let mut chunk = vec![0u8; 4096];
            let n = match timeout(Duration::from_secs(15), stream.read(&mut chunk)).await {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => {
                    if io_is_expected_peer_close(&e) {
                        return Ok(());
                    }
                    return Err(BlazeError::Io(e));
                }
                Err(_) => continue,
            };
            if n == 0 {
                return Ok(());
            }
            buffer.extend_from_slice(&chunk[..n]);

            // Legacy FireFrame envelope:
            // Size(u16) + Component(u16) + Command(u16) + Error(u16) + QType(u16) + PacketId(u16)
            while buffer.len() >= 12 {
                let payload_size = u16::from_be_bytes([buffer[0], buffer[1]]) as usize;
                if payload_size > 10000 {
                    let _ = buffer.split_to(1);
                    continue;
                }
                let total_packet_size = 12 + payload_size;
                if buffer.len() < total_packet_size {
                    break;
                }

                let component = u16::from_be_bytes([buffer[2], buffer[3]]);
                let command = u16::from_be_bytes([buffer[4], buffer[5]]);
                let packet_id = u16::from_be_bytes([buffer[10], buffer[11]]);
                let payload = buffer[12..total_packet_size].to_vec();
                let cmd_name = crate::blaze::components::get_command_name(component, command)
                    .map(|s| s.to_string());
                let is_ping_req = component == 9 && command == 2;
                if is_ping_req {
                    if let Some(name) = &cmd_name {
                        let line = format!(
                            "[Client→Blaze] {} Component={}, Command={}, Size={}, MsgType=REQUEST, MsgNum={}",
                            name, component, command, total_packet_size, packet_id
                        );
                        ping_burst.log_request(line);
                    } else {
                        let line = format!(
                            "[Client→Blaze] Component={}, Command={}, Size={}, MsgType=REQUEST, MsgNum={}",
                            component, command, total_packet_size, packet_id
                        );
                        ping_burst.log_request(line);
                    }
                } else {
                    ping_burst.flush();
                    let c2b_k = key_fire_c2b(component, command, total_packet_size);
                    if let Some(name) = &cmd_name {
                        let line = format!(
                            "[Client→Blaze] {} Component={}, Command={}, Size={}, MsgType=REQUEST, MsgNum={}",
                            name, component, command, total_packet_size, packet_id
                        );
                        info_coalesce.log(&c2b_k, line);
                    } else {
                        let line = format!(
                            "[Client→Blaze] Component={}, Command={}, Size={}, MsgType=REQUEST, MsgNum={}",
                            component, command, total_packet_size, packet_id
                        );
                        info_coalesce.log(&c2b_k, line);
                    }
                }
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                capture_packet(CapturedPacket {
                    timestamp,
                    direction: PacketDirection::ClientToBlaze,
                    component,
                    command,
                    msg_num: packet_id as u32,
                    msg_type: "REQUEST".to_string(),
                    payload_size: payload.len(),
                    payload: payload.clone(),
                    raw_packet: buffer[..total_packet_size].to_vec(),
                    command_name: cmd_name.clone(),
                    metadata_size: 0,
                });

                let response_payload = handle_packet_fields(component, command, &payload)?;

                let mut response_data = Vec::with_capacity(12 + response_payload.len());
                let response_size = response_payload.len() as u16;
                response_data.extend_from_slice(&response_size.to_be_bytes());
                response_data.extend_from_slice(&component.to_be_bytes());
                response_data.extend_from_slice(&command.to_be_bytes());
                response_data.extend_from_slice(&0u16.to_be_bytes()); // error
                response_data.extend_from_slice(&0x1000u16.to_be_bytes()); // reply qtype
                response_data.extend_from_slice(&packet_id.to_be_bytes());
                response_data.extend_from_slice(&response_payload);
                capture_packet(CapturedPacket {
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs_f64(),
                    direction: PacketDirection::BlazeToClient,
                    component,
                    command,
                    msg_num: packet_id as u32,
                    msg_type: "REPLY".to_string(),
                    payload_size: response_payload.len(),
                    payload: response_payload.to_vec(),
                    raw_packet: response_data.clone(),
                    command_name: cmd_name.clone(),
                    metadata_size: 0,
                });
                let b2c_k = key_fire_b2c(component, command, response_data.len());
                let is_ping_reply = component == 9 && command == 2;
                if is_ping_reply {
                    if let Some(name) = &cmd_name {
                        let line = format!(
                            "[Blaze→Client] {} Component={}, Command={}, Size={}, MsgType=REPLY, MsgNum={}",
                            name, component, command, response_data.len(), packet_id
                        );
                        ping_burst.log_reply(line);
                    } else {
                        let line = format!(
                            "[Blaze→Client] Component={}, Command={}, Size={}, MsgType=REPLY, MsgNum={}",
                            component, command, response_data.len(), packet_id
                        );
                        ping_burst.log_reply(line);
                    }
                } else {
                    ping_burst.flush();
                    if let Some(name) = &cmd_name {
                        let line = format!(
                            "[Blaze→Client] {} Component={}, Command={}, Size={}, MsgType=REPLY, MsgNum={}",
                            name, component, command, response_data.len(), packet_id
                        );
                        info_coalesce.log(&b2c_k, line);
                    } else {
                        let line = format!(
                            "[Blaze→Client] Component={}, Command={}, Size={}, MsgType=REPLY, MsgNum={}",
                            component, command, response_data.len(), packet_id
                        );
                        info_coalesce.log(&b2c_k, line);
                    }
                }

                if !blaze_send(&mut stream, &response_data, addr, name, "REPLY").await? {
                    return Ok(());
                }

                if crate::common::game::get_current_game_id() == "cnc"
                    && component == 0x0004
                    && (command == 0x0009 || command == 0x0019 || command == 0x0016)
                {
                    let pushes = if command == 0x0009 {
                        crate::client::cnc::fireframe::pushes_after_join_game(&payload)?
                    } else {
                        crate::client::cnc::fireframe::pushes_after_reset_dedicated_server(&payload)?
                    };
                    for (push_idx, push) in pushes.into_iter().enumerate() {
                        let ts = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs_f64();
                        capture_packet(CapturedPacket {
                            timestamp: ts,
                            direction: PacketDirection::BlazeToClient,
                            component: push.component,
                            command: push.command,
                            msg_num: 0,
                            msg_type: "NOTIFICATION".to_string(),
                            payload_size: push.tdf_body.len(),
                            payload: push.tdf_body.clone(),
                            raw_packet: push.wire.clone(),
                            command_name: crate::blaze::components::get_command_name(
                                push.component,
                                push.command,
                            )
                            .map(|s| s.to_string()),
                            metadata_size: 0,
                        });
                        if !blaze_send(
                            &mut stream,
                            &push.wire,
                            addr,
                            name,
                            push.blaze_send_label,
                        )
                        .await?
                        {
                            return Ok(());
                        }
                        let pl = push.wire.len();
                        info_coalesce.log(
                            &key_b2c_notif(push.component, push.command, pl),
                            push.info_log_line,
                        );
                        if push_idx < 2 {
                            sleep(Duration::from_millis(15)).await;
                        }
                    }
                }

                if crate::common::game::get_current_game_id() == "cnc"
                    && component == 0x0001
                    && command == 0x006e
                {
                    for push in crate::client::cnc::fireframe::pushes_after_login_persona()? {
                        capture_packet(CapturedPacket {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs_f64(),
                            direction: PacketDirection::BlazeToClient,
                            component: push.component,
                            command: push.command,
                            msg_num: 0,
                            msg_type: "NOTIFICATION".to_string(),
                            payload_size: push.tdf_body.len(),
                            payload: push.tdf_body.clone(),
                            raw_packet: push.wire.clone(),
                            command_name: crate::blaze::components::get_command_name(
                                push.component,
                                push.command,
                            )
                            .map(|s| s.to_string()),
                            metadata_size: 0,
                        });

                        if !blaze_send(
                            &mut stream,
                            &push.wire,
                            addr,
                            name,
                            push.blaze_send_label,
                        )
                        .await?
                        {
                            return Ok(());
                        }
                        let pl = push.wire.len();
                        info_coalesce.log(
                            &key_b2c_notif(push.component, push.command, pl),
                            push.info_log_line,
                        );
                    }

                    if let Some(sid) = state.blaze_session_id {
                        crate::session::blaze_sessions::mark_authenticated(sid);
                    }
                }

                let _ = buffer.split_to(total_packet_size);
            }
        }
    }

    async fn handle_gosredirector_blaze<S: AsyncReadExt + AsyncWriteExt + Unpin>(
        mut stream: S,
        addr: SocketAddr,
        protocol: &str,
    ) -> BlazeResult<()> {
        info!(
            "\x1b[38;2;150;150;255m[GOS]\x1b[0m handling request from {} ({})",
            addr, protocol
        );
        let mut buffer = vec![0u8; 4096];
        let n = match timeout(Duration::from_secs(5), stream.read(&mut buffer)).await {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => return Err(BlazeError::Io(e)),
            Err(_) => {
                return Err(BlazeError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "gosredirector read timeout",
                )))
            }
        };
        if n == 0 {
            return Ok(());
        }
        let mut acc = buffer[..n].to_vec();
        if find_get_server_instance(protocol, &acc).is_none() && n < 4096 {
            let n2 = match timeout(Duration::from_millis(500), stream.read(&mut buffer)).await {
                Ok(Ok(n2)) => n2,
                _ => 0,
            };
            if n2 > 0 {
                acc.extend_from_slice(&buffer[..n2]);
            }
        }
        let received = acc.as_slice();
        let frame = if let Some(f) = find_get_server_instance(protocol, received) {
            f
        } else {
            warn!(
                "\x1b[38;2;150;150;255m[GOS]\x1b[0m could not decode getServerInstance ({} bytes, {}), using fallback",
                received.len(),
                protocol
            );
            if protocol.eq_ignore_ascii_case("fireframe") {
                RedirectorWire::FireFrame { packet_id: 0 }
            } else {
                RedirectorWire::Fire2Frame { msg_num: 0 }
            }
        };

        match frame {
            RedirectorWire::FireFrame { packet_id } => {
                info!(
                    "\x1b[38;2;150;150;255m[GOS]\x1b[0m FireFrame getServerInstance packet_id={}",
                    packet_id
                );
            }
            RedirectorWire::Fire2Frame { msg_num } => {
                info!(
                    "\x1b[38;2;150;150;255m[GOS]\x1b[0m Fire2Frame getServerInstance msg_num={}",
                    msg_num
                );
            }
        }

        let response_payload = match crate::client::handle_redirector_get_server_instance(&[]) {
            Some(Ok(bytes)) => bytes,
            Some(Err(e)) => return Err(e),
            None => Bytes::new(),
        };
        let response_bytes = build_get_server_instance_reply(frame, response_payload)?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;
        info!(
            "\x1b[38;2;150;150;255m[GOS]\x1b[0m sent getServerInstance response to {}",
            addr
        );
        Ok(())
    }
}
