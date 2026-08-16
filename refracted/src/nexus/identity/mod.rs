//! Nexus identity: accounts (`users`) and attached `personas`.
//!
//! - **Desktop:** JSON / manual personas in `{data}/settings.json` (Settings → Accounts).
//! - **Headless `datasource=json`:** same JSON personas, **localized testing only**.
//! - **Headless `datasource=mysql`:** no JSON/manual personas. Game clients must
//!   authenticate before occupying a persona / joining a session (login/logout later).

mod migrate;
mod store;
mod auth;

pub use store::{BoundSession, IdentityStore, PersonaRecord, UserRecord};
pub use auth::{assert_bound_identity, IssuedCredentials};

use crate::common::app_env::{AppEnv, Datasource};

static IDENTITY: parking_lot::Mutex<Option<IdentityStore>> = parking_lot::Mutex::new(None);
/// Fail closed: JSON personas stay off until boot explicitly enables them.
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

/// Desktop, or headless `datasource=json` (localized testing). No-op after [`lock_identity_policy`].
pub(crate) fn enable_json_personas() {
    set_json_personas(true);
}

/// Headless production (`datasource=mysql`): JSON/manual personas are off.
pub(crate) fn disable_json_personas() {
    set_json_personas(false);
}

/// Freeze identity mode for process lifetime so a later call cannot flip mysql → json.
pub(crate) fn lock_identity_policy() {
    POLICY_LOCKED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// `true` on desktop and on headless only when `datasource=json`.
pub fn json_personas_allowed() -> bool {
    JSON_PERSONAS_ALLOWED.load(std::sync::atomic::Ordering::SeqCst)
}

/// Game client joining a session must be logged in. True on headless mysql.
/// Login/logout is not implemented yet; this is the policy gate.
pub fn client_join_requires_login() -> bool {
    !json_personas_allowed()
}

/// Connect to MySQL and run identity migrations. Does not import JSON profiles.
pub fn init_mysql_identity(env: &AppEnv) -> Result<(), String> {
    let store = IdentityStore::open_mysql(&env.mysql)?;
    store.migrate()?;
    let users = store.user_count()?;
    let personas = store.persona_count()?;
    crate::nexus::log_nexus_to_blaze(format!(
        "identity store ready (mysql) env={} users={users} personas={personas} (clients must authenticate)",
        env.environment.as_str()
    ));
    *IDENTITY.lock() = Some(store);
    Ok(())
}

pub fn current_identity_store() -> Option<IdentityStore> {
    IDENTITY.lock().clone()
}

/// Occupy a persona on mysql: the presented token/JWT must already be bound to that user+persona.
/// JSON/desktop skips this (localized Xevrac profile). Login will issue the token later.
pub fn bind_mysql_client(
    presented: &str,
    claimed_user: i64,
    claimed_persona: i64,
) -> Result<BoundSession, String> {
    if !client_join_requires_login() {
        return Err("bind_mysql_client is only for datasource=mysql".into());
    }
    let store = current_identity_store().ok_or("mysql identity store is not ready")?;
    store.bind_client(presented, claimed_user, claimed_persona)
}

pub fn log_headless_identity_policy(env: &AppEnv) {
    match env.datasource {
        Datasource::Json => {
            crate::nexus::log_nexus_to_blaze(format!(
                "headless datasource=json: localized testing — JSON/manual personas from {} (env={})",
                crate::common::paths::settings_json_path().display(),
                env.environment.as_str()
            ));
        }
        Datasource::Mysql => {
            crate::nexus::log_nexus_to_blaze(format!(
                "headless datasource=mysql: no JSON/manual personas; game clients must authenticate (env={})",
                env.environment.as_str()
            ));
        }
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
