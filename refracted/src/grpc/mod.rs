//! gRPC Module
//! 
//! Implements proper gRPC over HTTP/2 with:
//! - HTTP/2 framing
//! - gRPC frame format: [compression flag (1 byte)][message length (4 bytes)][data]
//! - gzip compression/decompression
//! - Protobuf encoding/decoding

pub mod grpc_frame;
pub mod grpc_protobuf;
pub mod grpc_handler;

pub use grpc_frame::*;
pub use grpc_protobuf::*;
pub use grpc_handler::*;





