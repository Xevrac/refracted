// Console capture module - provides a way to capture stdout and send to GUI
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct LogLine {
    pub text: String,
    pub colors: Vec<(usize, egui::Color32)>,
    pub segments: Vec<(String, egui::Color32)>,
    pub timestamp: f64,
    /// When set, Shell replaces the existing row with this key instead of appending (gRPC compact).
    pub upsert_key: Option<String>,
}

pub type LogBuffer = Arc<Mutex<Vec<LogLine>>>;

// Global buffer for console output
static GLOBAL_BUFFER: parking_lot::Mutex<Option<LogBuffer>> = parking_lot::const_mutex(None);

/// Tokio/tracing pushes here; egui drains into [`LogBuffer`] each frame (no mutex contention with writers).
static LOG_LINE_TX: OnceLock<std::sync::mpsc::Sender<LogLine>> = OnceLock::new();

pub fn init_log_line_sender(tx: std::sync::mpsc::Sender<LogLine>) {
    let _ = LOG_LINE_TX.set(tx);
}

pub fn push_log_line(line: LogLine) {
    if let Some(tx) = LOG_LINE_TX.get() {
        let _ = tx.send(line);
    }
}

/// Initialize the global log buffer
pub fn init_global_buffer(buffer: LogBuffer) {
    *GLOBAL_BUFFER.lock() = Some(buffer);
}

/// Get the global log buffer
pub fn get_global_buffer() -> Option<LogBuffer> {
    GLOBAL_BUFFER.lock().clone()
}

/// Parse ANSI escape codes from text
pub fn parse_ansi_codes(text: &str) -> (String, Vec<(usize, egui::Color32)>) {
    use eframe::egui;
    
    // First, replace literal "\x1b" strings with actual escape character
    // This handles cases where escape sequences are escaped/encoded as strings
    let text = text.replace("\\x1b", "\x1b");
    
    let mut result = String::new();
    let mut colors = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' || ch == '\u{001b}' {
            // Parse ANSI escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                let mut code = String::new();
                while let Some(&next) = chars.peek() {
                    if next == 'm' {
                        chars.next();
                        break;
                    }
                    if next.is_ascii_digit() || next == ';' {
                        code.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Byte offset in `result` where this color applies (must match str slicing).
                let pos = result.len();

                // Parse RGB color code: 38;2;r;g;b
                if code.starts_with("38;2;") {
                    let parts: Vec<&str> = code.split(';').collect();
                    if parts.len() >= 5 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            parts[2].parse::<u8>(),
                            parts[3].parse::<u8>(),
                            parts[4].parse::<u8>(),
                        ) {
                            colors.push((pos, egui::Color32::from_rgb(r, g, b)));
                        }
                    }
                } else if code == "0" {
                    // Reset color
                    colors.push((pos, egui::Color32::WHITE));
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    (result, colors)
}

fn make_log_line(text: &str) -> LogLine {
    use eframe::egui;
    use std::time::{SystemTime, UNIX_EPOCH};

    let (cleaned, colors) = parse_ansi_codes(text);

    let mut segments = Vec::new();
    let mut last_pos = 0;
    let mut current_color = egui::Color32::WHITE;

    for (pos, color) in &colors {
        if *pos > last_pos {
            segments.push((cleaned[last_pos..*pos].to_string(), current_color));
        }
        current_color = *color;
        last_pos = *pos;
    }
    if last_pos < cleaned.len() {
        segments.push((cleaned[last_pos..].to_string(), current_color));
    }
    if segments.is_empty() {
        segments.push((cleaned.clone(), egui::Color32::WHITE));
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    LogLine {
        text: cleaned,
        colors,
        segments,
        timestamp,
        upsert_key: None,
    }
}

/// gRPC compact: same endpoint updates one Shell row (`x1` → `x2` → …) instead of new lines.
pub fn push_grpc_compact_upsert(key: String, ansi_text: &str) {
    let mut line = make_log_line(ansi_text);
    line.upsert_key = Some(key);
    push_log_line(line);
}

/// Queue one line for the Shell (from UI thread before `init_log_line_sender`, this is a no-op).
pub fn capture_line(text: &str) {
    push_formatted_log_line(text);
}

/// Formatted text (ANSI ok) → Shell queue; used by tracing `LogWriter` and `capture_line`.
pub fn push_formatted_log_line(text: &str) {
    push_log_line(make_log_line(text));
}

/// Check if debug logging is enabled
pub fn is_debug_logging_enabled() -> bool {
    crate::common::settings::get_app_settings().debug_logging
}

/// Conditionally log debug message (only if debug logging is enabled)
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::core::console::is_debug_logging_enabled() {
            $crate::console_println!($($arg)*);
        }
    };
}

