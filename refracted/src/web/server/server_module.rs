use crate::common::error::{BlazeError, BlazeResult};
use crate::http::handlers::{HttpHandler, HttpResponse};
use bytes::Bytes;
use h2::server::{self, SendResponse};
use http::header::{HeaderName, HeaderValue};
use http::{Request, Response};
use parking_lot::Mutex as WebConnLogMutex;
use rustls::ServerConfig;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::{server::TlsStream as ServerTlsStream, TlsAcceptor};
use tracing::{error, info};

struct WebConnectionLogEntry {
    count: u32,
}

struct WebConnectionLogState {
    entries: HashMap<String, WebConnectionLogEntry>,
}

fn web_connection_log_state() -> &'static WebConnLogMutex<WebConnectionLogState> {
    static STATE: OnceLock<WebConnLogMutex<WebConnectionLogState>> = OnceLock::new();
    STATE.get_or_init(|| {
        WebConnLogMutex::new(WebConnectionLogState {
            entries: HashMap::new(),
        })
    })
}

/// One Shell row per listener + mode; increments `xN` like gRPC compact logs.
fn log_web_connection_compact(upsert_key: String, line_without_count: &str) {
    let count = {
        let mut state = web_connection_log_state().lock();
        let e = state
            .entries
            .entry(upsert_key.clone())
            .or_insert(WebConnectionLogEntry { count: 0 });
        e.count = e.count.saturating_add(1);
        e.count
    };
    let ansi = format!(
        "\x1b[38;2;100;200;255m[Web]\x1b[0m {} \x1b[38;2;140;140;140mx{}\x1b[0m",
        line_without_count,
        count.max(1)
    );
    crate::core::console::push_grpc_compact_upsert(upsert_key, &ansi);
}

/// Web Protocol Server - Handles HTTP requests for serving www content
/// This is a fully-fledged web server for titles that require serving web content
/// Consolidates HTTP (ports 80/443) and Web (ports 8080/8443) services
pub struct WebProtocolServer {
    host: String,
    ssl_context: Option<Arc<ServerConfig>>,
    http_handler: HttpHandler,
}

impl WebProtocolServer {
    /// Create new Web protocol server
    pub fn new(host: String, ssl_context: Option<Arc<ServerConfig>>) -> Self {
        Self {
            host,
            ssl_context,
            http_handler: HttpHandler::new(),
        }
    }

    pub fn ports_from_config(p: &crate::common::game::ServicePorts) -> Vec<(u16, String)> {
        vec![
            (p.web_http, "Web HTTP".into()),
            (p.web_https, "Web HTTPS".into()),
            (p.web_http_alt, "Web HTTP Alt".into()),
            (p.web_https_alt, "Web HTTPS Alt".into()),
        ]
    }

    /// Start Web protocol servers (ports from active title in `games.json`).
    pub async fn start_web_servers(&self, ports: &crate::common::game::ServicePorts) -> BlazeResult<()> {
        let web_http = ports.web_http;
        let host = self.host.clone();
        let http_handler = self.http_handler.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_http11_server(host, web_http, http_handler).await {
                error!("Web HTTP/1.1 server error: {}", e);
            }
        });

        let web_https = ports.web_https;
        let host = self.host.clone();
        let ssl_context = self.ssl_context.clone();
        let http_handler = self.http_handler.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_http2_server(host, web_https, ssl_context, http_handler).await {
                error!("Web HTTP/2 server error: {}", e);
            }
        });

        let web_http_alt = ports.web_http_alt;
        let host = self.host.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_web_http_server(host, web_http_alt).await {
                error!("Web HTTP server (alt) error: {}", e);
            }
        });

        let web_https_alt = ports.web_https_alt;
        let host = self.host.clone();
        let ssl_context = self.ssl_context.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_web_https_server(host, web_https_alt, ssl_context).await {
                error!("Web HTTPS server (alt) error: {}", e);
            }
        });

        Ok(())
    }

    /// Run Web HTTP server
    async fn run_web_http_server(
        host: String,
        port: u16,
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
            info!("Web HTTP server listening on {}", addr);
        }

        loop {
            let (stream, addr) = listener.accept().await?;
            crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m New HTTP connection accepted on port {} from {}", port, addr);
            tokio::spawn(async move {
                if let Err(e) = Self::handle_web_connection(stream, addr, false, port).await {
                    error!("Web HTTP connection error: {}", e);
                }
            });
        }
    }

    /// Run Web HTTPS server
    async fn run_web_https_server(
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
            info!("Web HTTPS server listening on {}", addr);
        }

        let ssl_context = ssl_context.ok_or_else(|| {
            BlazeError::InvalidPacket("SSL context required for HTTPS server".to_string())
        })?;

        loop {
            let (stream, addr) = listener.accept().await?;
            crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m New HTTPS connection accepted on port {} from {}", port, addr);
            let ssl_context = ssl_context.clone();
            tokio::spawn(async move {
                let acceptor = TlsAcceptor::from(ssl_context);
                crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Initiating TLS handshake for {} from {}", port, addr);
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m TLS handshake successful for {} from {}", port, addr);
                        if let Err(e) = Self::handle_web_connection(tls_stream, addr, true, port).await {
                            error!("Web HTTPS connection error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("TLS handshake failed for Web HTTPS: {}", e);
                        crate::debug_println!("\x1b[38;2;255;150;150m[Web]\x1b[0m TLS handshake failed for {} from {}: {}", port, addr, e);
                    }
                }
            });
        }
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
            info!("Web HTTP/1.1 server listening on {}", addr);
        }

        loop {
            let (stream, addr) = listener.accept().await?;
            crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m New HTTP/1.1 connection accepted on port {} from {}", port, addr);
            let http_handler = http_handler.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_http11_connection(stream, addr, port, http_handler)
                    .await
                {
                    error!("Web HTTP/1.1 connection error: {}", e);
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
            info!("Web HTTP/2 server listening on {}", addr);
        }

        let ssl_context = ssl_context.ok_or_else(|| {
            BlazeError::InvalidPacket("SSL context required for HTTPS server".to_string())
        })?;

        loop {
            let (stream, addr) = listener.accept().await?;
            crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m New HTTP/2 connection accepted on port {} from {}", port, addr);
            let ssl_context = ssl_context.clone();
            let http_handler = http_handler.clone();
            tokio::spawn(async move {
                let acceptor = TlsAcceptor::from(ssl_context);
                crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Initiating TLS handshake for HTTP/2 from {}", addr);
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let alpn_proto = tls_stream
                            .get_ref()
                            .1
                            .alpn_protocol()
                            .map(|p| p.to_vec());
                        
                        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m TLS handshake successful, ALPN protocol: {:?}", alpn_proto);
                        match alpn_proto.as_deref() {
                            Some(b"h2") => {
                                crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Routing to HTTP/2 handler for {}", addr);
                                if let Err(e) = Self::handle_http2_over_tls(
                                    tls_stream,
                                    addr,
                                    http_handler,
                                )
                                .await
                                {
                                    error!("Web HTTP/2 over TLS error: {}", e);
                                }
                            }
                            _ => {
                                crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Routing to HTTP/1.1 over TLS handler for {}", addr);
                                if let Err(e) =
                                    Self::handle_http11_tls_connection(tls_stream, addr, http_handler)
                                        .await
                                {
                                    error!("Web HTTP/1.1 over TLS error: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("TLS handshake failed for {}: {}", addr, e);
                        crate::debug_println!("\x1b[38;2;255;150;150m[Web]\x1b[0m TLS handshake failed for {}: {}", addr, e);
                    }
                }
            });
        }
    }

    /// Handle HTTP/1.1 connection
    async fn handle_http11_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        listen_port: u16,
        http_handler: HttpHandler,
    ) -> BlazeResult<()> {
        log_web_connection_compact(
            format!("web:conn:http11:{listen_port}"),
            &format!("HTTP/1.1 on :{listen_port}"),
        );
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m handle_http11_connection entered for {}", addr);

        // Read HTTP request
        let mut buffer = vec![0; 4096];
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Waiting to read HTTP request from {}", addr);
        let bytes_read = stream.read(&mut buffer).await?;
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Read {} bytes from {}", bytes_read, addr);
        if bytes_read == 0 {
            crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Connection closed by client {}", addr);
            return Ok(());
        }

        let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Request preview: {}", request_str.lines().next().unwrap_or(""));

        // Parse HTTP request
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Parsing HTTP request");
        let (method, path, host, body) = Self::parse_http_request(&request_str)?;
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Parsed request - Method: {}, Path: {}, Host: {}, Body size: {}", method, path, host, body.len());

        // Handle the request
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Routing request to handler");
        let response = http_handler.handle_request(&host, &path, &method, &body)?;
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Handler returned response (status: {}, body size: {})", response.status_code, response.body.len());

        // Send HTTP response
        let response_bytes = Self::format_http_response(&response);
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Web]\x1b[0m Sending HTTP response (size: {})",
            response_bytes.len()
        );
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Response sent successfully to {}", addr);

        Ok(())
    }

    /// Handle HTTP/1.1 over TLS connection
    async fn handle_http11_tls_connection<S: AsyncReadExt + AsyncWriteExt + Unpin>(
        mut stream: ServerTlsStream<S>,
        addr: SocketAddr,
        http_handler: HttpHandler,
    ) -> BlazeResult<()> {
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m handle_http11_tls_connection entered for {}", addr);
        let mut buffer = vec![0; 8192];
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Waiting to read HTTP/1.1 over TLS request from {}", addr);
        let bytes_read = match tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            stream.read(&mut buffer)
        ).await {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => {
                error!("[Web-HTTP-1.1-TLS] Read error from {}: {}", addr, e);
                return Err(BlazeError::Io(e));
            }
            Err(_) => {
                match stream.read(&mut buffer).await {
                    Ok(n) => n,
                    Err(e) => {
                        error!("[Web-HTTP-1.1-TLS] Read error from {}: {}", addr, e);
                        return Err(BlazeError::Io(e));
                    }
                }
            }
        };
        
        let final_bytes_read = if bytes_read == 0 {
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

        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Read {} bytes from {} (HTTP/1.1 over TLS)", final_bytes_read, addr);
        let request_str = String::from_utf8_lossy(&buffer[..final_bytes_read]);
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Request preview: {}", request_str.lines().next().unwrap_or(""));

        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Parsing HTTP/1.1 over TLS request");
        let (method, path, host, body) = match Self::parse_http_request(&request_str) {
            Ok(parsed) => parsed,
            Err(e) => {
                error!("[Web-HTTP-1.1-TLS] Failed to parse request from {}: {}", addr, e);
                crate::debug_println!("\x1b[38;2;255;150;150m[Web]\x1b[0m Parse error: {}", e);
                return Err(e);
            }
        };
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Parsed request - Method: {}, Path: {}, Host: {}, Body size: {}", method, path, host, body.len());
        
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Routing HTTP/1.1 over TLS request to handler");
        let response = match http_handler.handle_request(&host, &path, &method, &body) {
            Ok(r) => r,
            Err(e) => {
                error!("[Web-HTTP-1.1-TLS] Handler error for {} {}: {}", method, path, e);
                crate::debug_println!("\x1b[38;2;255;150;150m[Web]\x1b[0m Handler error: {}", e);
                return Err(e);
            }
        };
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Handler returned response (status: {}, body size: {})", response.status_code, response.body.len());
        let response_bytes = Self::format_http_response(&response);
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Web]\x1b[0m Sending HTTP/1.1 over TLS response (size: {})",
            response_bytes.len()
        );
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Response sent successfully to {}", addr);

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

    /// Generic HTTP/2 handler using h2 crate
    async fn handle_http2_connection<S>(
        stream: S,
        addr: SocketAddr,
        http_handler: HttpHandler,
    ) -> BlazeResult<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m handle_http2_connection entered for {}", addr);
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Performing HTTP/2 handshake");
        let mut connection = server::handshake(stream).await?;
        HttpHandler::flush_grpc_compact_log_on_new_http2_connection();
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m HTTP/2 handshake successful, waiting for requests");

        while let Some(request) = connection.accept().await {
            match request {
                Ok((req, respond)) => {
                    crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m HTTP/2 request accepted, method: {}, path: {}", 
                        req.method(), req.uri().path());
                    let handler = http_handler.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::process_http2_request(req, respond, handler).await {
                            error!("Web HTTP/2 request handling error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Web HTTP/2 accept error: {}", e);
                    crate::debug_println!("\x1b[38;2;255;150;150m[Web]\x1b[0m HTTP/2 accept error: {}", e);
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
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m process_http2_request entered");
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

        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m HTTP/2 request - Method: {}, Path: {}, Host: {}", method, path, host);

        let mut body_bytes = Vec::new();
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Reading HTTP/2 request body");
        while let Some(chunk) = request.body_mut().data().await {
            let data = chunk?;
            body_bytes.extend_from_slice(&data);
        }
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m HTTP/2 request body read (size: {})", body_bytes.len());

        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Routing HTTP/2 request to handler");
        let response = http_handler.handle_request(&host, &path, &method, &body_bytes)?;
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Handler returned response (status: {}, body size: {})", response.status_code, response.body.len());
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

    /// Handle Web connection (for alt ports 8080/8443)
    async fn handle_web_connection(
        mut stream: impl AsyncReadExt + AsyncWriteExt + Unpin,
        addr: SocketAddr,
        is_https: bool,
        listen_port: u16,
    ) -> BlazeResult<()> {
        let (key, label) = if is_https {
            (
                format!("web:conn:althttps:{listen_port}"),
                format!("HTTP alt (TLS) on :{listen_port}"),
            )
        } else {
            (
                format!("web:conn:althttp:{listen_port}"),
                format!("HTTP alt (plain) on :{listen_port}"),
            )
        };
        log_web_connection_compact(key, &label);
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m handle_web_connection entered for {} (HTTPS: {})", addr, is_https);

        // Minimal placeholder: extend here for title-specific static or routed www content.
        
        let response = "HTTP/1.1 200 OK\r\n\
                        Content-Type: text/html; charset=utf-8\r\n\
                        Content-Length: 45\r\n\
                        Connection: close\r\n\
                        \r\n\
                        <html><body>Web Server (Coming Soon)</body></html>";

        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Sending placeholder response to {}", addr);
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        crate::debug_println!("\x1b[38;2;100;200;255m[Web]\x1b[0m Response sent to {}", addr);

        Ok(())
    }
}
