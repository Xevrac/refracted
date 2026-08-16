//! Headless launch env (`refracted.env`) — production/staging config for `rfrcli` only.
//!
//! Desktop uses JSON under `{exe}/data` (settings, profiles, games). Default env path is
//! next to the executable and is created on first headless run if missing.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;

use super::paths;

pub const ENV_FILE_NAME: &str = "refracted.env";

pub const DEFAULT_ENV_CONTENTS: &str = "\
# what game
game=cnc
# what environment ie dev, prod, staging
environment=dev
# what datasource ie json (localized testing only), mysql
datasource=json
# mysql parameters
host=127.0.0.1
database=refracted
user=refracted
pass=
";

static CURRENT_ENV: Mutex<Option<AppEnv>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Staging,
    Prod,
}

impl Environment {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "prod" | "production" => Self::Prod,
            "staging" | "stage" => Self::Staging,
            _ => Self::Dev,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Staging => "staging",
            Self::Prod => "prod",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Datasource {
    Json,
    Mysql,
}

impl Datasource {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "json" | "file" | "local" => Ok(Self::Json),
            "mysql" | "sql" => Ok(Self::Mysql),
            other => Err(format!(
                "unknown datasource '{other}' (expected json or mysql)"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Mysql => "mysql",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MysqlParams {
    pub host: String,
    pub database: String,
    pub user: String,
    pub pass: String,
}

impl MysqlParams {
    pub fn host_port(&self) -> (String, u16) {
        parse_host_port(&self.host)
    }
}

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub path: PathBuf,
    pub game: String,
    pub environment: Environment,
    pub datasource: Datasource,
    pub mysql: MysqlParams,
    pub listen_host: Option<String>,
    pub data_dir: Option<PathBuf>,
    pub log_level: Option<String>,
}

impl AppEnv {
    pub fn apply_process_vars(&self) {
        std::env::set_var("REFRACTED_GAME", &self.game);
        std::env::set_var("REFRACTED_ENVIRONMENT", self.environment.as_str());
        std::env::set_var("REFRACTED_DATASOURCE", self.datasource.as_str());
        if let Some(dir) = &self.data_dir {
            std::env::set_var("REFRACTED_DATA_DIR", dir.display().to_string());
        }
        if let Some(level) = &self.log_level {
            if std::env::var_os("RUST_LOG").is_none() {
                std::env::set_var("RUST_LOG", level);
            }
        }
    }
}

/// Path used when `-env` is not given: `{exe}/refracted.env`.
pub fn default_env_path() -> PathBuf {
    paths::executable_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
        .join(ENV_FILE_NAME)
}

/// Load the env file, creating a default next to the application on first run.
pub fn load_or_create_app_env(cli_path: Option<&Path>) -> Result<AppEnv, String> {
    let path = cli_path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_env_path);

    if !path.exists() {
        write_default_env(&path)?;
        eprintln!(
            "created default env file at {} (edit this, then restart)",
            path.display()
        );
    }

    let env = load_app_env(&path)?;
    env.apply_process_vars();
    *CURRENT_ENV.lock() = Some(env.clone());
    Ok(env)
}

pub fn current_app_env() -> Option<AppEnv> {
    CURRENT_ENV.lock().clone()
}

pub fn load_app_env(path: &Path) -> Result<AppEnv, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read env file {}: {e}", path.display()))?;
    parse_app_env(path.to_path_buf(), &content)
}

pub fn parse_app_env(path: PathBuf, content: &str) -> Result<AppEnv, String> {
    let map = parse_env_map(content);
    let game = map
        .get("game")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "cnc".to_string());
    let environment = Environment::parse(map.get("environment").map(String::as_str).unwrap_or("dev"));
    let datasource = Datasource::parse(map.get("datasource").map(String::as_str).unwrap_or("json"))?;
    let mysql = MysqlParams {
        host: nonempty(map.get("host"), "127.0.0.1"),
        database: nonempty(map.get("database"), "refracted"),
        user: nonempty(map.get("user"), "refracted"),
        pass: map.get("pass").cloned().unwrap_or_default(),
    };
    let listen_host = map
        .get("listen_host")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let data_dir = map
        .get("data_dir")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    let log_level = map
        .get("log_level")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(AppEnv {
        path,
        game,
        environment,
        datasource,
        mysql,
        listen_host,
        data_dir,
        log_level,
    })
}

/// Rewrite `-env PATH` / `-env=PATH` / `-envPATH` to `--env PATH` for the headless CLI.
pub fn normalize_launch_args<I: IntoIterator<Item = String>>(args: I) -> Vec<String> {
    let mut iter = args.into_iter();
    let mut out = Vec::new();
    if let Some(exe) = iter.next() {
        out.push(exe);
    }
    while let Some(arg) = iter.next() {
        if arg == "-env" || arg == "--env" || arg.eq_ignore_ascii_case("/env") {
            out.push("--env".to_string());
            if let Some(path) = iter.next() {
                out.push(path);
            }
            continue;
        }
        if let Some(path) = arg.strip_prefix("--env=") {
            out.push("--env".to_string());
            out.push(path.to_string());
            continue;
        }
        if let Some(rest) = strip_env_prefix(&arg) {
            if let Some(path) = rest.strip_prefix('=') {
                out.push("--env".to_string());
                out.push(path.to_string());
                continue;
            }
            if looks_like_path(rest) {
                out.push("--env".to_string());
                out.push(rest.to_string());
                continue;
            }
        }
        out.push(arg);
    }
    out
}

fn strip_env_prefix(arg: &str) -> Option<&str> {
    arg.strip_prefix("-env")
        .or_else(|| arg.strip_prefix("-Env"))
        .or_else(|| arg.strip_prefix("-ENV"))
}

fn looks_like_path(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let b = s.as_bytes();
    s.starts_with('.')
        || s.starts_with('/')
        || s.starts_with('\\')
        || (b.len() >= 2 && b[1] == b':')
}

fn write_default_env(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "failed to create env directory {}: {e}",
                    parent.display()
                )
            })?;
        }
    }
    fs::write(path, DEFAULT_ENV_CONTENTS)
        .map_err(|e| format!("failed to write default env file {}: {e}", path.display()))
}

fn parse_env_map(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        if key.is_empty() {
            continue;
        }
        map.insert(key, unquote(value.trim()));
    }
    map
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let start = bytes[0];
        let end = bytes[bytes.len() - 1];
        if (start == b'"' && end == b'"') || (start == b'\'' && end == b'\'') {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn nonempty(value: Option<&String>, default: &str) -> String {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn parse_host_port(host: &str) -> (String, u16) {
    if let Some((h, p)) = host.rsplit_once(':') {
        if !h.is_empty() && !h.starts_with('[') {
            if let Ok(port) = p.parse::<u16>() {
                return (h.to_string(), port);
            }
        }
    }
    (host.to_string(), 3306)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_commented_template() {
        let env = parse_app_env(PathBuf::from("refracted.env"), DEFAULT_ENV_CONTENTS).unwrap();
        assert_eq!(env.game, "cnc");
        assert_eq!(env.environment, Environment::Dev);
        assert_eq!(env.datasource, Datasource::Json);
        assert_eq!(env.mysql.host, "127.0.0.1");
        assert_eq!(env.mysql.database, "refracted");
        assert_eq!(env.mysql.user, "refracted");
        assert!(env.mysql.pass.is_empty());
    }

    #[test]
    fn local_alias_is_json() {
        let env = parse_app_env(PathBuf::from("x.env"), "datasource=local\n").unwrap();
        assert_eq!(env.datasource, Datasource::Json);
    }

    #[test]
    fn parse_mysql_prod() {
        let src = "\
game=bf-labs
environment=production
datasource=mysql
host=db.internal:3307
database=nexus
user=blaze
pass=s3cret#hash
";
        let env = parse_app_env(PathBuf::from("x.env"), src).unwrap();
        assert_eq!(env.game, "bf-labs");
        assert_eq!(env.environment, Environment::Prod);
        assert_eq!(env.datasource, Datasource::Mysql);
        assert_eq!(env.mysql.host, "db.internal:3307");
        assert_eq!(env.mysql.host_port(), ("db.internal".to_string(), 3307));
        assert_eq!(env.mysql.pass, "s3cret#hash");
    }

    #[test]
    fn normalize_env_flag_forms() {
        let cases = [
            vec!["rfrcli", "-env", r"D:\prod\refracted.env"],
            vec!["rfrcli", "--env", r"D:\prod\refracted.env"],
            vec!["rfrcli", r"-envD:\prod\refracted.env"],
            vec!["rfrcli", r"--env=D:\prod\refracted.env"],
        ];
        for args in cases {
            let out = normalize_launch_args(args.into_iter().map(String::from));
            assert_eq!(out[1], "--env");
            assert_eq!(out[2], r"D:\prod\refracted.env");
        }
    }

    #[test]
    fn first_run_writes_default_env() {
        let dir = std::env::temp_dir().join(format!(
            "rfr-env-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(ENV_FILE_NAME);
        assert!(!path.exists());
        let env = load_or_create_app_env(Some(&path)).unwrap();
        assert!(path.exists());
        assert_eq!(env.game, "cnc");
        assert_eq!(env.datasource, Datasource::Json);
        let env2 = load_or_create_app_env(Some(&path)).unwrap();
        assert_eq!(env2.game, "cnc");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
