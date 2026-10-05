//! Nexus identity: `users` and attached `personas`.
//!
//! Desktop and headless `datasource=json` use JSON personas. Headless `datasource=mysql`
//! requires client authentication (login/logout later).

mod store;
mod auth;
mod handoff;

pub use store::{BoundSession, IdentityStore, MatchStats, PersonaRecord, UserRecord};
pub use auth::{assert_bound_identity, IssuedCredentials};
pub use handoff::{
    ensure_bound_from_handoff, handoff_file_present, handoff_path, try_bind_launcher_handoff,
};

use crate::common::app_env::{AppEnv, Datasource};

static IDENTITY: parking_lot::Mutex<Option<IdentityStore>> = parking_lot::Mutex::new(None);
static BOUND_SESSION: parking_lot::Mutex<Option<BoundSession>> = parking_lot::Mutex::new(None);
/// JSON personas stay off until boot enables them.
static JSON_PERSONAS_ALLOWED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static POLICY_LOCKED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn set_json_personas(allowed: bool) {
    if POLICY_LOCKED.load(std::sync::atomic::Ordering::SeqCst) {
        tracing::warn!(
            "identity policy is locked; ignoring json_personas={}",
            allowed
        );
        return;
    }
    JSON_PERSONAS_ALLOWED.store(allowed, std::sync::atomic::Ordering::SeqCst);
}

/// Desktop, or headless `datasource=json`. No-op after [`lock_identity_policy`].
pub(crate) fn enable_json_personas() {
    set_json_personas(true);
}

/// Headless `datasource=mysql`: JSON personas are off.
pub(crate) fn disable_json_personas() {
    set_json_personas(false);
}

/// Freeze identity mode for process lifetime.
pub(crate) fn lock_identity_policy() {
    POLICY_LOCKED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// True on desktop and on headless `datasource=json`.
pub fn json_personas_allowed() -> bool {
    JSON_PERSONAS_ALLOWED.load(std::sync::atomic::Ordering::SeqCst)
}

/// Join requires login on headless mysql.
pub fn client_join_requires_login() -> bool {
    !json_personas_allowed()
}

/// Connect to MySQL and verify the frontend-owned Nexus schema.
pub fn init_mysql_identity(env: &AppEnv) -> Result<(), String> {
    let store = IdentityStore::open_mysql(&env.mysql)?;
    store.verify_schema()?;
    let users = store.user_count()?;
    let personas = store.persona_count()?;
    crate::nexus::log_nexus_to_blaze(format!(
        "Identity: MySQL ({}), {users} users, {personas} personas, token pepper {}",
        env.environment.as_str(),
        auth::pepper_fingerprint()
    ));
    *IDENTITY.lock() = Some(store);
    Ok(())
}

pub fn current_identity_store() -> Option<IdentityStore> {
    IDENTITY.lock().clone()
}

/// Bind a mysql client to a user+persona using the presented token. JSON/desktop skips this.
pub fn bind_mysql_client(
    presented: &str,
    claimed_user: i64,
    claimed_persona: i64,
) -> Result<BoundSession, String> {
    if !client_join_requires_login() {
        return Err("bind_mysql_client is only for datasource=mysql".into());
    }
    let store = current_identity_store().ok_or("mysql identity store is not ready")?;
    let bound = store.bind_client(presented, claimed_user, claimed_persona)?;
    record_bound_game(&store, presented);
    *BOUND_SESSION.lock() = Some(bound.clone());
    crate::nexus::log_nexus_to_blaze(format!(
        "Signed in: {} (user {}, persona {})",
        bound.display_name, bound.user_id, bound.persona_id
    ));
    Ok(bound)
}

/// Why a game client's Blaze login was refused.
#[derive(Debug, Clone)]
pub enum ClientLoginRefusal {
    /// `until_unix`: `None` = permanent.
    Banned {
        until_unix: Option<i64>,
        user_id: i64,
        persona_id: i64,
        discord_id: Option<String>,
    },
    Unauthorized(String),
}

/// Bind a game client from the Nexus session token sent in Blaze `LoginRequest.TOKN`.
pub fn bind_presented_client(presented: &str) -> Result<BoundSession, ClientLoginRefusal> {
    if !client_join_requires_login() {
        return Err(ClientLoginRefusal::Unauthorized(
            "bind_presented_client is only for datasource=mysql".into(),
        ));
    }
    let store = current_identity_store()
        .ok_or_else(|| ClientLoginRefusal::Unauthorized("mysql identity store is not ready".into()))?;
    let bound = store.resolve_presented(presented).map_err(|e| {
        let Some(session) = store.presented_session(presented).ok().flatten() else {
            return ClientLoginRefusal::Unauthorized(e);
        };
        match store.active_ban_until(session.user_id).ok().flatten() {
            Some(until_unix) => ClientLoginRefusal::Banned {
                until_unix,
                user_id: session.user_id,
                persona_id: session.persona_id,
                discord_id: store.user_discord_id(session.user_id).ok().flatten(),
            },
            None => ClientLoginRefusal::Unauthorized(e),
        }
    })?;
    record_bound_game(&store, presented);
    *BOUND_SESSION.lock() = Some(bound.clone());
    crate::nexus::log_nexus_to_blaze(format!(
        "Signed in: {} (user {}, persona {})",
        bound.display_name, bound.user_id, bound.persona_id
    ));
    Ok(bound)
}

/// A failed stamp must not refuse an otherwise valid login.
fn record_bound_game(store: &IdentityStore, presented: &str) {
    let game = crate::common::game::get_current_game_id();
    if let Err(e) = store.record_session_game(presented, &game) {
        tracing::warn!("nexus session game stamp failed: {e}");
    }
}

/// Active bound Nexus session for this process (mysql only).
pub fn current_bound_session() -> Option<BoundSession> {
    BOUND_SESSION.lock().clone()
}

/// Clear the in-process bound session (logout / revoke locally).
pub fn clear_bound_session() {
    *BOUND_SESSION.lock() = None;
}

pub fn persona_match_stats(persona_id: i64) -> Result<MatchStats, String> {
    let store = current_identity_store().ok_or("mysql identity store is not ready")?;
    store.match_stats(persona_id)
}

pub fn record_persona_match(persona_id: i64, victory: bool) -> Result<MatchStats, String> {
    let store = current_identity_store().ok_or("mysql identity store is not ready")?;
    let stats = store.record_match_outcome(persona_id, victory)?;
    tracing::info!(
        persona = stats.persona_id,
        result = if victory { "win" } else { "loss" },
        wins = stats.wins,
        losses = stats.losses,
        win_streak = stats.win_streak,
        loss_streak = stats.loss_streak,
        "persona record"
    );
    Ok(stats)
}

pub fn log_headless_identity_policy(env: &AppEnv) {
    match env.datasource {
        Datasource::Json => {
            crate::nexus::log_nexus_to_blaze(format!(
                "Identity: local JSON personas from {} ({})",
                crate::common::paths::settings_json_path().display(),
                env.environment.as_str()
            ));
        }
        Datasource::Mysql => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_personas_gate_join_policy() {
        enable_json_personas();
        assert!(json_personas_allowed());
        assert!(!client_join_requires_login());
        disable_json_personas();
        assert!(!json_personas_allowed());
        assert!(client_join_requires_login());
    }
}
