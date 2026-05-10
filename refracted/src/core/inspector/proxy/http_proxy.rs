//! HTTP/HTTPS forwarding for research mode (capture alongside live upstream).

use crate::core::inspector::inspector_module::{capture_http, CapturedHttp, HttpDirection};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Start HTTP proxy server
/// Extracts destination from each request (Host header) and forwards to original destination
pub async fn start_http_proxy(
    listen_port: u16,
    _target_host: String,  // Kept for compatibility but not used
    _target_port: u16,     // Kept for compatibility but not used
    running: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", listen_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("HTTP proxy listening on {} (forwarding to original destination from Host header)", addr);

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Use tokio::select to check running flag periodically
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((client_stream, _client_addr)) => {
                        let running_clone = running.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = handle_http_proxy_connection(
                                client_stream,
                                running_clone,
                            ).await {
                                warn!("HTTP proxy connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("HTTP proxy accept error: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Periodic check of running flag
                if !running.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
            }
        }
    }

    info!("HTTP proxy stopped");
    Ok(())
}

/// Handle HTTP proxy connection
/// Extracts destination from Host header and forwards to original destination
async fn handle_http_proxy_connection(
    mut client_stream: TcpStream,
    _running: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read request from client
    let mut buffer = vec![0u8; 8192];
    let n = client_stream.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }

    let request_data = &buffer[..n];
    
    // Parse HTTP request
    let request_str = String::from_utf8_lossy(request_data);
    let lines: Vec<&str> = request_str.lines().collect();
    
    if lines.is_empty() {
        return Ok(());
    }

    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    
    if parts.len() < 3 {
        return Ok(());
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();
    
    // Extract destination from Host header
    let host_header_raw = lines.iter()
        .find(|line| line.to_lowercase().starts_with("host:"))
        .map(|line| line.split(':').nth(1).unwrap_or("").trim().to_string())
        .ok_or("Missing Host header")?;
    
    // Parse host:port from Host header
    let (target_host, target_port, host_header) = if host_header_raw.contains(':') {
        let parts: Vec<&str> = host_header_raw.split(':').collect();
        let host = parts[0].to_string();
        let port = parts[1].parse::<u16>().unwrap_or(80);
        (host.clone(), port, host)
    } else {
        (host_header_raw.clone(), 80, host_header_raw) // Default to port 80 for HTTP
    };

    // Capture request
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    let headers: Vec<(String, String)> = lines[1..]
        .iter()
        .take_while(|line| !line.is_empty())
        .filter_map(|line| {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                Some((key, value))
            } else {
                None
            }
        })
        .collect();

    let body_start = request_str.find("\r\n\r\n")
        .or_else(|| request_str.find("\n\n"))
        .map(|pos| pos + 4)
        .unwrap_or(request_data.len());
    
    let body = if body_start < request_data.len() {
        request_data[body_start..].to_vec()
    } else {
        Vec::new()
    };

    let captured_request = CapturedHttp {
        capture_seq: 0,
        timestamp,
        direction: HttpDirection::ClientToServer,
        method: method.clone(),
        path: path.clone(),
        host: host_header.clone(),
        headers: headers.clone(),
        body_size: body.len(),
        body: body.clone(),
        status_code: None,
    };
    capture_http(captured_request);

    // Connect to target server (extracted from Host header)
    let target_addr = format!("{}:{}", target_host, target_port);
    let mut target_stream = match TcpStream::connect(&target_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to target {}: {}", target_addr, e);
            let error_response = format!(
                "HTTP/1.1 502 Bad Gateway\r\n\
                 Content-Length: 0\r\n\
                 Connection: close\r\n\r\n"
            );
            let _ = client_stream.write_all(error_response.as_bytes()).await;
            return Err(format!("Connection failed: {}", e).into());
        }
    };

    // Forward request to target
    if let Err(e) = target_stream.write_all(request_data).await {
        error!("Failed to forward request: {}", e);
        return Err(format!("Forward failed: {}", e).into());
    }

    // Read response from target
    let mut response_buffer = Vec::new();
    let mut temp_buf = vec![0u8; 8192];
    
    loop {
        match target_stream.read(&mut temp_buf).await {
            Ok(0) => break,
            Ok(n) => {
                response_buffer.extend_from_slice(&temp_buf[..n]);
            }
            Err(e) => {
                error!("Error reading response: {}", e);
                break;
            }
        }
    }

    // Parse and capture response
    let response_str = String::from_utf8_lossy(&response_buffer);
    let response_lines: Vec<&str> = response_str.lines().collect();
    
    let status_code = response_lines.get(0)
        .and_then(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.get(1).and_then(|s| s.parse::<u16>().ok())
        });

    let response_headers: Vec<(String, String)> = response_lines[1..]
        .iter()
        .take_while(|line| !line.is_empty())
        .filter_map(|line| {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                Some((key, value))
            } else {
                None
            }
        })
        .collect();

    let response_body_start = response_str.find("\r\n\r\n")
        .or_else(|| response_str.find("\n\n"))
        .map(|pos| pos + 4)
        .unwrap_or(response_buffer.len());
    
    let response_body = if response_body_start < response_buffer.len() {
        response_buffer[response_body_start..].to_vec()
    } else {
        Vec::new()
    };

    let captured_response = CapturedHttp {
        capture_seq: 0,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64(),
        direction: HttpDirection::ServerToClient,
        method: method.clone(),
        path: path.clone(),
        host: host_header.clone(),
        headers: response_headers,
        body_size: response_body.len(),
        body: response_body.clone(),
        status_code,
    };
    capture_http(captured_response);

    // Forward response to client
    if let Err(e) = client_stream.write_all(&response_buffer).await {
        error!("Failed to forward response: {}", e);
        return Err(format!("Response forward failed: {}", e).into());
    }

    Ok(())
}

/// Start HTTPS proxy server (handles TLS)
/// Extracts destination from each request and forwards to original destination
pub async fn start_https_proxy(
    listen_port: u16,
    _target_host: String,  // Kept for compatibility but not used
    _target_port: u16,     // Kept for compatibility but not used
    running: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // HTTPS: simplified passthrough; full TLS interception would need local CA and cert generation.
    warn!("HTTPS proxy is not fully implemented; TLS interception needs certificate handling");
    start_http_proxy(listen_port, "".to_string(), 0, running).await
}

