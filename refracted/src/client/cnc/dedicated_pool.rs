//! Dedicated-server pool registry (`returnDedicatedServerToPool`, creator registration).

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::blaze::tdf::TdfEncoder;

static POOL: OnceLock<Mutex<HashMap<u64, DedicatedServerEntry>>> = OnceLock::new();
static ASSIGNMENTS: OnceLock<Mutex<HashMap<i64, GameAssignment>>> = OnceLock::new();

fn pool() -> &'static Mutex<HashMap<u64, DedicatedServerEntry>> {
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn assignments() -> &'static Mutex<HashMap<i64, GameAssignment>> {
    ASSIGNMENTS.get_or_init(|| Mutex::new(HashMap::new()))
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

#[derive(Debug, Clone)]
struct GameAssignment {
    _client_session_id: u64,
    dedicated_session_id: u64,
}

/// Dedicated host endpoints used in client `NotifyGameSetup` (`THST` / `HNET`).
#[derive(Debug, Clone, Copy)]
pub struct DedicatedHostContext {
    pub persona_id: i64,
    pub inip_ip: i32,
    pub inip_port: i32,
    pub exip_ip: i32,
    pub exip_port: i32,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn ipv4_to_cnc_int(ip: Ipv4Addr) -> i32 {
    u32::from(ip) as i32
}

fn parse_peer_ipv4(peer: &str) -> i32 {
    peer.parse::<SocketAddr>()
        .ok()
        .and_then(|sa| match sa.ip() {
            std::net::IpAddr::V4(v4) => Some(ipv4_to_cnc_int(v4)),
            std::net::IpAddr::V6(_) => None,
        })
        .unwrap_or(0)
}

/// Game UDP port for pooled dedicated servers (Prism `BindDedicatedPoolUdpListen`).
const DEDICATED_GAME_UDP_PORT: i32 = 25200;

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

pub fn is_dedicated_blaze_session(blaze_session_id: u64) -> bool {
    get_entry(blaze_session_id)
        .map(|e| is_pool_candidate(e.clnt.as_deref()))
        .unwrap_or(false)
}

/// Pick an idle pool member with `registerDynamicDedicatedServerCreator` completed.
pub fn acquire_idle_creator(exclude_session_id: u64) -> Option<DedicatedServerEntry> {
    sync_from_blaze_sessions();
    let m = pool().lock();
    let mut candidates: Vec<_> = m
        .values()
        .filter(|e| {
            e.blaze_session_id != exclude_session_id
                && e.creator_registered
                && matches!(
                    e.state,
                    DedicatedPoolState::Idle | DedicatedPoolState::CreatorRegistered
                )
        })
        .cloned()
        .collect();
    candidates.sort_by_key(|e| match e.state {
        DedicatedPoolState::Idle => 0,
        DedicatedPoolState::CreatorRegistered => 1,
        _ => 2,
    });
    candidates.into_iter().next()
}

pub fn host_for_gid(gid: i64) -> Option<DedicatedHostContext> {
    let assignment = assignments().lock().get(&gid).cloned()?;
    let entry = get_entry(assignment.dedicated_session_id)?;
    let persona = entry.persona_id.unwrap_or(0) as i64;
    let inip_ip = parse_peer_ipv4(&entry.peer);
    let inip_port = DEDICATED_GAME_UDP_PORT;
    let session = crate::session::get_user_session();
    let exip_ip = session
        .network_exip_ip
        .map(|u| u as i32)
        .filter(|&ip| ip != 0)
        .unwrap_or(inip_ip);
    let exip_port = session
        .network_exip_port
        .filter(|&p| p != 0)
        .unwrap_or(inip_port);
    Some(DedicatedHostContext {
        persona_id: persona,
        inip_ip,
        inip_port,
        exip_ip,
        exip_port,
    })
}

/// Assign a pooled `cnc.server.exe` to a client `resetDedicatedServer` and queue cmd 220 notify.
pub fn orchestrate_client_reset(
    client_session_id: u64,
    gid: i64,
    request_payload: &[u8],
) -> Option<u64> {
    if is_dedicated_blaze_session(client_session_id) {
        return None;
    }
    let dedicated = acquire_idle_creator(client_session_id)?;
    let dedicated_sid = dedicated.blaze_session_id;
    {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&dedicated_sid) {
            e.state = DedicatedPoolState::InUse;
            e.current_gid = Some(gid);
            e.last_event_unix_secs = now_secs();
        }
    }
    assignments().lock().insert(
        gid,
        GameAssignment {
            _client_session_id: client_session_id,
            dedicated_session_id: dedicated_sid,
        },
    );
    let notify = super::build_notify_create_dynamic_dedicated_server_game(gid, request_payload)
        .ok()?;
    let push = super::fireframe::OutgoingPush {
        wire: super::fireframe::notification_envelope(0x0004, 220, &notify),
        component: 0x0004,
        command: 220,
        tdf_body: notify.to_vec(),
        blaze_send_label: "NotifyCreateDynamicDedicatedServerGame",
        info_log_line: format!(
            "[Blaze→Dedicated] GameManager.NotifyCreateDynamicDedicatedServerGame Component=4, Command=220, gid={}, dedicated_session={}",
            gid, dedicated_sid
        ),
    };
    super::fireframe::enqueue_pending_pushes(dedicated_sid, vec![push]);
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m assigned session #{} → gid={} (client session #{})",
        dedicated_sid,
        gid,
        client_session_id
    );
    Some(dedicated_sid)
}

pub fn release_gid(gid: i64) {
    assignments().lock().remove(&gid);
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
        e.state = DedicatedPoolState::Idle;
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
    if let Some(g) = gid {
        release_gid(g);
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
    assignments().lock().retain(|_, a| a.dedicated_session_id != blaze_session_id);
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
