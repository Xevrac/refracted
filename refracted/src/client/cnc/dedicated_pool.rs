//! Dedicated-server pool registry (`returnDedicatedServerToPool`, creator registration).

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::blaze::tdf::TdfEncoder;

static POOL: OnceLock<Mutex<HashMap<u64, DedicatedServerEntry>>> = OnceLock::new();

fn pool() -> &'static Mutex<HashMap<u64, DedicatedServerEntry>> {
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Whether a Blaze `CLNT` string should appear in the dedicated pool UI.
/// Loose match: any `server` / `Server` substring (case variants via lowercase scan).
pub fn clnt_qualifies_for_pool(clnt: &str) -> bool {
    clnt.to_ascii_lowercase().contains("server")
}

fn is_pool_candidate(clnt: Option<&str>) -> bool {
    clnt.map(clnt_qualifies_for_pool).unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DedicatedPoolState {
    Connected,
    CreatorRegistered,
    Idle,
    InUse,
}

impl DedicatedPoolState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::CreatorRegistered => "registered",
            Self::Idle => "idle (pool)",
            Self::InUse => "in use",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedicatedServerEntry {
    pub blaze_session_id: u64,
    pub peer: String,
    pub clnt: Option<String>,
    pub display_name: Option<String>,
    pub persona_id: Option<u64>,
    pub state: DedicatedPoolState,
    pub current_gid: Option<i64>,
    pub game_name: Option<String>,
    pub last_event_unix_secs: u64,
    pub creator_registered: bool,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn upsert_from_blaze_session(
    entry: &mut DedicatedServerEntry,
    s: &crate::session::blaze_sessions::BlazeSessionInfo,
) {
    entry.peer = s.peer.clone();
    entry.clnt = s.clnt.clone();
    entry.display_name = s.display_name.clone();
    entry.persona_id = s.persona_id;
    entry.last_event_unix_secs = now_secs();
}

/// Called when a Blaze session's `CLNT` field is observed or updated.
pub fn on_clnt_updated(blaze_session_id: u64, clnt: &str) {
    if !clnt_qualifies_for_pool(clnt) {
        pool().lock().remove(&blaze_session_id);
        return;
    }
    sync_from_blaze_sessions();
}

pub fn sync_from_blaze_sessions() {
    use crate::session::blaze_sessions;
    let sessions = blaze_sessions::list_sessions();
    let active_ids: std::collections::HashSet<u64> = sessions.iter().map(|s| s.id).collect();
    let mut m = pool().lock();
    m.retain(|id, entry| {
        active_ids.contains(id) && is_pool_candidate(entry.clnt.as_deref())
    });
    for s in sessions {
        if !is_pool_candidate(s.clnt.as_deref()) {
            m.remove(&s.id);
            continue;
        }
        let entry = m.entry(s.id).or_insert_with(|| DedicatedServerEntry {
            blaze_session_id: s.id,
            peer: s.peer.clone(),
            clnt: s.clnt.clone(),
            display_name: s.display_name.clone(),
            persona_id: s.persona_id,
            state: DedicatedPoolState::Connected,
            current_gid: None,
            game_name: None,
            last_event_unix_secs: now_secs(),
            creator_registered: false,
        });
        upsert_from_blaze_session(entry, &s);
    }
}

pub fn on_register_creator(blaze_session_id: u64) {
    sync_from_blaze_sessions();
    let mut m = pool().lock();
    if let Some(e) = m.get_mut(&blaze_session_id) {
        e.creator_registered = true;
        e.state = DedicatedPoolState::CreatorRegistered;
        e.last_event_unix_secs = now_secs();
    }
}

pub fn on_unregister_creator(blaze_session_id: u64) {
    let mut m = pool().lock();
    if let Some(e) = m.get_mut(&blaze_session_id) {
        e.creator_registered = false;
        e.state = DedicatedPoolState::Connected;
        e.last_event_unix_secs = now_secs();
    }
}

pub fn on_return_to_pool(blaze_session_id: u64, payload: &[u8]) {
    sync_from_blaze_sessions();
    let gid = TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64);
    let mut m = pool().lock();
    if let Some(e) = m.get_mut(&blaze_session_id) {
        e.state = DedicatedPoolState::Idle;
        e.current_gid = gid;
        e.game_name = None;
        e.last_event_unix_secs = now_secs();
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} returned to pool (gid={:?})",
            blaze_session_id,
            gid
        );
    }
}

pub fn on_game_active(blaze_session_id: u64, gid: i64, game_name: Option<String>) {
    sync_from_blaze_sessions();
    let mut m = pool().lock();
    if let Some(e) = m.get_mut(&blaze_session_id) {
        e.state = DedicatedPoolState::InUse;
        e.current_gid = Some(gid);
        e.game_name = game_name;
        e.last_event_unix_secs = now_secs();
    }
}

pub fn on_session_gone(blaze_session_id: u64) {
    pool().lock().remove(&blaze_session_id);
}

pub fn list_entries() -> Vec<DedicatedServerEntry> {
    sync_from_blaze_sessions();
    let mut v: Vec<_> = pool().lock().values().cloned().collect();
    v.sort_by_key(|e| e.blaze_session_id);
    v
}

pub fn get_entry(blaze_session_id: u64) -> Option<DedicatedServerEntry> {
    sync_from_blaze_sessions();
    pool().lock().get(&blaze_session_id).cloned()
}

#[cfg(test)]
mod tests {
    use super::clnt_qualifies_for_pool;

    #[test]
    fn clnt_includes_server_substring() {
        assert!(clnt_qualifies_for_pool("RtsBlazeServer"));
        assert!(clnt_qualifies_for_pool("cnc.server"));
        assert!(clnt_qualifies_for_pool("SomeServerThing"));
        assert!(!clnt_qualifies_for_pool("RtsBlazeClient"));
    }
}
