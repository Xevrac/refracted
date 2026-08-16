use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use refracted::common::app_env::{self, AppEnv};
use refracted::common::boot::{boot_emulator, BootOptions};
use refracted::core::console::{
    colorize_channel_tags, enable_windows_vt, flush_cli_compact_line, push_formatted_log_line,
};
use refracted::core::server::BlazeServer;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::registry::LookupSpan;

#[derive(Parser, Debug)]
#[command(
    name = "rfrcli",
    about = "Refracted Blaze emulator (headless)",
    after_help = "\
Env file (created next to rfrcli on first run as refracted.env):
  game=cnc
  environment=dev
  datasource=json
  host=127.0.0.1
  database=refracted
  user=refracted
  pass=

datasource=json  localized testing only — JSON/manual personas from {exe}/data
datasource=mysql production identity tables; game clients must authenticate (login later)

Examples:
  rfrcli
  rfrcli -env D:\\prod\\refracted.env
  rfrcli --game cnc --listen-host 0.0.0.0
"
)]
struct Args {
    /// Path to env file (default: {exe}/refracted.env, created on first run)
    #[arg(long, short = 'e', value_name = "PATH")]
    env: Option<PathBuf>,

    /// Game id override (else env `game=`)
    #[arg(long, short = 'g')]
    game: Option<String>,

    /// Bind address override (else env `listen_host=`, default 0.0.0.0)
    #[arg(long)]
    listen_host: Option<String>,
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
        flush_cli_compact_line();
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
        flush_cli_compact_line();

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
                write!(writer, "{}", colorize_channel_tags(&message_string))?;
            } else if has_ansi {
                write!(writer, "{}", colorize_channel_tags(&message_string))?;
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
                    colorize_channel_tags(&message_string)
                )?;
            }
            writeln!(writer)
        } else if level == tracing::Level::ERROR {
            if has_ansi || has_rts || has_sim {
                write!(writer, "{}", colorize_channel_tags(&message_string))?;
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
                write!(writer, "{}", colorize_channel_tags(&message_string))?;
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
    enable_windows_vt();

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

fn resolve_listen_host(args: &Args, env: &AppEnv) -> String {
    args.listen_host
        .clone()
        .or_else(|| env.listen_host.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "0.0.0.0".to_string())
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse_from(app_env::normalize_launch_args(std::env::args()));

    let env = match app_env::load_or_create_app_env(args.env.as_deref()) {
        Ok(env) => env,
        Err(e) => {
            eprintln!("env load failed: {e}");
            return ExitCode::from(1);
        }
    };

    init_tracing(env.log_level.as_deref());

    if let Err(e) = boot_emulator(BootOptions {
        data_dir: env.data_dir.clone(),
        game_id: args.game.clone(),
        env: Some(env.clone()),
    }) {
        eprintln!("boot failed: {e}");
        return ExitCode::from(1);
    }

    let listen_host = resolve_listen_host(&args, &env);
    let game = refracted::common::game::get_current_game_id();
    info!(
        "rfrcli — game={game} env={} datasource={} listen={listen_host} file={}",
        env.environment.as_str(),
        env.datasource.as_str(),
        env.path.display()
    );

    let ports_in_use = BlazeServer::check_all_ports(&listen_host);
    if !ports_in_use.is_empty() {
        eprintln!("The following ports are already in use:");
        for (port, name) in &ports_in_use {
            eprintln!("  - Port {port} ({name})");
        }
        eprintln!("Free these ports before starting the server.");
        return ExitCode::from(2);
    }

    info!("Starting Refracted Emulator...");

    match BlazeServer::new(listen_host).await {
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
