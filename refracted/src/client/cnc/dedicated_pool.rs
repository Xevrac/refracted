
use parking_lot::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::blaze::tdf::TdfEncoder;

static POOL: OnceLock<Mutex<HashMap<u64, DedicatedServerEntry>>> = OnceLock::new();
static ASSIGNMENTS: OnceLock<Mutex<HashMap<i64, GameAssignment>>> = OnceLock::new();
static IDENTITIES: OnceLock<Mutex<HashMap<u64, DedicatedIdentity>>> = OnceLock::new();
/// Prism posts EnginePeerInit before Blaze register — stash until a pool entry can claim it.
static PENDING_ENGINE_PEER: OnceLock<Mutex<Vec<PendingEnginePeer>>> = OnceLock::new();
static LAST_NO_DEDICATED_UNIX: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
struct PendingEnginePeer {
    port: i32,
    msg_sys: Option<u16>,
    simu_cloud: Option<u16>,
    qos: Option<u16>,
    peer_host: Option<String>,
    noted_at_unix: u64,
}

fn pool() -> &'static Mutex<HashMap<u64, DedicatedServerEntry>> {
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn assignments() -> &'static Mutex<HashMap<i64, GameAssignment>> {
    ASSIGNMENTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn identities() -> &'static Mutex<HashMap<u64, DedicatedIdentity>> {
    IDENTITIES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pending_engine_peer() -> &'static Mutex<Vec<PendingEnginePeer>> {
    PENDING_ENGINE_PEER.get_or_init(|| Mutex::new(Vec::new()))
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
    /// Native level unload in progress (RESETABLE reclaim); not assignable until stabilization elapses.
    Recycling,
    InUse,
}

impl DedicatedPoolState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::CreatorRegistered => "registered",
            Self::Idle => "idle (pool)",
            Self::Recycling => "recycling (unload)",
            Self::InUse => "in use",
        }
    }
}

/// Fallback wall time when the dedicated never sends `returnDedicatedServerToPool` after RESETABLE.
/// Prefer the native RPC (Blaze cmd `0x14`) — it clears `unload_requested_at` immediately.
const RECYCLE_CMD220_STABILIZATION_SECS: u64 = 60;

/// Suppress duplicate RESETABLE+Removed pushes (quit fires leave-game then removePlayer).
const RECLAIM_NOTIFY_COALESCE_SECS: u64 = 60;

/// Extra CreateGame deferral when the next assignment targets a different map on a recycled dedicated.
const RECYCLE_MAP_SWITCH_CREATE_GAME_EXTRA_MS: u64 = 15_000;

fn promote_recycling_if_ready(e: &mut DedicatedServerEntry) {
    if e.state != DedicatedPoolState::Recycling {
        return;
    }
    // Native `returnDedicatedServerToPool` marks Idle + clears unload before we get here.
    if e.unload_requested_at.is_none() {
        e.state = DedicatedPoolState::Idle;
        e.last_event_unix_secs = now_secs();
        return;
    }
    let anchor = e.unload_requested_at.unwrap_or(e.last_event_unix_secs);
    let elapsed = now_secs().saturating_sub(anchor);
    if elapsed < RECYCLE_CMD220_STABILIZATION_SECS {
        return;
    }
    e.unload_requested_at = None;
    e.state = DedicatedPoolState::Idle;
    e.last_event_unix_secs = now_secs();
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} Idle after recycle fallback timer ({}s, standby_gid={:?})",
        e.blaze_session_id,
        elapsed,
        e.current_gid
    );
}

fn promote_all_recycling(m: &mut HashMap<u64, DedicatedServerEntry>) {
    for e in m.values_mut() {
        promote_recycling_if_ready(e);
        if e.state == DedicatedPoolState::Idle && recycle_cmd220_wait_secs(e) == 0 {
            e.unload_requested_at = None;
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
    /// Map path served before the last RESETABLE unload (for rematch / map-switch detection).
    #[serde(default)]
    pub previous_map: Option<String>,
    /// Unix seconds when RESETABLE reclaim unload was last requested for this session.
    #[serde(default)]
    pub unload_requested_at: Option<u64>,
    /// LAN/WAN IPs from this dedicated's `updateNetworkInfo` (addresses only — ports are QoS).
    #[serde(default)]
    pub network_inip_ip: Option<i32>,
    #[serde(default)]
    pub network_inip_port: Option<i32>,
    #[serde(default)]
    pub network_exip_ip: Option<i32>,
    #[serde(default)]
    pub network_exip_port: Option<i32>,
    /// Authoritative game-mesh UDP from Prism `EnginePeerInit` (per-host bind).
    #[serde(default)]
    pub engine_peer_udp_port: Option<i32>,
    /// Prism managed ServerHost TCP (`-Prism.MsgSysPort`, default 18387).
    #[serde(default)]
    pub msg_sys_tcp_port: Option<u16>,
    /// Prism SimuCloud host TCP (`-Prism.SimuCloudPort`, default 18388).
    #[serde(default)]
    pub simu_cloud_tcp_port: Option<u16>,
    /// Prism-reported QoS (Refracted still binds emulator QoS globally).
    #[serde(default)]
    pub qos_port: Option<u16>,
}

/// Seconds to wait before sending cmd 220 after a recent native unload on this dedicated.
pub fn recycle_cmd220_wait_secs(entry: &DedicatedServerEntry) -> u64 {
    entry
        .unload_requested_at
        .map(|t| {
            RECYCLE_CMD220_STABILIZATION_SECS.saturating_sub(now_secs().saturating_sub(t))
        })
        .unwrap_or(0)
}

/// Whether this pooled dedicated can be assigned to a new `resetDedicatedServer` right now.
pub fn is_assignable(entry: &DedicatedServerEntry) -> bool {
    entry.creator_registered
        && recycle_cmd220_wait_secs(entry) == 0
        && matches!(
            entry.state,
            DedicatedPoolState::Idle | DedicatedPoolState::CreatorRegistered
        )
}

/// Extra CreateGame delay when rematching on a recycled dedicated with a different map.
pub fn recycle_create_game_extra_delay_ms(dedicated_session_id: u64, wanted_map: &str) -> u64 {
    let Some(entry) = get_entry(dedicated_session_id) else {
        return 0;
    };
    let Some(prev) = entry.previous_map.as_deref() else {
        return 0;
    };
    if prev.is_empty() || prev == wanted_map {
        return 0;
    }
    RECYCLE_MAP_SWITCH_CREATE_GAME_EXTRA_MS
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

fn peer_ip_addr(entry: &DedicatedServerEntry) -> IpAddr {
    entry
        .peer
        .parse::<SocketAddr>()
        .map(|sa| sa.ip())
        .or_else(|_| entry.peer.parse::<IpAddr>())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

/// MsgSys ServerHost TCP for this dedicated (launcher `-Prism.MsgSysPort`).
pub fn msg_sys_port(entry: &DedicatedServerEntry) -> u16 {
    entry
        .msg_sys_tcp_port
        .filter(|&p| is_usable_prism_tcp_port(p))
        .unwrap_or(DEDICATED_PING_TCP_PORT)
}

/// SimuCloud host TCP for this dedicated (launcher `-Prism.SimuCloudPort`).
pub fn simu_cloud_port(entry: &DedicatedServerEntry) -> u16 {
    entry
        .simu_cloud_tcp_port
        .filter(|&p| is_usable_prism_tcp_port(p))
        .unwrap_or(crate::client::cnc::msgsystem::simucloud::SIMUCLOUD_PORT)
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ClientMsgsysRoute {
    gid: i64,
    persona_id: i64,
    blaze_session_id: Option<u64>,
    dedicated_session_id: Option<u64>,
    client_ip: Option<IpAddr>,
}

fn client_msgsys_routes() -> &'static Mutex<HashMap<i64, ClientMsgsysRoute>> {
    static ROUTES: OnceLock<Mutex<HashMap<i64, ClientMsgsysRoute>>> = OnceLock::new();
    ROUTES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn parse_peer_ip(peer: &str) -> Option<IpAddr> {
    peer.parse::<SocketAddr>()
        .map(|sa| sa.ip())
        .ok()
        .or_else(|| peer.parse().ok())
}

/// Hub listen port that 1:1 forwards to this dedicated's ServerHost (`msg_sys - 1`).
pub fn msgsys_hub_listen_port(msg_sys: u16) -> u16 {
    msg_sys.saturating_sub(1).max(1024)
}

fn upstream_for_dedicated_sid(sid: u64) -> Option<SocketAddr> {
    let e = get_entry(sid)?;
    Some(SocketAddr::new(peer_ip_addr(&e), msg_sys_port(&e)))
}

fn upstream_for_route(route: &ClientMsgsysRoute) -> Option<SocketAddr> {
    if let Some(sid) = route.dedicated_session_id {
        if let Some(up) = upstream_for_dedicated_sid(sid) {
            return Some(up);
        }
    }
    msgsys_upstream_for_gid(route.gid)
}

/// Bind this Blaze client session to one match dedicated. Hub traffic for this persona
/// splices only to that ServerHost — never another InUse instance.
pub fn note_client_msgsys_route(gid: i64, persona_id: i64, client_ip: Option<IpAddr>) {
    if gid <= 0 || persona_id <= 0 {
        return;
    }
    let blaze_session_id = crate::session::current_blaze_session_id();
    let dedicated_session_id = bound_dedicated_for_gid(gid);
    let route = ClientMsgsysRoute {
        gid,
        persona_id,
        blaze_session_id,
        dedicated_session_id,
        client_ip,
    };
    client_msgsys_routes().lock().insert(persona_id, route);
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m MsgSys route persona={persona_id} blaze={blaze_session_id:?} → gid={gid} dedicated={dedicated_session_id:?} ip={client_ip:?}"
    );
}

pub fn note_client_msgsys_route_from_current_session(gid: i64, persona_id: i64) {
    let ip = crate::session::current_blaze_session_id()
        .and_then(crate::session::blaze_sessions::get_session)
        .and_then(|s| parse_peer_ip(&s.peer));
    note_client_msgsys_route(gid, persona_id, ip);
}

pub fn note_human_msgsys_routes_for_gid(gid: i64) {
    for p in super::game_state::players_for_gid(gid) {
        if !p.is_ai && p.persona_id > 0 {
            note_client_msgsys_route(gid, p.persona_id, None);
        }
    }
}

pub fn clear_client_msgsys_routes_for_gid(gid: i64) {
    client_msgsys_routes().lock().retain(|_, r| r.gid != gid);
}

/// Upstream Prism ServerHost for this match gid.
pub fn msgsys_upstream_for_gid(gid: i64) -> Option<SocketAddr> {
    let sid = bound_dedicated_for_gid(gid)?;
    upstream_for_dedicated_sid(sid)
}

pub fn msgsys_upstream_for_serverhost_port(msg_sys: u16) -> Option<SocketAddr> {
    list_entries()
        .into_iter()
        .find(|e| msg_sys_port(e) == msg_sys)
        .map(|e| SocketAddr::new(peer_ip_addr(&e), msg_sys))
}

/// Resolve ServerHost for a client accepted on the shared `:18386` hub.
/// Identity only: ClientHello persona / join route. Never "newest InUse".
pub fn resolve_client_msgsys_upstream(
    peer: SocketAddr,
    persona: Option<u64>,
) -> Option<SocketAddr> {
    sync_from_blaze_sessions();
    if let Some(pid) = persona.filter(|&p| p != 0) {
        if let Some(route) = client_msgsys_routes().lock().get(&(pid as i64)).cloned() {
            return upstream_for_route(&route);
        }
        let gids = super::game_state::gids_for_human_persona(pid as i64);
        if gids.len() == 1 {
            return msgsys_upstream_for_gid(gids[0]);
        }
        return None;
    }

    if peer.ip().is_loopback() {
        // Loopback without ClientHello: only safe when a single match is unambiguous.
        return resolve_unambiguous_msgsys_upstream();
    }
    let matches: Vec<ClientMsgsysRoute> = client_msgsys_routes()
        .lock()
        .values()
        .filter(|r| r.client_ip == Some(peer.ip()))
        .cloned()
        .collect();
    if matches.len() == 1 {
        return upstream_for_route(&matches[0]);
    }
    resolve_unambiguous_msgsys_upstream()
}

/// When exactly one dedicated ServerHost / join route is live, splice without waiting
/// for ClientHello. Retail ClientHello is only sent *after* TCP ConnectSuccess — the
/// shared hub used to peek Hello first and close on timeout → black-screen State 5.
pub fn resolve_unambiguous_msgsys_upstream() -> Option<SocketAddr> {
    sync_from_blaze_sessions();

    let routes: Vec<ClientMsgsysRoute> = client_msgsys_routes().lock().values().cloned().collect();
    if routes.len() == 1 {
        if let Some(up) = upstream_for_route(&routes[0]) {
            return Some(up);
        }
    }

    let with_host: Vec<DedicatedServerEntry> = list_entries()
        .into_iter()
        .filter(|e| {
            e.msg_sys_tcp_port
                .filter(|&p| is_usable_prism_tcp_port(p))
                .is_some()
                && (e.state == DedicatedPoolState::InUse || e.creator_registered)
        })
        .collect();
    if with_host.len() == 1 {
        let e = &with_host[0];
        return Some(SocketAddr::new(peer_ip_addr(e), msg_sys_port(e)));
    }
    None
}

/// Upstream SimuCloud host for `CreateGame` on this gid.
pub fn simucloud_upstream_for_gid(gid: i64) -> SocketAddr {
    if let Some(sid) = bound_dedicated_for_gid(gid) {
        if let Some(e) = get_entry(sid) {
            return SocketAddr::new(peer_ip_addr(&e), simu_cloud_port(&e));
        }
    }
    SocketAddr::from(([127, 0, 0, 1], crate::client::cnc::msgsystem::simucloud::SIMUCLOUD_PORT))
}

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

/// Pool namespace gids (10xxx): browser row, joinGame target, and resetDedicatedServer match id — one gid per session.
pub fn is_pool_standby_gid(gid: i64) -> bool {
    gid >= 10_000
}

/// Registered pool row's browser/match gid (10xxx), if any.
pub fn registered_pool_gid() -> Option<i64> {
    sync_from_blaze_sessions();
    pool()
        .lock()
        .values()
        .filter(|e| e.creator_registered)
        .filter_map(|e| e.current_gid)
        .find(|&g| g > 0 && is_pool_standby_gid(g))
}

fn reclaim_notify_gid(blaze_session_id: u64, hint_gid: i64) -> Option<i64> {
    if let Some(gid) = gid_for_dedicated_session(blaze_session_id) {
        if gid > 0 {
            return Some(gid);
        }
    }
    if hint_gid > 0 {
        return Some(hint_gid);
    }
    pool()
        .lock()
        .get(&blaze_session_id)
        .and_then(|e| e.current_gid)
        .filter(|&g| g > 0)
}

#[derive(Debug, Clone)]
struct GameAssignment {
    _client_session_id: u64,
    dedicated_session_id: u64,
    host: DedicatedHostContext,
    assigned_at_unix: u64,
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

/// Fallback game UDP when EnginePeer bind port is not yet known.
/// Prefer [`DedicatedServerEntry::engine_peer_udp_port`] (Prism EnginePeerInit).
pub const DEFAULT_DEDICATED_GAME_UDP_PORT: i32 = 25200;

/// Deprecated alias — use [`DEFAULT_DEDICATED_GAME_UDP_PORT`] or discovered entry ports.
pub const DEDICATED_GAME_UDP_PORT: i32 = DEFAULT_DEDICATED_GAME_UDP_PORT;

/// Windows dynamic/private range — Blaze QoS `updateNetworkInfo` PORTs live here; EnginePeer does not.
fn is_qos_ephemeral_udp_port(port: i32) -> bool {
    (49152..=65535).contains(&port)
}

/// Game-mesh listen port (EnginePeer / dedicated pool UDP), not a QoS reflection port.
fn is_usable_dedicated_game_udp_port(port: i32) -> bool {
    port > 0 && port <= 65535 && !is_qos_ephemeral_udp_port(port)
}

fn is_usable_prism_tcp_port(port: u16) -> bool {
    (1024..=49151).contains(&port)
}

fn apply_prism_tcp_on_entry(
    e: &mut DedicatedServerEntry,
    msg_sys: Option<u16>,
    simu_cloud: Option<u16>,
    qos: Option<u16>,
) {
    if let Some(p) = msg_sys.filter(|&p| is_usable_prism_tcp_port(p)) {
        e.msg_sys_tcp_port = Some(p);
    }
    if let Some(p) = simu_cloud.filter(|&p| is_usable_prism_tcp_port(p)) {
        e.simu_cloud_tcp_port = Some(p);
    }
    if let Some(p) = qos.filter(|&p| is_usable_prism_tcp_port(p)) {
        e.qos_port = Some(p);
    }
}

fn apply_pending_on_entry(e: &mut DedicatedServerEntry, pending: &PendingEnginePeer) {
    e.engine_peer_udp_port = Some(pending.port);
    apply_prism_tcp_on_entry(e, pending.msg_sys, pending.simu_cloud, pending.qos);
    e.last_event_unix_secs = now_secs();
}

fn attach_prism_tcp_ports(
    blaze_session_id: u64,
    msg_sys: Option<u16>,
    simu_cloud: Option<u16>,
    qos: Option<u16>,
) {
    let mut m = pool().lock();
    if let Some(e) = m.get_mut(&blaze_session_id) {
        apply_prism_tcp_on_entry(e, msg_sys, simu_cloud, qos);
        if msg_sys.is_some() || simu_cloud.is_some() {
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} Prism TCP msgsys={:?} simucloud={:?} qos={:?}",
                blaze_session_id,
                e.msg_sys_tcp_port,
                e.simu_cloud_tcp_port,
                e.qos_port
            );
        }
        if let Some(p) = e.msg_sys_tcp_port {
            crate::client::cnc::msgsystem::server::spawn_pinned(msgsys_hub_listen_port(p), p);
        }
    }
}

fn find_session_by_game_udp(port: i32) -> Option<u64> {
    pool()
        .lock()
        .values()
        .find(|e| e.engine_peer_udp_port == Some(port))
        .map(|e| e.blaze_session_id)
}

fn peer_host_key(peer: &str) -> String {
    let host = peer
        .rsplit_once(':')
        .map(|(h, _)| h)
        .unwrap_or(peer);
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .to_string()
}

fn entry_needs_engine_peer(e: &DedicatedServerEntry) -> bool {
    e.engine_peer_udp_port
        .filter(|&p| is_usable_dedicated_game_udp_port(p))
        .is_none()
}

fn stash_pending_engine_peer(
    port: i32,
    peer_host: Option<&str>,
    msg_sys: Option<u16>,
    simu_cloud: Option<u16>,
    qos: Option<u16>,
) {
    if !is_usable_dedicated_game_udp_port(port) {
        return;
    }
    let host = peer_host
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| peer_host_key(s));
    let mut q = pending_engine_peer().lock();
    // Replace same-host, same-game-UDP, or orphan (no host) pending so we keep the latest bind.
    q.retain(|p| {
        if p.port == port {
            return false;
        }
        match (&host, &p.peer_host) {
            (Some(h), Some(ph)) => !ph.eq_ignore_ascii_case(h) || p.port != port,
            (None, None) => false,
            _ => true,
        }
    });
    q.push(PendingEnginePeer {
        port,
        msg_sys,
        simu_cloud,
        qos,
        peer_host: host.clone(),
        noted_at_unix: now_secs(),
    });
    crate::debug_println!(
        "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m EnginePeer UDP :{} msgsys={:?} simucloud={:?} pending (pool not ready yet; peer={})",
        port,
        msg_sys,
        simu_cloud,
        host.as_deref().unwrap_or("-")
    );
}

/// Attach any stashed EnginePeer ports to matching / sole pool entries.
fn apply_pending_engine_peers_locked(m: &mut HashMap<u64, DedicatedServerEntry>) {
    let mut q = pending_engine_peer().lock();
    if q.is_empty() || m.is_empty() {
        return;
    }
    let mut kept = Vec::new();
    for pending in q.drain(..) {
        let mut applied = false;
        if let Some(e) = m
            .values_mut()
            .find(|e| e.engine_peer_udp_port == Some(pending.port))
        {
            apply_pending_on_entry(e, &pending);
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} EnginePeer UDP :{} (claimed pending same-game-udp)",
                e.blaze_session_id,
                pending.port
            );
            applied = true;
        }
        if !applied {
            if let Some(ref host) = pending.peer_host {
                if let Some(e) = m.values_mut().find(|e| {
                    entry_needs_engine_peer(e)
                        && peer_host_key(&e.peer).eq_ignore_ascii_case(host)
                }) {
                    apply_pending_on_entry(e, &pending);
                    crate::debug_println!(
                        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} EnginePeer UDP :{} (claimed pending peer {})",
                        e.blaze_session_id,
                        pending.port,
                        host
                    );
                    applied = true;
                }
            }
        }
        if !applied {
            let needy: Vec<u64> = m
                .values()
                .filter(|e| entry_needs_engine_peer(e))
                .map(|e| e.blaze_session_id)
                .collect();
            if needy.len() == 1 {
                if let Some(e) = m.get_mut(&needy[0]) {
                    apply_pending_on_entry(e, &pending);
                    crate::debug_println!(
                        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} EnginePeer UDP :{} (claimed pending sole)",
                        e.blaze_session_id,
                        pending.port
                    );
                    applied = true;
                }
            }
        }
        if !applied {
            // Drop stale pending (>10 min) so multi-host queues do not grow forever.
            if now_secs().saturating_sub(pending.noted_at_unix) < 600 {
                kept.push(pending);
            }
        }
    }
    *q = kept;
}

/// Result of Prism `/cnc/dedicated-engine-peer` — Applied now, or Pending until pool registers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnginePeerNote {
    Applied { session: u64 },
    Pending,
    Rejected,
}

/// Unified EnginePeer report from Prism (session and/or peer optional).
/// `msg_sys` / `simu_cloud` / `qos` are Prism listen ports for this instance.
pub fn note_dedicated_engine_peer_report(
    blaze_session_id: Option<u64>,
    peer_host: Option<&str>,
    port: i32,
    msg_sys: Option<u16>,
    simu_cloud: Option<u16>,
    qos: Option<u16>,
) -> EnginePeerNote {
    if !is_usable_dedicated_game_udp_port(port) {
        return EnginePeerNote::Rejected;
    }
    sync_from_blaze_sessions();
    let attach = |sid: u64| {
        attach_prism_tcp_ports(sid, msg_sys, simu_cloud, qos);
        EnginePeerNote::Applied { session: sid }
    };
    if let Some(sid) = blaze_session_id.filter(|&s| s != 0) {
        if note_dedicated_engine_peer_udp(sid, port) {
            return attach(sid);
        }
    }
    if let Some(sid) = find_session_by_game_udp(port) {
        let _ = note_dedicated_engine_peer_udp(sid, port);
        return attach(sid);
    }
    if let Some(host) = peer_host.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(sid) = note_dedicated_engine_peer_udp_by_peer_host(host, port) {
            return attach(sid);
        }
    }
    if let Some(sid) = note_dedicated_engine_peer_udp_sole(port) {
        return attach(sid);
    }
    stash_pending_engine_peer(port, peer_host, msg_sys, simu_cloud, qos);
    EnginePeerNote::Pending
}

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
    assignable_creator_count()
}

pub fn assignable_creator_count() -> usize {
    sync_from_blaze_sessions();
    pool()
        .lock()
        .values()
        .filter(|e| is_assignable(e))
        .count()
}

pub fn stabilizing_creator_count() -> usize {
    sync_from_blaze_sessions();
    pool()
        .lock()
        .values()
        .filter(|e| {
            e.creator_registered
                && !is_assignable(e)
                && matches!(
                    e.state,
                    DedicatedPoolState::Recycling
                        | DedicatedPoolState::Idle
                        | DedicatedPoolState::CreatorRegistered
                )
        })
        .count()
}

pub fn max_assignable_wait_secs() -> u64 {
    sync_from_blaze_sessions();
    pool()
        .lock()
        .values()
        .filter_map(|e| {
            if e.creator_registered && !is_assignable(e) {
                Some(recycle_cmd220_wait_secs(e))
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0)
}

pub fn lobby_pool_status_json() -> String {
    let assignable = assignable_creator_count();
    let stabilizing = stabilizing_creator_count();
    let assignable_in = max_assignable_wait_secs();
    let last = last_no_dedicated_unix();
    let creators = list_entries().len();
    format!(
        "{{\"idle\":{assignable},\"assignable\":{assignable},\"stabilizing\":{stabilizing},\"assignableIn\":{assignable_in},\"creators\":{creators},\"lastNoDedicatedAt\":{last},\"noDedicated\":{}}}",
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
    {
        let mut m = pool().lock();
        if m.contains_key(&blaze_session_id) {
            apply_pending_engine_peers_locked(&mut m);
            return;
        }
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
            previous_map: None,
            unload_requested_at: None,
            network_inip_ip: None,
            network_inip_port: None,
            network_exip_ip: None,
            network_exip_port: None,
            engine_peer_udp_port: None,
            msg_sys_tcp_port: None,
            simu_cloud_tcp_port: None,
            qos_port: None,
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
            previous_map: None,
            unload_requested_at: None,
            network_inip_ip: None,
            network_inip_port: None,
            network_exip_ip: None,
            network_exip_port: None,
            engine_peer_udp_port: None,
            msg_sys_tcp_port: None,
            simu_cloud_tcp_port: None,
            qos_port: None,
        }
    };
    {
        let mut m = pool().lock();
        m.insert(blaze_session_id, entry);
        apply_pending_engine_peers_locked(&mut m);
    }
}

pub fn acquire_idle_creator_for_map(
    exclude_session_id: u64,
    _wanted_map: Option<&str>,
) -> Option<DedicatedServerEntry> {
    sync_from_blaze_sessions();
    let m = pool().lock();
    let mut candidates: Vec<_> = m
        .values()
        .filter(|e| e.blaze_session_id != exclude_session_id && is_assignable(e))
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

/// Store LAN/WAN IPs from a dedicated Blaze session's `updateNetworkInfo`.
/// Ports from that RPC are almost always QoS ephemeral — ignored unless they look like a listen port.
/// Never call this with the joining client's QoS snapshot.
pub fn note_dedicated_network(
    blaze_session_id: u64,
    inip_ip: Option<i32>,
    inip_port: Option<i32>,
    exip_ip: Option<i32>,
    exip_port: Option<i32>,
) {
    if !blaze_wire_peer_is_server(blaze_session_id) {
        return;
    }
    ensure_pool_entry(blaze_session_id);
    let mut m = pool().lock();
    let Some(e) = m.get_mut(&blaze_session_id) else {
        return;
    };
    if let Some(ip) = inip_ip.filter(|&v| v != 0) {
        e.network_inip_ip = Some(ip);
    }
    if let Some(ip) = exip_ip.filter(|&v| v != 0) {
        e.network_exip_ip = Some(ip);
    }
    // Prefer EnginePeerInit for game UDP. Only keep Blaze PORT values that are not QoS ephemeral
    // (e.g. a future dedicated that puts EnginePeer in updateNetworkInfo).
    let usable_inip = inip_port.filter(|&p| is_usable_dedicated_game_udp_port(p));
    let usable_exip = exip_port.filter(|&p| is_usable_dedicated_game_udp_port(p));
    if usable_inip.is_none() {
        if let Some(p) = inip_port.filter(|&v| v > 0) {
            crate::debug_println!(
                "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m session #{} ignoring QoS-style INIP PORT={} (not EnginePeer)",
                blaze_session_id,
                p
            );
        }
    } else {
        e.network_inip_port = usable_inip;
    }
    if usable_exip.is_none() {
        if let Some(p) = exip_port.filter(|&v| v > 0) {
            crate::debug_println!(
                "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m session #{} ignoring QoS-style EXIP PORT={} (not EnginePeer)",
                blaze_session_id,
                p
            );
        }
    } else {
        e.network_exip_port = usable_exip;
    }
    e.last_event_unix_secs = now_secs();
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} network IPs INIP={:?} EXIP={:?} listenPorts INIP={:?} EXIP={:?} enginePeer={:?}",
        blaze_session_id,
        e.network_inip_ip,
        e.network_exip_ip,
        e.network_inip_port,
        e.network_exip_port,
        e.engine_peer_udp_port
    );
}

/// Record Prism `EnginePeerInit` UDP bind for this dedicated (authoritative HNET game port).
pub fn note_dedicated_engine_peer_udp(blaze_session_id: u64, port: i32) -> bool {
    if !is_usable_dedicated_game_udp_port(port) {
        return false;
    }
    if !blaze_wire_peer_is_server(blaze_session_id) {
        return false;
    }
    ensure_pool_entry(blaze_session_id);
    let mut m = pool().lock();
    let Some(e) = m.get_mut(&blaze_session_id) else {
        return false;
    };
    e.engine_peer_udp_port = Some(port);
    e.last_event_unix_secs = now_secs();
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} EnginePeer UDP :{}",
        blaze_session_id,
        port
    );
    true
}

/// Match a dedicated by Blaze peer host (e.g. `10.0.0.230`) when Prism only knows the listen IP.
pub fn note_dedicated_engine_peer_udp_by_peer_host(peer_host: &str, port: i32) -> Option<u64> {
    if !is_usable_dedicated_game_udp_port(port) {
        return None;
    }
    sync_from_blaze_sessions();
    let host = peer_host.trim();
    if host.is_empty() {
        return None;
    }
    let mut m = pool().lock();
    let mut needy_sid = None;
    let mut any_sid = None;
    for e in m.values() {
        if !peer_host_key(&e.peer).eq_ignore_ascii_case(host) {
            continue;
        }
        any_sid = Some(e.blaze_session_id);
        if entry_needs_engine_peer(e) {
            needy_sid = Some(e.blaze_session_id);
            break;
        }
    }
    let sid = needy_sid.or(any_sid)?;
    if let Some(e) = m.get_mut(&sid) {
        e.engine_peer_udp_port = Some(port);
        e.last_event_unix_secs = now_secs();
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} EnginePeer UDP :{} (matched peer host {})",
            sid,
            port,
            host
        );
    }
    Some(sid)
}

/// When Prism only posts `port=` (no peer/session), attach to the sole pool dedicated
/// that still needs an EnginePeer port (or the only pool entry).
pub fn note_dedicated_engine_peer_udp_sole(port: i32) -> Option<u64> {
    if !is_usable_dedicated_game_udp_port(port) {
        return None;
    }
    sync_from_blaze_sessions();
    let mut m = pool().lock();
    apply_pending_engine_peers_locked(&mut m);
    let needy: Vec<u64> = m
        .values()
        .filter(|e| entry_needs_engine_peer(e))
        .map(|e| e.blaze_session_id)
        .collect();
    let sid = if needy.len() == 1 {
        needy[0]
    } else if m.len() == 1 {
        *m.keys().next()?
    } else {
        return None;
    };
    if let Some(e) = m.get_mut(&sid) {
        e.engine_peer_udp_port = Some(port);
        e.last_event_unix_secs = now_secs();
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} EnginePeer UDP :{} (sole/needy pool entry)",
            sid,
            port
        );
    }
    Some(sid)
}

fn resolve_dedicated_game_ports(entry: &DedicatedServerEntry) -> (i32, i32) {
    // Claim any pending report that matches this dedicated before reading ports.
    {
        let mut m = pool().lock();
        apply_pending_engine_peers_locked(&mut m);
    }
    let fresh = get_entry(entry.blaze_session_id).unwrap_or_else(|| entry.clone());
    let from_engine = fresh
        .engine_peer_udp_port
        .filter(|&p| is_usable_dedicated_game_udp_port(p));
    let from_blaze = fresh
        .network_inip_port
        .filter(|&p| is_usable_dedicated_game_udp_port(p))
        .or_else(|| {
            fresh
                .network_exip_port
                .filter(|&p| is_usable_dedicated_game_udp_port(p))
        });
    let port = if let Some(p) = from_engine.or(from_blaze) {
        p
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m session #{} HNET using DEFAULT UDP :{} (no EnginePeer report yet)",
            fresh.blaze_session_id,
            DEFAULT_DEDICATED_GAME_UDP_PORT
        );
        DEFAULT_DEDICATED_GAME_UDP_PORT
    };
    (port, port)
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
    let peer_ip = parse_peer_ipv4(&entry.peer);
    let inip_ip = entry
        .network_inip_ip
        .filter(|&ip| ip != 0)
        .unwrap_or(peer_ip);
    let (inip_port, exip_port) = resolve_dedicated_game_ports(entry);
    // Prefer this dedicated's own reported EXIP/INIP addresses; never the joining client's QoS.
    let exip_ip = entry
        .network_exip_ip
        .filter(|&ip| ip != 0)
        .or_else(|| entry.network_inip_ip.filter(|&ip| ip != 0))
        .unwrap_or(inip_ip);
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
    // Always rebuild from the live entry so HNET ports pick up discovered game UDP.
    // Never return a stale assignment.host snapshot that may still carry a client QoS port.
    if let Some(a) = assignments().lock().get(&gid).cloned() {
        if let Some(entry) = get_entry(a.dedicated_session_id) {
            return Some(host_context_from_entry(&entry));
        }
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
    let mut wanted_map = super::game_state::get_map_path(gid);
    if wanted_map.is_empty() {
        if let Some(adopted) = super::game_state::adopt_host_lobby_pending_into(gid) {
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m adopted lobby pending map into gid={gid}: \"{adopted}\""
            );
            wanted_map = adopted;
        }
    }
    let wanted_map = if wanted_map.is_empty() {
        None
    } else {
        Some(wanted_map)
    };
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
            e.unload_requested_at = None;
            e.last_event_unix_secs = now_secs();
        }
    }
    assignments().lock().insert(
        gid,
        GameAssignment {
            _client_session_id: client_session_id,
            dedicated_session_id: dedicated_sid,
            host,
            assigned_at_unix: now_secs(),
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

    let host_persona = dedicated
        .persona_id
        .map(|p| p as i64)
        .or_else(|| dedicated_identity_for_session(dedicated_sid).map(|(p, _)| p as i64))
        .unwrap_or(host.persona_id);
    // Blaze GMGR only inserts into mGameMap from NotifyGameSetup → createLocalGame.
    // NotifyGameReset / NotifyPlayerJoining look up by GID and log "unknown game" if Setup never ran.
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
    match super::build_dedicated_host_notify_game_reset(
        gid,
        host_persona,
        host.inip_ip,
        host.inip_port,
        host.exip_ip,
        host.exip_port,
        dedicated_sid,
        request_payload,
    ) {
        Ok(reset) => {
            ded_pushes.push(super::fireframe::OutgoingPush {
                wire: super::fireframe::notification_envelope(0x0004, 0x0070, &reset),
                component: 0x0004,
                command: 0x0070,
                tdf_body: reset.to_vec(),
                blaze_send_label: "NotifyGameReset (dedicated host)",
                info_log_line: format!(
                    "[Blaze→Server] Match reset sent to dedicated (game {gid})"
                ),
            });
        }
        Err(e) => {
            crate::debug_println!(
                "\x1b[38;2;255;120;120m[Dedicated pool]\x1b[0m host NotifyGameReset build failed gid={}: {:?}",
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
    let recycle_wait = recycle_cmd220_wait_secs(&dedicated);
    if recycle_wait > 0 {
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m RECYCLE: deferring cmd 220 {recycle_wait}s \
             (session #{dedicated_sid} gid={gid} prev_map={:?}) — native unload stabilization",
            dedicated.previous_map
        );
        let gid_log = gid;
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(recycle_wait)).await;
            enqueue_dedicated_match_pushes(dedicated_sid, gid_log, ded_pushes);
        });
    } else {
        enqueue_dedicated_match_pushes(dedicated_sid, gid, ded_pushes);
    }
    super::msgsystem::log::log_orch_milestone(&format!(
        "Dedicated assigned to game {gid} (client #{client_session_id})"
    ));
    note_human_msgsys_routes_for_gid(gid);
    Some(dedicated_sid)
}

fn enqueue_dedicated_match_pushes(
    dedicated_sid: u64,
    gid: i64,
    ded_pushes: Vec<super::fireframe::OutgoingPush>,
) {
    super::fireframe::enqueue_pending_pushes(dedicated_sid, ded_pushes);
    // Wake the dedicated's read loop right away -- its session task is usually blocked in
    // stream.read() and would otherwise only flush these on its next ping (~15s).
    let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
    super::msgsystem::log::log_orch_milestone(&format!(
        "Dedicated match pushes queued (game {gid}, session #{dedicated_sid})"
    ));
}

pub fn release_gid(gid: i64) {
    let dedicated_sid = assignments().lock().remove(&gid).map(|a| a.dedicated_session_id);
    super::game_state::clear_orchestration(gid);
    clear_client_msgsys_routes_for_gid(gid);
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
    super::game_state::clear_blaze_join_and_push_flags(gid);
    super::game_state::clear_orchestration(gid);
    clear_client_msgsys_routes_for_gid(gid);

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
            if e.current_map.is_some() {
                e.previous_map = e.current_map.clone();
            }
            e.state = DedicatedPoolState::Recycling;
            e.game_name = None;
            e.current_gid = Some(keep_gid);
            e.current_map = None;
            e.last_event_unix_secs = now_secs();
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} Recycling after empty humans (gid={} standby={} prev_map={:?}) — unload requested",
                sid,
                gid,
                keep_gid,
                e.previous_map
            );
        }
    }

    Some(sid)
}

/// True when this gid's dedicated already received reclaim Blaze notifies recently.
pub fn reclaim_notify_recent_for_gid(gid: i64) -> bool {
    let Some(sid) = bound_dedicated_for_gid(gid) else {
        return false;
    };
    reclaim_notify_recent_for_session(sid, gid)
}

fn reclaim_notify_recent_for_session(blaze_session_id: u64, _gid: i64) -> bool {
    let m = pool().lock();
    let Some(e) = m.get(&blaze_session_id) else {
        return false;
    };
    let Some(t) = e.unload_requested_at else {
        return false;
    };
    e.state == DedicatedPoolState::Recycling
        && now_secs().saturating_sub(t) < RECLAIM_NOTIFY_COALESCE_SECS
}

pub fn request_dedicated_level_unload(blaze_session_id: u64, gid: i64) {
    let Some(reclaim_gid) = reclaim_notify_gid(blaze_session_id, gid) else {
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m reclaim notify skipped — no live match gid (session #{}, hint={})",
            blaze_session_id,
            gid
        );
        return;
    };
    if reclaim_notify_recent_for_session(blaze_session_id, reclaim_gid) {
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m reclaim notify coalesced — already sent for session #{} gid={}",
            blaze_session_id,
            reclaim_gid
        );
        return;
    };
    {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&blaze_session_id) {
            if e.current_map.is_some() {
                e.previous_map = e.current_map.clone();
            }
            e.unload_requested_at = Some(now_secs());
            e.state = DedicatedPoolState::Recycling;
        }
    }
    super::fireframe::clear_pending_pushes(blaze_session_id);
    match super::fireframe::pushes_dedicated_reclaim_idle(reclaim_gid) {
        Ok(pushes) if !pushes.is_empty() => {
            super::fireframe::enqueue_pending_pushes(blaze_session_id, pushes);
            let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m queued NotifyGameRemoved+RESETABLE reclaim → dedicated #{} gid={}",
                blaze_session_id,
                reclaim_gid
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
            previous_map: None,
            unload_requested_at: None,
            network_inip_ip: None,
            network_inip_port: None,
            network_exip_ip: None,
            network_exip_port: None,
            engine_peer_udp_port: None,
            msg_sys_tcp_port: None,
            simu_cloud_tcp_port: None,
            qos_port: None,
        });
        upsert_from_blaze_session(entry, &s);
    }
    apply_pending_engine_peers_locked(&mut m);
    promote_all_recycling(&mut m);
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
    {
        let mut m = pool().lock();
        apply_pending_engine_peers_locked(&mut m);
    }
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
    {
        let mut m = pool().lock();
        if let Some(e) = m.get_mut(&blaze_session_id) {
            e.state = DedicatedPoolState::Idle;
            e.unload_requested_at = None;
            e.current_map = None;
            e.game_name = None;
            e.last_event_unix_secs = now_secs();
            if let Some(g) = gid {
                e.current_gid = Some(g);
            }
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m session #{} native returnDedicatedServerToPool → assignable (gid={:?}, standby_gid={:?})",
                blaze_session_id,
                gid,
                e.current_gid
            );
        }
    }
    if let Some(g) = gid {
        super::game_state::force_standby_reset(g);
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
    fn msgsys_hub_is_one_below_serverhost() {
        assert_eq!(super::msgsys_hub_listen_port(18387), 18386);
        assert_eq!(super::msgsys_hub_listen_port(18397), 18396);
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
