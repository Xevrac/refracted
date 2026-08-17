//! CNC Toolkit — native Frostbite `.layout` UI editor (ported from layout_editor.py).

mod model;
mod ui;

pub use ui::{cnc_layout_editor_available, render_layout_editor, LayoutEditorState};
