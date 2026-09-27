//! Shared startup for desktop and headless binaries.
//!
//! Desktop uses JSON under `{exe}/data` unless a launcher Nexus handoff is present
//! and `refracted.env` has `datasource=mysql`. Headless follows `refracted.env`.

use std::path::PathBuf;

use crate::common::app_env::{self, AppEnv, Datasource};
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
        None => boot_desktop_identity()?,
        Some(env) => {
            identity::log_headless_identity_policy(env);
            match env.datasource {
                Datasource::Json => {
                    identity::enable_json_personas();
                    user_profile::sync_profile_to_session();
                    blaze_sessions::load_persisted_sessions();
                    if identity::handoff_file_present() {
                        crate::nexus::log_nexus_to_blaze(
                            "launcher handoff present but datasource=json — Nexus bind skipped",
                        );
                    }
                }
                Datasource::Mysql => {
                    identity::disable_json_personas();
                    identity::init_mysql_identity(env)?;
                    let _ = identity::try_bind_launcher_handoff();
                }
            }
        }
    }

    identity::lock_identity_policy();
    Ok(())
}

/// Desktop: JSON by default. If the launcher wrote a handoff **and** `refracted.env`
/// is `datasource=mysql`, switch to Nexus bind so GetProfile returns the Discord persona.
fn boot_desktop_identity() -> Result<(), String> {
    if identity::handoff_file_present() {
        match load_mysql_env_if_configured() {
            Ok(Some(env)) => {
                crate::nexus::log_nexus_to_blaze(
                    "desktop: launcher handoff + datasource=mysql — binding Nexus persona",
                );
                identity::disable_json_personas();
                identity::init_mysql_identity(&env)?;
                let _ = identity::try_bind_launcher_handoff();
                return Ok(());
            }
            Ok(None) => {
                crate::nexus::log_nexus_to_blaze(
                    "desktop: launcher handoff present but refracted.env is not datasource=mysql — using JSON personas",
                );
            }
            Err(e) => {
                crate::nexus::log_nexus_to_blaze(format!(
                    "desktop: launcher handoff present but mysql env failed ({e}) — using JSON personas"
                ));
            }
        }
    }

    identity::enable_json_personas();
    user_profile::sync_profile_to_session();
    blaze_sessions::load_persisted_sessions();
    Ok(())
}

/// Load `{exe}/refracted.env` only if it already exists and asks for mysql. Never create it here.
fn load_mysql_env_if_configured() -> Result<Option<AppEnv>, String> {
    let path = app_env::default_env_path();
    if !path.is_file() {
        return Ok(None);
    }
    let env = app_env::load_app_env(&path)?;
    if env.datasource != Datasource::Mysql {
        return Ok(None);
    }
    app_env::set_current_app_env(env.clone());
    Ok(Some(env))
}

/// Known game ids from the current registry (after [`boot_emulator`] / settings init).
pub fn list_game_ids() -> Vec<String> {
    game::get_all_game_definitions()
        .into_iter()
        .map(|g| g.id)
        .collect()
}
