use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use refracted::common::boot::{boot_emulator, BootOptions};
use refracted::core::console::push_formatted_log_line;
use refracted::core::server::BlazeServer;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::registry::LookupSpan;

#[derive(Parser, Debug)]
#[command(
    name = "refracted-headless",
    about = "Refracted Blaze emulator (headless) — game selection + terminal logs only"
)]
struct Args {
    /// Game id from games.json (e.g. cnc, bf-labs)
    #[arg(long, short = 'g')]
    game: String,

    /// Directory for settings.json / games.json (default: {exe}/data or REFRACTED_DATA_DIR)
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Bind address for service listeners
    #[arg(long, default_value = "0.0.0.0")]
    listen_host: String,

    /// Tracing filter (overrides RUST_LOG when set)
    #[arg(long)]
    log_level: Option<String>,
}

struct LogWriter {
    stdout: io::Stdout,
}

/// Tracing's field formatter escapes ESC as the literal `\x1b`; undo that for real ANSI terminals.
fn unescape_ansi_escapes(s: &str) -> String {
    s.replace("\\x1b", "\x1b").replace("\\u{001b}", "\x1b")
}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        for line in text.lines() {
            if !line.trim().is_empty() {
                let rendered = unescape_ansi_escapes(line);
                let _ = self.stdout.write_all(format!("{rendered}\n").as_bytes());
                push_formatted_log_line(line);
            }
        }
        let _ = self.stdout.flush();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
}

struct HeadlessFormatter;

struct OtherTimer;

impl FormatTime for OtherTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let datetime =
            chrono::DateTime::<chrono::Utc>::from_timestamp(now.as_secs() as i64, now.subsec_nanos())
                .unwrap_or_default();
        write!(w, "[{}]", datetime.format("%Y-%m-%dT%H:%M:%S%.6fZ"))
    }
}

impl<S, N> FormatEvent<S, N> for HeadlessFormatter
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
        let mut message_string = String::new();
        {
            let mut message_writer = tracing_subscriber::fmt::format::Writer::new(&mut message_string);
            ctx.format_fields(message_writer.by_ref(), event)?;
        }

        let has_ansi_escape = message_string.contains('\x1b');
        let has_literal_escape = message_string.contains("\\x1b");
        let has_client_arrow = message_string.contains("[Client→");
        let has_server_arrow = message_string.contains("[Server→");
        let has_blaze_arrow = message_string.contains("[Blaze→");
        let has_rts = message_string.contains("[RTS]")
            || message_string.contains("[RTS→")
            || message_string.contains("→RTS]");
        let has_sim = message_string.contains("[SIM]")
            || message_string.contains("[Orchestration]")
            || message_string.contains("[CNC orchestration]");
        let has_qos = message_string.contains("[QoS]");
        let has_ansi = has_ansi_escape
            || has_literal_escape
            || has_client_arrow
            || has_server_arrow
            || has_blaze_arrow;

        const QOS_TAG_GREEN: &str = "\x1b[38;2;80;200;120m[QoS]\x1b[0m";
        const ERROR_TAG_RED: &str = "\x1b[38;2;255;150;150m[ERROR]\x1b[0m";

        fn strip_plain_qos(s: &str) -> &str {
            let mut t = s;
            while let Some(x) = t.strip_prefix("[QoS]") {
                t = x.trim_start();
            }
            t
        }

        if level == tracing::Level::INFO {
            if has_client_arrow || has_server_arrow || has_blaze_arrow || has_rts || has_sim {
                write!(writer, "{}", message_string)?;
            } else if has_ansi {
                write!(writer, "{}", message_string)?;
            } else if has_qos {
                write!(
                    writer,
                    "{} {}",
                    QOS_TAG_GREEN,
                    strip_plain_qos(message_string.as_str())
                )?;
            } else {
                write!(
                    writer,
                    "\x1b[38;2;128;128;128m[Console]\x1b[0m {}",
                    message_string
                )?;
            }
            writeln!(writer)
        } else if level == tracing::Level::ERROR {
            if has_ansi || has_rts || has_sim {
                write!(writer, "{}", message_string)?;
            } else if has_qos {
                write!(
                    writer,
                    "{} {}",
                    QOS_TAG_GREEN,
                    strip_plain_qos(message_string.as_str())
                )?;
            } else {
                write!(writer, "{} {}", ERROR_TAG_RED, message_string)?;
            }
            writeln!(writer)
        } else if level == tracing::Level::WARN {
            if has_ansi || has_rts || has_sim {
                write!(writer, "{}", message_string)?;
            } else if has_qos {
                write!(
                    writer,
                    "{} {}",
                    QOS_TAG_GREEN,
                    strip_plain_qos(message_string.as_str())
                )?;
            } else {
                write!(
                    writer,
                    "\x1b[38;2;128;128;128m[Console]\x1b[0m \x1b[38;2;255;200;0mWARN\x1b[0m {}",
                    message_string
                )?;
            }
            writeln!(writer)
        } else {
            let timer = OtherTimer;
            timer.format_time(&mut writer)?;
            write!(writer, "  {} {}", level, message_string)?;
            writeln!(writer)
        }
    }
}

fn init_tracing(log_level: Option<&str>) {
    // Windows consoles need VT processing enabled or RGB escapes are ignored / look wrong.
    #[cfg(windows)]
    {
        let _ = colored::control::set_virtual_terminal(true);
    }

    let filter = if let Some(level) = log_level {
        EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };
    let filter = filter
        .add_directive("rustls=warn".parse().unwrap())
        .add_directive("h2=warn".parse().unwrap());

    let make_writer = || {
        Box::new(LogWriter {
            stdout: io::stdout(),
        }) as Box<dyn Write + Send>
    };

    let _ = tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(filter)
        .with_writer(make_writer)
        .event_format(HeadlessFormatter)
        .try_init();
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    init_tracing(args.log_level.as_deref());

    if let Err(e) = boot_emulator(BootOptions {
        data_dir: args.data_dir.clone(),
        game_id: Some(args.game.clone()),
    }) {
        eprintln!("boot failed: {e}");
        return ExitCode::from(1);
    }

    let game = refracted::common::game::get_current_game_id();
    info!("Refracted headless — game={game} listen={}", args.listen_host);

    let ports_in_use = BlazeServer::check_all_ports(&args.listen_host);
    if !ports_in_use.is_empty() {
        eprintln!("The following ports are already in use:");
        for (port, name) in &ports_in_use {
            eprintln!("  - Port {port} ({name})");
        }
        eprintln!("Free these ports before starting the server.");
        return ExitCode::from(2);
    }

    info!("Starting Refracted Emulator...");

    match BlazeServer::new(args.listen_host).await {
        Ok(mut emulator) => match emulator.start_emulator().await {
            Ok(()) => {
                info!("Refracted Emulator has been shut down gracefully");
                ExitCode::SUCCESS
            }
            Err(e) => {
                error!("Emulator error: {e}");
                ExitCode::from(3)
            }
        },
        Err(e) => {
            error!("Failed to create emulator: {e}");
            ExitCode::from(4)
        }
    }
}
