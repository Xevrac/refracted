// gRPC Request/Response Handler
// Handles gRPC requests with proper framing, compression, and protobuf encoding

use crate::common::error::BlazeResult;
use crate::grpc::{build_grpc_frame, parse_grpc_frame, client_accepts_gzip};
use crate::grpc::grpc_protobuf::*;
use std::collections::HashMap;

/// Extract JWT token from gRPC request body
/// The JWT is typically in field 6 of the request
pub fn extract_jwt_from_grpc_request(body: &[u8]) -> Option<String> {
    // Parse gRPC frame first
    let (_, protobuf_data) = parse_grpc_frame(body).ok()?;
    
    // Extract field 6 (JWT token)
    extract_string_field(&protobuf_data, 6)
}

/// Build GetAuthForToken protobuf payload (field layout below).
/// Shape:
/// {
///   "1": "cbdede63-d594-49aa-bbc1-1f86f6f2507b",  // UUID
///   "2": { "1": 1012711274866, "2": 1006276674866, "3": 1 },  // User info struct
///   "4": { "1": "824762863", "2": "824762863" },  // IDs struct
///   "5": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",  // JWT token
///   "6": "0.17.1"  // Version
/// }
pub fn build_get_auth_for_token_protobuf(
    uuid: &str,
    user_id: u64,
    persona_id: u64,
    id1: &str,
    id2: &str,
    jwt_token: &str,
    version: &str,
) -> Vec<u8> {
    let mut response = Vec::new();
    
    // Field 1: UUID string
    response.extend_from_slice(&encode_string_field(1, uuid));
    
    // Field 2: User info struct
    let mut user_info = Vec::new();
    user_info.extend_from_slice(&encode_uint64_field(1, user_id));
    user_info.extend_from_slice(&encode_uint64_field(2, persona_id));
    user_info.extend_from_slice(&encode_int64_field(3, 1));
    response.extend_from_slice(&encode_message_field(2, &user_info));
    
    // Field 4: IDs struct
    let mut ids_struct = Vec::new();
    ids_struct.extend_from_slice(&encode_string_field(1, id1));
    ids_struct.extend_from_slice(&encode_string_field(2, id2));
    response.extend_from_slice(&encode_message_field(4, &ids_struct));
    
    // Field 5: JWT token string
    response.extend_from_slice(&encode_string_field(5, jwt_token));
    
    // Field 6: Version string
    response.extend_from_slice(&encode_string_field(6, version));
    
    response
}

/// Build GetAuthForToken response with Code messages (alternative format)
/// This format uses repeated Code messages in field 1
pub fn build_get_auth_for_token_code_format(token: &str, token_url: &str) -> Vec<u8> {
    let mut response = Vec::new();
    
    // First Code message: field 1 (repeated), contains Code { field 1: token }
    let mut code1_msg = Vec::new();
    code1_msg.extend_from_slice(&encode_string_field(1, token));
    response.extend_from_slice(&encode_message_field(1, &code1_msg));
    
    // Second Code message: field 1 (repeated), contains Code { field 1: token_url }
    let mut code2_msg = Vec::new();
    code2_msg.extend_from_slice(&encode_string_field(1, token_url));
    response.extend_from_slice(&encode_message_field(1, &code2_msg));
    
    response
}

/// Build complete gRPC response with proper framing and compression
pub fn build_grpc_response(
    protobuf_data: &[u8],
    headers: &HashMap<String, String>,
) -> BlazeResult<(Vec<u8>, HashMap<String, String>)> {
    let use_gzip = client_accepts_gzip(headers);
    
    // Build gRPC frame with optional gzip compression
    let frame = build_grpc_frame(protobuf_data, use_gzip)?;
    
    // Update headers
    let mut response_headers = headers.clone();
    if use_gzip {
        response_headers.insert("grpc-encoding".to_string(), "gzip".to_string());
    } else {
        response_headers.insert("grpc-encoding".to_string(), "identity".to_string());
    }
    response_headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
    response_headers.insert("grpc-status".to_string(), "0".to_string());
    
    Ok((frame, response_headers))
}





