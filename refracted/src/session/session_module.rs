use parking_lot::Mutex;
use std::net::SocketAddr;

pub use crate::common::build_profile::BuildProfile;

#[derive(Debug, Clone)]
struct BuildDetectionState {
    profile: BuildProfile,
    source: String,
}

impl Default for BuildDetectionState {
    fn default() -> Self {
        Self {
            profile: BuildProfile::Unknown,
            source: "none".to_string(),
        }
    }
}

/// User session information extracted from LSX authentication
#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: u64,
    pub persona_id: u64,
    pub display_name: String,
    pub email: String,
    pub psid: u32,
    pub jwt_token: Option<String>,
    pub update_network_info_count: u32,
    pub hwfg: u32, // Hardware flags state
    /// Echo from last `updateNetworkInfo` TDF (used in `UserSessionExtendedDataUpdate`).
    pub network_exip_ip: Option<u32>,
    pub network_inip_ip: Option<u32>,
    pub network_exip_port: Option<i32>,
    pub network_inip_port: Option<i32>,
    pub network_bps: Option<String>,
    pub next_message_id: u32, // Next message ID for Messaging.sendMessage
}

impl Default for UserSession {
    fn default() -> Self {
        Self {
            user_id: 1012711274866,
            persona_id: 1016820078927,
            display_name: "Xevrac".to_string(),
            email: "xevrac@ea.com".to_string(),
            psid: 0,
            jwt_token: None,
            update_network_info_count: 0,
            hwfg: 0,
            network_exip_ip: None,
            network_inip_ip: None,
            network_exip_port: None,
            network_inip_port: None,
            network_bps: None,
            next_message_id: 1160000, // Start from a reasonable message ID
        }
    }
}

/// Global session state shared between LSX and HTTP handlers
static GLOBAL_SESSION: Mutex<Option<UserSession>> = Mutex::new(None);
/// IPv4 (Blaze / network byte order) last seen as the QoS TCP peer — fills `EXIP` when the client
/// still sends 0.0.0.0 in `updateNetworkInfo`.
static LAST_QOS_OBSERVED_EXIP_IP: Mutex<Option<u32>> = Mutex::new(None);
static BUILD_DETECTION: Mutex<BuildDetectionState> = Mutex::new(BuildDetectionState {
    profile: BuildProfile::Unknown,
    source: String::new(),
});

/// Last `Util.fetchClientConfig` snapshot for Sessions UI (CFID, gRPC tenancy / URL from CONF map).
#[derive(Debug, Clone, Default)]
pub struct LastFetchClientConfig {
    pub cfid: String,
    pub client_grpc_tenancy: String,
    pub client_grpc_url: String,
}

static LAST_FETCH_CLIENT_CONFIG: Mutex<LastFetchClientConfig> = Mutex::new(LastFetchClientConfig {
    cfid: String::new(),
    client_grpc_tenancy: String::new(),
    client_grpc_url: String::new(),
});

/// Blaze version string echoed in preAuth / similar (matches `util_handlers` SVER).
pub const BLAZE_SERVER_VERSION_LABEL: &str = "Blaze 18.3.0 (CL# 2087509)";

pub fn record_last_fetch_client_config(cfid: &str, grpc_tenancy: &str, grpc_url: &str) {
    let mut g = LAST_FETCH_CLIENT_CONFIG.lock();
    g.cfid = cfid.to_string();
    g.client_grpc_tenancy = grpc_tenancy.to_string();
    g.client_grpc_url = grpc_url.to_string();
}

pub fn last_fetch_client_config() -> LastFetchClientConfig {
    LAST_FETCH_CLIENT_CONFIG.lock().clone()
}

/// Set the current user session from LSX authentication
pub fn set_user_session(session: UserSession) {
    let mut global = GLOBAL_SESSION.lock();
    *global = Some(session);
    drop(global);
    crate::session::blaze_sessions::sync_all_from_global_session();
}

/// Real LSX/HTTP session only — never the [`UserSession::default`] placeholder.
pub fn clone_user_session_if_set() -> Option<UserSession> {
    GLOBAL_SESSION.lock().clone()
}

/// Get the current user session, or default if not set
pub fn get_user_session() -> UserSession {
    let global = GLOBAL_SESSION.lock();
    global.clone().unwrap_or_default()
}

/// Increment updateNetworkInfo call count and return the count
pub fn increment_update_network_info_count() -> u32 {
    let mut global = GLOBAL_SESSION.lock();
    if let Some(ref mut session) = *global {
        session.update_network_info_count += 1;
        session.update_network_info_count
    } else {
        1
    }
}

/// Update hardware flags state
pub fn set_hwfg(value: u32) {
    let mut global = GLOBAL_SESSION.lock();
    if let Some(ref mut session) = *global {
        session.hwfg = value;
    }
}

/// Parsed from `updateNetworkInfo` TDF; merged into [`UserSession`] for extended-data echo.
#[derive(Default)]
pub struct NetworkSnapshot {
    pub exip_ip: Option<u32>,
    pub inip_ip: Option<u32>,
    pub exip_port: Option<i32>,
    pub inip_port: Option<i32>,
    pub bps: Option<String>,
}

pub fn merge_network_snapshot(n: NetworkSnapshot) {
    let mut global = GLOBAL_SESSION.lock();
    let Some(ref mut session) = *global else {
        return;
    };
    if n.exip_ip.is_some() {
        session.network_exip_ip = n.exip_ip;
    }
    if n.inip_ip.is_some() {
        session.network_inip_ip = n.inip_ip;
    }
    if n.exip_port.is_some() {
        session.network_exip_port = n.exip_port;
    }
    if n.inip_port.is_some() {
        session.network_inip_port = n.inip_port;
    }
    if n.bps.is_some() {
        session.network_bps = n.bps;
    }
}

/// Record the client's IPv4 as seen by our QoS listener (NAT reflection / WAN path). Skips
/// loopback and unspecified; merges into [`UserSession`] when logged in and retains a process-wide
/// hint for [`peek_qos_observed_exip_ip`] (covers QoS before Blaze session is ready).
pub fn record_qos_observed_client_endpoint(peer: SocketAddr) {
    let std::net::IpAddr::V4(v4) = peer.ip() else {
        return;
    };
    if v4.is_loopback() || v4.is_unspecified() {
        return;
    }
    let bits = v4.to_bits();
    *LAST_QOS_OBSERVED_EXIP_IP.lock() = Some(bits);
    merge_network_snapshot(NetworkSnapshot {
        exip_ip: Some(bits),
        ..Default::default()
    });
}

pub fn peek_qos_observed_exip_ip() -> Option<u32> {
    *LAST_QOS_OBSERVED_EXIP_IP.lock()
}

/// Get and increment next message ID
pub fn get_next_message_id() -> u32 {
    let mut global = GLOBAL_SESSION.lock();
    if let Some(ref mut session) = *global {
        let id = session.next_message_id;
        session.next_message_id += 1;
        id
    } else {
        1160000
    }
}

pub fn get_build_profile() -> BuildProfile {
    BUILD_DETECTION.lock().profile
}

pub fn get_build_profile_name() -> &'static str {
    match get_build_profile() {
        BuildProfile::Unknown => "Unknown",
        BuildProfile::Labs => "BF Labs",
        BuildProfile::LabsAlpha => "BF Labs Alpha",
        BuildProfile::OpenBeta => "BF6 Open Beta",
    }
}

pub fn get_build_profile_source() -> String {
    BUILD_DETECTION.lock().source.clone()
}

pub fn set_build_profile(profile: BuildProfile, source: &str) {
    let print_conf: Option<(BuildProfile, String)> = {
        let mut state = BUILD_DETECTION.lock();
        if state.profile != profile {
            state.profile = profile;
            state.source = source.to_string();
            Some((profile, source.to_string()))
        } else {
            if !source.is_empty() {
                state.source = source.to_string();
            }
            None
        }
    };
    if let Some((profile, _src)) = print_conf {
        let profile_name = match profile {
            BuildProfile::Unknown => "Unknown",
            BuildProfile::Labs => "BF Labs",
            BuildProfile::LabsAlpha => "BF Labs Alpha",
            BuildProfile::OpenBeta => "BF6 Open Beta",
        };
        crate::console_println!(
            "\x1b[38;2;205;127;50m[CONF]\x1b[0m Profile discovered: \x1b[38;2;205;127;50m{}\x1b[0m",
            profile_name
        );
    }
}

/// Delegates to the active [`crate::client`] implementation (e.g. Battlefield Labs heuristics).
pub fn hint_build_profile_from_text(text: &str) -> BuildProfile {
    crate::client::hint_build_profile_from_text(text)
}

pub fn detect_and_set_build_profile_from_text(text: &str, source: &str) {
    let p = hint_build_profile_from_text(text);
    if p != BuildProfile::Unknown {
        set_build_profile(p, source);
    }
}

