
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone)]
pub struct LogLine {
    pub text: String,
    pub colors: Vec<(usize, RgbColor)>,
    pub segments: Vec<(String, RgbColor)>,
    pub timestamp: f64,
    /// When set, Shell replaces the existing row with this key instead of appending (gRPC compact).
    pub upsert_key: Option<String>,
}

pub type LogBuffer = Arc<Mutex<Vec<LogLine>>>;

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

pub fn init_global_buffer(buffer: LogBuffer) {
    *GLOBAL_BUFFER.lock() = Some(buffer);
}

pub fn get_global_buffer() -> Option<LogBuffer> {
    GLOBAL_BUFFER.lock().clone()
}

pub fn parse_ansi_codes(text: &str) -> (String, Vec<(usize, RgbColor)>) {
    let text = text.replace("\\x1b", "\x1b");

    let mut result = String::new();
    let mut colors = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' || ch == '\u{001b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
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

                let pos = result.len();

                if code.starts_with("38;2;") {
                    let parts: Vec<&str> = code.split(';').collect();
                    if parts.len() >= 5 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            parts[2].parse::<u8>(),
                            parts[3].parse::<u8>(),
                            parts[4].parse::<u8>(),
                        ) {
                            colors.push((pos, RgbColor::rgb(r, g, b)));
                        }
                    }
                } else if code == "0" {
                    colors.push((pos, RgbColor::WHITE));
                }
            }
        } else {
            result.push(ch);
        }
    }

    (result, colors)
}

fn make_log_line(text: &str) -> LogLine {
    use std::time::{SystemTime, UNIX_EPOCH};

    let (cleaned, colors) = parse_ansi_codes(text);

    let mut segments = Vec::new();
    let mut last_pos = 0;
    let mut current_color = RgbColor::WHITE;

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
        segments.push((cleaned.clone(), RgbColor::WHITE));
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    let text = cleaned.trim().to_string();

    LogLine {
        text,
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

pub fn capture_line(text: &str) {
    push_formatted_log_line(text);
}

pub fn push_formatted_log_line(text: &str) {
    push_log_line(make_log_line(text));
}

pub fn is_debug_logging_enabled() -> bool {
    crate::common::settings::get_app_settings().debug_logging
}

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::core::console::is_debug_logging_enabled() {
            $crate::console_println!($($arg)*);
        }
    };
}
