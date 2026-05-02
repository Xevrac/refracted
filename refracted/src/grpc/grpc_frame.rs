// gRPC Frame Format
// 
// gRPC frames have the format:
// [compression flag (1 byte)][message length (4 bytes, big-endian)][message data]
// 
// Compression flag:
// - 0x00 = no compression
// - 0x01 = compressed (gzip)

use crate::common::error::{BlazeError, BlazeResult};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// Hard cap so random protobuf bytes misread as length do not imply multi‑GB frames.
pub const MAX_GRPC_MESSAGE_PAYLOAD: usize = 64 * 1024 * 1024;

/// Peel one or more concatenated **gRPC over HTTP/2 DATA** payloads:
/// `[compressed u8][length u32 BE][payload]*` repeating. Each emitted chunk is protobuf bytes
/// (`gzip`-inflated when the flag byte is `0x01`; pass-through when `0x00`).
/// Returns `(messages, slack_bytes_after_last_full_frame)` per IOActive / gRPC wire layout.
///
/// `first_payload_len_override`: if `Some(L)`, uses `L` as the declared payload byte length for
/// the **first** frame only (bytes after the 5-byte header); subsequent frames always read BE
/// length from the stream. Intended for malformed captures where wire length disagrees from reality.
pub fn peel_grpc_data_frames_detailed_with_first_len_override(
    body: &[u8],
    first_payload_len_override: Option<u32>,
) -> (Vec<Vec<u8>>, &[u8], bool) {
    let mut out = Vec::new();
    let mut rest = body;
    let mut any_compressed_flag = false;
    let mut is_first_outer_frame = true;
    while rest.len() >= 5 {
        let flag = rest[0];
        if flag != 0x00 && flag != 0x01 {
            break;
        }
        if flag == 0x01 {
            any_compressed_flag = true;
        }
        let len = if is_first_outer_frame {
            is_first_outer_frame = false;
            match first_payload_len_override {
                Some(ov) => ov as usize,
                None => u32::from_be_bytes([rest[1], rest[2], rest[3], rest[4]]) as usize,
            }
        } else {
            u32::from_be_bytes([rest[1], rest[2], rest[3], rest[4]]) as usize
        };

        if len > MAX_GRPC_MESSAGE_PAYLOAD {
            break;
        }
        let Some(total) = 5usize.checked_add(len) else {
            break;
        };
        if rest.len() < total {
            break;
        }
        let slice = &rest[5..total];
        match flag {
            0x00 => out.push(slice.to_vec()),
            0x01 => match decompress_gzip_bytes(slice) {
                Ok(dec) => out.push(dec),
                Err(_) => break,
            },
            _ => break,
        }
        rest = &rest[total..];
    }
    (out, rest, any_compressed_flag)
}

pub fn peel_grpc_data_frames_detailed(body: &[u8]) -> (Vec<Vec<u8>>, &[u8], bool) {
    peel_grpc_data_frames_detailed_with_first_len_override(body, None)
}

/// Fields captured for toolkit / Listen (proto bytes per message, optional slack).
#[derive(Clone, Debug)]
pub struct GrpcBodyDecodeCapture {
    /// One entry per peeled gRPC data frame on the HTTP/2 body (protobuf-ready).
    pub protobuf_chunks: Vec<Vec<u8>>,
    /// Tail bytes after the last parseable frame (trailers framing, stray bytes, incomplete frame).
    pub slack: Vec<u8>,
    pub any_frame_was_compressed: bool,
}

pub fn grpc_body_decode_capture(body: &[u8]) -> GrpcBodyDecodeCapture {
    let (chunks, slack, any_compressed) = peel_grpc_data_frames_detailed(body);
    GrpcBodyDecodeCapture {
        protobuf_chunks: chunks,
        slack: slack.to_vec(),
        any_frame_was_compressed: any_compressed,
    }
}

/// Parse gRPC frame from HTTP/2 body
/// Returns (compressed flag, decompressed data)
pub fn parse_grpc_frame(body: &[u8]) -> BlazeResult<(bool, Vec<u8>)> {
    if body.len() < 5 {
        return Err(BlazeError::InvalidPacket(
            format!("gRPC frame too short: {} bytes", body.len())
        ));
    }

    let compression_flag = body[0];
    let message_length =
        u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
    if message_length > MAX_GRPC_MESSAGE_PAYLOAD {
        return Err(BlazeError::InvalidPacket(format!(
            "gRPC declared length unreasonable: {}",
            message_length
        )));
    }

    if body.len() < 5 + message_length {
        return Err(BlazeError::InvalidPacket(
            format!(
                "gRPC frame incomplete: have {} bytes, need {} bytes",
                body.len(),
                5 + message_length
            )
        ));
    }

    let message_data = &body[5..5 + message_length];
    let is_compressed = compression_flag == 0x01;

    if is_compressed {
        // Decompress with gzip
        let mut decoder = GzDecoder::new(message_data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| BlazeError::InvalidPacket(format!("gzip decompression failed: {}", e)))?;
        Ok((true, decompressed))
    } else {
        Ok((false, message_data.to_vec()))
    }
}

/// Build gRPC frame from protobuf data
/// If use_gzip is true, compresses the data with gzip
pub fn build_grpc_frame(data: &[u8], use_gzip: bool) -> BlazeResult<Vec<u8>> {
    let (compression_flag, message_data) = if use_gzip {
        // Compress with gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| BlazeError::InvalidPacket(format!("gzip compression failed: {}", e)))?;
        let compressed = encoder.finish()
            .map_err(|e| BlazeError::InvalidPacket(format!("gzip compression finish failed: {}", e)))?;
        (0x01u8, compressed)
    } else {
        (0x00u8, data.to_vec())
    };

    let mut frame = Vec::new();
    frame.push(compression_flag);
    frame.extend_from_slice(&(message_data.len() as u32).to_be_bytes());
    frame.extend_from_slice(&message_data);

    Ok(frame)
}

/// Decompress a gzip stream (standalone `.gz`, or protobuf bytes wrapped once in gzip).
pub fn decompress_gzip_bytes(body: &[u8]) -> BlazeResult<Vec<u8>> {
    let mut decoder = GzDecoder::new(body);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| BlazeError::InvalidPacket(format!("gzip decompression failed: {}", e)))?;
    Ok(out)
}

/// gzip magic (`1f 8b`) at slice start.
pub fn looks_like_gzip_prefix(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}

/// Check if client accepts gzip compression from headers
pub fn client_accepts_gzip(headers: &std::collections::HashMap<String, String>) -> bool {
    headers
        .get("grpc-accept-encoding")
        .map(|v| v.contains("gzip"))
        .unwrap_or(false)
}





