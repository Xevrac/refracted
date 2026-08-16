use parking_lot::Mutex;
use std::path::{Path, PathBuf};

static APP_DATA_DIR_OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn executable_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

pub fn set_app_data_dir(path: PathBuf) {
    *APP_DATA_DIR_OVERRIDE.lock() = Some(path);
}

pub fn app_data_dir() -> PathBuf {
    if let Some(dir) = APP_DATA_DIR_OVERRIDE.lock().clone() {
        return dir;
    }
    if let Ok(dir) = std::env::var("REFRACTED_DATA_DIR") {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
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
