//! Shared startup for desktop and headless binaries.
//!
//! Desktop uses JSON under `{exe}/data`. Headless uses JSON only when `datasource=json`;
//! `datasource=mysql` is production identity.

use std::path::PathBuf;

use crate::common::app_env::{AppEnv, Datasource};
use crate::common::game;
use crate::common::paths;
use crate::common::settings;
use crate::common::user_profile;
use crate::nexus::identity;
use crate::session::blaze_sessions;

/// Options for emulator boot.
#[derive(Debug, Clone, Default)]
pub struct BootOptions {
    /// Override `{exe}/data`.
    pub data_dir: Option<PathBuf>,
    /// Game id from `games.json`. Overrides env `game=` when set.
    pub game_id: Option<String>,
    /// Headless only. Desktop leaves this `None`.
    pub env: Option<AppEnv>,
}

/// Initialize data, settings, games registry, and identity policy.
pub fn boot_emulator(opts: BootOptions) -> Result<(), String> {
    if let Some(dir) = opts
        .data_dir
        .or_else(|| opts.env.as_ref().and_then(|e| e.data_dir.clone()))
    {
        paths::set_app_data_dir(dir);
    }

    paths::ensure_app_data_dir().map_err(|e| format!("Failed to create app data dir: {e}"))?;

    let settings_path = paths::settings_json_path();
    settings::init_settings(settings_path)?;

    let game_id = opts
        .game_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            opts.env.as_ref().and_then(|e| {
                let g = e.game.trim();
                (!g.is_empty()).then_some(g)
            })
        });
    if let Some(game_id) = game_id {
        game::set_current_game(game_id)?;
    }

    match &opts.env {
        None => {
            identity::enable_json_personas();
            user_profile::sync_profile_to_session();
            blaze_sessions::load_persisted_sessions();
        }
        Some(env) => {
            identity::log_headless_identity_policy(env);
            match env.datasource {
                Datasource::Json => {
                    identity::enable_json_personas();
                    user_profile::sync_profile_to_session();
                    blaze_sessions::load_persisted_sessions();
                }
                Datasource::Mysql => {
                    identity::disable_json_personas();
                    identity::init_mysql_identity(env)?;
                }
            }
        }
    }

    identity::lock_identity_policy();
    Ok(())
}

/// Known game ids from the current registry (after [`boot_emulator`] / settings init).
pub fn list_game_ids() -> Vec<String> {
    game::get_all_game_definitions()
        .into_iter()
        .map(|g| g.id)
        .collect()
}
