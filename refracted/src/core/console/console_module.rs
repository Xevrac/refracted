
use parking_lot::Mutex;
use std::io::{self, Write};
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

/// True when the desktop GUI registered a Shell log drain.
pub fn has_log_line_consumer() -> bool {
    LOG_LINE_TX.get().is_some()
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

/// Paint known channel tags. Safe when the line already has ANSI (e.g. gray `xN`).
pub fn colorize_channel_tags(message: &str) -> String {
    let mut out = message
        .replace("\\x1b", "\x1b")
        .replace("\\u{001b}", "\x1b");
    const PAIRS: &[(&str, &str)] = &[
        (
            "[Client→Blaze]",
            "\x1b[38;2;100;200;255m[Client→Blaze]\x1b[0m",
        ),
        (
            "[Blaze→Client]",
            "\x1b[38;2;100;200;255m[Blaze→Client]\x1b[0m",
        ),
        (
            "[Server→Blaze]",
            "\x1b[38;2;255;180;80m[Server→Blaze]\x1b[0m",
        ),
        (
            "[Blaze→Server]",
            "\x1b[38;2;255;180;80m[Blaze→Server]\x1b[0m",
        ),
        ("[RTS]", "\x1b[38;2;56;156;220m[RTS]\x1b[0m"),
        ("[SIM]", "\x1b[38;2;140;180;140m[SIM]\x1b[0m"),
        (
            "[Orchestration]",
            "\x1b[38;2;140;180;220m[Orchestration]\x1b[0m",
        ),
        ("[GOS]", "\x1b[38;2;150;150;255m[GOS]\x1b[0m"),
        ("[CNC]", "\x1b[38;2;255;215;0m[CNC]\x1b[0m"),
        ("[QoS]", "\x1b[38;2;80;200;120m[QoS]\x1b[0m"),
        ("[Nexus → Blaze]", "\x1b[38;2;180;140;255m[Nexus → Blaze]\x1b[0m"),
        ("[Blaze → Nexus]", "\x1b[38;2;180;140;255m[Blaze → Nexus]\x1b[0m"),
    ];
    for (plain, colored) in PAIRS {
        if out.contains(colored) {
            continue;
        }
        if out.contains(plain) {
            out = out.replacen(plain, colored, 1);
        }
    }
    out
}

struct CliCompactMirror {
    open_key: Option<String>,
    open: bool,
}

static CLI_COMPACT: Mutex<CliCompactMirror> = Mutex::new(CliCompactMirror {
    open_key: None,
    open: false,
});

/// Finish an in-progress compact CLI line before other stdout (tracing) writes.
pub fn flush_cli_compact_line() {
    let mut g = CLI_COMPACT.lock();
    if !g.open {
        return;
    }
    let _ = writeln!(io::stdout());
    let _ = io::stdout().flush();
    g.open = false;
    g.open_key = None;
}

fn mirror_compact_to_cli(key: &str, ansi_text: &str) {
    let display = colorize_channel_tags(ansi_text);
    let mut g = CLI_COMPACT.lock();
    let mut out = io::stdout();
    if g.open {
        if g.open_key.as_deref() == Some(key) {
            // Same upsert key: rewrite one terminal row (GUI Shell behavior).
            let _ = write!(out, "\r\x1b[2K{display}");
            let _ = out.flush();
            return;
        }
        let _ = writeln!(out);
        g.open = false;
        g.open_key = None;
    }
    let _ = write!(out, "{display}");
    let _ = out.flush();
    g.open_key = Some(key.to_string());
    g.open = true;
}

/// Enable Windows console VT processing so RGB ANSI escapes render.
pub fn enable_windows_vt() {
    #[cfg(windows)]
    {
        type BOOL = i32;
        type DWORD = u32;
        type HANDLE = *mut std::ffi::c_void;
        extern "system" {
            fn GetStdHandle(n_std_handle: DWORD) -> HANDLE;
            fn SetConsoleMode(h_console_handle: HANDLE, dw_mode: DWORD) -> BOOL;
            fn GetConsoleMode(h_console_handle: HANDLE, lp_mode: *mut DWORD) -> BOOL;
        }
        const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5; // (DWORD)-11
        const ENABLE_VIRTUAL_TERMINAL_PROCESSING: DWORD = 0x0004;
        unsafe {
            let hout = GetStdHandle(STD_OUTPUT_HANDLE);
            if !hout.is_null() && hout != (-1isize as HANDLE) {
                let mut mode: DWORD = 0;
                if GetConsoleMode(hout, &mut mode) != 0 {
                    let _ = SetConsoleMode(hout, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
                }
            }
        }
        let _ = colored::control::set_virtual_terminal(true);
    }
}

/// gRPC compact: update one Shell row. Headless uses in-place `\r` instead of `info!`.
pub fn push_grpc_compact_upsert(key: String, ansi_text: &str) {
    let mut line = make_log_line(ansi_text);
    line.upsert_key = Some(key.clone());
    push_log_line(line);
    if !has_log_line_consumer() {
        mirror_compact_to_cli(&key, ansi_text);
    }
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
