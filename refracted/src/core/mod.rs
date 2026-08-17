//! Core Application Module
//! This module contains the core application logic:
//! - Logging setup and management
//! - Console output capture
//! - Server orchestration

pub mod console;
pub mod logging;
pub mod server;
pub mod inspector;

#[cfg(feature = "desktop")]
pub mod frostex;

#[cfg(feature = "desktop")]
pub mod layout_editor;

#[cfg(feature = "desktop")]
pub mod initfs;

// Re-export commonly used types
pub use console::*;
pub use server::*;
pub use inspector::*;

#[cfg(feature = "desktop")]
pub use frostex::*;

#[cfg(feature = "desktop")]
pub use layout_editor::{cnc_layout_editor_available, render_layout_editor, LayoutEditorState};

#[cfg(feature = "desktop")]
pub use initfs::{render_initfs, InitfsState};
