//! Blaze TCP forwarding for research mode (capture alongside live upstream).

use crate::core::inspector::inspector_module::{capture_packet, CapturedPacket, PacketDirection};
use crate::blaze::protocol::fire2frame::fire2frame_module::Fire2FrameHeader;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use bytes::BytesMut;

/// Start Blaze protocol proxy server
/// Note: Blaze is a custom protocol, so target is still required (can't extract from protocol)
pub async fn start_blaze_proxy(
    listen_port: u16,
    target_host: String,
    target_port: u16,
    running: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", listen_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Blaze proxy listening on {} -> {}:{}", addr, target_host, target_port);

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((client_stream, _client_addr)) => {
                        let target_host = target_host.clone();
                        let running_clone = running.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = handle_blaze_proxy_connection(
                                client_stream,
                                target_host,
                                target_port,
                                running_clone,
                            ).await {
                                warn!("Blaze proxy connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Blaze proxy accept error: {}", e);
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

    info!("Blaze proxy stopped");
    Ok(())
}

/// Handle Blaze protocol proxy connection
async fn handle_blaze_proxy_connection(
    client_stream: TcpStream,
    target_host: String,
    target_port: u16,
    running: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to target server
    let target_addr = format!("{}:{}", target_host, target_port);
    let target_stream = match TcpStream::connect(&target_addr).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to target {}: {}", target_addr, e);
            return Err(format!("Connection failed: {}", e).into());
        }
    };

    // Split streams for bidirectional forwarding with capture
    let (mut client_read, mut client_write) = tokio::io::split(client_stream);
    let (mut target_read, mut target_write) = tokio::io::split(target_stream);

    // Buffers for accumulating packet data
    let mut client_buffer = BytesMut::new();
    let mut target_buffer = BytesMut::new();

    // Clone running flag for both tasks
    let running_clone1 = running.clone();
    let running_clone2 = running.clone();

    // Forward client -> target with capture
    let client_to_target = tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        loop {
            if !running_clone1.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            
            match client_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    client_buffer.extend_from_slice(&buf[..n]);
                    
                    // Process complete packets from buffer
                    while let Some(packet) = extract_blaze_packet(&mut client_buffer, PacketDirection::ClientToBlaze) {
                        capture_packet(packet.clone());
                        
                        // Forward the packet
                        if target_write.write_all(&packet.raw_packet).await.is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Forward target -> client with capture
    let target_to_client = tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        loop {
            if !running_clone2.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            
            match target_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    target_buffer.extend_from_slice(&buf[..n]);
                    
                    // Process complete packets from buffer
                    while let Some(packet) = extract_blaze_packet(&mut target_buffer, PacketDirection::BlazeToClient) {
                        capture_packet(packet.clone());
                        
                        // Forward the packet
                        if client_write.write_all(&packet.raw_packet).await.is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    let _ = tokio::join!(client_to_target, target_to_client);
    Ok(())
}

/// Extract a complete Blaze packet from buffer
/// Returns Some(packet) if a complete packet is found, None otherwise
fn extract_blaze_packet(buffer: &mut BytesMut, direction: PacketDirection) -> Option<CapturedPacket> {
    // Need at least header size to check
    if buffer.len() < Fire2FrameHeader::HEADER_SIZE {
        return None;
    }

    // Try to parse Fire2Frame header
    match Fire2FrameHeader::from_bytes(&buffer[..Fire2FrameHeader::HEADER_SIZE]) {
        Ok(header) => {
            let total_size = header.get_total_size();
            
            // Check if we have the complete packet
            if buffer.len() < total_size {
                return None; // Need more data
            }

            // Extract the complete packet
            let packet_data = buffer.split_to(total_size).to_vec();
            
            // Parse the packet
            match parse_blaze_packet(&packet_data, direction, &header) {
                Ok(packet) => Some(packet),
                Err(e) => {
                    warn!("Failed to parse Blaze packet: {}", e);
                    None
                }
            }
        }
        Err(_) => {
            // Invalid header - skip one byte and try again
            if !buffer.is_empty() {
                let _ = buffer.split_to(1);
            }
            None
        }
    }
}

/// Parse Blaze packet from raw data using Fire2Frame header
fn parse_blaze_packet(
    data: &[u8],
    direction: PacketDirection,
    header: &Fire2FrameHeader,
) -> Result<CapturedPacket, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    // Extract payload (skip header and metadata)
    let payload_start = Fire2FrameHeader::HEADER_SIZE + header.metadata_size as usize;
    let payload = if payload_start < data.len() {
        data[payload_start..].to_vec()
    } else {
        Vec::new()
    };

    Ok(CapturedPacket {
        timestamp,
        direction,
        component: header.component,
        command: header.command,
        msg_num: header.msg_num,
        msg_type: header.msg_type.to_string().to_string(),
        payload_size: payload.len(),
        payload,
        raw_packet: data.to_vec(),
        command_name: None,
        metadata_size: header.metadata_size,
    })
}

