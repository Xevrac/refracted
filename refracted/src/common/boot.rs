//! Shared startup for desktop and headless binaries.
//!
//! Loads settings / games registry, selects a game, and syncs the local profile
//! into the Blaze [`UserSession`](crate::session::session_module::UserSession).

use std::path::PathBuf;

use crate::common::game;
use crate::common::paths;
use crate::common::settings;
use crate::common::user_profile;
use crate::session::blaze_sessions;

/// Options for emulator boot (settings dir, optional game override).
#[derive(Debug, Clone, Default)]
pub struct BootOptions {
    /// Override `{exe}/data` (also settable via `REFRACTED_DATA_DIR`).
    pub data_dir: Option<PathBuf>,
    /// Game id from `games.json` (e.g. `cnc`, `bf-labs`). When set, updates preference.
    pub game_id: Option<String>,
}

/// Initialize app data, settings, games registry, profile→session, and persisted Blaze sessions.
pub fn boot_emulator(opts: BootOptions) -> Result<(), String> {
    if let Some(dir) = opts.data_dir {
        paths::set_app_data_dir(dir);
    }

    paths::ensure_app_data_dir().map_err(|e| format!("Failed to create app data dir: {e}"))?;

    let settings_path = paths::settings_json_path();
    settings::init_settings(settings_path)?;

    if let Some(game_id) = opts.game_id.as_deref() {
        game::set_current_game(game_id)?;
    }

    user_profile::sync_profile_to_session();
    blaze_sessions::load_persisted_sessions();

    Ok(())
}

/// Known game ids from the current registry (after [`boot_emulator`] / settings init).
pub fn list_game_ids() -> Vec<String> {
    game::get_all_game_definitions()
        .into_iter()
        .map(|g| g.id)
        .collect()
}
