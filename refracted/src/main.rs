#![cfg_attr(windows, windows_subsystem = "windows")]

use anyhow::Result;
use refracted::core::console::{
    init_global_buffer, init_log_line_sender, LogBuffer, LogLine, push_formatted_log_line,
};
use refracted::core::inspector::{
    init_global_packet_buffer, init_global_grpc_buffer, init_global_http_buffer,
    init_global_lsx_buffer,
    PacketBuffer, GrpcBuffer, HttpBuffer, LsxBuffer,
    render_toolkit, InspectorUiState, start_proxy_servers
};
use refracted::core::server::BlazeServer;
use eframe::egui;
use parking_lot::Mutex;
use std::io::{self, Write};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

// Embedded icon image from workspace root icon.png
const ICON_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../icon.png"));

// Embedded banner as base64
// The include_str! macro embeds the file content at compile time into the binary
// No external file is needed at runtime - the base64 string is embedded in the executable
const BANNER_BASE64: &str = include_str!("core/ui/banner_base64.txt");

const EMULATOR_LISTEN_HOST: &str = "0.0.0.0";

struct LogWriter {
    stdout: io::Stdout,
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        let lines: Vec<&str> = text.lines().collect();

        if !lines.is_empty() {
            for line in lines {
                if !line.trim().is_empty() {
                    let formatted_line = format!("{}\n", line);
                    let _ = self.stdout.write_all(formatted_line.as_bytes());
                    push_formatted_log_line(line);
                }
            }
            let _ = self.stdout.flush();
        } else {
            let _ = self.stdout.write_all(buf);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
}

/// Custom stdout writer that captures output to both stdout and our buffer
#[allow(dead_code)]
struct StdoutCapture {
    buffer: LogBuffer,
    inner: io::Stdout,
}

impl Write for StdoutCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to actual stdout
        let _ = self.inner.write_all(buf);
        let _ = self.inner.flush();
        
        // Also capture to buffer
        let text = String::from_utf8_lossy(buf);
        let lines: Vec<&str> = text.lines().collect();
        
        if !lines.is_empty() {
            let mut buffer = self.buffer.lock();
            for line in lines {
                if !line.trim().is_empty() {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs_f64();
                    
                    let (text, colors) = refracted::core::console::parse_ansi_codes(line);
                    // Build segments from colors
                    use eframe::egui;
                    let mut segments = Vec::new();
                    let mut last_pos = 0;
                    let mut current_color = egui::Color32::WHITE;
                    
                    for (pos, color) in &colors {
                        if *pos > last_pos {
                            segments.push((text[last_pos..*pos].to_string(), current_color));
                        }
                        current_color = *color;
                        last_pos = *pos;
                    }
                    if last_pos < text.len() {
                        segments.push((text[last_pos..].to_string(), current_color));
                    }
                    if segments.is_empty() {
                        segments.push((text.clone(), egui::Color32::WHITE));
                    }
                    
                    buffer.push(refracted::core::console::LogLine {
                        text,
                        colors,
                        segments,
                        timestamp,
                        upsert_key: None,
                    });
                }
            }
            let len = buffer.len();
            if len > 10000 {
                let remove_count = len - 10000;
                buffer.drain(0..remove_count);
            }
        }
        
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Draw a checkmark using egui primitives
fn draw_checkmark(ui: &mut egui::Ui, color: egui::Color32, size: f32) {
    let rect = ui.available_rect_before_wrap();
    let center = rect.center();
    let painter = ui.painter();
    
    let stroke = egui::Stroke::new(2.0, color);
    
    // Draw checkmark: two lines forming a check
    let start1 = egui::pos2(center.x - size * 0.3, center.y);
    let mid = egui::pos2(center.x - size * 0.1, center.y + size * 0.3);
    let end = egui::pos2(center.x + size * 0.3, center.y - size * 0.3);
    
    painter.line_segment([start1, mid], stroke);
    painter.line_segment([mid, end], stroke);
}

/// Draw an X using egui primitives
fn draw_x(ui: &mut egui::Ui, color: egui::Color32, size: f32) {
    let rect = ui.available_rect_before_wrap();
    let center = rect.center();
    let painter = ui.painter();
    
    let stroke = egui::Stroke::new(2.0, color);
    
    // Draw X: two diagonal lines
    let offset = size * 0.35;
    let top_left = egui::pos2(center.x - offset, center.y - offset);
    let top_right = egui::pos2(center.x + offset, center.y - offset);
    let bottom_left = egui::pos2(center.x - offset, center.y + offset);
    let bottom_right = egui::pos2(center.x + offset, center.y + offset);
    
    painter.line_segment([top_left, bottom_right], stroke);
    painter.line_segment([top_right, bottom_left], stroke);
}

fn draw_disclaimer_arrow_icon(
    ui: &egui::Ui,
    rect: egui::Rect,
    expanded: bool,
    hovered: bool,
) {
    let stroke_color = if hovered {
        egui::Color32::from_rgb(235, 235, 235)
    } else {
        egui::Color32::from_rgb(200, 200, 200)
    };
    let stroke = egui::Stroke::new(2.0, stroke_color);
    let c = rect.center();
    let w = 5.0;
    let h = 3.0;
    let (top, bottom) = if expanded {
        (
            egui::pos2(c.x - w, c.y + h),
            egui::pos2(c.x + w, c.y + h),
        )
    } else {
        (
            egui::pos2(c.x - w, c.y - h),
            egui::pos2(c.x + w, c.y - h),
        )
    };
    let apex = egui::pos2(c.x, if expanded { c.y - h } else { c.y + h });
    ui.painter().line_segment([top, apex], stroke);
    ui.painter().line_segment([apex, bottom], stroke);
}

struct RefractedApp {
    log_buffer: LogBuffer,
    /// Drained into `log_buffer` each frame so tokio never blocks on GUI mutex.
    log_rx: Receiver<LogLine>,
    packet_buffer: PacketBuffer,
    grpc_buffer: GrpcBuffer,
    http_buffer: HttpBuffer,
    lsx_buffer: LsxBuffer,
    server_running: bool,
    proxy_running: bool, // Track if proxy mode is active
    auto_scroll: bool,
    selected_tab: String,
    selected_blaze_session: Option<u64>,
    shutdown_tx: Option<broadcast::Sender<()>>,
    server_handle: Option<tokio::task::JoinHandle<()>>, // Track server task to wait for completion
    last_line_count: usize,
    last_startup_message: Option<String>, // Track last startup message to detect changes
    show_about: bool, // Show about window
    show_accounts: bool, // Show accounts window
    show_games: bool, // Show games window
    show_options: bool, // Show options window
    show_proxy: bool, // Show proxy settings window
    show_disclaimer_popup: bool,
    inspector_state: InspectorUiState, // Inspector UI state
    startup_maximize_pending: bool,
}

impl RefractedApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        log_buffer: LogBuffer,
        packet_buffer: PacketBuffer,
        log_rx: Receiver<LogLine>,
    ) -> Self {
        // Set global log buffer for stdout capture
        init_global_buffer(log_buffer.clone());
        
        // Set global packet buffer for packet capture
        init_global_packet_buffer(packet_buffer.clone());
        
        // Initialize inspector buffers
        let grpc_buffer = Arc::new(Mutex::new(Vec::new()));
        let http_buffer = Arc::new(Mutex::new(Vec::new()));
        let lsx_buffer = Arc::new(Mutex::new(Vec::new()));
        
        init_global_grpc_buffer(grpc_buffer.clone());
        init_global_http_buffer(http_buffer.clone());
        init_global_lsx_buffer(lsx_buffer.clone());
        
        // Configure egui style
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(12.0, egui::FontFamily::Monospace),
        );
        cc.egui_ctx.set_style(style);

        let _ = refracted::common::paths::ensure_app_data_dir();
        let settings_path = refracted::common::paths::settings_json_path();
        if let Err(e) = refracted::common::settings::init_settings(settings_path) {
            eprintln!("Failed to initialize settings: {}", e);
        } else {
            // Sync profile to session state
            refracted::common::user_profile::sync_profile_to_session();
            
            // Apply theme
            let settings = refracted::common::settings::get_settings();
            apply_theme(&cc.egui_ctx, &settings.app_settings.theme);
        }

        refracted::session::blaze_sessions::load_persisted_sessions();

        // Initialize inspector state
        let mut inspector_state = InspectorUiState::new();
        let proxy_settings = refracted::common::settings::get_proxy_settings();
        inspector_state.proxy_config_http_listen = proxy_settings.http_listen_port.to_string();
        inspector_state.proxy_config_https_listen = proxy_settings.https_listen_port.to_string();
        inspector_state.proxy_config_grpc_listen = proxy_settings.grpc_listen_port.to_string();
        inspector_state.proxy_config_blaze_listen = proxy_settings.blaze_listen_port.to_string();
        inspector_state.proxy_config_lsx_listen = proxy_settings.lsx_listen_port.to_string();
        inspector_state.proxy_config_target_host = proxy_settings.target_host.clone();
        inspector_state.proxy_config_target_http = proxy_settings.target_http_port.to_string();
        inspector_state.proxy_config_target_https = proxy_settings.target_https_port.to_string();
        inspector_state.proxy_config_target_grpc = proxy_settings.target_grpc_port.to_string();
        inspector_state.proxy_config_target_blaze = proxy_settings.target_blaze_port.to_string();
        inspector_state.proxy_config_target_lsx = proxy_settings.target_lsx_port.to_string();
        inspector_state.proxy_config_enable_http = proxy_settings.enable_http;
        inspector_state.proxy_config_enable_https = proxy_settings.enable_https;
        inspector_state.proxy_config_enable_grpc = proxy_settings.enable_grpc;
        inspector_state.proxy_config_enable_blaze = proxy_settings.enable_blaze;
        inspector_state.proxy_config_enable_lsx = proxy_settings.enable_lsx;

        Self {
            log_buffer,
            log_rx,
            packet_buffer,
            grpc_buffer,
            http_buffer,
            lsx_buffer,
            server_running: false,
            proxy_running: false,
            auto_scroll: true,
            selected_tab: "Shell".to_string(),
            selected_blaze_session: None,
            shutdown_tx: None,
            server_handle: None,
            last_line_count: 0,
            last_startup_message: None,
            show_about: false,
            show_accounts: false,
            show_games: false,
            show_options: false,
            show_proxy: false,
            show_disclaimer_popup: false,
            inspector_state,
            startup_maximize_pending: true,
        }
    }

    fn start_server(&mut self) {
        if self.server_running {
            return;
        }

        // Check if all required ports are available
        let ports_in_use =
            refracted::core::server::BlazeServer::check_all_ports(EMULATOR_LISTEN_HOST);
        if !ports_in_use.is_empty() {
            // Write error messages directly to log buffer (tracing not initialized yet)
            
            // Format matches CustomFormatter ERROR level ([ERROR] only, no [Console])
            let error_header = "\x1b[38;2;255;150;150m[ERROR]\x1b[0m The following ports are already in use:";
            println!("{}", error_header);
            refracted::core::console::capture_line(error_header);
            
            for (port, name) in &ports_in_use {
                let port_line = format!("  - Port {} ({})", port, name);
                println!("{}", port_line);
                refracted::core::console::capture_line(&port_line);
            }
            
            println!();
            refracted::core::console::capture_line("");
            
            let footer = "Please free these ports before starting the server.";
            println!("{}", footer);
            refracted::core::console::capture_line(footer);
            
            // Force stdout flush to ensure messages appear
            use std::io::Write;
            let _ = std::io::stdout().flush();
            
            // Don't start the server
            return;
        }

        self.server_running = true;

        // Spawn server in background
        let (shutdown_tx, shutdown_rx) = broadcast::channel(16);
        self.shutdown_tx = Some(shutdown_tx.clone());
        
        let handle = tokio::spawn(async move {
            // Initialize logging with custom writer and timestamp format
            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
                .add_directive("rustls=warn".parse().unwrap())
                .add_directive("h2=warn".parse().unwrap());

            let make_writer = move || {
                Box::new(LogWriter {
                    stdout: io::stdout(),
                }) as Box<dyn Write + Send>
            };

            // Custom formatter for INFO messages as [Console]
            use tracing_subscriber::fmt::format::{FormatEvent, FormatFields};
            use tracing_subscriber::fmt::time::FormatTime;
            use tracing_subscriber::registry::LookupSpan;
            
            struct CustomFormatter;
            #[allow(dead_code)]
            struct InfoTimer; // Empty timer for INFO
            struct OtherTimer; // Timer with brackets for other levels
            
            impl FormatTime for InfoTimer {
                fn format_time(&self, _w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
                    Ok(()) // No timestamp for INFO
                }
            }
            
            impl FormatTime for OtherTimer {
                fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
                    use std::time::SystemTime;
                    let now = SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap();
                    let secs = now.as_secs();
                    let nanos = now.subsec_nanos();
                    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, nanos)
                        .unwrap_or_default();
                    write!(w, "[{}]", datetime.format("%Y-%m-%dT%H:%M:%S%.6fZ"))
                }
            }
            
            impl<S, N> FormatEvent<S, N> for CustomFormatter
            where
                S: tracing::Subscriber + for<'a> LookupSpan<'a>,
                N: for<'a> FormatFields<'a> + 'static,
            {
                fn format_event(
                    &self,
                    ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
                    mut writer: tracing_subscriber::fmt::format::Writer<'_>,
                    event: &tracing::Event<'_>,
                ) -> std::fmt::Result {
                    let level = *event.metadata().level();
                    
                    // Capture the formatted message first to check if it has ANSI codes
                    let mut message_string = String::new();
                    {
                        let mut message_writer = tracing_subscriber::fmt::format::Writer::new(&mut message_string);
                        ctx.format_fields(message_writer.by_ref(), event)?;
                    }
                    
                    // Check if message contains ANSI escape codes or Blaze packet markers
                    // Check for actual escape char first, then literal string, then Blaze markers
                    let has_ansi_escape = message_string.contains('\x1b');
                    let has_literal_escape = message_string.contains("\\x1b");
                    let has_client_arrow = message_string.contains("[Client→");
                    let has_blaze_arrow = message_string.contains("[Blaze→");
                    let has_qos = message_string.contains("[QoS]");
                    let has_ansi = has_ansi_escape || has_literal_escape || has_client_arrow || has_blaze_arrow;
                    // INFO: ANSI first (so embedded [QoS] in colored debug lines is not prefixed again), then plain [QoS]
                    const QOS_TAG_GREEN: &str = "\x1b[38;2;80;200;120m[QoS]\x1b[0m";
                    const ERROR_TAG_RED: &str = "\x1b[38;2;255;150;150m[ERROR]\x1b[0m";
                    // Strip plain-text [QoS] prefix(es); must not run on ANSI-prefixed lines (those take has_ansi branch).
                    fn strip_plain_qos(s: &str) -> &str {
                        let mut t = s;
                        while let Some(x) = t.strip_prefix("[QoS]") {
                            t = x.trim_start();
                        }
                        t
                    }

                    if level == tracing::Level::INFO {
                        if has_client_arrow || has_blaze_arrow {
                            // Blaze packet log - write plain text without [Console] prefix
                            // Keep it simple - just write the message as-is
                            write!(writer, "{}", message_string)?;
                        } else if has_ansi {
                            // ANSI first: debug_println embeds [QoS] after escape codes — do not add a second green [QoS]
                            write!(writer, "{}", message_string)?;
                        } else if has_qos {
                            let rest = strip_plain_qos(message_string.as_str());
                            write!(writer, "{} {}", QOS_TAG_GREEN, rest)?;
                        } else {
                            // Format INFO as grey [Console] without timestamp
                            write!(writer, "\x1b[38;2;128;128;128m[Console]\x1b[0m {}", message_string)?;
                        }
                        writeln!(writer)
                    } else if level == tracing::Level::ERROR {
                        if has_ansi {
                            write!(writer, "{}", message_string)?;
                        } else if has_qos {
                            let rest = strip_plain_qos(message_string.as_str());
                            write!(writer, "{} {}", QOS_TAG_GREEN, rest)?;
                        } else {
                            write!(writer, "{} {}", ERROR_TAG_RED, message_string)?;
                        }
                        writeln!(writer)
                    } else if level == tracing::Level::WARN {
                        if has_ansi {
                            write!(writer, "{}", message_string)?;
                        } else if has_qos {
                            let rest = strip_plain_qos(message_string.as_str());
                            write!(writer, "{} {}", QOS_TAG_GREEN, rest)?;
                        } else {
                            // Format WARN as [Console] with WARN in yellow
                            write!(writer, "\x1b[38;2;128;128;128m[Console]\x1b[0m \x1b[38;2;255;200;0mWARN\x1b[0m {}", message_string)?;
                        }
                        writeln!(writer)
                    } else {
                        // Format other levels with timestamp
                        let timer = OtherTimer;
                        timer.format_time(&mut writer)?;
                        write!(writer, "  {} {}", level, message_string)?;
                        writeln!(writer)
                    }
                }
            }
            
            // Only initialize if not already set (prevents panic on restart)
            let _ = tracing_subscriber::fmt()
                .with_target(false)
                .with_env_filter(filter)
                .with_writer(make_writer)
                .event_format(CustomFormatter)
                .try_init();

            // Note: console_println! macro will capture all output automatically

            info!("Starting Refracted Emulator...");

            // Create and start the emulator
            match BlazeServer::new(EMULATOR_LISTEN_HOST.to_string()).await {
                Ok(mut emulator) => {
                    // Get the emulator's shutdown channel before moving it
                    let emulator_shutdown_tx = emulator.shutdown_tx.clone();
                    
                    // Spawn task to monitor our shutdown signal and forward to emulator
                    let mut shutdown_monitor = shutdown_rx.resubscribe();
                    let shutdown_forwarder = tokio::spawn(async move {
                        if shutdown_monitor.recv().await.is_ok() {
                            info!("Shutdown signal received, requesting emulator shutdown");
                            // Send shutdown signal to emulator
                            let _ = emulator_shutdown_tx.send(());
                        }
                    });
                    
                    // Start emulator (this will block until shutdown)
                    let emulator_result = emulator.start_emulator().await;
                    
                    // Cancel shutdown forwarder
                    shutdown_forwarder.abort();
                    
                    match emulator_result {
                        Err(e) => error!("Emulator error: {}", e),
                        Ok(_) => info!("Refracted Emulator has been shut down gracefully"),
                    }
                }
                Err(e) => {
                    error!("Failed to create emulator: {}", e);
                }
            }
        });
        
        self.server_handle = Some(handle);
    }

    fn reload_app(&self) {
        // Get the current executable path
        if let Ok(exe_path) = std::env::current_exe() {
            // Spawn a new instance of the application
            if let Err(e) = std::process::Command::new(&exe_path)
                .spawn()
            {
                error!("Failed to restart application: {}", e);
                return;
            }
            // Exit the current instance
            std::process::exit(0);
        } else {
            error!("Failed to get current executable path");
        }
    }

    fn stop_server(&mut self) {
        if !self.server_running {
            return;
        }

        // Send shutdown signal
        if let Some(ref shutdown_tx) = self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }

        self.server_running = false;
        self.server_handle = None;
    }

    fn start_proxy(&mut self) {
        if self.proxy_running {
            return;
        }

        self.proxy_running = true;
        
        // Update inspector state to research mode
        self.inspector_state.mode = refracted::core::inspector::InspectorMode::Research;
        
        // Update proxy config from UI state
        let mut config = self.inspector_state.proxy_state.config.lock();
        config.http_listen_port = self.inspector_state.proxy_config_http_listen.parse().unwrap_or(80);
        config.https_listen_port = self.inspector_state.proxy_config_https_listen.parse().unwrap_or(443);
        config.grpc_listen_port = self.inspector_state.proxy_config_grpc_listen.parse().unwrap_or(443);
        config.blaze_listen_port = self.inspector_state.proxy_config_blaze_listen.parse().unwrap_or(10042);
        config.lsx_listen_port = self.inspector_state.proxy_config_lsx_listen.parse().unwrap_or(3216);
        config.target_host = self.inspector_state.proxy_config_target_host.clone();
        config.target_http_port = self.inspector_state.proxy_config_target_http.parse().unwrap_or(80);
        config.target_https_port = self.inspector_state.proxy_config_target_https.parse().unwrap_or(443);
        config.target_grpc_port = self.inspector_state.proxy_config_target_grpc.parse().unwrap_or(443);
        config.target_blaze_port = self.inspector_state.proxy_config_target_blaze.parse().unwrap_or(10042);
        config.target_lsx_port = self.inspector_state.proxy_config_target_lsx.parse().unwrap_or(3216);
        config.enable_http = self.inspector_state.proxy_config_enable_http;
        config.enable_https = self.inspector_state.proxy_config_enable_https;
        config.enable_grpc = self.inspector_state.proxy_config_enable_grpc;
        config.enable_blaze = self.inspector_state.proxy_config_enable_blaze;
        config.enable_lsx = self.inspector_state.proxy_config_enable_lsx;
        drop(config);
        
        // Start proxy servers
        start_proxy_servers(self.inspector_state.proxy_state.clone());
        
        // Show hint in shell
        use refracted::core::console::capture_line;
        let hint = "\x1b[38;2;128;128;128m[Console]\x1b[0m \x1b[38;2;100;200;255mINFO\x1b[0m Started in Proxy mode - Use Toolkit tab to view intercepted traffic!";
        println!("{}", hint);
        capture_line(hint);
    }

    fn stop_proxy(&mut self) {
        if !self.proxy_running {
            return;
        }

        self.proxy_running = false;
        
        // Stop proxy servers
        self.inspector_state.proxy_state.stop();
        
        // Update inspector state to emulator mode
        self.inspector_state.mode = refracted::core::inspector::InspectorMode::Emulator;
    }

    fn show_toolkit(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        render_toolkit(
            ctx,
            ui,
            &mut self.inspector_state,
            self.packet_buffer.clone(),
            self.grpc_buffer.clone(),
            self.http_buffer.clone(),
            self.lsx_buffer.clone(),
        );
    }

    fn show_sessions(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        use chrono::TimeZone;
        use refracted::session::blaze_sessions;
        use refracted::session::session_module::{
            last_fetch_client_config, BLAZE_SERVER_VERSION_LABEL,
        };

        let sessions = blaze_sessions::list_sessions();
        let active_n = sessions.len();
        let auth_n = sessions.iter().filter(|s| s.authenticated).count();
        if let Some(sel) = self.selected_blaze_session {
            if !sessions.iter().any(|s| s.id == sel) {
                self.selected_blaze_session = None;
            }
        }

        ui.horizontal(|ui| {
            ui.heading("Blaze sessions");
            ui.label(format!(
                "Active: {} · Authenticated: {}",
                active_n, auth_n
            ));
        });
        ui.separator();

        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.label(egui::RichText::new("Connections").heading());
                egui::ScrollArea::vertical()
                    .id_source("blaze_sessions_list")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if sessions.is_empty() {
                            ui.label("No active Blaze clients.");
                            return;
                        }
                        for s in &sessions {
                            let sel = self.selected_blaze_session == Some(s.id);
                            let name = s
                                .display_name
                                .as_deref()
                                .map(str::trim)
                                .filter(|n| !n.is_empty())
                                .unwrap_or("—");
                            let label = format!("#{} {} {}", s.id, name, s.peer);
                            if ui.selectable_label(sel, label).clicked() {
                                self.selected_blaze_session = Some(s.id);
                            }
                        }
                    });
            });

            cols[1].vertical(|ui| {
                ui.label(egui::RichText::new("Details").heading());
                ui.separator();
                let detail = self
                    .selected_blaze_session
                    .and_then(blaze_sessions::get_session);
                if let Some(s) = detail {
                    let when = chrono::Utc
                        .timestamp_opt(s.connected_unix_secs as i64, 0)
                        .single()
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| s.connected_unix_secs.to_string());

                    ui.label(format!("Session ID: {}", s.id));
                    ui.label(format!("Peer: {}", s.peer));
                    ui.label(format!("Connected: {}", when));
                    ui.label(format!(
                        "Authenticated: {}",
                        if s.authenticated { "yes" } else { "no" }
                    ));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("User").weak());
                    if let Some(ref n) = s.display_name {
                        ui.label(format!("Username: {}", n));
                    } else {
                        ui.label("Username: —");
                    }
                    if let Some(uid) = s.user_id {
                        ui.label(format!("UID: {}", uid));
                    }
                    if let Some(pid) = s.persona_id {
                        ui.label(format!("PID: {}", pid));
                    }
                    if let Some(ref e) = s.email {
                        ui.label(format!("Email: {}", e));
                    }
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Client").weak());
                    ui.label(format!("Profile: {}", s.build_profile));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("CONF").weak());
                    let conf = last_fetch_client_config();
                    ui.label(format!(
                        "CFID: {}",
                        if conf.cfid.is_empty() {
                            "—".to_string()
                        } else {
                            conf.cfid
                        }
                    ));
                    ui.label(format!(
                        "Tenancy: {}",
                        if conf.client_grpc_tenancy.is_empty() {
                            "—".to_string()
                        } else {
                            conf.client_grpc_tenancy
                        }
                    ));
                    let url_disp = if conf.client_grpc_url.is_empty() {
                        "—".to_string()
                    } else {
                        let u = conf.client_grpc_url.as_str();
                        const MAX: usize = 72;
                        if u.len() <= MAX {
                            u.to_string()
                        } else {
                            let mut cut = MAX - 1;
                            while cut > 0 && !u.is_char_boundary(cut) {
                                cut -= 1;
                            }
                            format!("{}…", &u[..cut])
                        }
                    };
                    ui.label(format!("Gateway: {}", url_disp));
                    ui.label(format!("Build: {}", BLAZE_SERVER_VERSION_LABEL));
                    if !s.build_source.is_empty() {
                        ui.label(format!("Detection: {}", s.build_source));
                    }
                } else {
                    ui.label("Select a session to view details.");
                }
            });
        });
    }

    fn collect_shell_output_text(&self) -> String {
        let buffer = self.log_buffer.lock();
        let mut full_text = String::new();
        for line in buffer.iter() {
            for (text, _) in &line.segments {
                full_text.push_str(text);
            }
            full_text.push('\n');
        }
        full_text
    }
}

impl eframe::App for RefractedApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.startup_maximize_pending {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            self.startup_maximize_pending = false;
        }

        while let Ok(line) = self.log_rx.try_recv() {
            let mut buf = self.log_buffer.lock();
            if let Some(ref k) = line.upsert_key {
                if let Some(i) = buf.iter().rposition(|l| l.upsert_key.as_ref() == Some(k)) {
                    buf[i] = line;
                } else {
                    buf.push(line);
                }
            } else {
                buf.push(line);
            }
            let len = buf.len();
            if len > 10000 {
                buf.drain(0..len - 10000);
            }
        }

        // Continuous repaint for smooth updates - request repaint every frame
        // This ensures log lines appear immediately without delay
        ctx.request_repaint();

        if let Some(line) = refracted::common::dev_env_banner::dev_env_banner_line() {
            egui::TopBottomPanel::top("refracted_dev_env_banner")
                .exact_height(24.0)
                .frame(
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(72, 58, 18))
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0)),
                )
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            egui::RichText::new("DEV ENV")
                                .strong()
                                .color(egui::Color32::from_rgb(255, 190, 90)),
                        );
                        ui.label(
                            egui::RichText::new(line)
                                .family(egui::FontFamily::Monospace)
                                .color(egui::Color32::from_rgb(255, 245, 210)),
                        );
                    });
                });
        }
        
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Actions", |ui| {
                    // Start Emulator button (disabled when proxy is running)
                    let emulator_response = ui.add_enabled(!self.proxy_running, |ui: &mut egui::Ui| {
                        if self.server_running {
                            ui.button("Reload App")
                        } else {
                            ui.button("Start Emulator")
                        }
                    });
                    if emulator_response.clicked() {
                        if self.server_running {
                            self.reload_app();
                        } else {
                            // Stop proxy if running
                            if self.proxy_running {
                                self.stop_proxy();
                            }
                            self.start_server();
                        }
                        ui.close_menu();
                    }
                    
                    // Start Proxy button (disabled when emulator is running)
                    let proxy_response = ui.add_enabled(!self.server_running, |ui: &mut egui::Ui| {
                        if self.proxy_running {
                            ui.button("Stop Proxy")
                        } else {
                            ui.button("Start Proxy")
                        }
                    });
                    if proxy_response.clicked() {
                        if self.proxy_running {
                            self.stop_proxy();
                        } else {
                            // Stop emulator if running
                            if self.server_running {
                                self.stop_server();
                            }
                            self.start_proxy();
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Copy to Clipboard").clicked() {
                        let buffer = self.log_buffer.lock();
                        let mut full_text = String::new();
                        for line in buffer.iter() {
                            for (text, _) in &line.segments {
                                full_text.push_str(text);
                            }
                            full_text.push('\n');
                        }
                        ctx.copy_text(full_text);
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Settings", |ui| {
                    if ui.button("Accounts").clicked() {
                        self.show_accounts = true;
                        ui.close_menu();
                    }
                    if ui.button("Games").clicked() {
                        self.show_games = true;
                        ui.close_menu();
                    }
                    if ui.button("Options").clicked() {
                        self.show_options = true;
                        ui.close_menu();
                    }
                    if ui.button("Proxy").clicked() {
                        self.show_proxy = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("About", |ui| {
                    if ui.button("About Us").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // About window
        if self.show_about {
            let mut is_open = self.show_about;
            about_window(ctx, &mut is_open);
            self.show_about = is_open;
        }
        
        // Accounts window
        if self.show_accounts {
            let mut is_open = self.show_accounts;
            accounts_window(ctx, &mut is_open);
            self.show_accounts = is_open;
        }
        
        // Games window
        if self.show_games {
            let mut is_open = self.show_games;
            games_window(ctx, &mut is_open);
            self.show_games = is_open;
        }
        
        // Options window
        if self.show_options {
            let mut is_open = self.show_options;
            options_window(ctx, &mut is_open);
            self.show_options = is_open;
        }
        
        // Proxy settings window
        if self.show_proxy {
            let mut is_open = self.show_proxy;
            proxy_window(ctx, &mut is_open, &mut self.inspector_state);
            self.show_proxy = is_open;
        }

        if self.show_disclaimer_popup {
            let mut is_open = self.show_disclaimer_popup;
            disclaimer_window(ctx, &mut is_open);
            self.show_disclaimer_popup = is_open;
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Header
                ui.horizontal(|ui| {
                    let title = if let Some(current_game) = refracted::common::game::get_current_game() {
                        format!("Refracted: {}", current_game.name)
                    } else {
                        "Refracted".to_string()
                    };
                    ui.heading(title);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.proxy_running {
                            // Draw checkmark for proxy mode
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(20.0, 20.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    draw_checkmark(ui, egui::Color32::from_rgb(100, 200, 255), 12.0);
                                },
                            );
                            ui.label(egui::RichText::new("Proxy Mode").color(egui::Color32::from_rgb(100, 200, 255)));
                        } else if self.server_running {
                            // Draw checkmark
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(20.0, 20.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    draw_checkmark(ui, egui::Color32::GREEN, 12.0);
                                },
                            );
                            ui.label("Running");
                        } else {
                            // Draw X
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(20.0, 20.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    draw_x(ui, egui::Color32::RED, 12.0);
                                },
                            );
                            ui.label("Stopped");
                        }
                    });
                });

                ui.separator();

                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_tab, "Shell".to_string(), "Shell");
                    ui.selectable_value(&mut self.selected_tab, "Toolkit".to_string(), "Toolkit");
                    ui.label(egui::RichText::new("|").weak());
                    ui.selectable_value(&mut self.selected_tab, "Sessions".to_string(), "Sessions");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.selected_tab == "Shell" {
                            if ui.button("Save As").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Save Shell Output")
                                    .set_file_name("shell-output.txt")
                                    .save_file()
                                {
                                    let full_text = self.collect_shell_output_text();
                                    if let Err(e) = std::fs::write(&path, full_text) {
                                        println!("\x1b[38;2;255;150;150m[Console]\x1b[0m Failed to save shell output: {}", e);
                                    } else {
                                        println!("\x1b[38;2;100;200;255m[Console]\x1b[0m Shell output saved to {}", path.display());
                                    }
                                }
                            }
                            if ui.button("📋").on_hover_text("Copy to clipboard").clicked() {
                                ctx.copy_text(self.collect_shell_output_text());
                            }
                        }
                    });
                });

                ui.separator();

                // Show content based on selected tab
                if self.selected_tab == "Shell" {
                    // Shell tab content - reserve space so footer is always visible.
                    let footer_height = 52.0;
                    let scroll_height = (ui.available_height() - footer_height).max(0.0);

                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), scroll_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .stick_to_bottom(self.auto_scroll)
                                .show(ui, |ui| {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let current_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs_f64();
                        
                        // Check for startup progress message
                        let startup_msg = refracted::common::startup_progress::get_current_startup_message();
                        
                        // If startup message changed, request repaint
                        if startup_msg != self.last_startup_message {
                            self.last_startup_message = startup_msg.clone();
                            ctx.request_repaint();
                        }
                        
                        // Show startup progress message if active (single updating line)
                        if let Some(ref msg) = startup_msg {
                            let alpha_u8 = 255;
                            let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha_u8);
                            ui.label(
                                egui::RichText::new(msg)
                                    .family(egui::FontFamily::Monospace)
                                    .size(11.0)
                                    .color(color),
                            );
                            return; // Don't show log buffer during startup
                        }
                        
                        // Show proxy mode hint if in proxy mode
                        if self.proxy_running {
                            let hint_color = egui::Color32::from_rgba_unmultiplied(100, 200, 255, 255);
                            ui.label(
                                egui::RichText::new("Started in Proxy mode - Use Toolkit tab to view intercepted traffic!")
                                    .family(egui::FontFamily::Monospace)
                                    .size(11.0)
                                    .color(hint_color),
                            );
                            ui.add_space(5.0);
                            ui.separator();
                            ui.add_space(5.0);
                        }

                        let debug_logging_enabled =
                            refracted::common::settings::get_app_settings().debug_logging;
                        let is_light_mode =
                            refracted::common::settings::get_app_settings().theme == "light";

                        let shell_lines: Vec<LogLine> = {
                            let buffer = self.log_buffer.lock();
                            let current_line_count = buffer.len();
                            if current_line_count > self.last_line_count {
                                ctx.request_repaint();
                                self.last_line_count = current_line_count;
                            }
                            buffer.iter().cloned().collect()
                        };

                        // Helper function to invert color for light mode
                        // Only invert white/light text colors, keep colored tags as-is
                        let invert_color = |color: egui::Color32| -> egui::Color32 {
                            if is_light_mode {
                                // Check if color is white (255, 255, 255) or very close to white
                                // Only invert pure white or very light gray text, keep all colored tags unchanged
                                let is_white_text = (color.r() == 255 && color.g() == 255 && color.b() == 255) ||
                                                   (color.r() > 240 && color.g() > 240 && color.b() > 240);
                                
                                if is_white_text {
                                    // Invert white text to black
                                    egui::Color32::from_rgb(0, 0, 0)
                                } else {
                                    // Keep colored tags unchanged (all non-white colors)
                                    color
                                }
                            } else {
                                color
                            }
                        };
                        
                        for line in &shell_lines {
                            // Filter [Blaze] debug logs if debug logging is disabled
                            if !debug_logging_enabled && line.text.contains("[Blaze]") {
                                continue;
                            }
                            
                            // Calculate fade-in alpha (fade in over 0.3 seconds)
                            let age = current_time - line.timestamp;
                            let alpha = (age / 0.3).min(1.0);
                            
                            // Check if this is a Blaze log line (has [Client→Blaze] or [Blaze→Client])
                            let is_blaze_log = line.text.contains("[Client→Blaze]") || line.text.contains("[Blaze→Client]");
                            
                            // Render line with color segments
                            ui.horizontal(|ui| {
                                let alpha_u8 = (255.0 * alpha) as u8;
                                
                                if is_blaze_log {
                                    // Special handling for Blaze logs - apply colors directly in GUI
                                    // Format: [Client (orange) → (white) Blaze] (orange) rest
                                    // Don't invert colored tags (orange), only invert white text
                                    let blaze_orange = egui::Color32::from_rgb(254, 60, 0); // Keep orange as-is
                                    let blaze_orange_alpha = egui::Color32::from_rgba_unmultiplied(
                                        blaze_orange.r(), blaze_orange.g(), blaze_orange.b(), alpha_u8
                                    );
                                    let white_base = if is_light_mode {
                                        egui::Color32::from_rgb(0, 0, 0) // Black in light mode
                                    } else {
                                        egui::Color32::from_rgb(255, 255, 255) // White in dark mode
                                    };
                                    let white_alpha = egui::Color32::from_rgba_unmultiplied(
                                        white_base.r(), white_base.g(), white_base.b(), alpha_u8
                                    );
                                    
                                    if line.text.contains("[Client→Blaze]") {
                                        // Split at [Client→Blaze]
                                        if let Some(pos) = line.text.find("[Client→Blaze]") {
                                            // Text before marker
                                            if pos > 0 {
                                                ui.label(
                                                    egui::RichText::new(&line.text[..pos])
                                                        .family(egui::FontFamily::Monospace)
                                                        .size(11.0)
                                                        .color(white_alpha),
                                                );
                                            }
                                            // [Client (orange)
                                            ui.label(
                                                egui::RichText::new("[Client")
                                                    .family(egui::FontFamily::Monospace)
                                                    .size(11.0)
                                                    .color(blaze_orange_alpha),
                                            );
                                            // → (white)
                                            ui.label(
                                                egui::RichText::new("→")
                                                    .family(egui::FontFamily::Monospace)
                                                    .size(11.0)
                                                    .color(white_alpha),
                                            );
                                            // Blaze] (orange)
                                            ui.label(
                                                egui::RichText::new("Blaze]")
                                                    .family(egui::FontFamily::Monospace)
                                                    .size(11.0)
                                                    .color(blaze_orange_alpha),
                                            );
                                            // Rest of message (white)
                                            let rest_start = pos + "[Client→Blaze]".len();
                                            if rest_start < line.text.len() {
                                                let rest = &line.text[rest_start..];
                                                let gray_base = egui::Color32::from_rgb(140, 140, 140);
                                                let gray_alpha = egui::Color32::from_rgba_unmultiplied(
                                                    gray_base.r(), gray_base.g(), gray_base.b(), alpha_u8
                                                );
                                                if let Some((base, suffix_digits)) = rest.rsplit_once(" x") {
                                                    if !suffix_digits.is_empty() && suffix_digits.chars().all(|c| c.is_ascii_digit()) {
                                                        ui.label(
                                                            egui::RichText::new(base)
                                                                .family(egui::FontFamily::Monospace)
                                                                .size(11.0)
                                                                .color(white_alpha),
                                                        );
                                                        ui.label(
                                                            egui::RichText::new(format!(" x{}", suffix_digits))
                                                                .family(egui::FontFamily::Monospace)
                                                                .size(11.0)
                                                                .color(gray_alpha),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            egui::RichText::new(rest)
                                                                .family(egui::FontFamily::Monospace)
                                                                .size(11.0)
                                                                .color(white_alpha),
                                                        );
                                                    }
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new(rest)
                                                            .family(egui::FontFamily::Monospace)
                                                            .size(11.0)
                                                            .color(white_alpha),
                                                    );
                                                }
                                            }
                                        }
                                    } else if line.text.contains("[Blaze→Client]") {
                                        // Split at [Blaze→Client]
                                        if let Some(pos) = line.text.find("[Blaze→Client]") {
                                            // Text before marker
                                            if pos > 0 {
                                                ui.label(
                                                    egui::RichText::new(&line.text[..pos])
                                                        .family(egui::FontFamily::Monospace)
                                                        .size(11.0)
                                                        .color(white_alpha),
                                                );
                                            }
                                            // [Blaze (orange)
                                            ui.label(
                                                egui::RichText::new("[Blaze")
                                                    .family(egui::FontFamily::Monospace)
                                                    .size(11.0)
                                                    .color(blaze_orange_alpha),
                                            );
                                            // → (white)
                                            ui.label(
                                                egui::RichText::new("→")
                                                    .family(egui::FontFamily::Monospace)
                                                    .size(11.0)
                                                    .color(white_alpha),
                                            );
                                            // Client] (orange)
                                            ui.label(
                                                egui::RichText::new("Client]")
                                                    .family(egui::FontFamily::Monospace)
                                                    .size(11.0)
                                                    .color(blaze_orange_alpha),
                                            );
                                            // Rest of message (white)
                                            let rest_start = pos + "[Blaze→Client]".len();
                                            if rest_start < line.text.len() {
                                                let rest = &line.text[rest_start..];
                                                let gray_base = egui::Color32::from_rgb(140, 140, 140);
                                                let gray_alpha = egui::Color32::from_rgba_unmultiplied(
                                                    gray_base.r(), gray_base.g(), gray_base.b(), alpha_u8
                                                );
                                                if let Some((base, suffix_digits)) = rest.rsplit_once(" x") {
                                                    if !suffix_digits.is_empty() && suffix_digits.chars().all(|c| c.is_ascii_digit()) {
                                                        ui.label(
                                                            egui::RichText::new(base)
                                                                .family(egui::FontFamily::Monospace)
                                                                .size(11.0)
                                                                .color(white_alpha),
                                                        );
                                                        ui.label(
                                                            egui::RichText::new(format!(" x{}", suffix_digits))
                                                                .family(egui::FontFamily::Monospace)
                                                                .size(11.0)
                                                                .color(gray_alpha),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            egui::RichText::new(rest)
                                                                .family(egui::FontFamily::Monospace)
                                                                .size(11.0)
                                                                .color(white_alpha),
                                                        );
                                                    }
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new(rest)
                                                            .family(egui::FontFamily::Monospace)
                                                            .size(11.0)
                                                            .color(white_alpha),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Normal log line - use existing color segments
                                    for (text, color) in &line.segments {
                                        if !text.is_empty() {
                                            let inverted_color = invert_color(*color);
                                            let seg_color = egui::Color32::from_rgba_unmultiplied(
                                                inverted_color.r(),
                                                inverted_color.g(),
                                                inverted_color.b(),
                                                alpha_u8,
                                            );
                                            ui.label(
                                                egui::RichText::new(text)
                                                    .family(egui::FontFamily::Monospace)
                                                    .size(11.0)
                                                    .color(seg_color),
                                            );
                                        }
                                    }
                                }
                            });
                        }
                    });
                        },
                    );

                    // Footer controls
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let buffer = self.log_buffer.lock();
                            ui.label(format!("Lines: {}", buffer.len()));
                        });
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(
                                "Refracted is not endorsed, sponsored, or affiliated with Electronic Arts Inc. (\"EA\").",
                            )
                            .size(11.0)
                            .color(egui::Color32::from_rgb(220, 220, 220)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (rect, response) = ui.allocate_exact_size(
                                egui::Vec2::new(22.0, 18.0),
                                egui::Sense::click(),
                            );
                            if response.clicked() {
                                self.show_disclaimer_popup = !self.show_disclaimer_popup;
                            }
                            let bg = if response.hovered() {
                                egui::Color32::from_rgb(48, 48, 48)
                            } else {
                                egui::Color32::from_rgb(34, 34, 34)
                            };
                            ui.painter().rect(
                                rect,
                                3.0,
                                bg,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
                            );
                            draw_disclaimer_arrow_icon(
                                ui,
                                rect.shrink(3.0),
                                self.show_disclaimer_popup,
                                response.hovered(),
                            );
                            response.on_hover_text("Show legal disclaimer details");
                        });
                    });
                } else if self.selected_tab == "Sessions" {
                    self.show_sessions(ctx, ui);
                } else {
                    self.show_toolkit(ctx, ui);
                }
            });
        });
    }
}

/// Load icon from embedded image bytes
fn load_icon() -> Result<egui::IconData> {
    let image = image::load_from_memory(ICON_BYTES)?;
    let rgba = image.to_rgba8();
    let size = rgba.dimensions();
    let pixels = rgba.into_raw();
    
    Ok(egui::IconData {
        rgba: pixels,
        width: size.0,
        height: size.1,
    })
}

/// Load banner image from embedded base64 data
fn load_banner() -> Option<egui::ColorImage> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    
    // Decode base64 banner
    // Remove UTF-8 BOM (Byte Order Mark) if present
    let trimmed = BANNER_BASE64.trim_start_matches('\u{FEFF}').trim();
    
    if trimmed.is_empty() {
        return None;
    }
    
    let banner_bytes = STANDARD.decode(trimmed).ok()?;
    
    if banner_bytes.is_empty() {
        return None;
    }
    
    let image = image::load_from_memory(&banner_bytes).ok()?;
    let rgba = image.to_rgba8();
    let size = rgba.dimensions();
    let pixels = rgba.into_raw();
    
    Some(egui::ColorImage::from_rgba_unmultiplied([size.0 as usize, size.1 as usize], &pixels))
}

/// Get version from Cargo.toml at runtime
fn get_version_from_cargo_toml() -> Option<String> {
    use std::fs;
    use std::path::Path;
    
    let cargo_toml_path = Path::new("Cargo.toml");
    if let Ok(contents) = fs::read_to_string(cargo_toml_path) {
        // Simple parsing - look for version = "x.y.z"
        for line in contents.lines() {
            let line = line.trim();
            if line.starts_with("version") {
                if let Some(start) = line.find('"') {
                    let start = start + 1;
                    if let Some(end) = line[start..].find('"') {
                        return Some(line[start..start + end].to_string());
                    }
                }
            }
        }
    }
    None
}

/// About window
fn about_window(ctx: &egui::Context, open: &mut bool) {
    let mut should_close = false;
    
    egui::Window::new("About Us")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .default_width(550.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Banner image - display at top
                // Cache the texture using OnceLock to avoid reloading every frame
                use std::sync::OnceLock;
                static BANNER_TEXTURE: OnceLock<Option<(egui::TextureHandle, f32, f32)>> = OnceLock::new();
                
                let banner_data = BANNER_TEXTURE.get_or_init(|| {
                    load_banner().map(|banner_image| {
                        let width = banner_image.width() as f32;
                        let height = banner_image.height() as f32;
                        let texture = ctx.load_texture("about_banner", banner_image, Default::default());
                        (texture, width, height)
                    })
                });
                
                if let Some((ref texture, width, height)) = banner_data {
                    let aspect_ratio = *width / *height;
                    let display_width = 450.0;
                    let display_height = display_width / aspect_ratio;
                    ui.image((texture.id(), egui::Vec2::new(display_width, display_height)));
                    ui.add_space(15.0);
                } else {
                    // Banner failed to load
                    ui.label(egui::RichText::new("⚠ Banner image not found").color(egui::Color32::YELLOW));
                    ui.add_space(15.0);
                }
                
                ui.label("A modern lightweight service layer emulator.");
                ui.add_space(10.0);
                
                // Version - read from Cargo.toml at runtime
                let version = get_version_from_cargo_toml().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
                ui.label(format!("Version: {}", version));
                ui.add_space(10.0);
                
                ui.separator();
                ui.add_space(10.0);
                
                // About section
                ui.label(egui::RichText::new("About").heading());
                ui.label("Refracted is an emulator for Frostbite titles and their service layers, providing local development and testing capabilities.");
                ui.add_space(10.0);
                
                // Disclaimer
                ui.label(egui::RichText::new("Disclaimer").heading());
                ui.label(egui::RichText::new("Refracted is not endorsed, sponsored, or affiliated with Electronic Arts Inc. (\"EA\") or related companies in any way. This is a educational project and not intended for any unauthorized use.").color(egui::Color32::from_rgb(200, 100, 100)));
                ui.add_space(10.0);
                
                // Credits
                ui.label(egui::RichText::new("Credits").heading());
                ui.label("Built with Rust, egui, and tokio");
                ui.label("Xevrac");
                ui.add_space(10.0);
                
                ui.separator();
                ui.add_space(10.0);
                
                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });
        });
    if should_close {
        *open = false;
    }
}

fn disclaimer_window(ctx: &egui::Context, open: &mut bool) {
    let mut should_close = false;

    egui::Window::new("Legal Disclaimer")
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_width(720.0)
        .default_height(220.0)
        .show(ctx, |ui| {
            ui.label(
                "This passion project is created solely for preservation and educational purposes. It is not intended for commercial use. All intellectual property rights, trademarks, and copyrights related to games belong to their respective owners, including but not limited to Electronic Arts Inc. (\"EA\") and its subsidiaries.",
            );
            ui.add_space(8.0);
            ui.label(
                "This project is not endorsed, sponsored, or affiliated with EA. It is an independent educational initiative aiming to explore and understand various programming and development concepts in an effort to preserve video games over time.",
            );
            ui.add_space(12.0);
            if ui.button("Close").clicked() {
                should_close = true;
            }
        });

    if should_close {
        *open = false;
    }
}

fn accounts_window(ctx: &egui::Context, open: &mut bool) {
    let mut should_close = false;
    
    // Use egui data to persist state across frames
    let show_advanced = ctx.data(|data| {
        data.get_temp::<bool>(egui::Id::new("show_advanced")).unwrap_or(false)
    });
    let mut show_advanced = show_advanced;
    
    let new_profile_name = ctx.data(|data| {
        data.get_temp::<String>(egui::Id::new("new_profile_name")).unwrap_or_default()
    });
    let mut new_profile_name = new_profile_name;
    
    let show_add_profile = ctx.data(|data| {
        data.get_temp::<bool>(egui::Id::new("show_add_profile")).unwrap_or(false)
    });
    let mut show_add_profile = show_add_profile;
    
    egui::Window::new("Accounts")
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_size([600.0, 500.0])
        .show(ctx, |ui| {
            let profiles = refracted::common::user_profile::get_profiles();
            let current_profile_name = profiles.current_profile.clone();
            // Profile changes desync the live Blaze session: the client already
            // baked DSNM/PID/MAIL into its UserManager during login, so we lock
            // the editor while any authenticated session is alive.
            let session_locked =
                refracted::session::blaze_sessions::authenticated_count() > 0;
            
            ui.vertical(|ui| {
                ui.heading("User Profiles");
                ui.label(
                    egui::RichText::new(
                        "Nucleus layer for profile data use when emulating Blaze.",
                    )
                    .small()
                    .weak(),
                );
                ui.add_space(4.0);

                if session_locked {
                    ui.add_space(4.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(60, 40, 0))
                        .rounding(4.0)
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 200, 80),
                                "Profile is locked while a game client is authenticated. \
                                 Disconnect the client to edit or switch profiles.",
                            );
                        });
                    ui.add_space(6.0);
                }
                
                // Profile selector
                ui.horizontal(|ui| {
                    ui.label("Current Profile:");
                    ui.add_enabled_ui(!session_locked, |ui| {
                        egui::ComboBox::from_id_source("profile_selector")
                            .selected_text(&current_profile_name)
                            .show_ui(ui, |ui| {
                                for (name, _) in &profiles.profiles {
                                    if ui.selectable_label(name == &current_profile_name, name).clicked() {
                                        if let Err(e) = refracted::common::user_profile::set_current_profile(name) {
                                            eprintln!("Failed to set current profile: {}", e);
                                        } else {
                                            refracted::common::user_profile::sync_profile_to_session();
                                        }
                                    }
                                }
                            });
                    });
                    
                    if ui.add_enabled(!session_locked, egui::Button::new("Add Profile")).clicked() {
                        show_add_profile = true;
                        new_profile_name.clear();
                    }
                });
                
                ui.add_space(10.0);
                
                // Current profile editor
                if let Some(selected_profile) = profiles.profiles.get(&current_profile_name) {
                    let edit_profile_name_id = egui::Id::new("accounts_edit_profile_name");
                    let edit_profile_id = egui::Id::new("accounts_edit_profile");
                    let edit_profile_changed_id = egui::Id::new("accounts_edit_profile_changed");

                    let mut edit_profile_name = ctx
                        .data(|data| data.get_temp::<String>(edit_profile_name_id))
                        .unwrap_or_default();
                    let mut profile = ctx
                        .data(|data| {
                            data.get_temp::<refracted::common::user_profile::UserProfile>(
                                edit_profile_id,
                            )
                        })
                        .unwrap_or_else(|| selected_profile.clone());
                    let mut profile_changed = ctx
                        .data(|data| data.get_temp::<bool>(edit_profile_changed_id))
                        .unwrap_or(false);

                    if edit_profile_name != current_profile_name {
                        edit_profile_name = current_profile_name.clone();
                        profile = selected_profile.clone();
                        profile_changed = false;
                    }
                    
                    ui.label(egui::RichText::new("Profile Settings").heading());
                    
                    // Username - greyed out, matches profile name
                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        let mut username_display = current_profile_name.clone();
                        ui.add(egui::TextEdit::singleline(&mut username_display)
                            .interactive(false)
                            .desired_width(200.0));
                        ui.label(egui::RichText::new("(from profile name)").small().weak());
                    });
                    
                    // Display Name - greyed out, matches profile name
                    ui.horizontal(|ui| {
                        ui.label("Display Name:");
                        let mut display_name_display = current_profile_name.clone();
                        ui.add(egui::TextEdit::singleline(&mut display_name_display)
                            .interactive(false)
                            .desired_width(200.0));
                        ui.label(egui::RichText::new("(from profile name)").small().weak());
                    });
                    
                    // Email - editable field (read-only while client session is live)
                    ui.horizontal(|ui| {
                        ui.label("Email:");
                        ui.add_enabled_ui(!session_locked, |ui| {
                            if ui.text_edit_singleline(&mut profile.email).changed() {
                                profile_changed = true;
                            }
                        });
                    });
                    
                    ui.add_space(5.0);
                    ui.checkbox(&mut show_advanced, "Show Advanced Settings");
                    
                    if show_advanced {
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("Advanced Settings").heading());
                        
                        ui.add_enabled_ui(!session_locked, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("User ID");
                                ui.label(egui::RichText::new("(UID)").color(egui::Color32::from_rgb(100, 150, 255)));
                                ui.label(":");
                                let mut user_id_str = profile.user_id.to_string();
                                if ui.text_edit_singleline(&mut user_id_str).changed() {
                                    if let Ok(uid) = user_id_str.parse::<u64>() {
                                        profile.user_id = uid;
                                        profile_changed = true;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Persona ID");
                                ui.label(egui::RichText::new("(PID)").color(egui::Color32::from_rgb(100, 150, 255)));
                                ui.label(":");
                                let mut persona_id_str = profile.persona_id.to_string();
                                if ui.text_edit_singleline(&mut persona_id_str).changed() {
                                    if let Ok(pid) = persona_id_str.parse::<u64>() {
                                        profile.persona_id = pid;
                                        profile_changed = true;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Display Name");
                                ui.label(egui::RichText::new("(DSNM)").color(egui::Color32::from_rgb(100, 150, 255)));
                                ui.label(":");
                                let mut display_name_display = current_profile_name.clone();
                                ui.add(egui::TextEdit::singleline(&mut display_name_display)
                                    .interactive(false)
                                    .desired_width(200.0));
                                ui.label(egui::RichText::new("(from profile name)").small().weak());
                            });

                            ui.horizontal(|ui| {
                                ui.label("PSID");
                                ui.label(egui::RichText::new("(PSID)").color(egui::Color32::from_rgb(100, 150, 255)));
                                ui.label(":");
                                let mut psid_str = profile.psid.to_string();
                                if ui.text_edit_singleline(&mut psid_str).changed() {
                                    if let Ok(psid) = psid_str.parse::<u32>() {
                                        profile.psid = psid;
                                        profile_changed = true;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("AUSRC");
                                ui.label(egui::RichText::new("(AUSRC)").color(egui::Color32::from_rgb(100, 150, 255)));
                                ui.label(":");
                                if ui.text_edit_singleline(&mut profile.ausrc).changed() {
                                    profile_changed = true;
                                }
                            });
                        });
                    }
                    
                    ui.add_space(10.0);
                    
                    if profile_changed && !session_locked {
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                if let Err(e) = refracted::common::user_profile::save_profile(&current_profile_name, profile.clone()) {
                                    eprintln!("Failed to save profile: {}", e);
                                } else {
                                    refracted::common::user_profile::sync_profile_to_session();
                                    profile_changed = false;
                                }
                            }
                            
                            if ui.button("Cancel").clicked() {
                                profile = selected_profile.clone();
                                profile_changed = false;
                            }
                        });
                    }
                    
                    ui.add_space(10.0);
                    
                    // Delete profile button
                    if profiles.profiles.len() > 1 {
                        if ui
                            .add_enabled(
                                !session_locked,
                                egui::Button::new(
                                    egui::RichText::new("Delete Profile").color(egui::Color32::RED),
                                ),
                            )
                            .clicked()
                        {
                            if let Err(e) = refracted::common::user_profile::delete_profile(&current_profile_name) {
                                eprintln!("Failed to delete profile: {}", e);
                            } else {
                                refracted::common::user_profile::sync_profile_to_session();
                            }
                        }
                    }

                    // Persist editor draft for the selected profile.
                    ctx.data_mut(|data| {
                        data.insert_temp(edit_profile_name_id, edit_profile_name);
                        data.insert_temp(edit_profile_id, profile);
                        data.insert_temp(edit_profile_changed_id, profile_changed);
                    });
                }
                
                // Add profile dialog
                if show_add_profile {
                    if session_locked {
                        // Auto-dismiss if a session became active while the form was open.
                        show_add_profile = false;
                        new_profile_name.clear();
                    } else {
                        ui.add_space(10.0);
                        ui.group(|ui| {
                            ui.label("Add New Profile");
                            ui.horizontal(|ui| {
                                ui.label("Profile Name:");
                                ui.text_edit_singleline(&mut new_profile_name);
                            });

                            ui.horizontal(|ui| {
                                if ui.button("Create").clicked() {
                                    if !new_profile_name.is_empty() {
                                        // Create new profile with randomized UUIDs and incremented PSID
                                        // Username and display_name will be set from profile name in save_profile
                                        let new_profile = refracted::common::user_profile::create_new_profile();
                                        if let Err(e) = refracted::common::user_profile::save_profile(&new_profile_name, new_profile) {
                                            eprintln!("Failed to create profile: {}", e);
                                        } else {
                                            if let Err(e) = refracted::common::user_profile::set_current_profile(&new_profile_name) {
                                                eprintln!("Failed to set current profile: {}", e);
                                            } else {
                                                refracted::common::user_profile::sync_profile_to_session();
                                            }
                                            show_add_profile = false;
                                            new_profile_name.clear();
                                        }
                                    }
                                }
                                if ui.button("Cancel").clicked() {
                                    show_add_profile = false;
                                    new_profile_name.clear();
                                }
                            });
                        });
                    }
                }
                
                ui.add_space(10.0);
                
                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });
        });
    
    // Persist state to data
    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new("show_advanced"), show_advanced);
        data.insert_temp(egui::Id::new("new_profile_name"), new_profile_name);
        data.insert_temp(egui::Id::new("show_add_profile"), show_add_profile);
    });
    
    if should_close {
        *open = false;
    }
}

#[derive(Clone)]
struct AddGameDraft {
    id: String,
    name: String,
    protocol: String,
    frostbite_build: String,
    blaze_build: String,
    redirector_tls: bool,
    svc_lsx: bool,
    svc_neptune: bool,
    svc_blaze: bool,
    svc_grpc: bool,
    svc_web: bool,
    svc_qos: bool,
    svc_rtm: bool,
    ports: refracted::common::game::ServicePorts,
}

impl Default for AddGameDraft {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            protocol: "Fire2Frame".into(),
            frostbite_build: String::new(),
            blaze_build: String::new(),
            redirector_tls: true,
            svc_lsx: true,
            svc_neptune: false,
            svc_blaze: true,
            svc_grpc: false,
            svc_web: true,
            svc_qos: true,
            svc_rtm: true,
            ports: refracted::common::game::ServicePorts::default(),
        }
    }
}

fn games_window(ctx: &egui::Context, open: &mut bool) {
    let mut should_close = false;
    
    egui::Window::new("Games")
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_size([920.0, 680.0])
        .show(ctx, |ui| {
            let selection = refracted::common::game::get_game_selection();
            let current_game_id = selection.current_game.clone();
            
            ui.vertical(|ui| {
                ui.heading("Game Selection");
                ui.add_space(10.0);
                
                ui.label("Select the game you want Refracted to emulate:");
                ui.add_space(10.0);
                
                // Game selector
                ui.horizontal(|ui| {
                    ui.label("Game:");
                    egui::ComboBox::from_id_source("game_selector")
                        .selected_text(
                            selection.available_games
                                .iter()
                                .find(|g| g.id == current_game_id)
                                .map(|g| g.name.as_str())
                                .unwrap_or("Unknown")
                        )
                        .show_ui(ui, |ui| {
                            for game in &selection.available_games {
                                let is_selected = game.id == current_game_id;
                                if ui.selectable_label(is_selected, &game.name).clicked() {
                                    if let Err(e) = refracted::common::game::set_current_game(&game.id) {
                                        eprintln!("Failed to set current game: {}", e);
                                    }
                                }
                            }
                        });
                });
                
                ui.horizontal(|ui| {
                    ui.label("Need help?");
                    ui.hyperlink_to("Raise a request", "https://github.com/Xevrac/refracted/issues/new");
                });

                ui.add_space(8.0);

                let add_draft_id = egui::Id::new("refracted_add_game_draft");
                let mut add_draft = ctx
                    .data_mut(|d| d.get_temp_mut_or_insert_with(add_draft_id, AddGameDraft::default).clone());

                ui.horizontal(|ui| {
                    if ui.button("Load selected title into form").clicked() {
                        if let Some(g) = refracted::common::game::get_current_game() {
                            add_draft = AddGameDraft {
                                id: g.id.clone(),
                                name: g.name.clone(),
                                protocol: g.protocol.clone(),
                                frostbite_build: g.frostbite_build.clone(),
                                blaze_build: g.blaze_build.clone(),
                                redirector_tls: g.redirector_tls,
                                svc_lsx: g.enabled_services.iter().any(|s| s.eq_ignore_ascii_case("LSX")),
                                svc_neptune: g
                                    .enabled_services
                                    .iter()
                                    .any(|s| s.eq_ignore_ascii_case("Neptune")),
                                svc_blaze: g
                                    .enabled_services
                                    .iter()
                                    .any(|s| s.eq_ignore_ascii_case("Blaze")),
                                svc_grpc: g
                                    .enabled_services
                                    .iter()
                                    .any(|s| s.eq_ignore_ascii_case("gRPC")),
                                svc_web: g.enabled_services.iter().any(|s| s.eq_ignore_ascii_case("Web")),
                                svc_qos: g.enabled_services.iter().any(|s| s.eq_ignore_ascii_case("QoS")),
                                svc_rtm: g.enabled_services.iter().any(|s| s.eq_ignore_ascii_case("RTM")),
                                ports: g.service_ports.clone(),
                            };
                        }
                    }
                });

                ui.collapsing("Add or edit title", |ui| {
                    egui::Grid::new("add_edit_title_grid")
                        .num_columns(2)
                        .spacing([10.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Id:");
                            ui.add(
                                egui::TextEdit::singleline(&mut add_draft.id)
                                    .desired_width(220.0)
                                    .hint_text("e.g. my-game"),
                            );
                            ui.end_row();

                            ui.label("Display name:");
                            ui.add(
                                egui::TextEdit::singleline(&mut add_draft.name)
                                    .desired_width(300.0)
                                    .hint_text("Shown in UI"),
                            );
                            ui.end_row();

                            ui.label("Protocol:");
                            ui.add(egui::TextEdit::singleline(&mut add_draft.protocol).desired_width(180.0));
                            ui.end_row();

                            ui.label("Frostbite:");
                            ui.add(egui::TextEdit::singleline(&mut add_draft.frostbite_build).desired_width(180.0));
                            ui.end_row();

                            ui.label("Blaze:");
                            ui.add(egui::TextEdit::singleline(&mut add_draft.blaze_build).desired_width(180.0));
                            ui.end_row();

                            ui.label("Redirector TLS:");
                            ui.checkbox(&mut add_draft.redirector_tls, "Use TLS on gosredirector");
                            ui.end_row();
                        });
                    ui.label("Services:");
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut add_draft.svc_lsx, "LSX");
                        ui.checkbox(&mut add_draft.svc_neptune, "Neptune");
                        ui.checkbox(&mut add_draft.svc_blaze, "Blaze");
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut add_draft.svc_grpc, "gRPC");
                        ui.checkbox(&mut add_draft.svc_web, "Web");
                        ui.checkbox(&mut add_draft.svc_qos, "QoS");
                        ui.checkbox(&mut add_draft.svc_rtm, "RTM");
                    });
                    ui.collapsing("Listen ports (per title)", |ui| {
                        let p = &mut add_draft.ports;
                        ui.columns(3, |cols| {
                            cols[0].vertical(|ui| {
                                egui::Grid::new("add_ports_c0")
                                    .num_columns(2)
                                    .spacing([8.0, 6.0])
                                    .show(ui, |ui| {
                                        ui.label("Web HTTP");
                                        ui.add(egui::DragValue::new(&mut p.web_http).speed(1));
                                        ui.end_row();
                                        ui.label("Web HTTPS");
                                        ui.add(egui::DragValue::new(&mut p.web_https).speed(1));
                                        ui.end_row();
                                        ui.label("Web HTTP alt");
                                        ui.add(egui::DragValue::new(&mut p.web_http_alt).speed(1));
                                        ui.end_row();
                                        ui.label("Web HTTPS alt");
                                        ui.add(egui::DragValue::new(&mut p.web_https_alt).speed(1));
                                        ui.end_row();
                                        ui.label("GOS");
                                        ui.add(egui::DragValue::new(&mut p.blaze_gosredirector).speed(1));
                                        ui.end_row();
                                    });
                            });
                            cols[1].vertical(|ui| {
                                egui::Grid::new("add_ports_c1")
                                    .num_columns(2)
                                    .spacing([8.0, 6.0])
                                    .show(ui, |ui| {
                                        ui.label("Blaze gosca");
                                        ui.add(egui::DragValue::new(&mut p.blaze_gosca).speed(1));
                                        ui.end_row();
                                        ui.label("Blaze main");
                                        ui.add(egui::DragValue::new(&mut p.blaze_main).speed(1));
                                        ui.end_row();
                                        ui.label("Blaze alt");
                                        ui.add(egui::DragValue::new(&mut p.blaze_alt).speed(1));
                                        ui.end_row();
                                        ui.label("Blaze sec");
                                        ui.add(egui::DragValue::new(&mut p.blaze_sec).speed(1));
                                        ui.end_row();
                                        ui.label("QoS coordinator");
                                        ui.add(egui::DragValue::new(&mut p.qos_coordinator).speed(1));
                                        ui.end_row();
                                    });
                            });
                            cols[2].vertical(|ui| {
                                egui::Grid::new("add_ports_c2")
                                    .num_columns(2)
                                    .spacing([8.0, 6.0])
                                    .show(ui, |ui| {
                                        ui.label("QoS data");
                                        ui.add(egui::DragValue::new(&mut p.qos_data).speed(1));
                                        ui.end_row();
                                        ui.label("QoS alt");
                                        ui.add(egui::DragValue::new(&mut p.qos_alt).speed(1));
                                        ui.end_row();
                                        ui.label("RTM");
                                        ui.add(egui::DragValue::new(&mut p.rtm).speed(1));
                                        ui.end_row();
                                        ui.label("LSX");
                                        ui.add(egui::DragValue::new(&mut p.lsx).speed(1));
                                        ui.end_row();
                                    });
                            });
                        });
                    });
                    if ui.button("Save").clicked() {
                        let d = add_draft.clone();
                        let mut enabled = Vec::new();
                        if d.svc_lsx {
                            enabled.push("LSX".to_string());
                        }
                        if d.svc_neptune {
                            enabled.push("Neptune".to_string());
                        }
                        if d.svc_blaze {
                            enabled.push("Blaze".to_string());
                        }
                        if d.svc_grpc {
                            enabled.push("gRPC".to_string());
                        }
                        if d.svc_web {
                            enabled.push("Web".to_string());
                        }
                        if d.svc_qos {
                            enabled.push("QoS".to_string());
                        }
                        if d.svc_rtm {
                            enabled.push("RTM".to_string());
                        }
                        if d.id.trim().is_empty() || d.name.trim().is_empty() {
                            eprintln!("Add game: id and display name are required.");
                        } else {
                            let g = refracted::common::game::GameInfo::new(
                                d.id.trim().to_string(),
                                d.name.trim().to_string(),
                                d.protocol.trim().to_string(),
                                d.frostbite_build.trim().to_string(),
                                d.blaze_build.trim().to_string(),
                                d.redirector_tls,
                                enabled,
                                d.ports.clone(),
                            );
                            if let Err(e) = refracted::common::game::upsert_game(g) {
                                eprintln!("Failed to save game: {}", e);
                            }
                        }
                    }
                });

                ctx.data_mut(|d| {
                    *d.get_temp_mut_or_insert_with(add_draft_id, AddGameDraft::default) = add_draft;
                });

                ui.add_space(6.0);
                if ui
                    .add_enabled(
                        selection.available_games.len() > 1,
                        egui::Button::new("Remove selected title"),
                    )
                    .on_disabled_hover_text("Keep at least one title (add another before removing).")
                    .clicked()
                {
                    if let Err(e) =
                        refracted::common::game::remove_registered_game(&current_game_id)
                    {
                        eprintln!("Failed to remove game: {}", e);
                    }
                }
                
                ui.add_space(10.0);
                
                // Display current game info
                if let Some(current_game) = refracted::common::game::get_current_game() {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Current Game Information").heading());
                        ui.add_space(5.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Game ID:");
                            ui.label(egui::RichText::new(&current_game.id).monospace());
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.label(egui::RichText::new(&current_game.name).monospace());
                        });
                        
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(5.0);
                        
                        // Protocol (unselectable text field)
                        ui.horizontal(|ui| {
                            ui.label("Protocol:");
                            let mut protocol_text = current_game.protocol.clone();
                            ui.add(egui::TextEdit::singleline(&mut protocol_text)
                                .interactive(false)
                                .desired_width(150.0));
                        });
                        
                        ui.add_space(5.0);
                        
                        // Frostbite build number (unselectable text field)
                        ui.horizontal(|ui| {
                            ui.label("Frostbite:");
                            let mut frostbite_build = current_game.frostbite_build.clone();
                            ui.add(egui::TextEdit::singleline(&mut frostbite_build)
                                .interactive(false)
                                .desired_width(150.0));
                        });
                        
                        ui.add_space(5.0);
                        
                        // Blaze build number (unselectable text field)
                        ui.horizontal(|ui| {
                            ui.label("Blaze:");
                            let mut blaze_build = current_game.blaze_build.clone();
                            ui.add(egui::TextEdit::singleline(&mut blaze_build)
                                .interactive(false)
                                .desired_width(150.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Redirector TLS:");
                            let mut redirector_tls = current_game.redirector_tls;
                            ui.add_enabled(false, egui::Checkbox::new(&mut redirector_tls, ""));
                        });
                        
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(5.0);
                        
                        // Services (unselectable checkboxes)
                        ui.label(egui::RichText::new("Services").heading());
                        ui.add_space(5.0);
                        
                        // List of all possible services
                        let all_services = vec![
                            "LSX", "Neptune", "Blaze", "gRPC", "Web", "QoS", "RTM"
                        ];
                        
                        // Display services in 2 columns
                        ui.columns(2, |columns| {
                            for (idx, service) in all_services.iter().enumerate() {
                                let column = &mut columns[idx % 2];
                                let is_enabled = current_game.enabled_services.contains(&service.to_string());
                                column.horizontal(|ui| {
                                    // Display checkbox state (read-only) using a visual indicator
                                    if is_enabled {
                                        // Color #2246a7
                                        ui.label(egui::RichText::new("☑").color(egui::Color32::from_rgb(0x22, 0x46, 0xa7)));
                                    } else {
                                        ui.label(egui::RichText::new("☐").color(egui::Color32::GRAY));
                                    }
                                    ui.label(*service);
                                });
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("Listen ports").heading());
                        ui.add_space(4.0);
                        let pr = &current_game.service_ports;
                        ui.columns(3, |cols| {
                            cols[0].vertical(|ui| {
                                egui::Grid::new("cur_ports_c0")
                                    .num_columns(2)
                                    .spacing([8.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Web HTTP");
                                        ui.label(egui::RichText::new(pr.web_http.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("Web HTTPS");
                                        ui.label(egui::RichText::new(pr.web_https.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("Web HTTP alt");
                                        ui.label(egui::RichText::new(pr.web_http_alt.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("Web HTTPS alt");
                                        ui.label(egui::RichText::new(pr.web_https_alt.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("GOS");
                                        ui.label(egui::RichText::new(pr.blaze_gosredirector.to_string()).monospace());
                                        ui.end_row();
                                    });
                            });
                            cols[1].vertical(|ui| {
                                egui::Grid::new("cur_ports_c1")
                                    .num_columns(2)
                                    .spacing([8.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Blaze gosca");
                                        ui.label(egui::RichText::new(pr.blaze_gosca.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("Blaze main");
                                        ui.label(egui::RichText::new(pr.blaze_main.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("Blaze alt");
                                        ui.label(egui::RichText::new(pr.blaze_alt.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("Blaze sec");
                                        ui.label(egui::RichText::new(pr.blaze_sec.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("QoS coordinator");
                                        ui.label(egui::RichText::new(pr.qos_coordinator.to_string()).monospace());
                                        ui.end_row();
                                    });
                            });
                            cols[2].vertical(|ui| {
                                egui::Grid::new("cur_ports_c2")
                                    .num_columns(2)
                                    .spacing([8.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("QoS data");
                                        ui.label(egui::RichText::new(pr.qos_data.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("QoS alt");
                                        ui.label(egui::RichText::new(pr.qos_alt.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("RTM");
                                        ui.label(egui::RichText::new(pr.rtm.to_string()).monospace());
                                        ui.end_row();
                                        ui.label("LSX");
                                        ui.label(egui::RichText::new(pr.lsx.to_string()).monospace());
                                        ui.end_row();
                                    });
                            });
                        });
                    });
                }
                
                ui.add_space(10.0);
                
                ui.separator();
                ui.add_space(10.0);
                
                ui.label(egui::RichText::new("Note: Reload the app after changing ports or the selected title.").size(14.0));
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                    
                    if ui.button("Reload").clicked() {
                        // Reload the app to apply game selection changes
                        if let Ok(exe_path) = std::env::current_exe() {
                            if let Err(e) = std::process::Command::new(&exe_path).spawn() {
                                eprintln!("Failed to reload application: {}", e);
                            } else {
                                std::process::exit(0);
                            }
                        }
                    }
                });
            });
        });
    
    if should_close {
        *open = false;
    }
}

/// Apply theme to egui context
fn apply_theme(ctx: &egui::Context, theme: &str) {
    if theme == "light" {
        // Light theme - use egui's built-in light visuals
        ctx.set_visuals(egui::Visuals::light());
    } else {
        // Dark theme (default) - use egui's built-in dark visuals
        ctx.set_visuals(egui::Visuals::dark());
    }
}

/// Options window
fn options_window(ctx: &egui::Context, open: &mut bool) {
    let mut should_close = false;
    let mut settings = refracted::common::settings::get_settings();
    let mut app_settings = settings.app_settings.clone();
    let mut settings_changed = false;
    
    egui::Window::new("Options")
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_size([400.0, 300.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Application Settings");
                ui.add_space(10.0);
                
                // Debug logging toggle
                if ui.checkbox(&mut app_settings.debug_logging, "Enable Debug Logging").changed() {
                    settings_changed = true;
                }
                
                ui.add_space(10.0);
                
                // Theme toggle
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    if ui.selectable_label(app_settings.theme == "dark", "Dark").clicked() {
                        app_settings.theme = "dark".to_string();
                        settings_changed = true;
                    }
                    if ui.selectable_label(app_settings.theme == "light", "Light").clicked() {
                        app_settings.theme = "light".to_string();
                        settings_changed = true;
                    }
                });

                if settings_changed {
                    settings.app_settings = app_settings.clone();
                    if let Err(e) = refracted::common::settings::update_settings(settings.clone()) {
                        eprintln!("Failed to save settings: {}", e);
                    } else {
                        // Apply theme immediately
                        apply_theme(ctx, &app_settings.theme);
                        settings_changed = false;
                    }
                }
                
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            });
        });
    
    if should_close {
        *open = false;
    }
}

/// Proxy settings window
fn proxy_window(ctx: &egui::Context, open: &mut bool, inspector_state: &mut InspectorUiState) {
    let mut should_close = false;
    let proxy_settings = refracted::common::settings::get_proxy_settings();
    
    egui::Window::new("Proxy Settings")
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_size([600.0, 600.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Proxy Listen Ports");
                ui.add_space(10.0);
                ui.label("Configure the ports the proxy will listen on for each protocol:");
                ui.add_space(10.0);
                
                // HTTP
                ui.horizontal(|ui| {
                    ui.checkbox(&mut inspector_state.proxy_config_enable_http, "");
                    ui.label("HTTP:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_http_listen)
                        .desired_width(100.0)
                        .interactive(inspector_state.proxy_config_enable_http));
                });
                
                // HTTPS
                ui.horizontal(|ui| {
                    ui.checkbox(&mut inspector_state.proxy_config_enable_https, "");
                    ui.label("HTTPS:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_https_listen)
                        .desired_width(100.0)
                        .interactive(inspector_state.proxy_config_enable_https));
                });
                
                // gRPC
                ui.horizontal(|ui| {
                    ui.checkbox(&mut inspector_state.proxy_config_enable_grpc, "");
                    ui.label("gRPC:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_grpc_listen)
                        .desired_width(100.0)
                        .interactive(inspector_state.proxy_config_enable_grpc));
                });
                
                // Blaze
                ui.horizontal(|ui| {
                    ui.checkbox(&mut inspector_state.proxy_config_enable_blaze, "");
                    ui.label("Blaze:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_blaze_listen)
                        .desired_width(100.0)
                        .interactive(inspector_state.proxy_config_enable_blaze));
                });
                
                // LSX
                ui.horizontal(|ui| {
                    ui.checkbox(&mut inspector_state.proxy_config_enable_lsx, "");
                    ui.label("LSX:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_lsx_listen)
                        .desired_width(100.0)
                        .interactive(inspector_state.proxy_config_enable_lsx));
                });
                
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Target server configuration
                ui.heading("Target Server");
                ui.add_space(10.0);
                
                // Target host
                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_target_host)
                        .desired_width(200.0));
                });
                
                // Target ports (only Blaze and LSX - HTTP/HTTPS/gRPC extract from Host header)
                ui.label(egui::RichText::new("Note: HTTP, HTTPS, and gRPC proxies automatically extract the destination from the Host header.")
                    .size(11.0)
                    .color(egui::Color32::GRAY));
                ui.horizontal(|ui| {
                    ui.label("Blaze Port:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_target_blaze)
                        .desired_width(100.0));
                    ui.label("LSX Port:");
                    ui.add(egui::TextEdit::singleline(&mut inspector_state.proxy_config_target_lsx)
                        .desired_width(100.0));
                });
                
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Save button
                if ui.button("Save").clicked() {
                    // Validate listen ports
                    let http_port = inspector_state.proxy_config_http_listen.parse::<u16>().unwrap_or(80);
                    let https_port = inspector_state.proxy_config_https_listen.parse::<u16>().unwrap_or(443);
                    let grpc_port = inspector_state.proxy_config_grpc_listen.parse::<u16>().unwrap_or(443);
                    let blaze_port = inspector_state.proxy_config_blaze_listen.parse::<u16>().unwrap_or(10042);
                    let lsx_port = inspector_state.proxy_config_lsx_listen.parse::<u16>().unwrap_or(3216);
                    
                    // Validate target ports (HTTP/HTTPS/gRPC use defaults, not from UI)
                    let target_http_port = 80;   // Default, not configurable (extracted from Host header)
                    let target_https_port = 443; // Default, not configurable (extracted from Host header)
                    let target_grpc_port = 443;  // Default, not configurable (extracted from Host header)
                    let target_blaze_port = inspector_state.proxy_config_target_blaze.parse::<u16>().unwrap_or(10042);
                    let target_lsx_port = inspector_state.proxy_config_target_lsx.parse::<u16>().unwrap_or(3216);
                    
                    // Update settings
                    let mut new_settings = proxy_settings.clone();
                    new_settings.http_listen_port = http_port;
                    new_settings.https_listen_port = https_port;
                    new_settings.grpc_listen_port = grpc_port;
                    new_settings.blaze_listen_port = blaze_port;
                    new_settings.lsx_listen_port = lsx_port;
                    new_settings.target_host = inspector_state.proxy_config_target_host.clone();
                    new_settings.target_http_port = target_http_port;
                    new_settings.target_https_port = target_https_port;
                    new_settings.target_grpc_port = target_grpc_port;
                    new_settings.target_blaze_port = target_blaze_port;
                    new_settings.target_lsx_port = target_lsx_port;
                    new_settings.enable_http = inspector_state.proxy_config_enable_http;
                    new_settings.enable_https = inspector_state.proxy_config_enable_https;
                    new_settings.enable_grpc = inspector_state.proxy_config_enable_grpc;
                    new_settings.enable_blaze = inspector_state.proxy_config_enable_blaze;
                    new_settings.enable_lsx = inspector_state.proxy_config_enable_lsx;
                    
                    if let Err(e) = refracted::common::settings::update_proxy_settings(new_settings.clone()) {
                        eprintln!("Failed to save proxy settings: {}", e);
                    } else {
                        // Update inspector state
                        inspector_state.proxy_config_http_listen = http_port.to_string();
                        inspector_state.proxy_config_https_listen = https_port.to_string();
                        inspector_state.proxy_config_grpc_listen = grpc_port.to_string();
                        inspector_state.proxy_config_blaze_listen = blaze_port.to_string();
                        inspector_state.proxy_config_lsx_listen = lsx_port.to_string();
                        // HTTP/HTTPS/gRPC target ports are not configurable (extracted from Host header)
                        inspector_state.proxy_config_target_blaze = target_blaze_port.to_string();
                        inspector_state.proxy_config_target_lsx = target_lsx_port.to_string();
                        
                        // Update proxy config if proxy is running
                        if inspector_state.proxy_state.is_running() {
                            let mut config = inspector_state.proxy_state.config.lock();
                            config.http_listen_port = http_port;
                            config.https_listen_port = https_port;
                            config.grpc_listen_port = grpc_port;
                            config.blaze_listen_port = blaze_port;
                            config.lsx_listen_port = lsx_port;
                            config.target_host = inspector_state.proxy_config_target_host.clone();
                            config.target_http_port = target_http_port;
                            config.target_https_port = target_https_port;
                            config.target_grpc_port = target_grpc_port;
                            config.target_blaze_port = target_blaze_port;
                            config.target_lsx_port = target_lsx_port;
                            config.enable_http = inspector_state.proxy_config_enable_http;
                            config.enable_https = inspector_state.proxy_config_enable_https;
                            config.enable_grpc = inspector_state.proxy_config_enable_grpc;
                            config.enable_blaze = inspector_state.proxy_config_enable_blaze;
                            config.enable_lsx = inspector_state.proxy_config_enable_lsx;
                        }
                    }
                }
                
                ui.add_space(10.0);
                
                // Check if proxy is running
                let proxy_running = inspector_state.proxy_state.is_running();
                if proxy_running {
                    ui.label(egui::RichText::new("⚠ Proxy is currently running. Port changes will take effect after you stop and restart the proxy.")
                        .size(12.0)
                        .color(egui::Color32::YELLOW));
                } else {
                    ui.label(egui::RichText::new("Note: Changes take effect when you start the proxy.")
                        .size(12.0)
                        .color(egui::Color32::GRAY));
                }
                
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            });
        });
    
    if should_close {
        *open = false;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let log_buffer = Arc::new(Mutex::new(Vec::new()));
    let (log_tx, log_rx) = std::sync::mpsc::channel::<LogLine>();
    init_log_line_sender(log_tx);

    // Load application icon from embedded base64
    let icon_data = load_icon().ok();
    
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_title("Refracted")
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([800.0, 600.0])
        .with_maximized(true);
    
    if let Some(icon) = icon_data {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    let packet_buffer = Arc::new(Mutex::new(Vec::new()));
    
    if let Err(e) = eframe::run_native(
        "Refracted",
        options,
        Box::new(move |cc| {
            Box::new(RefractedApp::new(
                cc,
                log_buffer.clone(),
                packet_buffer.clone(),
                log_rx,
            ))
        }),
    ) {
        eprintln!("Error running GUI: {}", e);
        return Err(anyhow::anyhow!("GUI error: {}", e));
    }

    Ok(())
}
