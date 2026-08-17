//! Initfs editor, extractor, and syntax-aware viewer (InitfsTools 2.15 parity).
//!
//! Skips console injection, launch-with-changes, type dumper, presets, and anticheat patching.

mod codec;
mod db;
mod payload;
mod syntax;
mod ui;

pub use ui::{render_initfs, InitfsState};
