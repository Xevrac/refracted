use std::path::{Path, PathBuf};

pub fn executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

/// `{exe_dir}/data`, or `./data` if the executable path is unavailable.
pub fn app_data_dir() -> PathBuf {
    executable_dir()
        .map(|d| d.join("data"))
        .unwrap_or_else(|| PathBuf::from("data"))
}

pub fn ensure_app_data_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(app_data_dir())
}

pub fn settings_json_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn sessions_json_path() -> PathBuf {
    app_data_dir().join("sessions.json")
}

/// User-defined titles (`GameInfo` list), seeded from embedded `resources/default_games.json`.
pub fn games_json_path() -> PathBuf {
    app_data_dir().join("games.json")
}

pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}
