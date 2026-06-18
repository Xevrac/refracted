use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::paths::{ensure_parent_dir, sessions_json_path};

use super::session_module::{
    clone_user_session_if_set, get_build_profile_name, get_build_profile_source,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn registry() -> &'static Mutex<HashMap<u64, BlazeSessionEntry>> {
    static REG: OnceLock<Mutex<HashMap<u64, BlazeSessionEntry>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlazeSessionEntry {
    pub id: u64,
    pub peer: String,
    pub listener: String,
    pub connected_unix_secs: u64,
    pub crypto_enabled: bool,
    pub authenticated: bool,
    pub display_name: Option<String>,
    pub user_id: Option<u64>,
    pub persona_id: Option<u64>,
    pub email: Option<String>,
    /// Blaze client name/type string from request `CLNT` (e.g. `RtsBlazeClient` vs dedicated server).
    #[serde(default)]
    pub clnt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BlazeSessionInfo {
    pub id: u64,
    pub peer: String,
    pub listener: String,
    pub connected_unix_secs: u64,
    pub crypto_enabled: bool,
    pub authenticated: bool,
    pub display_name: Option<String>,
    pub user_id: Option<u64>,
    pub persona_id: Option<u64>,
    pub email: Option<String>,
    pub clnt: Option<String>,
    pub build_profile: String,
    pub build_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionsFile {
    next_id: u64,
    sessions: Vec<BlazeSessionEntry>,
}

fn persist_sessions_to_disk() {
    let path = sessions_json_path();
    let next_id = NEXT_ID.load(Ordering::Relaxed);
    let sessions: Vec<BlazeSessionEntry> = {
        let m = registry().lock();
        m.values().cloned().collect()
    };
    let doc = SessionsFile { next_id, sessions };
    let Ok(json) = serde_json::to_string_pretty(&doc) else {
        return;
    };
    if ensure_parent_dir(&path).is_err() {
        return;
    }
    if let Err(e) = fs::write(&path, json) {
        tracing::warn!("failed to write {}: {}", path.display(), e);
    }
}

/// Load registry from `data/sessions.json` (e.g. after restart). Safe if file is missing or invalid.
pub fn load_persisted_sessions() {
    let path = sessions_json_path();
    if !path.exists() {
        return;
    }
    let Ok(text) = fs::read_to_string(&path) else {
        tracing::warn!("failed to read {}", path.display());
        return;
    };
    let Ok(file) = serde_json::from_str::<SessionsFile>(&text) else {
        tracing::warn!("invalid JSON in {}", path.display());
        return;
    };
    let mut m = registry().lock();
    m.clear();
    for e in file.sessions {
        m.insert(e.id, e);
    }
    let floor = m.keys().copied().max().unwrap_or(0).saturating_add(1);
    let next = file.next_id.max(floor);
    NEXT_ID.store(next, Ordering::Relaxed);
}

fn entry_to_info(e: BlazeSessionEntry, build_profile: String, build_source: String) -> BlazeSessionInfo {
    BlazeSessionInfo {
        id: e.id,
        peer: e.peer,
        listener: e.listener,
        connected_unix_secs: e.connected_unix_secs,
        crypto_enabled: e.crypto_enabled,
        authenticated: e.authenticated,
        display_name: e.display_name,
        user_id: e.user_id,
        persona_id: e.persona_id,
        email: e.email,
        clnt: e.clnt.clone(),
        build_profile,
        build_source,
    }
}

pub fn register(peer: std::net::SocketAddr, listener: &str) -> u64 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let connected_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut m = registry().lock();
        m.insert(
        id,
        BlazeSessionEntry {
            id,
            peer: peer.to_string(),
            listener: listener.to_string(),
            connected_unix_secs,
            crypto_enabled: false,
            authenticated: false,
            display_name: None,
            user_id: None,
            persona_id: None,
            email: None,
            clnt: None,
        },
    );
    drop(m);
    sync_all_from_global_session();
    id
}

/// Copy LSX/global user fields into every active Blaze row (for Sessions UI). Safe to call often.
pub fn sync_all_from_global_session() {
    let snapshot = clone_user_session_if_set();
    let mut m = registry().lock();
    if let Some(s) = snapshot {
        for e in m.values_mut() {
            if !s.display_name.is_empty() {
                e.display_name = Some(s.display_name.clone());
            }
            e.user_id = Some(s.user_id);
            e.persona_id = Some(s.persona_id);
            if !s.email.is_empty() {
                e.email = Some(s.email.clone());
            } else {
                e.email = None;
            }
        }
    }
    drop(m);
    persist_sessions_to_disk();
}

pub fn unregister(id: u64) {
    let mut m = registry().lock();
    m.remove(&id);
    drop(m);
    if crate::common::game::get_current_game_id().as_str() == "cnc" {
        crate::client::cnc::dedicated_pool::on_session_gone(id);
    }
    persist_sessions_to_disk();
}

pub struct BlazeSessionGuard(u64);

impl BlazeSessionGuard {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl Drop for BlazeSessionGuard {
    fn drop(&mut self) {
        unregister(self.0);
    }
}

/// Returns `true` if the registry was updated. Uses `try_lock` so Blaze I/O never blocks on the Sessions UI.
pub fn set_crypto_enabled(id: u64, enabled: bool) -> bool {
    let Some(mut m) = registry().try_lock() else {
        return false;
    };
    if let Some(e) = m.get_mut(&id) {
        e.crypto_enabled = enabled;
    }
    drop(m);
    persist_sessions_to_disk();
    true
}

/// If a Util `preAuth` payload contains `CLNT`, persist it on the Blaze session row.
pub fn note_clnt_if_preauth(id: u64, component: u16, command: u16, payload: &[u8]) {
    if component != 0x0009 || command != 0x0007 {
        return;
    }
    note_clnt_from_payload(id, payload);
}

fn note_clnt_from_payload(id: u64, payload: &[u8]) {
    let Some(clnt) = crate::blaze::tdf::TdfEncoder::find_string_field(payload, "CLNT") else {
        return;
    };
    let clnt = clnt.trim().to_string();
    if clnt.is_empty() {
        return;
    }
    let changed = {
        let mut m = registry().lock();
        let Some(e) = m.get_mut(&id) else {
            return;
        };
        if e.clnt.as_deref() == Some(clnt.as_str()) {
            false
        } else {
            e.clnt = Some(clnt.clone());
            true
        }
    };
    if !changed {
        return;
    }
    persist_sessions_to_disk();
    if crate::common::game::get_current_game_id().as_str() == "cnc" {
        crate::client::cnc::dedicated_pool::on_clnt_updated(id, &clnt);
    }
}

pub fn mark_authenticated(id: u64) {
    sync_all_from_global_session();
    let mut m = registry().lock();
    if let Some(e) = m.get_mut(&id) {
        e.authenticated = true;
    } else {
        tracing::warn!(
            "mark_authenticated: no registry entry for blaze session id {}",
            id
        );
    }
    drop(m);
    persist_sessions_to_disk();
}

pub fn list_sessions() -> Vec<BlazeSessionInfo> {
    let entries: Vec<BlazeSessionEntry> = {
        let m = registry().lock();
        m.values().cloned().collect()
    };
    let build_profile = get_build_profile_name().to_string();
    let build_source = get_build_profile_source();
    let mut v: Vec<BlazeSessionInfo> = entries
        .into_iter()
        .map(|e| entry_to_info(e, build_profile.clone(), build_source.clone()))
        .collect();
    v.sort_by_key(|i| i.id);
    v
}

pub fn active_count() -> usize {
    registry().lock().len()
}

pub fn authenticated_count() -> usize {
    registry().lock().values().filter(|e| e.authenticated).count()
}

pub fn get_session(id: u64) -> Option<BlazeSessionInfo> {
    let e = {
        let m = registry().lock();
        m.get(&id).cloned()?
    };
    let build_profile = get_build_profile_name().to_string();
    let build_source = get_build_profile_source();
    Some(entry_to_info(e, build_profile, build_source))
}
