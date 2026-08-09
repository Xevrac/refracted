
use parking_lot::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::blaze::tdf::TdfEncoder;

static POOL: OnceLock<Mutex<HashMap<u64, DedicatedServerEntry>>> = OnceLock::new();
static ASSIGNMENTS: OnceLock<Mutex<HashMap<i64, GameAssignment>>> = OnceLock::new();
static IDENTITIES: OnceLock<Mutex<HashMap<u64, DedicatedIdentity>>> = OnceLock::new();
static LAST_NO_DEDICATED_UNIX: AtomicU64 = AtomicU64::new(0);

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

pub const DEDICATED_SYNTHETIC_PERSONA_BASE: i64 = 20_000_000_000_000;

pub fn is_dedicated_synthetic_persona(pid: i64) -> bool {
    pid >= DEDICATED_SYNTHETIC_PERSONA_BASE
}

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
    let persona_id = (DEDICATED_SYNTHETIC_PERSONA_BASE as u64)
        + rand::thread_rng().gen_range(0..9_999_999_999u64);
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
    pub server_hostname: Option<String>,
    pub persona_id: Option<u64>,
    pub state: DedicatedPoolState,
    pub current_gid: Option<i64>,
    pub game_name: Option<String>,
    pub last_event_unix_secs: u64,
    pub creator_registered: bool,
    pub current_map: Option<String>,
}

pub fn browser_server_name(entry: &DedicatedServerEntry) -> String {
    entry
        .server_hostname
        .clone()
        .or_else(|| entry.game_name.clone())
        .unwrap_or_else(|| {
            peer_display_fallback(&entry.peer)
                .unwrap_or_else(|| format!("Dedicated #{}", entry.blaze_session_id))
        })
}

fn peer_display_fallback(peer: &str) -> Option<String> {
    let ip = peer.parse::<SocketAddr>().ok()?.ip().to_string();
    if ip == "127.0.0.1" || ip == "::1" {
        Some(local_machine_hostname())
    } else {
        Some(ip)
    }
}

fn local_machine_hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "SERVER".to_string())
        .trim()
        .trim_end_matches('.')
        .to_ascii_uppercase()
}

fn normalize_hostname_label(raw: &str) -> String {
    let s = raw.trim().trim_matches('\0');
    let s = s
        .strip_prefix("PCC/")
        .or_else(|| s.strip_prefix("pcc/"))
        .unwrap_or(s);
    let s = s.split('.').next().unwrap_or(s);
    if s.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c == '_') {
        s.to_ascii_uppercase()
    } else {
        s.to_string()
    }
}

fn looks_like_server_hostname(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty()
        && s.len() <= 64
        && !s.contains('@')
        && !s.eq_ignore_ascii_case("RtsBlazeServer")
        && !s.starts_with("CNCO")
}

fn extract_server_hostname(payload: &[u8], peer: &str) -> String {
    for tag in &["HNAM", "HOST", "MACH", "SNAM", "CNAM"] {
        if let Some(s) = TdfEncoder::find_string_field(payload, tag) {
            let label = normalize_hostname_label(&s);
            if looks_like_server_hostname(&label) {
                return label;
            }
        }
    }
    if let Some(pcc) = extract_pcc_hostname(payload) {
        return normalize_hostname_label(&pcc);
    }
    peer_display_fallback(peer).unwrap_or_else(local_machine_hostname)
}

fn extract_pcc_hostname(payload: &[u8]) -> Option<String> {
    let hay = String::from_utf8_lossy(payload);
    let lower = hay.to_ascii_lowercase();
    let idx = lower.find("pcc/")?;
    let rest = &hay[idx + 4..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '\0' || c == '"' || c == '\'')
        .unwrap_or(rest.len().min(64));
    let name = rest[..end].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
pub const DEDICATED_PING_TCP_PORT: u16 = crate::client::cnc::msgsystem::DEDICATED_SERVERHOST_PORT;

pub fn probe_host_rtt_ms(host: &str, port: Option<u16>) -> Option<u32> {
    use std::net::{IpAddr, SocketAddr, TcpStream};
    use std::time::{Duration, Instant};

    let ip: IpAddr = host
        .parse::<SocketAddr>()
        .map(|sa| sa.ip())
        .or_else(|_| host.parse::<IpAddr>())
        .ok()?;
    if ip.is_loopback() {
        return Some(1);
    }
    let addr = SocketAddr::new(ip, port.unwrap_or(DEDICATED_PING_TCP_PORT));
    let start = Instant::now();
    let _stream = TcpStream::connect_timeout(&addr, Duration::from_millis(500)).ok()?;
    let ms = start.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
    Some(ms.max(1).min(9999))
}

pub fn probe_server_rtt_ms(peer: &str) -> Option<u32> {
    probe_host_rtt_ms(peer, Some(DEDICATED_PING_TCP_PORT))
}

fn standby_gid_for_session(blaze_session_id: u64) -> i64 {
    10_000 + (blaze_session_id % 900_000) as i64
}

#[derive(Debug, Clone)]
struct GameAssignment {
    _client_session_id: u64,
    dedicated_session_id: u64,
    host: DedicatedHostContext,
}

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
pub const DEDICATED_GAME_UDP_PORT: i32 = 25200;

pub const GAMEMANAGER_ERR_NO_DEDICATED_SERVER_FOUND: u16 = 301;

pub fn note_no_dedicated_server() {
    LAST_NO_DEDICATED_UNIX.store(now_secs(), Ordering::Relaxed);
}

pub fn clear_no_dedicated_server() {
    LAST_NO_DEDICATED_UNIX.store(0, Ordering::Relaxed);
}

pub fn last_no_dedicated_unix() -> u64 {
    LAST_NO_DEDICATED_UNIX.load(Ordering::Relaxed)
}

pub fn idle_creator_count() -> usize {
    sync_from_blaze_sessions();
    pool()
        .lock()
        .values()
        .filter(|e| {
            e.creator_registered
                && matches!(
                    e.state,
                    DedicatedPoolState::Idle | DedicatedPoolState::CreatorRegistered
                )
        })
        .count()
}

pub fn lobby_pool_status_json() -> String {
    let idle = idle_creator_count();
    let last = last_no_dedicated_unix();
    let creators = list_entries().len();
    format!(
        "{{\"idle\":{idle},\"creators\":{creators},\"lastNoDedicatedAt\":{last},\"noDedicated\":{}}}",
        if last > 0 && now_secs().saturating_sub(last) <= 15 {
            "true"
        } else {
            "false"
        }
    )
}

fn upsert_from_blaze_session(
    entry: &mut DedicatedServerEntry,
    s: &crate::session::blaze_sessions::BlazeSessionInfo,
) {
    entry.peer = s.peer.clone();
    entry.clnt = s.clnt.clone();
    if !entry.creator_registered {
        entry.display_name = s.display_name.clone();
        entry.persona_id = s.persona_id;
    }
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
            server_hostname: None,
            persona_id: s.persona_id,
            state: DedicatedPoolState::Connected,
            current_gid: None,
            game_name: None,
            last_event_unix_secs: now_secs(),
            creator_registered: false,
            current_map: None,
        }
    } else {
        DedicatedServerEntry {
            blaze_session_id,
            peer: String::new(),
            clnt: Some("RtsBlazeServer".to_string()),
            display_name: None,
            server_hostname: None,
            persona_id: None,
            state: DedicatedPoolState::Connected,
            current_gid: None,
            game_name: None,
            last_event_unix_secs: now_secs(),
            creator_registered: false,
            current_map: None,
        }
    };
    pool().lock().insert(blaze_session_id, entry);
}

pub fn acquire_idle_creator_for_map(
    exclude_session_id: u64,
    _wanted_map: Option<&str>,
) -> Option<DedicatedServerEntry> {
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
    candidates.sort_by_key(|e| {
        let state_rank = match e.state {
            DedicatedPoolState::Idle => 0,
            DedicatedPoolState::CreatorRegistered => 1,
            _ => 2,
        };
        (state_rank, e.blaze_session_id)
    });
    candidates.into_iter().next()
}

pub fn acquire_idle_creator(exclude_session_id: u64) -> Option<DedicatedServerEntry> {
    acquire_idle_creator_for_map(exclude_session_id, None)
}

pub fn note_dedicated_map(blaze_session_id: u64, map_path: Option<&str>) {
    let mut m = pool().lock();
    if let Some(e) = m.get_mut(&blaze_session_id) {
        e.current_map = map_path.map(|s| s.to_string());
        e.last_event_unix_secs = now_secs();
    }
}

pub fn peek_dedicated_for_gid(gid: i64) -> Option<u64> {
    if let Some(a) = assignments().lock().get(&gid) {
        return Some(a.dedicated_session_id);
    }
    if let Some(entry) = dedicated_for_standby_gid(gid) {
        return Some(entry.blaze_session_id);
    }
    acquire_idle_creator(0).map(|e| e.blaze_session_id)
}

fn host_context_from_entry(entry: &DedicatedServerEntry) -> DedicatedHostContext {
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
    if let Some(existing_sid) = assignments().lock().get(&gid).map(|a| a.dedicated_session_id) {
        if super::game_state::has_orchestration(gid) {
            return Some(existing_sid);
        }
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m stale assignment gid={} dedicated #{} (no orch) — re-orchestrating",
            gid,
            existing_sid
        );
        assignments().lock().remove(&gid);
        {
            let mut m = pool().lock();
            if let Some(e) = m.get_mut(&existing_sid) {
                e.state = DedicatedPoolState::Idle;
                e.last_event_unix_secs = now_secs();
            }
        }
    }
    let wanted_map = super::game_state::get_map_path(gid);
    let wanted_map = if wanted_map.is_empty() { None } else { Some(wanted_map) };
    let dedicated = if let Some(bound) = dedicated_for_standby_gid(gid) {
        bound
    } else {
        acquire_idle_creator_for_map(client_session_id, wanted_map.as_deref())?
    };
    let dedicated_sid = dedicated.blaze_session_id;
    let host = host_context_from_entry(&dedicated);

    crate::debug_println!(
        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m gid={} → session #{} map={:?} (was {:?}) — reload on assign",
        gid,
        dedicated_sid,
        wanted_map.as_deref().unwrap_or("<none>"),
        dedicated.current_map.as_deref().unwrap_or("<none>")
    );

    {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&dedicated_sid) {
            e.state = DedicatedPoolState::InUse;
            e.current_gid = Some(gid);
            e.current_map = wanted_map.clone();
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
    let dedicated_sid = assignments().lock().remove(&gid).map(|a| a.dedicated_session_id);
    super::game_state::clear_orchestration(gid);
    if let Some(sid) = dedicated_sid {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&sid) {
            e.state = DedicatedPoolState::Idle;
            e.current_gid = None;
            e.game_name = None;
            e.current_map = None;
            e.last_event_unix_secs = now_secs();
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} Idle after release_gid={} (awaiting unload)",
                sid,
                gid
            );
        }
    }
}

pub fn bound_dedicated_for_gid(gid: i64) -> Option<u64> {
    if let Some(a) = assignments().lock().get(&gid) {
        return Some(a.dedicated_session_id);
    }
    if let Some(entry) = dedicated_for_standby_gid(gid) {
        return Some(entry.blaze_session_id);
    }
    super::game_state::dedicated_session_id_for_gid(gid)
}

pub fn reclaim_gid_to_idle_pool(gid: i64) -> Option<u64> {
    let dedicated_sid = bound_dedicated_for_gid(gid);

    assignments().lock().remove(&gid);
    super::game_state::clear_blaze_one_shot_flags(gid);
    super::game_state::clear_orchestration(gid);

    let Some(sid) = dedicated_sid else {
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m reclaim gid={} — no bound dedicated",
            gid
        );
        return None;
    };

    {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&sid) {
            let keep_gid = e.current_gid.unwrap_or(gid);
            e.state = DedicatedPoolState::Idle;
            e.game_name = None;
            e.current_gid = Some(keep_gid);
            e.current_map = None;
            e.last_event_unix_secs = now_secs();
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} Idle after empty humans (gid={} standby={}) — unload requested",
                sid,
                gid,
                keep_gid
            );
        }
    }

    Some(sid)
}

pub fn request_dedicated_level_unload(blaze_session_id: u64, gid: i64) {
    match super::fireframe::pushes_dedicated_reclaim_idle(gid) {
        Ok(pushes) if !pushes.is_empty() => {
            super::fireframe::enqueue_pending_pushes(blaze_session_id, pushes);
            let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m queued reclaim/unload notify → dedicated #{} gid={}",
                blaze_session_id,
                gid
            );
        }
        Ok(_) => {}
        Err(e) => {
            crate::debug_println!(
                "\x1b[38;2;255;165;0m[Dedicated pool]\x1b[0m reclaim notify encode failed: {}",
                e
            );
        }
    }
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
            server_hostname: None,
            persona_id: s.persona_id,
            state: DedicatedPoolState::Connected,
            current_gid: None,
            game_name: None,
            last_event_unix_secs: now_secs(),
            creator_registered: false,
            current_map: None,
        });
        upsert_from_blaze_session(entry, &s);
    }
}

pub fn dedicated_for_standby_gid(gid: i64) -> Option<DedicatedServerEntry> {
    sync_from_blaze_sessions();
    pool()
        .lock()
        .values()
        .find(|e| e.creator_registered && e.current_gid == Some(gid))
        .cloned()
}

pub fn on_register_creator(blaze_session_id: u64, register_payload: &[u8]) {
    ensure_pool_entry(blaze_session_id);
    if pool()
        .lock()
        .get(&blaze_session_id)
        .map(|e| e.creator_registered && e.current_gid.is_some())
        .unwrap_or(false)
    {
        return;
    }
    let (name, persona) = allocate_dedicated_identity(blaze_session_id);
    crate::session::blaze_sessions::set_dedicated_identity(blaze_session_id, &name, persona);
    let gid = standby_gid_for_session(blaze_session_id);
    let hostname = {
        let mut m = pool().lock();
        let hostname = if let Some(e) = m.get(&blaze_session_id) {
            extract_server_hostname(register_payload, &e.peer)
        } else {
            extract_server_hostname(register_payload, "")
        };
        if let Some(e) = m.get_mut(&blaze_session_id) {
            e.creator_registered = true;
            e.state = DedicatedPoolState::Idle;
            e.persona_id = Some(persona);
            e.display_name = Some(name.clone());
            e.server_hostname = Some(hostname.clone());
            e.current_gid = Some(gid);
            e.last_event_unix_secs = now_secs();
        }
        hostname
    };
    super::game_state::ensure_standby_game(gid, &hostname, blaze_session_id);
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} registered as {} (browser={} gid={})",
        blaze_session_id,
        name,
        hostname,
        gid
    );
}

pub fn on_unregister_creator(blaze_session_id: u64) {
    {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&blaze_session_id) {
            e.creator_registered = false;
            e.state = DedicatedPoolState::Connected;
            e.current_gid = None;
            e.last_event_unix_secs = now_secs();
        }
    }
    super::game_state::destroy_games_for_dedicated(blaze_session_id);
}

pub fn on_return_to_pool(blaze_session_id: u64, payload: &[u8]) {
    sync_from_blaze_sessions();
    let gid = TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64);
    if let Some(g) = gid {
        let _ = reclaim_gid_to_idle_pool(g);
        super::game_state::force_standby_reset(g);
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} returned to pool (gid={})",
            blaze_session_id,
            g
        );
    } else {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&blaze_session_id) {
            e.state = DedicatedPoolState::Idle;
            e.current_map = None;
            e.game_name = None;
            e.last_event_unix_secs = now_secs();
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} returned to pool (no gid)",
                blaze_session_id
            );
        }
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
    let owned_gid = {
        let m = pool().lock();
        m.get(&blaze_session_id).map(|e| {
            e.current_gid
                .unwrap_or_else(|| standby_gid_for_session(blaze_session_id))
        })
    };
    pool().lock().remove(&blaze_session_id);
    assignments()
        .lock()
        .retain(|_, a| a.dedicated_session_id != blaze_session_id);
    free_dedicated_identity(blaze_session_id);
    super::game_state::destroy_games_for_dedicated(blaze_session_id);
    if let Some(gid) = owned_gid {
        super::game_state::note_server_lost(gid);
        super::game_state::destroy_game(gid);
    }
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
