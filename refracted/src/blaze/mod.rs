//! Blaze Protocol Module
//! 
//! This module contains all Blaze protocol-related functionality:
//! - Component definitions and command routing
//! - TDF encoding/decoding
//! - Protocol handlers
//! - Error codes
//! - Protocol packet formats (Fire2Frame, FireFrame)

pub mod components;
pub mod errors;
pub mod handlers;
pub mod protocol;
pub mod server;
pub mod tdf;

// Re-export commonly used types
pub use components::{get_component_name, get_command_name};
pub use errors::{get_error_name, get_error_code, create_error_response};
pub use protocol::{Fire2FrameHeader, Fire2FramePacket, MessageType};
pub use tdf::TdfEncoder;

