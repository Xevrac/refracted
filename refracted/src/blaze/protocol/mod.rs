//! Blaze Protocol Implementations
//! 
//! This module contains the different Blaze protocol frame formats:
//! - Fire2Frame: 16-byte header format (newer)
//! - FireFrame: 12-byte header format (older)

pub mod compact_blaze_envelope;
pub mod fire2frame;
pub mod fireframe;
pub mod redirector;

// Re-export commonly used types from Fire2Frame (primary protocol)
pub use fire2frame::{Fire2FramePacket, Fire2FrameHeader, MessageType};
pub use fireframe::{FireFramePacket, FireFrameHeader};
pub use redirector::{build_get_server_instance_reply, find_get_server_instance, RedirectorWire};

