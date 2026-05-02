use crate::common::error::{BlazeError, BlazeResult};
use crate::http::handlers::{HttpHandler, HttpResponse};
use bytes::Bytes;
use h2::server::{self, SendResponse};
use http::header::{HeaderName, HeaderValue};
use http::{Request, Response};
use rustls::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::{server::TlsStream as ServerTlsStream, TlsAcceptor};
use tracing::{error, info};
use std::pin::Pin;
use std::task::{Context, Poll};

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
            if self.buffer_pos == self.buffer.len() && !self.buffer.is_empty() {
                // Log when we've finished providing buffered bytes
                error!("[TLS] BufferedStream: Provided {} buffered bytes to TLS library", self.buffer.len());
            }
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

/// HTTP Protocol Server - Handles HTTP/1.1 and HTTP/2 communications
pub struct HttpProtocolServer {
    host: String,
    ssl_context: Option<Arc<ServerConfig>>,
    http_handler: HttpHandler,
}

impl HttpProtocolServer {
    /// Create new HTTP protocol server
    pub fn new(host: String, ssl_context: Option<Arc<ServerConfig>>) -> Self {
        Self {
            host,
            ssl_context,
            http_handler: HttpHandler::new(),
        }
    }

    /// Get the ports this server will use
    pub fn get_ports() -> Vec<(u16, &'static str)> {
        vec![
            (80, "HTTP/1.1"),
            (443, "HTTP/2"),
        ]
    }

    /// Start HTTP/HTTPS protocol servers
    pub async fn start_http_servers(&self) -> BlazeResult<()> {
        // Port 80 - HTTP/1.1 only
        let host = self.host.clone();
        let http_handler = self.http_handler.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_http11_server(host, 80, http_handler).await {
                error!("HTTP/1.1 server error: {}", e);
            }
        });
        // HTTP/1.1 server started (logged by startup progress)

        // Port 443 - HTTP/2 support
        let host = self.host.clone();
        let ssl_context = self.ssl_context.clone();
        let http_handler = self.http_handler.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_http2_server(host, 443, ssl_context, http_handler).await {
                error!("HTTP/2 server error: {}", e);
            }
        });
        // HTTP/2 server started (logged by startup progress)

        Ok(())
    }

    /// Run HTTP/1.1 server
    async fn run_http11_server(
        host: String,
        port: u16,
        http_handler: HttpHandler,
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
            info!("HTTP/1.1 server listening on {}", addr);
        }

        loop {
            let (stream, addr) = listener.accept().await?;
            let http_handler = http_handler.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_http11_connection(stream, addr, http_handler).await {
                    error!("HTTP/1.1 connection error: {}", e);
                }
            });
        }
    }

    /// Run HTTP/2 server
    async fn run_http2_server(
        host: String,
        port: u16,
        ssl_context: Option<Arc<ServerConfig>>,
        http_handler: HttpHandler,
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
            info!("HTTP/2 server listening on {}", addr);
        }

        loop {
            let (stream, addr) = listener.accept().await?;
            let ssl_context = ssl_context.clone();
            let http_handler = http_handler.clone();
            tokio::spawn(async move {
                if let Some(config) = ssl_context {
                    let acceptor = TlsAcceptor::from(config);
                    match acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            let alpn_proto = tls_stream
                                .get_ref()
                                .1
                                .alpn_protocol()
                                .map(|p| p.to_vec());
                            
                            match alpn_proto.as_deref() {
                                Some(b"h2") => {
                                    if let Err(e) = Self::handle_http2_over_tls(
                                        tls_stream,
                                        addr,
                                        http_handler,
                                    )
                                    .await
                                    {
                                        error!("HTTP/2 over TLS error: {}", e);
                                    }
                                }
                                _ => {
                                    if let Err(e) =
                                        Self::handle_http11_tls_connection(tls_stream, addr, http_handler)
                                            .await
                                    {
                                        error!("HTTP/1.1 over TLS error: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("TLS handshake failed for {}: {}", addr, e);
                            
                            // Log detailed error information
                            let error_str = format!("{}", e);
                            let error_debug = format!("{:?}", e);
                            error!("[TLS] Error details: {}", error_debug);
                            
                            if error_str.contains("AlertReceived") {
                                error!("Client sent TLS alert - likely certificate validation failure");
                            } else if error_str.contains("CorruptMessage") {
                                error!("TLS message corruption detected - possible protocol mismatch");
                            } else if error_str.contains("InappropriateMessage") {
                                error!("TLS protocol error - client/server version mismatch?");
                            } else if error_str.contains("UnsupportedNameType") {
                                error!("TLS name type unsupported - certificate hostname issue?");
                            } else if error_str.contains("NoApplicationProtocol") {
                                error!("ALPN negotiation failed - no common protocol");
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
                            
                            error!("This is likely due to the client rejecting the self-signed certificate");
                            error!("The client may need to be configured to accept self-signed certificates");
                        }
                    }
                } else {
                    // Handle plain HTTP/2 connection
                    if let Err(e) = Self::handle_http2_plain(stream, addr, http_handler).await {
                        error!("Plain HTTP/2 error: {}", e);
                    }
                }
            });
        }
    }

    /// Handle HTTP/1.1 connection
    async fn handle_http11_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        http_handler: HttpHandler,
    ) -> BlazeResult<()> {
        info!("HTTP/1.1 connection from {}", addr);

        // Read HTTP request
        let mut buffer = vec![0; 4096];
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Ok(());
        }

        let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);

        // Parse HTTP request
        let (method, path, host, body) = Self::parse_http_request(&request_str)?;

        // Handle the request
        let response = http_handler.handle_request(&host, &path, &method, &body)?;

        // Send HTTP response
        let response_bytes = Self::format_http_response(&response);
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Handle HTTP/1.1 over TLS connection
    async fn handle_http11_tls_connection<S: AsyncReadExt + AsyncWriteExt + Unpin>(
        mut stream: ServerTlsStream<S>,
        addr: SocketAddr,
        http_handler: HttpHandler,
    ) -> BlazeResult<()> {
        // Try reading with a short timeout to handle cases where client sends data slightly after connecting
        let mut buffer = vec![0; 8192];
        let bytes_read = match tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            stream.read(&mut buffer)
        ).await {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => {
                error!("[HTTP-1.1-TLS] Read error from {}: {}", addr, e);
                return Err(BlazeError::Io(e));
            }
            Err(_) => {
                // Timeout - try reading again without timeout (client might be slow)
                match stream.read(&mut buffer).await {
                    Ok(n) => n,
                    Err(e) => {
                        error!("[HTTP-1.1-TLS] Read error from {}: {}", addr, e);
                        return Err(BlazeError::Io(e));
                    }
                }
            }
        };
        
        let final_bytes_read = if bytes_read == 0 {
            // Try one more read in case data arrives late
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            match stream.read(&mut buffer).await {
                Ok(n) if n > 0 => n,
                _ => {
                    return Ok(());
                }
            }
        } else {
            bytes_read
        };

        let request_str = String::from_utf8_lossy(&buffer[..final_bytes_read]);

        let (method, path, host, body) = match Self::parse_http_request(&request_str) {
            Ok(parsed) => parsed,
            Err(e) => {
                error!("[HTTP-1.1-TLS] Failed to parse request from {}: {}", addr, e);
                return Err(e);
            }
        };
        
        let response = match http_handler.handle_request(&host, &path, &method, &body) {
            Ok(r) => r,
            Err(e) => {
                error!("[HTTP-1.1-TLS] Handler error for {} {}: {}", method, path, e);
                return Err(e);
            }
        };
        let response_bytes = Self::format_http_response(&response);
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Handle HTTP/2 over TLS
    async fn handle_http2_over_tls<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        stream: ServerTlsStream<S>,
        addr: SocketAddr,
        http_handler: HttpHandler,
    ) -> BlazeResult<()> {
        Self::handle_http2_connection(stream, addr, http_handler).await
    }

    /// Handle plain HTTP/2
    async fn handle_http2_plain(
        stream: TcpStream,
        addr: SocketAddr,
        http_handler: HttpHandler,
    ) -> BlazeResult<()> {
        Self::handle_http2_connection(stream, addr, http_handler).await
    }

    /// Generic HTTP/2 handler using h2 crate
    async fn handle_http2_connection<S>(
        stream: S,
        _addr: SocketAddr,
        http_handler: HttpHandler,
    ) -> BlazeResult<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let mut connection = server::handshake(stream).await?;
        HttpHandler::flush_grpc_compact_log_on_new_http2_connection();

        while let Some(request) = connection.accept().await {
            match request {
                Ok((req, respond)) => {
                    let handler = http_handler.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::process_http2_request(req, respond, handler).await {
                            error!("HTTP/2 request handling error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("HTTP/2 accept error: {}", e);
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    async fn process_http2_request(
        mut request: Request<h2::RecvStream>,
        mut respond: SendResponse<Bytes>,
        http_handler: HttpHandler,
    ) -> BlazeResult<()> {
        let method = request.method().as_str().to_string();
        let path = request
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str().to_string())
            .unwrap_or_else(|| "/".to_string());

        let host = request
            .headers()
            .get("host")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| request.uri().authority().map(|a| a.as_str().to_string()))
            .unwrap_or_default();

        let mut body_bytes = Vec::new();
        while let Some(chunk) = request.body_mut().data().await {
            let data = chunk?;
            body_bytes.extend_from_slice(&data);
        }

        let response = http_handler.handle_request(&host, &path, &method, &body_bytes)?;
        let HttpResponse {
            status_code,
            content_type,
            body,
            headers,
        } = response;
        let has_body = !body.is_empty();

        let mut response_builder = Response::builder().status(status_code);
        response_builder = response_builder.header("content-type", content_type.as_str());
        response_builder = response_builder.header("content-length", body.len().to_string());

        let mut trailer_headers: Vec<(HeaderName, HeaderValue)> = Vec::new();
        let mut has_grpc_status = false;

        for (key, value) in &headers {
            if key.eq_ignore_ascii_case("grpc-status") || key.eq_ignore_ascii_case("grpc-message") {
                let header_name = HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| crate::common::error::BlazeError::Http2(e.to_string()))?;
                if key.eq_ignore_ascii_case("grpc-status") {
                    has_grpc_status = true;
                }
                let header_value = HeaderValue::from_str(value)
                    .map_err(|e| crate::common::error::BlazeError::Http2(e.to_string()))?;
                trailer_headers.push((header_name, header_value));
            } else {
                response_builder = response_builder.header(key, value);
            }
        }
        let needs_trailers =
            !trailer_headers.is_empty() || content_type.contains("application/grpc");

        let response_head = response_builder
            .body(())
            .map_err(|e| crate::common::error::BlazeError::Http2(e.to_string()))?;

        let send_end_stream = !has_body && !needs_trailers;
        let mut send_stream = respond.send_response(response_head, send_end_stream)?;

        if has_body {
            let end_stream_after_body = !needs_trailers;
            send_stream.send_data(Bytes::from(body), end_stream_after_body)?;
        }

        if needs_trailers {
            if !has_grpc_status && content_type.contains("application/grpc") {
                trailer_headers.push((
                    HeaderName::from_static("grpc-status"),
                    HeaderValue::from_static("0"),
                ));
            }

            let mut trailers = http::HeaderMap::new();
            for (name, value) in trailer_headers {
                trailers.append(name, value);
            }
            send_stream.send_trailers(trailers)?;
        }

        Ok(())
    }

    /// Parse HTTP request
    fn parse_http_request(request_str: &str) -> BlazeResult<(String, String, String, Vec<u8>)> {
        let lines: Vec<&str> = request_str.lines().collect();
        if lines.is_empty() {
            return Err(crate::common::error::BlazeError::InvalidPacket(
                "Empty request".to_string(),
            ));
        }

        // Parse request line
        let request_line = lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(crate::common::error::BlazeError::InvalidPacket(
                "Invalid request line".to_string(),
            ));
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();

        // Parse headers
        let mut host = String::new();
        let mut content_length = 0;

        for line in &lines[1..] {
            if line.is_empty() {
                break; // End of headers
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                match key.as_str() {
                    "host" => host = value.to_string(),
                    "content-length" => {
                        if let Ok(len) = value.parse::<usize>() {
                            content_length = len;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Extract body if present
        let body_start = request_str.find("\r\n\r\n").unwrap_or(request_str.len()) + 4;
        let body = if body_start < request_str.len() && content_length > 0 {
            request_str[body_start..].as_bytes().to_vec()
        } else {
            Vec::new()
        };

        Ok((method, path, host, body))
    }

    /// Format HTTP response
    fn format_http_response(response: &HttpResponse) -> Vec<u8> {
        let reason = match response.status_code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "OK",
        };

        let mut header = format!("HTTP/1.1 {} {}\r\n", response.status_code, reason);
        header.push_str(&format!("Content-Type: {}\r\n", response.content_type));
        header.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
        header.push_str("Connection: close\r\n");
        header.push_str("Accept-Ranges: bytes\r\n");

        if !response.headers.contains_key("Server") {
            header.push_str("Server: Refracted/1.0\r\n");
        }

        for (key, value) in &response.headers {
            header.push_str(&format!("{}: {}\r\n", key, value));
        }

        header.push_str("\r\n");

        let mut out = header.into_bytes();
        out.extend_from_slice(&response.body);
        out
    }
}
