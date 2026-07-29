//! Dedicated-server pool registry (`returnDedicatedServerToPool`, creator registration).

use parking_lot::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::blaze::tdf::TdfEncoder;

static POOL: OnceLock<Mutex<HashMap<u64, DedicatedServerEntry>>> = OnceLock::new();
static ASSIGNMENTS: OnceLock<Mutex<HashMap<i64, GameAssignment>>> = OnceLock::new();
static IDENTITIES: OnceLock<Mutex<HashMap<u64, DedicatedIdentity>>> = OnceLock::new();

fn pool() -> &'static Mutex<HashMap<u64, DedicatedServerEntry>> {
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn assignments() -> &'static Mutex<HashMap<i64, GameAssignment>> {
    ASSIGNMENTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn identities() -> &'static Mutex<HashMap<u64, DedicatedIdentity>> {
    IDENTITIES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
struct DedicatedIdentity {
    number: u32,
    name: String,
    persona_id: u64,
}

/// Allocate (or return the existing) distinct identity for a pooled dedicated `cnc.server.exe`:
/// display name `CNCO<N>` -- N = lowest number not currently in use, so a freed slot is reused --
/// plus a random persona id in a range that never collides with player profiles (static client is
/// 1201618778; generated profiles use 10_000_000_000_000+). Idempotent per session. Giving each
/// dedicated its OWN persona is what keeps `THST.HPID` distinct from every real client, so a joining
/// client connects OUT to the dedicated instead of assuming the topology-host role itself.
pub fn allocate_dedicated_identity(session_id: u64) -> (String, u64) {
    let mut m = identities().lock();
    if let Some(existing) = m.get(&session_id) {
        return (existing.name.clone(), existing.persona_id);
    }
    let used: std::collections::HashSet<u32> = m.values().map(|d| d.number).collect();
    let mut number = 1u32;
    while used.contains(&number) {
        number += 1;
    }
    let persona_id = 20_000_000_000_000u64 + rand::thread_rng().gen_range(0..9_999_999_999u64);
    let name = format!("CNCO{number}");
    m.insert(
        session_id,
        DedicatedIdentity { number, name: name.clone(), persona_id },
    );
    (name, persona_id)
}

fn free_dedicated_identity(session_id: u64) {
    identities().lock().remove(&session_id);
}

/// Return the pooled-dedicated identity (persona id, `CNCO<N>` name) assigned to `session_id`, if any.
/// Used so a dedicated server's own Blaze responses report its own persona rather than the client's.
pub fn dedicated_identity_for_session(session_id: u64) -> Option<(u64, String)> {
    identities()
        .lock()
        .get(&session_id)
        .map(|d| (d.persona_id, d.name.clone()))
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
    /// Host endpoints snapshotted when the dedicated was assigned, so NotifyGameSetup (`THST`/`HNET`)
    /// and the shell `serverID` stay correct even if the pool entry is later churned by a sync.
    host: DedicatedHostContext,
}

/// Dedicated host endpoints used in client `NotifyGameSetup` (`THST` / `HNET`).
#[derive(Debug, Clone, Copy)]
pub struct DedicatedHostContext {
    pub blaze_session_id: u64,
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
        .map(|e| e.creator_registered || is_pool_candidate(e.clnt.as_deref()))
        .unwrap_or(false)
}

/// Whether this Blaze TCP session is the dedicated game server (CLNT contains `server`).
pub fn blaze_wire_peer_is_server(blaze_session_id: u64) -> bool {
    if is_dedicated_blaze_session(blaze_session_id) {
        return true;
    }
    crate::session::blaze_sessions::get_session(blaze_session_id)
        .and_then(|s| s.clnt)
        .map(|clnt| clnt_qualifies_for_pool(&clnt))
        .unwrap_or(false)
}

/// Shell tag peer name for Blaze wire logs (`Client` vs `Server`).
pub fn blaze_wire_peer_label(blaze_session_id: u64) -> &'static str {
    if blaze_wire_peer_is_server(blaze_session_id) {
        "Server"
    } else {
        "Client"
    }
}

/// Dedicated sessions log as `[Server→Blaze]` / `[Blaze→Server]` instead of client tags.
pub fn normalize_blaze_wire_log_line(blaze_session_id: u64, line: impl AsRef<str>) -> String {
    let line = line.as_ref();
    if !blaze_wire_peer_is_server(blaze_session_id) {
        return line.to_string();
    }
    line.replace("[Client→Blaze]", "[Server→Blaze]")
        .replace("[Blaze→Client]", "[Blaze→Server]")
}

fn ensure_pool_entry(blaze_session_id: u64) {
    sync_from_blaze_sessions();
    if pool().lock().contains_key(&blaze_session_id) {
        return;
    }
    let entry = if let Some(s) = crate::session::blaze_sessions::get_session(blaze_session_id) {
        DedicatedServerEntry {
            blaze_session_id,
            peer: s.peer,
            clnt: s
                .clnt
                .clone()
                .or_else(|| Some("RtsBlazeServer".to_string())),
            display_name: s.display_name,
            persona_id: s.persona_id,
            state: DedicatedPoolState::Connected,
            current_gid: None,
            game_name: None,
            last_event_unix_secs: now_secs(),
            creator_registered: false,
        }
    } else {
        DedicatedServerEntry {
            blaze_session_id,
            peer: String::new(),
            clnt: Some("RtsBlazeServer".to_string()),
            display_name: None,
            persona_id: None,
            state: DedicatedPoolState::Connected,
            current_gid: None,
            game_name: None,
            last_event_unix_secs: now_secs(),
            creator_registered: false,
        }
    };
    pool().lock().insert(blaze_session_id, entry);
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

/// Dedicated session id a `resetDedicatedServer` for `gid` will resolve to: the existing
/// assignment if one was already made, otherwise the idle creator `orchestrate_client_reset`
/// would pick. Non-reserving, so it is safe to call while building the reset reply (before the
/// assignment is created) -- this is what lets the reply carry `GSID`/`SRVR` (shell `serverID`).
pub fn peek_dedicated_for_gid(gid: i64) -> Option<u64> {
    if let Some(a) = assignments().lock().get(&gid) {
        return Some(a.dedicated_session_id);
    }
    acquire_idle_creator(0).map(|e| e.blaze_session_id)
}

/// Build the host context for a pooled dedicated entry (persona + INIP/EXIP endpoints).
fn host_context_from_entry(entry: &DedicatedServerEntry) -> DedicatedHostContext {
    // The dedicated host identity (THST.HPID) must NOT equal a real player's persona. If it does --
    // which happens in test setups where the client and the cnc.server.exe authenticate with the same
    // profile -- the joining client sees THST.HPID == its own persona, concludes it is the topology
    // host itself, and never connects out to the dedicated at HNET (25200). Fall back to a synthetic
    // per-session id whenever the pooled dedicated's persona collides with the local client persona.
    let client_persona = crate::session::get_user_session().persona_id as u64;
    let persona = match entry.persona_id {
        Some(p) if p != 0 && p != client_persona => p,
        _ => 900_000_000_000 + entry.blaze_session_id,
    } as i64;
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
    DedicatedHostContext {
        blaze_session_id: entry.blaze_session_id,
        persona_id: persona,
        inip_ip,
        inip_port,
        exip_ip,
        exip_port,
    }
}

pub fn host_for_gid(gid: i64) -> Option<DedicatedHostContext> {
    // Prefer the snapshot taken at assignment time; fall back to a live pool lookup.
    if let Some(a) = assignments().lock().get(&gid) {
        return Some(a.host);
    }
    let entry = get_entry(acquire_idle_creator(0)?.blaze_session_id)?;
    Some(host_context_from_entry(&entry))
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
    if let Some(existing) = assignments().lock().get(&gid) {
        return Some(existing.dedicated_session_id);
    }
    let dedicated = acquire_idle_creator(client_session_id)?;
    let dedicated_sid = dedicated.blaze_session_id;
    let host = host_context_from_entry(&dedicated);
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
            host,
        },
    );
    super::game_state::begin_reset_orchestration(gid, client_session_id, dedicated_sid);
    let notify = super::build_notify_create_dynamic_dedicated_server_game(gid, request_payload)
        .ok()?;
    let push = super::fireframe::OutgoingPush {
        wire: super::fireframe::notification_envelope(0x0004, 220, &notify),
        component: 0x0004,
        command: 220,
        tdf_body: notify.to_vec(),
        blaze_send_label: "NotifyCreateDynamicDedicatedServerGame",
        info_log_line: format!(
            "[Blaze→Server] Match assignment sent (game {gid})"
        ),
    };
    let mut ded_pushes = vec![push];

    // Host notify + roster join after cmd 220 (retail pool semantics stay intact).
    let host_persona = dedicated
        .persona_id
        .map(|p| p as i64)
        .or_else(|| dedicated_identity_for_session(dedicated_sid).map(|(p, _)| p as i64))
        .unwrap_or(host.persona_id);
    match super::build_dedicated_host_notify_game_setup(
        gid,
        host_persona,
        host.inip_ip,
        host.inip_port,
        host.exip_ip,
        host.exip_port,
        dedicated_sid,
        request_payload,
    ) {
        Ok(setup) => {
            ded_pushes.push(super::fireframe::OutgoingPush {
                wire: super::fireframe::notification_envelope(0x0004, 0x0014, &setup),
                component: 0x0004,
                command: 0x0014,
                tdf_body: setup.to_vec(),
                blaze_send_label: "NotifyGameSetup (dedicated host)",
                info_log_line: format!(
                    "[Blaze→Server] Match setup sent to dedicated (game {gid})"
                ),
            });
        }
        Err(e) => {
            crate::debug_println!(
                "\x1b[38;2;255;120;120m[Dedicated pool]\x1b[0m host NotifyGameSetup build failed gid={}: {:?}",
                gid,
                e
            );
        }
    }

    // Announce the joining client to the dedicated roster -- not the host.
    // The game is seeded with the host only, so the client must be added to the roster first and
    // then carried in NotifyPlayerJoining's PDAT; otherwise the dedicated is told "the host is
    // joining its own game" and no client player is ever created.
    let client_session = crate::session::blaze_sessions::get_session(client_session_id);
    let client_persona = client_session
        .as_ref()
        .and_then(|session| session.persona_id)
        .unwrap_or_else(|| crate::session::get_user_session().persona_id) as i64;
    let client_name = client_session
        .as_ref()
        .and_then(|session| session.display_name.as_deref())
        .unwrap_or("");
    match super::game_state::ensure_client_player(gid, client_persona, client_name) {
        Some(client_player) => match super::build_game_manager_notify_player_joining(&client_player, gid) {
            Ok(pj) => {
                ded_pushes.push(super::fireframe::OutgoingPush {
                    wire: super::fireframe::notification_envelope(0x0004, 0x0015, &pj),
                    component: 0x0004,
                    command: 0x0015,
                    tdf_body: pj.to_vec(),
                    blaze_send_label: "NotifyPlayerJoining (client -> dedicated roster)",
                    info_log_line: format!(
                        "[Blaze→Server] Player join sent to dedicated roster (game {gid}, persona {client_persona})"
                    ),
                });
            }
            Err(e) => {
                crate::debug_println!(
                    "\x1b[38;2;255;120;120m[Dedicated pool]\x1b[0m NotifyPlayerJoining build failed gid={}: {:?}",
                    gid,
                    e
                );
            }
        },
        None => {
            crate::debug_println!(
                "\x1b[38;2;255;120;120m[Dedicated pool]\x1b[0m ensure_client_player returned None gid={}",
                gid
            );
        }
    }
    super::fireframe::enqueue_pending_pushes(dedicated_sid, ded_pushes);
    // Wake the dedicated's read loop right away -- its session task is usually blocked in
    // stream.read() and would otherwise only flush these on its next ping (~15s).
    let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
    super::msgsystem::log::log_orch_milestone(&format!(
        "Dedicated assigned to game {gid} (client #{client_session_id})"
    ));
    Some(dedicated_sid)
}

pub fn release_gid(gid: i64) {
    assignments().lock().remove(&gid);
    super::game_state::clear_orchestration(gid);
}

/// Blaze session id of the client that owns this `gid` assignment.
pub fn client_session_for_gid(gid: i64) -> Option<u64> {
    assignments()
        .lock()
        .get(&gid)
        .map(|a| a._client_session_id)
}

/// Reverse lookup: which game is the dedicated session hosting?
pub fn gid_for_dedicated_session(dedicated_session_id: u64) -> Option<i64> {
    assignments()
        .lock()
        .iter()
        .find(|(_, a)| a.dedicated_session_id == dedicated_session_id)
        .map(|(gid, _)| *gid)
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
        active_ids.contains(id)
            && (entry.creator_registered || is_pool_candidate(entry.clnt.as_deref()))
    });
    for s in sessions {
        // A session that already completed registerDynamicDedicatedServerCreator stays in the pool
        // regardless of its `clnt` string -- the CLNT->RtsBlazeServer patch does not always reach
        // blaze-main's preAuth CINF.CLNT, and evicting a registered creator here is what made
        // host_for_gid/acquire_idle_creator return None right after orchestrate assigned it.
        let already_registered = m.get(&s.id).map(|e| e.creator_registered).unwrap_or(false);
        if !already_registered && !is_pool_candidate(s.clnt.as_deref()) {
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
    ensure_pool_entry(blaze_session_id);
    // A dedicated that registers as a pool creator gets its OWN identity (CNCO<N> + distinct persona),
    // stamped onto the Blaze session so it never carries the shared client persona. This is the
    // reliable detection point -- the CLNT->RtsBlazeServer patch does not always reach preAuth.
    let (name, persona) = allocate_dedicated_identity(blaze_session_id);
    crate::session::blaze_sessions::set_dedicated_identity(blaze_session_id, &name, persona);
    let mut m = pool().lock();
    if let Some(e) = m.get_mut(&blaze_session_id) {
        e.creator_registered = true;
        e.state = DedicatedPoolState::Idle;
        e.persona_id = Some(persona);
        e.display_name = Some(name);
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
    // Free the CNCO<N> slot so a later dedicated can reuse the lowest free number.
    free_dedicated_identity(blaze_session_id);
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
    use super::{clnt_qualifies_for_pool, normalize_blaze_wire_log_line};

    #[test]
    fn clnt_includes_server_substring() {
        assert!(clnt_qualifies_for_pool("RtsBlazeServer"));
        assert!(clnt_qualifies_for_pool("cnc.server"));
        assert!(clnt_qualifies_for_pool("SomeServerThing"));
        assert!(!clnt_qualifies_for_pool("RtsBlazeClient"));
    }

    #[test]
    fn normalize_rewrites_client_tags_for_server_sessions() {
        // blaze_session_id 0 won't match pool; test the string rewrite in isolation.
        let line = "[Client→Blaze] PreAuthRequest clnt=RtsBlazeServer";
        assert_eq!(
            normalize_blaze_wire_log_line(0, line),
            line,
            "unknown session id should not rewrite"
        );
    }
}
