//! Core Application Module
//! 
//! This module contains the core application logic:
//! - Logging setup and management
//! - Console output capture
//! - Server orchestration

pub mod console;
pub mod logging;
pub mod server;
pub mod inspector;
pub mod frostex;

// Re-export commonly used types
pub use console::*;
pub use server::*;
pub use inspector::*;
pub use frostex::*;
