//! gRPC forwarding for research mode (capture alongside live upstream).

use crate::core::inspector::inspector_module::{capture_grpc, CapturedGrpc, GrpcDirection};
use crate::grpc::grpc_body_decode_capture;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Start gRPC proxy server
/// Extracts destination from each request (Host header) and forwards to original destination
pub async fn start_grpc_proxy(
    listen_port: u16,
    _target_host: String,  // Kept for compatibility but not used
    _target_port: u16,     // Kept for compatibility but not used
    running: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", listen_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("gRPC proxy listening on {} (forwarding to original destination from Host header)", addr);

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((client_stream, _client_addr)) => {
                        let running_clone = running.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = handle_grpc_proxy_connection(
                                client_stream,
                                running_clone,
                            ).await {
                                warn!("gRPC proxy connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("gRPC proxy accept error: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                if !running.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
            }
        }
    }

    info!("gRPC proxy stopped");
    Ok(())
}

/// Handle gRPC proxy connection (HTTP/2 based)
/// Extracts destination from Host header and forwards to original destination
async fn handle_grpc_proxy_connection(
    mut client_stream: TcpStream,
    _running: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // gRPC uses HTTP/2, so we need to handle HTTP/2 frames
    // For now, we'll do a simplified pass-through with capture
    // Full HTTP/2 support would require proper frame parsing
    
    // Read initial data from client
    let mut buffer = vec![0u8; 16384];
    let n = client_stream.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }

    let request_data = &buffer[..n];
    
    // Try to parse as HTTP/2 or HTTP/1.1 (gRPC can use either)
    let request_str = String::from_utf8_lossy(request_data);
    
    // Check if it's HTTP/1.1 style (for gRPC over HTTP/1.1)
    if request_str.starts_with("POST") || request_str.starts_with("GET") {
        // Parse as HTTP/1.1 gRPC request
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
        let host_header = lines.iter()
            .find(|line| line.to_lowercase().starts_with("host:"))
            .map(|line| line.split(':').nth(1).unwrap_or("").trim().to_string())
            .ok_or("Missing Host header")?;
        
        // Parse host:port from Host header
        let (target_host, target_port) = if host_header.contains(':') {
            let parts: Vec<&str> = host_header.split(':').collect();
            let host = parts[0].to_string();
            let port = parts[1].parse::<u16>().unwrap_or(443);
            (host, port)
        } else {
            (host_header.clone(), 443) // Default to port 443 for gRPC
        };

        // Extract headers
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

        // Extract body (gRPC frame)
        let body_start = request_str.find("\r\n\r\n")
            .or_else(|| request_str.find("\n\n"))
            .map(|pos| pos + 4)
            .unwrap_or(request_data.len());
        
        let body = if body_start < request_data.len() {
            request_data[body_start..].to_vec()
        } else {
            Vec::new()
        };

        let decoded = grpc_body_decode_capture(&body);
        let is_compressed = decoded.any_frame_was_compressed;
        let protobuf_data = decoded.protobuf_chunks.first().cloned();

        // Capture request
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let captured_request = CapturedGrpc {
            capture_seq: 0,
            timestamp,
            direction: GrpcDirection::ClientToServer,
            method: method.clone(),
            path: path.clone(),
            host: host_header.clone(),
            headers: headers.clone(),
            body_size: body.len(),
            body: body.clone(),
            protobuf_data: protobuf_data.clone(),
            protobuf_chunks: decoded.protobuf_chunks.clone(),
            is_compressed,
            grpc_status: None,
        };
        capture_grpc(captured_request);

        // Connect to target server
        let target_addr = format!("{}:{}", target_host, target_port);
        let mut target_stream = match TcpStream::connect(&target_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to target {}: {}", target_addr, e);
                return Err(format!("Connection failed: {}", e).into());
            }
        };

        // Forward request to target
        if let Err(e) = target_stream.write_all(request_data).await {
            error!("Failed to forward gRPC request: {}", e);
            return Err(format!("Forward failed: {}", e).into());
        }

        // Read response from target
        let mut response_buffer = Vec::new();
        let mut temp_buf = vec![0u8; 16384];
        
        loop {
            match target_stream.read(&mut temp_buf).await {
                Ok(0) => break,
                Ok(n) => {
                    response_buffer.extend_from_slice(&temp_buf[..n]);
                }
                Err(e) => {
                    error!("Error reading gRPC response: {}", e);
                    break;
                }
            }
        }

        // Parse and capture response
        let response_str = String::from_utf8_lossy(&response_buffer);
        let response_lines: Vec<&str> = response_str.lines().collect();
        
        let grpc_status = response_lines.iter()
            .find(|line| line.to_lowercase().starts_with("grpc-status:"))
            .and_then(|line| line.split(':').nth(1).map(|s| s.trim().to_string()));

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

        let resp_decoded = grpc_body_decode_capture(&response_body);
        let resp_is_compressed = resp_decoded.any_frame_was_compressed;
        let resp_protobuf_data = resp_decoded.protobuf_chunks.first().cloned();

        let captured_response = CapturedGrpc {
            capture_seq: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
            direction: GrpcDirection::ServerToClient,
            method: method.clone(),
            path: path.clone(),
            host: host_header.clone(),
            headers: response_headers,
            body_size: response_body.len(),
            body: response_body.clone(),
            protobuf_data: resp_protobuf_data,
            protobuf_chunks: resp_decoded.protobuf_chunks.clone(),
            is_compressed: resp_is_compressed,
            grpc_status,
        };
        capture_grpc(captured_response);

        // Forward response to client
        if let Err(e) = client_stream.write_all(&response_buffer).await {
            error!("Failed to forward gRPC response: {}", e);
            return Err(format!("Response forward failed: {}", e).into());
        }
    } else {
        // HTTP/2 binary protocol - pass through for now
        // Full HTTP/2 frame parsing would be needed for proper capture and destination extraction
        warn!("HTTP/2 binary protocol detected - cannot extract destination, connection refused");
        return Err("HTTP/2 binary protocol requires :authority header parsing (not yet implemented)".into());
    }

    Ok(())
}

