//! Command & Conquer 
//! Wire dispatch from [`crate::client`] and Blaze/HTTP handlers once this title is implemented.

pub mod dedicated_pool;
pub mod fireframe;
pub mod game_state;
pub mod msgsystem;

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use bytes::Bytes;
use crate::blaze::tdf::TdfEncoder;
use crate::common::error::BlazeResult;
use crate::http::handlers::handlers_module::HttpResponse;
use crate::session::session_module::{get_user_session, set_user_session};

// Blaze AuthenticationComponent `PLAT` (ClientPlatformType): 1=XBL2, 2=PS3, 3=WII, 4=PC
#[allow(dead_code)]
const PLAT_INVALID: i32 = 0;
#[allow(dead_code)]
const PLAT_XBL2: i32 = 1;
#[allow(dead_code)]
const PLAT_PS3: i32 = 2;
#[allow(dead_code)]
const PLAT_WII: i32 = 3;
const PLAT_PC: i32 = 4;

// Blaze AuthenticationComponent `STAS` (PersonaStatus) constants
#[allow(dead_code)]
const STAS_UNKNOWN: i32 = 0;
#[allow(dead_code)]
const STAS_INACTIVE: i32 = 1;
const STAS_ACTIVE: i32 = 2;

// Blaze GameManager `JGS` (JoinGameState) constants
const JGS_JOINED_GAME: i32 = 0;
#[allow(dead_code)]
const JGS_IN_QUEUE: i32 = 1;
#[allow(dead_code)]
const JGS_GROUP_PART_JOIN: i32 = 2;

// Blaze GameManager `NTOP` (NetworkTopology) constants -- values verified
// against this CNC build's TDF dump (1 = CLIENT_SERVER_DEDICATED).
#[allow(dead_code)]
const NTOP_NETWORK_DISABLED: i32 = 0;
#[allow(dead_code)]
const NTOP_CLIENT_SERVER_DEDICATED: i32 = 1;
#[allow(dead_code)]
const NTOP_CLIENT_SERVER_PEER_HOSTED: i32 = 2;
#[allow(dead_code)]
const NTOP_PEER_TO_PEER_FULL_MESH: i32 = 3;
#[allow(dead_code)]
const NTOP_PEER_TO_PEER_PARTIAL_MESH: i32 = 4;

const NTOP_DEFAULT: i32 = NTOP_CLIENT_SERVER_DEDICATED;
const CNC_TEST_DEDICATED_PORT: i32 = 25200;

fn cnc_blaze_conf_map() -> indexmap::IndexMap<String, String> {
    let mut conf_map = indexmap::IndexMap::new();
    conf_map.insert("associationListSkipInitialSet".to_string(), "1".to_string());
    conf_map.insert("autoReconnectEnabled".to_string(), "0".to_string());
    conf_map.insert("cachedUserRefreshInterval".to_string(), "1s".to_string());
    conf_map.insert("clientUserMetricsUpdateRate".to_string(), "60000".to_string());
    conf_map.insert("connIdleTimeout".to_string(), "90s".to_string());
    conf_map.insert("defaultRequestTimeout".to_string(), "20s".to_string());
    conf_map.insert("enableLoginQueueEstimate".to_string(), "false".to_string());
    conf_map.insert("loginRateSeconds".to_string(), "200".to_string());
    conf_map.insert("maxReconnectAttempts".to_string(), "30".to_string());
    conf_map.insert("nonResumableTimeoutScale".to_string(), "2.0".to_string());
    conf_map.insert("nucleusConnect".to_string(), "https://accounts.ea.com".to_string());
    conf_map.insert(
        "nucleusConnectTrusted".to_string(),
        "https://accounts2s.ea.com".to_string(),
    );
    conf_map.insert("nucleusPortal".to_string(), "https://signin.ea.com".to_string());
    conf_map.insert("nucleusProxy".to_string(), "https://gateway.ea.com".to_string());
    conf_map.insert("pingPeriod".to_string(), "30s".to_string());
    conf_map.insert("userManagerMaxCachedUsers".to_string(), "0".to_string());
    conf_map
}

/// Blaze `preAuth` **QOSS** for CNC 3.19.4 (`BWPS` / `LNP` / `LTPS` / `SVID`).
fn cnc_qos_ping_site_struct(qos_host: &str, qos_port: i32, site_name: &str) -> Vec<u8> {
    let mut s = Vec::new();
    s.extend_from_slice(&TdfEncoder::encode_string("PSA ", qos_host));
    s.extend_from_slice(&TdfEncoder::encode_int("PSP ", qos_port));
    s.extend_from_slice(&TdfEncoder::encode_string("SNA ", site_name));
    s
}

fn cnc_encode_preauth_qoss_field() -> Vec<u8> {
    let qos_ports = crate::common::game::current_service_ports();
    let qos_host = "127.0.0.1";
    let qos_port = qos_ports.qos_data as i32;
    let coordinator = "qoscoordinator.gameservices.ea.com";

    let mut qoss_struct = Vec::new();

    let bwps = cnc_qos_ping_site_struct(qos_host, qos_port, coordinator);
    qoss_struct.extend_from_slice(&TdfEncoder::encode_struct("BWPS", &bwps));

    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("LNP ", 10));

    let mut ltps_map = indexmap::IndexMap::new();
    let regions = [
        ("aws-bah", qos_host),
        ("aws-brz", qos_host),
        ("aws-cmh", qos_host),
        ("aws-cpt", qos_host),
        ("aws-dub", qos_host),
        ("aws-fra", qos_host),
        ("aws-hkg", qos_host),
        ("aws-iad", qos_host),
        ("aws-icn", qos_host),
        ("aws-lhr", qos_host),
        ("aws-nrt", qos_host),
        ("aws-pdx", qos_host),
        ("aws-sin", qos_host),
        ("aws-sjc", qos_host),
        ("aws-syd", qos_host),
    ];
    for (alias, host) in regions {
        // SNA should match the LTPS alias; empty SNA leaves QosManager unable to
        // select a bandwidth site after latency probes complete.
        let region = cnc_qos_ping_site_struct(host, qos_port, alias);
        ltps_map.insert(alias.to_string(), region);
    }
    qoss_struct.extend_from_slice(&TdfEncoder::encode_string_struct_map_ordered("LTPS", &ltps_map));

    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("SVID", 1_161_889_797));

    TdfEncoder::encode_struct("QOSS", &qoss_struct).to_vec()
}

fn cnc_data_runtime_dir() -> PathBuf {
    crate::common::paths::app_data_dir()
        .join("client")
        .join("cnc")
}

fn sanitize_relative_request_path(raw: &str) -> Option<PathBuf> {
    let clean = raw.split('?').next().unwrap_or(raw).trim_start_matches('/');
    if clean.is_empty() {
        return None;
    }

    let mut rel = PathBuf::new();
    for comp in Path::new(clean).components() {
        match comp {
            Component::Normal(seg) => rel.push(seg),
            Component::CurDir => {}
            _ => return None,
        }
    }

    if rel.as_os_str().is_empty() {
        None
    } else {
        Some(rel)
    }
}

fn content_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html",
        "js" => "text/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        // is often rejected for @font-face.
        "woff" => "application/font-woff",
        "woff2" => "font/woff2",
        "ttf" => "application/x-font-ttf",
        "otf" => "application/x-font-opentype",
        "cfg" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// CNC probe HTTP routes (`/cnc/probe-dump`, `/cnc/online-count`, `/cnc/build-info`).
pub fn try_handle_cnc_post(method: &str, path: &str, body: &[u8]) -> Option<HttpResponse> {
    let is_post = method.eq_ignore_ascii_case("POST");
    let is_get = method.eq_ignore_ascii_case("GET");
    if !is_post && !is_get {
        return None;
    }
    let (base, query) = path
        .split_once('?')
        .map(|(b, q)| (b, Some(q)))
        .unwrap_or((path, None));
    let base = base.trim_start_matches('/');
    if base == "cnc/online-count" && is_get {
        let _ = body;
        return Some(handle_cnc_online_count());
    }
    // GET /cnc/build-info -- RFR (Cargo) + Prism (running instance sidecar/log) + cnc_rl.
    if base == "cnc/build-info" && is_get {
        let _ = body;
        return Some(handle_cnc_build_info());
    }
    // POST /cnc/api/start-battle -- advance game state for a given GID (testing API).
    if base == "cnc/api/start-battle" && is_post {
        return Some(handle_cnc_start_battle(body));
    }
    if base == "cnc/select-map" {
        return Some(handle_cnc_select_map(query, body));
    }
    if base == "cnc/player-attrs" && is_post {
        return Some(handle_cnc_player_attrs(query, body));
    }
    if base == "cnc/shell-theme" {
        return Some(handle_cnc_shell_theme(is_post, body));
    }
    // GET /cnc/player-probe?gid= -- validate map + player lobby/CreateGame fields.
    if base == "cnc/player-probe" && is_get {
        return Some(handle_cnc_player_probe(query));
    }
    if base == "cnc/game-list" && is_get {
        let _ = body;
        return Some(handle_cnc_game_list());
    }
    if base == "cnc/game-password" && is_post {
        return Some(handle_cnc_game_password(query, body));
    }
    if base == "cnc/verify-game-password" && is_post {
        return Some(handle_cnc_verify_game_password(query, body));
    }
    if base == "cnc/server-ping" && is_get {
        return Some(handle_cnc_server_ping(query));
    }
    if base == "cnc/leave-game" && is_post {
        return Some(handle_cnc_leave_game(query));
    }
    if base == "cnc/player-ready" && is_post {
        return Some(handle_cnc_player_ready(query));
    }
    if base == "cnc/lobby-roster" && is_get {
        return Some(handle_cnc_lobby_roster(query));
    }
    if base == "cnc/dedicated-pool" && is_get {
        let _ = body;
        return Some(HttpResponse::new(
            200,
            "application/json",
            dedicated_pool::lobby_pool_status_json().into_bytes(),
        ));
    }
    if !is_post {
        return None;
    }
    if base != "cnc/probe-dump" {
        return None;
    }
    let filename = query
        .and_then(|q| {
            q.split('&').find_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                if k == "filename" {
                    Some(sanitize_probe_dump_filename(&percent_decode_plus(v)))
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| "cnc-probe-log.txt".to_string());
    let mut headers = HashMap::new();
    headers.insert(
        "Content-Disposition".to_string(),
        format!("attachment; filename=\"{filename}\""),
    );
    Some(HttpResponse::new_with_headers(
        200,
        "text/plain; charset=utf-8",
        body.to_vec(),
        headers,
    ))
}

fn percent_decode_plus(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let h = std::str::from_utf8(&b[i + 1..i + 3]).ok();
            if let Some(two) = h {
                if let Ok(byte) = u8::from_str_radix(two, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        if b[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(b[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Lobby HUD: authenticated Blaze presence (`GET /cnc/online-count`).
/// `players` / `servers` split by CLNT (server substring = dedicated); `count` stays player total for older callers.
fn handle_cnc_online_count() -> HttpResponse {
    use crate::session::blaze_sessions;
    let players = blaze_sessions::authenticated_player_count();
    let servers = blaze_sessions::authenticated_server_count();
    let body = serde_json::json!({
        "ok": true,
        "count": players,
        "players": players,
        "servers": servers,
        "active": blaze_sessions::active_count(),
    });
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn handle_cnc_build_info() -> HttpResponse {
    let body = shell_build_info_json();
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

const CNC_RL_BUILD: &str = "150805";

fn shell_build_info_json() -> serde_json::Value {
    let rfr = env!("CARGO_PKG_VERSION").to_string();
    let prism = resolve_prism_version().unwrap_or_else(|| "?".to_string());
    serde_json::json!({
        "ok": true,
        "rfr": rfr,
        "prism": prism,
        "cnc": CNC_RL_BUILD,
        "cnc_rl": CNC_RL_BUILD,
    })
}

fn resolve_prism_version() -> Option<String> {
    if let Ok(v) = std::env::var("CNC_PRISM_VERSION") {
        let t = v.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    for dir in prism_version_search_dirs() {
        if let Some(v) = read_prism_version_file(&dir.join("prism.version")) {
            return Some(v);
        }
        if let Some(v) = parse_prism_version_from_log(&dir.join("prism.log")) {
            return Some(v);
        }
        if let Some(v) = parse_prism_version_from_log(&dir.join("prism.log.prev")) {
            return Some(v);
        }
    }
    None
}

fn prism_version_search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let push = |dirs: &mut Vec<PathBuf>, p: PathBuf| {
        if p.is_dir() && !dirs.iter().any(|d| d == &p) {
            dirs.push(p);
        }
    };
    if let Ok(d) = std::env::var("CNC_GAME_DIR") {
        push(&mut dirs, PathBuf::from(d));
    }
    if let Ok(cwd) = std::env::current_dir() {
        push(&mut dirs, cwd);
    }
    if let Some(exe) = crate::common::paths::executable_dir() {
        push(&mut dirs, exe);
    }
    for p in cnc_process_dirs() {
        push(&mut dirs, p);
    }
    dirs
}

fn read_prism_version_file(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let line = raw.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    Some(line.to_string())
}

fn parse_prism_version_from_log(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    for line in raw.lines() {
        let t = line.trim();
        // Strip common ANSI / log prefixes then match splash "Version: x.y.z".
        let bare = strip_ansi_approx(t);
        let bare = bare.trim();
        if let Some(rest) = bare.strip_prefix("Version:") {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn strip_ansi_approx(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

#[cfg(windows)]
fn cnc_process_dirs() -> Vec<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use winapi::shared::minwindef::{DWORD, FALSE, MAX_PATH};
    use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::psapi::GetModuleFileNameExW;
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
    use winapi::um::winnt::PROCESS_VM_READ;

    let mut out = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return out;
        }
        let mut pe: PROCESSENTRY32W = std::mem::zeroed();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut pe) != FALSE {
            loop {
                let name = {
                    let len = pe
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(pe.szExeFile.len());
                    String::from_utf16_lossy(&pe.szExeFile[..len]).to_ascii_lowercase()
                };
                if name == "cnc.exe" || name == "cnc.server.exe" {
                    let access = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
                    let proc = OpenProcess(access, FALSE, pe.th32ProcessID);
                    if !proc.is_null() {
                        let mut buf = [0u16; MAX_PATH];
                        let n = GetModuleFileNameExW(proc, std::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as DWORD);
                        CloseHandle(proc);
                        if n > 0 {
                            let path = std::ffi::OsString::from_wide(&buf[..n as usize]);
                            if let Some(parent) = Path::new(&path).parent() {
                                out.push(parent.to_path_buf());
                            }
                        }
                    }
                }
                if Process32NextW(snap, &mut pe) == FALSE {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    out
}

#[cfg(not(windows))]
fn cnc_process_dirs() -> Vec<PathBuf> {
    Vec::new()
}

fn shell_theme_prefs_path() -> PathBuf {
    cnc_data_runtime_dir()
        .join("cncg2")
        .join("shell")
        .join("prefs")
        .join("ui-theme.json")
}

fn normalize_shell_theme_id(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "aurora" => "aurora",
        "classic" | "cnc-alpha" | "alpha" => "classic",
        _ => "aurora",
    }
}

/// Body/file JSON: `{ "theme": "classic"|"aurora", "defaultTheme": "classic"|"aurora" }`.
fn handle_cnc_shell_theme(is_post: bool, body: &[u8]) -> HttpResponse {
    let path = shell_theme_prefs_path();
    if is_post {
        let mut theme = "aurora".to_string();
        let mut default_theme = "aurora".to_string();
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
            if let Some(t) = v.get("theme").and_then(|t| t.as_str()) {
                theme = normalize_shell_theme_id(t).to_string();
            }
            if let Some(t) = v.get("defaultTheme").and_then(|t| t.as_str()) {
                default_theme = normalize_shell_theme_id(t).to_string();
            } else {
                default_theme = theme.clone();
            }
        } else if let Ok(s) = std::str::from_utf8(body) {
            let s = s.trim();
            if !s.is_empty() {
                theme = normalize_shell_theme_id(s).to_string();
                default_theme = theme.clone();
            }
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let payload = serde_json::json!({
            "theme": theme,
            "defaultTheme": default_theme
        });
        match std::fs::write(&path, payload.to_string()) {
            Ok(()) => HttpResponse::new(200, "application/json", payload.to_string().into_bytes()),
            Err(e) => HttpResponse::new(
                500,
                "application/json",
                serde_json::json!({ "ok": false, "error": e.to_string() })
                    .to_string()
                    .into_bytes(),
            ),
        }
    } else {
        let (theme, default_theme) = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .map(|v| {
                let theme = v
                    .get("theme")
                    .and_then(|t| t.as_str())
                    .map(normalize_shell_theme_id)
                    .unwrap_or("aurora")
                    .to_string();
                let default_theme = v
                    .get("defaultTheme")
                    .and_then(|t| t.as_str())
                    .map(normalize_shell_theme_id)
                    .unwrap_or(theme.as_str())
                    .to_string();
                (theme, default_theme)
            })
            .unwrap_or_else(|| ("aurora".to_string(), "aurora".to_string()));
        let payload = serde_json::json!({
            "theme": theme,
            "defaultTheme": default_theme
        });
        HttpResponse::new(200, "application/json", payload.to_string().into_bytes())
    }
}

fn handle_cnc_select_map(query: Option<&str>, body: &[u8]) -> HttpResponse {
    use crate::client::cnc::game_state;
    let mut gid: i64 = 1;
    let mut path = String::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                match k {
                    "gid" => gid = percent_decode_plus(v).parse().unwrap_or(1),
                    "path" | "level" => path = percent_decode_plus(v),
                    _ => {}
                }
            }
        }
    }
    if path.is_empty() {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
            if let Some(g) = v.get("gid").and_then(|g| g.as_i64()) {
                gid = g;
            }
            if let Some(p) = v
                .get("path")
                .or_else(|| v.get("level"))
                .and_then(|p| p.as_str())
            {
                path = p.to_string();
            }
        }
    }
    if path.is_empty() {
        return HttpResponse::new(
            400,
            "application/json",
            br#"{"ok":false,"error":"missing path"}"#.to_vec(),
        );
    }
    game_state::set_map_path(gid, &path);
    tracing::info!(
        target: "cnc",
        "[CNC] select-map gid={} path=\"{}\"",
        gid,
        path
    );
    let body = serde_json::json!({ "ok": true, "gid": gid, "path": path });
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

/// Query or JSON: `gid`, `pid` (0 = host), `faction`, `team`, `startpoint`, `general`, `isai`.
fn handle_cnc_player_attrs(query: Option<&str>, body: &[u8]) -> HttpResponse {
    use crate::client::cnc::game_state;
    use indexmap::IndexMap;

    let mut gid: i64 = 1;
    let mut pid: i64 = 0;
    let mut attrs = IndexMap::new();

    let push_attr = |attrs: &mut IndexMap<String, String>, k: &str, v: String| {
        match k {
            "faction" | "_faction" => {
                attrs.insert("_faction".into(), v);
            }
            "team" | "_team" => {
                attrs.insert("_team".into(), v);
            }
            "startpoint" | "start" | "_startpoint" => {
                attrs.insert("_startpoint".into(), v);
            }
            "general" | "_general" => {
                attrs.insert("_general".into(), v);
            }
            "isai" | "_isai" => {
                attrs.insert("_isai".into(), v);
            }
            "difficulty" | "_difficulty" => {
                attrs.insert("_difficulty".into(), v);
            }
            _ => {}
        }
    };

    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                let decoded = percent_decode_plus(v);
                match k {
                    "gid" => gid = decoded.parse().unwrap_or(1),
                    "pid" | "persona" | "player" => pid = decoded.parse().unwrap_or(0),
                    _ => push_attr(&mut attrs, k, decoded),
                }
            }
        }
    }
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(g) = v.get("gid").and_then(|g| g.as_i64()) {
            gid = g;
        }
        if let Some(p) = v
            .get("pid")
            .or_else(|| v.get("persona"))
            .and_then(|p| p.as_i64().or_else(|| p.as_u64().map(|u| u as i64)))
        {
            pid = p;
        }
        for key in [
            "_faction",
            "faction",
            "_team",
            "team",
            "_startpoint",
            "startpoint",
            "_general",
            "general",
            "_isai",
            "isai",
            "_difficulty",
            "difficulty",
        ] {
            if let Some(s) = v.get(key).and_then(|x| {
                x.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| x.as_i64().map(|n| n.to_string()))
                    .or_else(|| x.as_u64().map(|n| n.to_string()))
            }) {
                push_attr(&mut attrs, key, s);
            }
        }
        if let Some(obj) = v.get("attrs").and_then(|a| a.as_object()) {
            for (k, val) in obj {
                if let Some(s) = val.as_str().map(|s| s.to_string()).or_else(|| {
                    val.as_i64()
                        .map(|n| n.to_string())
                        .or_else(|| val.as_u64().map(|n| n.to_string()))
                }) {
                    push_attr(&mut attrs, k, s);
                }
            }
        }
    }

    if attrs.is_empty() {
        return HttpResponse::new(
            400,
            "application/json",
            br#"{"ok":false,"error":"missing attrs"}"#.to_vec(),
        );
    }

    crate::debug_println!(
        "[CNC] /cnc/player-attrs gid={} pid={} attrs={:?}",
        gid,
        pid,
        attrs
    );
    game_state::set_pending_player_attrs(gid, pid, attrs.clone());
    let probe = game_state::player_data_probe(gid);
    let body = serde_json::json!({
        "ok": true,
        "gid": gid,
        "pid": pid,
        "attrs": attrs,
        "probe": probe,
    });
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn handle_cnc_player_probe(query: Option<&str>) -> HttpResponse {
    use crate::client::cnc::game_state;
    let mut gid: i64 = 1;
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if k == "gid" {
                    gid = percent_decode_plus(v).parse().unwrap_or(1);
                }
            }
        }
    }
    let body = game_state::player_data_probe(gid);
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn handle_cnc_game_list() -> HttpResponse {
    let body = game_state::browser_game_list_json();
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn parse_gid_pid_password(query: Option<&str>, body: &[u8]) -> (i64, i64, String) {
    let mut gid: i64 = 0;
    let mut pid: i64 = 0;
    let mut password = String::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                let decoded = percent_decode_plus(v);
                match k {
                    "gid" => gid = decoded.parse().unwrap_or(0),
                    "pid" | "persona" | "player" => pid = decoded.parse().unwrap_or(0),
                    "password" | "pass" | "pwd" => password = decoded,
                    _ => {}
                }
            }
        }
    }
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(g) = v.get("gid").and_then(|g| g.as_i64()) {
            gid = g;
        }
        if let Some(p) = v
            .get("pid")
            .or_else(|| v.get("persona"))
            .and_then(|p| p.as_i64().or_else(|| p.as_u64().map(|u| u as i64)))
        {
            pid = p;
        }
        if let Some(s) = v
            .get("password")
            .or_else(|| v.get("pass"))
            .or_else(|| v.get("pwd"))
            .and_then(|x| x.as_str())
        {
            password = s.to_string();
        }
    }
    if pid <= 0 {
        let session = crate::session::get_user_session();
        if session.persona_id != 0 {
            pid = session.persona_id as i64;
        }
    }
    (gid, pid, password)
}

fn handle_cnc_game_password(query: Option<&str>, body: &[u8]) -> HttpResponse {
    let (gid, pid, password) = parse_gid_pid_password(query, body);
    if gid <= 0 {
        return HttpResponse::new(
            400,
            "application/json",
            br#"{"ok":false,"error":"gid required"}"#.to_vec(),
        );
    }
    let resp = game_state::set_game_password(gid, pid, &password);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m game-password gid={} pid={} protected={}",
        gid,
        pid,
        resp.get("passwordProtected").and_then(|v| v.as_bool()).unwrap_or(false)
    );
    HttpResponse::new(200, "application/json", resp.to_string().into_bytes())
}

fn handle_cnc_verify_game_password(query: Option<&str>, body: &[u8]) -> HttpResponse {
    let (gid, pid, password) = parse_gid_pid_password(query, body);
    if gid <= 0 {
        return HttpResponse::new(
            400,
            "application/json",
            br#"{"ok":false,"error":"gid required"}"#.to_vec(),
        );
    }
    let resp = game_state::verify_game_password(gid, pid, &password);
    let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m verify-game-password gid={} pid={} ok={}",
        gid,
        pid,
        ok
    );
    HttpResponse::new(
        if ok { 200 } else { 403 },
        "application/json",
        resp.to_string().into_bytes(),
    )
}

fn handle_cnc_server_ping(query: Option<&str>) -> HttpResponse {
    let mut host = String::new();
    let mut port = dedicated_pool::DEDICATED_PING_TCP_PORT;
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                match k {
                    "host" => host = percent_decode_plus(v),
                    "port" => {
                        if let Ok(p) = percent_decode_plus(v).parse::<u16>() {
                            port = p;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let ms = if host.is_empty() {
        None
    } else {
        dedicated_pool::probe_host_rtt_ms(&host, Some(port))
    };
    let body = serde_json::json!({ "ok": ms.is_some(), "pingMs": ms, "host": host, "port": port });
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn handle_cnc_leave_game(query: Option<&str>) -> HttpResponse {
    let mut gid: i64 = 0;
    let mut pid: i64 = 0;
    let mut force = false;
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                match k {
                    "gid" => gid = percent_decode_plus(v).parse().unwrap_or(0),
                    "pid" => pid = percent_decode_plus(v).parse().unwrap_or(0),
                    "force" => {
                        let s = percent_decode_plus(v);
                        force = s == "1" || s.eq_ignore_ascii_case("true");
                    }
                    _ => {}
                }
            }
        }
    }
    if pid <= 0 {
        let session = crate::session::get_user_session();
        if session.persona_id != 0 {
            pid = session.persona_id as i64;
        }
    }
    if gid <= 0 {
        return HttpResponse::new(
            400,
            "application/json",
            br#"{"ok":false,"error":"gid required"}"#.to_vec(),
        );
    }
    let body = game_state::leave_gameroom_ex(gid, pid, force || pid <= 0);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m leave-game gid={} pid={} force={} ? {}",
        gid,
        pid,
        force || pid <= 0,
        body
    );
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn handle_cnc_player_ready(query: Option<&str>) -> HttpResponse {
    let mut gid: i64 = 1;
    let mut pid: i64 = 0;
    let mut ready = true;
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                match k {
                    "gid" => gid = percent_decode_plus(v).parse().unwrap_or(1),
                    "pid" => pid = percent_decode_plus(v).parse().unwrap_or(0),
                    "ready" => {
                        let s = percent_decode_plus(v);
                        ready = s == "1" || s.eq_ignore_ascii_case("true");
                    }
                    _ => {}
                }
            }
        }
    }
    if pid <= 0 {
        let session = crate::session::get_user_session();
        pid = if session.persona_id == 0 {
            1000
        } else {
            session.persona_id as i64
        };
    }
    let ok = game_state::set_player_ready(gid, pid, ready);
    let body = serde_json::json!({
        "ok": ok,
        "gid": gid,
        "pid": pid,
        "ready": ready,
        "allReady": game_state::all_humans_ready(gid),
        "admin": game_state::host_persona_for_gid(gid),
    });
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn handle_cnc_lobby_roster(query: Option<&str>) -> HttpResponse {
    let mut gid: i64 = 1;
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if k == "gid" {
                    gid = percent_decode_plus(v).parse().unwrap_or(1);
                }
            }
        }
    }
    let body = game_state::lobby_roster_json(gid);
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

/// POST `/cnc/api/start-battle` -- advance game state for a given GID (testing API).
fn handle_cnc_start_battle(body: &[u8]) -> HttpResponse {
    use crate::client::cnc::game_state;
    use crate::client::cnc::fireframe;

    let gid = std::str::from_utf8(body)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("gid").and_then(|g| g.as_i64()))
        .unwrap_or(1);

    let phase = game_state::get_phase(gid);
    if phase == game_state::GamePhase::InGame {
        return HttpResponse::new(200, "application/json", br#"{"ok":true,"info":"already in game"}"#.to_vec());
    }

    if !game_state::all_humans_ready(gid) {
        let body = serde_json::json!({
            "ok": false,
            "error": "not_all_ready",
            "gid": gid,
            "allReady": false,
            "admin": game_state::host_persona_for_gid(gid),
        });
        return HttpResponse::new(409, "application/json", body.to_string().into_bytes());
    }

    game_state::set_phase(gid, game_state::GamePhase::InGame);
    let players = game_state::players_for_gid(gid);

    // Enqueue advanceGameState notifications for each human player's Blaze session.
    for p in &players {
        if p.is_ai {
            continue;
        }
        let sessions = crate::session::blaze_sessions::list_sessions();
        for s in &sessions {
            if s.persona_id == Some(p.persona_id as u64) {
                if let Ok(pushes) = fireframe::pushes_after_advance_game_state(gid) {
                    fireframe::enqueue_pending_pushes(s.id, pushes);
                    // Wake the session's read loop so the pushes go out immediately
                    // instead of waiting for its next inbound packet / idle timeout.
                    let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
                }
                break;
            }
        }
    }

    let body = serde_json::json!({
        "ok": true,
        "gid": gid,
        "phase": phase.label(),
        "player_count": players.len(),
    });
    HttpResponse::new(200, "application/json", body.to_string().into_bytes())
}

fn sanitize_probe_dump_filename(raw: &str) -> String {
    let mut s = String::new();
    for c in raw.chars().take(120) {
        if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
            s.push(c);
        } else if c == '%' {
            // skip; minimal encoding not supported
        } else {
            s.push('_');
        }
    }
    if s.is_empty() || s == "." {
        "cnc-probe-log.txt".to_string()
    } else {
        s
    }
}

fn cnc_local_request_relative_path(clean: &str) -> Option<String> {
    if let Some(rest) = clean.strip_prefix("/cnc/data/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = clean.strip_prefix("/cncg2/") {
        return Some(format!("cncg2/{rest}"));
    }
    if let Some(rest) = clean.strip_prefix("/config.cncg2/") {
        return Some(format!("config.cncg2/{rest}"));
    }
    if clean == "/config.cncg2" || clean == "/config.cncg2/" {
        return Some("config.cncg2/cncprod150805.cfg".to_string());
    }
    None
}

pub fn try_handle_http_request(method: &str, path: &str) -> Option<HttpResponse> {
    let is_head = method == "HEAD";
    if method != "GET" && !is_head {
        return None;
    }

    let clean = path.split('?').next().unwrap_or(path);
    let request_rel = cnc_local_request_relative_path(clean)?;

    let rel = sanitize_relative_request_path(&request_rel)?;
    for root in cnc_http_data_roots() {
        if let Some(response) = try_read_http_file(&root, &rel, is_head) {
            return Some(response);
        }
    }

    Some(HttpResponse::new(404, "text/plain", b"Not Found".to_vec()))
}

fn cnc_http_data_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    roots.push(cnc_data_runtime_dir());
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("data").join("client").join("cnc"));
        }
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("client").join("cnc").join("data"));
    roots
}

fn try_read_http_file(root: &Path, rel: &Path, is_head: bool) -> Option<HttpResponse> {
    let full = root.join(rel);

    let mut try_paths = if full.is_dir() {
        vec![full.join("index.html"), full.join("devWrapper.html")]
    } else {
        vec![full.clone(), full.join("index.html"), full.join("devWrapper.html")]
    };
    if full.extension().is_none() {
        try_paths.push(full.with_extension("html"));
    }

    for candidate in try_paths {
        if let Ok(bytes) = std::fs::read(&candidate) {
            let ct = content_type_for(&candidate);
            let body = if is_head {
                Vec::new()
            } else if ct == "text/html" {
                inject_profile_script(&bytes)
            } else {
                bytes
            };
            let mut response = HttpResponse::new(200, ct, body);
            if matches!(
                ct,
                "text/html" | "application/javascript" | "text/javascript" | "text/css"
            ) {
                response.headers.insert(
                    "Cache-Control".to_string(),
                    "no-cache, no-store, must-revalidate".to_string(),
                );
                response
                    .headers
                    .insert("Pragma".to_string(), "no-cache".to_string());
            }
            return Some(response);
        }
    }
    None
}

/// Templates the active Refracted user profile into served HTML so the JS shell
/// can authenticate as the chosen persona instead of a hardcoded placeholder.
fn inject_profile_script(html: &[u8]) -> Vec<u8> {
    let s = match std::str::from_utf8(html) {
        Ok(s) => s,
        Err(_) => return html.to_vec(),
    };

    let p = crate::common::user_profile::get_current_profile();
    let json = serde_json::json!({
        "email": p.email,
        "username": p.username,
        "displayName": p.display_name,
        "personaId": p.persona_id,
        "userId": p.user_id,
    });
    let build = shell_build_info_json();
    let script = format!(
        "<script>window.__CNC_PROFILE={};window.__CNC_BUILD={};</script>",
        json, build
    );

    let lower = s.to_ascii_lowercase();
    let insert_at = lower
        .find("<head>")
        .map(|i| i + "<head>".len())
        .or_else(|| lower.find("<head ").and_then(|i| s[i..].find('>').map(|j| i + j + 1)));

    match insert_at {
        Some(i) => {
            let mut out = String::with_capacity(s.len() + script.len());
            out.push_str(&s[..i]);
            out.push_str(&script);
            out.push_str(&s[i..]);
            out.into_bytes()
        }
        None => html.to_vec(),
    }
}

pub fn handle_redirector_get_server_instance(_payload: &[u8]) -> BlazeResult<Bytes> {
    let ports = crate::common::game::current_service_ports();
    let host = "127.0.0.1";
    let ip = u32::from_be_bytes(std::net::Ipv4Addr::new(127, 0, 0, 1).octets()) as i32;

    let mut response = Vec::new();
    response.extend_from_slice(&encode_union_struct("ADDR", 0, |valu| {
        valu.extend_from_slice(&TdfEncoder::encode_string("HOST", host));
        valu.extend_from_slice(&TdfEncoder::encode_int("IP\0\0", ip));
        valu.extend_from_slice(&TdfEncoder::encode_int("PORT", ports.blaze_main as i32));
    }));
    response.extend_from_slice(&TdfEncoder::encode_int("SECU", 0));
    response.extend_from_slice(&TdfEncoder::encode_int("XDNS", 0));
    Ok(Bytes::from(response))
}

pub fn handle_packet_fields(
    component: u16,
    command: u16,
    payload: &[u8],
) -> Option<BlazeResult<Bytes>> {
    match (component, command) {
        (0x0009, 0x02) => Some(handle_util_ping(payload)),
        (0x0009, 0x01) => Some(handle_util_fetch_client_config(payload)),
        (0x0009, 0x08) => Some(handle_util_post_auth(payload)),
        (0x0009, 0x05) => Some(handle_util_get_telemetry_server(payload)),
        (0x0009, 0x09) => Some(handle_util_set_client_state(payload)),
        (0x0009, 0x16) => Some(handle_util_set_client_metrics(payload)),
        (0x0009, 0x1c) => Some(handle_util_set_client_state_28(payload)),
        (0x0001, 0x0a) => Some(handle_auth_login(payload)),
        (0x0001, 0x28) => Some(handle_auth_login(payload)),
        (0x0001, 0x3c) => Some(handle_auth_login(payload)),
        (0x0001, 0x6e) => Some(handle_auth_login_persona(payload)),
        (0x0001, 0x46) => Some(handle_auth_logout(payload)),
        (0x000F, 0x01) => Some(handle_messaging_send_message(payload)),
        (0x7802, 0x01) => Some(handle_user_sessions_command_1(payload)),
        (0x7802, 0x08) => Some(handle_user_sessions_update_hardware_flags(payload)),
        (0x7802, 0x0c) => Some(handle_user_sessions_lookup_user(payload)),
        (0x7802, 0x0d) => Some(handle_user_sessions_lookup_users(payload)),
        (0x7802, 0x14) => Some(handle_user_sessions_update_network_info(payload)),
        (0x7802, 0x0b) => Some(handle_user_sessions_set_user_cross_platform_opt_in(payload)),
        (0x7802, 0x15) => Some(handle_user_sessions_lookup_users(payload)),
        (0x7802, 0x3c) => Some(handle_user_sessions_command_60(payload)),
        (0x0007, 0x00) => Some(handle_stats_command_0(payload)),
        (0x0007, 0xf00) => Some(handle_stats_command_3840(payload)),
        (0x0007, 0x2900) => Some(handle_stats_command_10496(payload)),
        (0x0007, 0x3700) => Some(handle_stats_command_14080(payload)),
        (0x0007, 0x4100) => Some(handle_stats_command_16640(payload)),
        (0x0007, 0x4f00) => Some(handle_stats_command_20224(payload)),
        (0x0007, 0x5900) => Some(handle_stats_command_22784(payload)),
        (0x0007, 0x7100) => Some(handle_stats_command_28928(payload)),
        // createGame (RPC 1) -- some CNC client builds send this instead of resetDedicatedServer (0x16/0x19).
        (0x0004, 0x01) => Some(handle_game_manager_reset_dedicated_server(payload)),
        (0x0004, 0x03) => Some(handle_game_manager_advance_game_state(payload)),
        (0x0004, 0x04) => Some(handle_game_manager_set_game_settings(payload)),
        (0x0004, 0x05) => Some(handle_game_manager_destroy_game(payload)),
        (0x0004, 0x07) => Some(handle_game_manager_command_7(payload)),
        (0x0004, 0x09) => Some(handle_game_manager_join_game(payload)),
        (0x0004, 0x08) => Some(handle_game_manager_set_player_attributes(payload)),
        (0x0004, 0x0b) => Some(handle_game_manager_remove_player(payload)),
        // updateMeshConnection (RPC 29) -- joining client reports it connected to the dedicated endpoint.
        (0x0004, 0x1d) => Some(handle_game_manager_update_mesh_connection(payload)),
        (0x0004, 0x0d) => Some(handle_game_manager_finalize_game_creation(payload)),
        // CNC 3.19.4: `finalizeGameCreation` is RPC **15** (`0x0F`) with `UpdateGameSessionRequest`.
        (0x0004, 0x0f) => Some(handle_game_manager_finalize_game_creation(payload)),
        (0x0004, 0x12) => Some(handle_game_manager_set_player_custom_data(payload)),
        (0x0004, 0x0a) => Some(handle_game_manager_command_10(payload)),
        (0x0004, 0x10) => Some(handle_game_manager_command_16(payload)),
        // CNC: returnDedicatedServerToPool is RPC id 20 (0x14), not 17 (0x11 = removePlayer on EA table).
        (0x0004, 0x14) => Some(handle_game_manager_return_dedicated_server_to_pool(payload)),
        (0x0004, 0x26) => Some(handle_game_manager_add_queued_player_to_game(payload)),
        (0x0004, 0x96) => Some(handle_game_manager_register_dynamic_dedicated_server_creator(payload)),
        (0x0004, 0x97) => Some(handle_game_manager_unregister_dynamic_dedicated_server_creator(payload)),
        (0x0004, 0x64) => Some(handle_game_manager_get_game_list_snapshot(payload)),
        (0x0004, 0x0e) => Some(handle_game_manager_list_games(payload)),
        (0x0004, 0x22) => Some(handle_game_manager_list_game_data(payload)),
        // getFullGameData (0x2C in some tables, 0x67 in CNC 3.19.4)
        (0x0004, 0x2c) => Some(handle_game_manager_get_full_game_data(payload)),
        (0x0004, 0x67) => Some(handle_game_manager_get_full_game_data(payload)),
        // CNC Blaze 3.19.4: dedicated reset uses 0x0019; official table lists reset at 0x16 -- both return JoinGameResponse.
        (0x0004, 0x16) => Some(handle_game_manager_reset_dedicated_server(payload)),
        (0x0004, 0x19) => Some(handle_game_manager_reset_dedicated_server(payload)),
        (0x0004, 0x41) => Some(handle_game_manager_mesh_endpoints_connected(payload)),
        (0x0004, 0x71) => Some(handle_game_manager_command_113(payload)),
        // RedirectorComponent::getServerInstance
        (0x0005, 0x0001) => Some(handle_redirector_get_server_instance(payload)),
        // UtilComponent::preAuth
        (0x0009, 0x0007) => Some(handle_util_preauth(payload)),
        // Blaze::Rooms -- hub assigns component id at runtime (~`0x7800` segment). Extend when discovery captures `(id,opcode)`.
        _ => None,
    }
}

pub fn handle_util_preauth(payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_string("ASRC", "324320"));
    response.extend_from_slice(&TdfEncoder::encode_list(
        "CIDS",
        &[
            30728, 1, 30729, 25, 30730, 555, 30731, 4, 30732, 9, 10, 63490, 403, 13, 15, 30720,
            30721, 30722, 30723, 30724, 30725, 30726, 30727,
        ],
    ));

    let mut conf_struct = Vec::new();
    let conf_map = cnc_blaze_conf_map();
    conf_struct.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("CONF", &conf_map));
    response.extend_from_slice(&TdfEncoder::encode_struct("CONF", &conf_struct));

    response.extend_from_slice(&TdfEncoder::encode_bool("EEFA", true));
    response.extend_from_slice(&TdfEncoder::encode_string("ESRC", "324320"));
    response.extend_from_slice(&TdfEncoder::encode_string("INST", "cncprod150805"));
    response.extend_from_slice(&TdfEncoder::encode_int("MINR", 0));
    response.extend_from_slice(&TdfEncoder::encode_string("NASP", "cem_ea_id"));
    response.extend_from_slice(&TdfEncoder::encode_string("PILD", ""));
    response.extend_from_slice(&TdfEncoder::encode_string("PLAT", "pc"));
    response.extend_from_slice(&cnc_encode_preauth_qoss_field());
    response.extend_from_slice(&TdfEncoder::encode_string("RSRC", "324320"));
    response.extend_from_slice(&TdfEncoder::encode_string("SVER", "Blaze 3.19.4.0"));

    let cfid = TdfEncoder::find_string_field(payload, "CFID").unwrap_or_else(|| "BlazeSDK".to_string());
    let web = crate::common::game::current_service_ports().web_http;
    let grpc_url = format!("http://127.0.0.1:{web}");
    crate::session::session_module::record_last_fetch_client_config(&cfid, "cnc", &grpc_url);

    Ok(Bytes::from(response))
}

pub fn handle_util_fetch_client_config(payload: &[u8]) -> BlazeResult<Bytes> {
    let cfid = TdfEncoder::find_string_field(payload, "CFID").unwrap_or_else(|| "BlazeSDK".to_string());
    let web = crate::common::game::current_service_ports().web_http;
    let grpc_url = format!("http://127.0.0.1:{web}");
    crate::session::session_module::record_last_fetch_client_config(&cfid, "cnc", &grpc_url);
    let conf_map = cnc_blaze_conf_map();
    Ok(Bytes::from(TdfEncoder::encode_string_string_map_ordered(
        "CONF", &conf_map,
    )))
}

pub fn handle_util_post_auth(_payload: &[u8]) -> BlazeResult<Bytes> {
    let session = crate::session::get_user_session();
    let uid = if session.persona_id == 0 { 1000 } else { session.persona_id as i64 };

    let mut response = Vec::new();

    // Ascending packed-tag order: ADRS CSIG OIDS PJID PORT RPRT TIID.
    let mut pss = Vec::new();
    pss.extend_from_slice(&TdfEncoder::encode_string("ADRS", "127.0.0.1"));
    pss.extend_from_slice(&TdfEncoder::encode_struct("CSIG", &[]));
    pss.extend_from_slice(&TdfEncoder::encode_object_id_list("OIDS", &[]));
    pss.extend_from_slice(&TdfEncoder::encode_string("PJID", "123071"));
    pss.extend_from_slice(&TdfEncoder::encode_int("PORT", 80));
    pss.extend_from_slice(&TdfEncoder::encode_int("RPRT", 9));
    pss.extend_from_slice(&TdfEncoder::encode_int("TIID", 0));
    response.extend_from_slice(&TdfEncoder::encode_struct("PSS", &pss));

    // Field order aligned with Labs `postAuth` TELE so Prism / strict TDF decoders stay in sync.
    let disa = "AD,AF,AG,AI,AL,AM,AN,AO,AQ,AR,AS,AW,AX,AZ,BA,BB,BD,BF,BH,BI,BJ,BM,BN,BO,BR,BS,BT,BV,BW,BY,BZ,CC,CD,CF,CG,CI,CK,CL,CM,CN,CO,CR,CU,CV,CX,DJ,DM,DO,DZ,EC,EG,EH,ER,ET,FJ,FK,FM,FO,GA,GD,GE,GF,GG,GH,GI,GL,GM,GN,GP,GQ,GS,GT,GU,GW,GY,HM,HN,HT,ID,IL,IM,IN,IO,IQ,IR,IS,JE,JM,JO,KE,KG,KH,KI,KM,KN,KP,KR,KW,KY,KZ,LA,LB,LC,LI,LK,LR,LS,LY,MA,MC,MD,ME,MG,MH,ML,MM,MN,MO,MP,MQ,MR,MS,MU,MV,MW,MY,MZ,NA,NC,NE,NF,NG,NI,NP,NR,NU,OM,PA,PE,PF,PG,PH,PK,PM,PN,PS,PW,PY,QA,RE,RS,RW,SA,SB,SC,SD,SG,SH,SJ,SL,SM,SN,SO,SR,ST,SV,SY,SZ,TC,TD,TF,TG,TH,TJ,TK,TL,TM,TN,TO,TT,TV,TZ,UA,UG,UM,UY,UZ,VA,VC,VE,VG,VN,VU,WF,WS,YE,YT,ZM,ZW,ZZ";
    let mut tele = Vec::new();
    tele.extend_from_slice(&TdfEncoder::encode_string("ADRS", "127.0.0.1"));
    tele.extend_from_slice(&TdfEncoder::encode_int("ANON", 0));
    tele.extend_from_slice(&TdfEncoder::encode_string("BKEY", ""));
    tele.extend_from_slice(&TdfEncoder::encode_int("CTRY", 0));
    tele.extend_from_slice(&TdfEncoder::encode_string("DISA", disa));
    tele.extend_from_slice(&TdfEncoder::encode_int("ECCT", 0));
    tele.extend_from_slice(&TdfEncoder::encode_int("EDCT", 0));
    tele.extend_from_slice(&TdfEncoder::encode_string("FILT", "-GAME/COMM/EXPD"));
    tele.extend_from_slice(&TdfEncoder::encode_int("LOC", 2053653326));
    tele.extend_from_slice(&TdfEncoder::encode_int("MINR", 0));
    tele.extend_from_slice(&TdfEncoder::encode_string("NOOK", "US,CA,MX"));
    tele.extend_from_slice(&TdfEncoder::encode_string("PENV", "prod"));
    tele.extend_from_slice(&TdfEncoder::encode_int("PORT", 80));
    tele.extend_from_slice(&TdfEncoder::encode_string(
        "PURL",
        "https://pin-river.data.ea.com",
    ));
    tele.extend_from_slice(&TdfEncoder::encode_int("SDLY", 15000));
    tele.extend_from_slice(&TdfEncoder::encode_string("SESS", "tele_sess"));
    tele.extend_from_slice(&TdfEncoder::encode_string("SKEY", "some_tele_key"));
    tele.extend_from_slice(&TdfEncoder::encode_int("SPCT", 75));
    tele.extend_from_slice(&TdfEncoder::encode_string("STIM", "Default"));
    tele.extend_from_slice(&TdfEncoder::encode_string("SVNM", "telemetry-3-common"));
    response.extend_from_slice(&TdfEncoder::encode_struct("TELE", &tele));

    let mut tick = Vec::new();
    tick.extend_from_slice(&TdfEncoder::encode_string("ADRS", "127.0.0.1"));
    tick.extend_from_slice(&TdfEncoder::encode_int("PORT", 8999));
    tick.extend_from_slice(&TdfEncoder::encode_string(
        "SKEY",
        &format!("{uid},127.0.0.1:80,cncprod150805,10,50,50,50,50,0,0"),
    ));
    response.extend_from_slice(&TdfEncoder::encode_struct("TICK", &tick));

    let mut urop = Vec::new();
    urop.extend_from_slice(&TdfEncoder::encode_int("TMOP", 1));
    urop.extend_from_slice(&TdfEncoder::encode_long("UID", uid));
    response.extend_from_slice(&TdfEncoder::encode_struct("UROP", &urop));
    Ok(Bytes::from(response))
}

pub fn handle_util_get_telemetry_server(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_string("ADRS", "https://river.data.ea.com"));
    response.extend_from_slice(&TdfEncoder::encode_int("ANON", 0));
    response.extend_from_slice(&TdfEncoder::encode_binary("BKEY", &[]));
    response.extend_from_slice(&TdfEncoder::encode_int("CTRY", 17230));
    response.extend_from_slice(&TdfEncoder::encode_string("PENV", "prod"));
    response.extend_from_slice(&TdfEncoder::encode_int("PORT", 443));
    response.extend_from_slice(&TdfEncoder::encode_string("PURL", "https://pin-river.data.ea.com"));
    response.extend_from_slice(&TdfEncoder::encode_int("SDLY", 15000));
    response.extend_from_slice(&TdfEncoder::encode_string("SKEY", "1"));
    response.extend_from_slice(&TdfEncoder::encode_int("SPCT", 75));
    response.extend_from_slice(&TdfEncoder::encode_string("STIM", "Default"));
    Ok(Bytes::from(response))
}

/// Identity for auth / user-session responses on the CURRENT Blaze session: a pooled dedicated
/// server responds as its own `CNCO<N>` persona; every other session uses the shared client profile.
pub fn cnc_effective_identity() -> (u64, String) {
    if let Some(sid) = crate::session::session_module::current_blaze_session_id() {
        if let Some((persona, name)) = dedicated_pool::dedicated_identity_for_session(sid) {
            return (persona, name);
        }
    }
    let s = get_user_session();
    let persona = if s.persona_id == 0 { 1000 } else { s.persona_id };
    let name = if s.display_name.is_empty() {
        "Player".to_string()
    } else {
        s.display_name.clone()
    };
    (persona, name)
}

pub fn handle_auth_login(payload: &[u8]) -> BlazeResult<Bytes> {
    let has_tokn = TdfEncoder::find_string_field(payload, "TOKN")
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);
    let mail = TdfEncoder::find_string_field(payload, "MAIL").filter(|m| !m.is_empty());
    if has_tokn && mail.is_none() {
        // Token login with no email = a pooled dedicated server (cnc.server.exe). Give it its own
        // CNCO<N> identity now, so this login response (and later user-session notifies) report the
        // dedicated persona instead of the shared client profile.
        if let Some(sid) = crate::session::session_module::current_blaze_session_id() {
            let (name, persona) = dedicated_pool::allocate_dedicated_identity(sid);
            crate::session::blaze_sessions::set_dedicated_identity(sid, &name, persona);
        }
    } else if let Some(mail) = mail {
        let mut s = get_user_session();
        s.email = mail;
        set_user_session(s);
    }

    let (uid_u, display_name) = cnc_effective_identity();
    let uid = uid_u as i64;
    let session = crate::session::get_user_session();
    let session_key =
        crate::client::labs::payload_auth::blaze_session_key(uid, uid);

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_bool("ANON", false));
    response.extend_from_slice(&TdfEncoder::encode_bool("NTOS", false));
    response.extend_from_slice(&TdfEncoder::encode_string("PCTK", ""));

    let _ = &session;
    let mut profile_struct = Vec::new();
    profile_struct.extend_from_slice(&TdfEncoder::encode_string("DSNM", &display_name));
    profile_struct.extend_from_slice(&TdfEncoder::encode_int("LAST", 0));
    profile_struct.extend_from_slice(&TdfEncoder::encode_long("PID ", uid));
    profile_struct.extend_from_slice(&TdfEncoder::encode_int("PLAT", PLAT_PC));
    profile_struct.extend_from_slice(&TdfEncoder::encode_int("STAS", STAS_ACTIVE));
    profile_struct.extend_from_slice(&TdfEncoder::encode_long("XREF", 0));
    response.extend_from_slice(&encode_struct_list("PLST", &[profile_struct]));

    response.extend_from_slice(&TdfEncoder::encode_string("SKEY", &session_key));
    response.extend_from_slice(&TdfEncoder::encode_bool("SPAM", false));
    response.extend_from_slice(&TdfEncoder::encode_long("UID ", uid));
    response.extend_from_slice(&TdfEncoder::encode_bool("UNDR", false));
    Ok(Bytes::from(response))
}

pub fn handle_auth_login_persona(payload: &[u8]) -> BlazeResult<Bytes> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Don't let a pooled dedicated server's loginPersona overwrite the shared client profile.
    let is_dedicated = crate::session::session_module::current_blaze_session_id()
        .map(|sid| dedicated_pool::dedicated_identity_for_session(sid).is_some())
        .unwrap_or(false);
    if !is_dedicated {
        let mut session = crate::session::get_user_session();
        if let Some(pnam) = TdfEncoder::find_string_field(payload, "PNAM") {
            if !pnam.is_empty() {
                session.display_name = pnam;
            }
        }
        if session.persona_id == 0 {
            session.persona_id = 1000;
            session.user_id = 1000;
        }
        set_user_session(session);
    }

    let (uid_u, display_name) = cnc_effective_identity();
    let uid = uid_u as i64;
    let mail = if is_dedicated {
        String::new()
    } else {
        get_user_session().email
    };
    let session_key = crate::client::labs::payload_auth::blaze_session_key(uid, uid);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("BUID", uid));
    response.extend_from_slice(&TdfEncoder::encode_bool("FRST", false));
    response.extend_from_slice(&TdfEncoder::encode_string("KEY ", &session_key));
    response.extend_from_slice(&TdfEncoder::encode_long("LLOG", now));
    response.extend_from_slice(&TdfEncoder::encode_string("MAIL", &mail));

    let mut pdtl = Vec::new();
    pdtl.extend_from_slice(&TdfEncoder::encode_string("DSNM", &display_name));
    pdtl.extend_from_slice(&TdfEncoder::encode_long("LAST", now));
    pdtl.extend_from_slice(&TdfEncoder::encode_long("PID ", uid));
    pdtl.extend_from_slice(&TdfEncoder::encode_int("PLAT", PLAT_PC));
    pdtl.extend_from_slice(&TdfEncoder::encode_int("STAS", STAS_ACTIVE));
    pdtl.extend_from_slice(&TdfEncoder::encode_long("XREF", 0));
    response.extend_from_slice(&TdfEncoder::encode_struct("PDTL", &pdtl));
    response.extend_from_slice(&TdfEncoder::encode_long("UID ", uid));
    Ok(Bytes::from(response))
}

pub fn handle_auth_logout(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_bool("SUCC", true));
    Ok(Bytes::from(response))
}

pub fn handle_util_ping(payload: &[u8]) -> BlazeResult<Bytes> {
    if payload.is_empty() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut response = Vec::new();
        let stim = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;
        response.extend_from_slice(&TdfEncoder::encode_int("STIM", stim));
        Ok(Bytes::from(response))
    } else {
        Ok(Bytes::from(vec![payload[0]]))
    }
}

pub fn handle_util_set_client_state(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_util_set_client_metrics(payload: &[u8]) -> BlazeResult<Bytes> {
    let ubfl = TdfEncoder::find_int_field(payload, "UBFL").unwrap_or(0);
    let udev = TdfEncoder::find_string_field(payload, "UDEV").unwrap_or_default();
    let uflg = TdfEncoder::find_int_field(payload, "UFLG").unwrap_or(0);
    let ulrc = TdfEncoder::find_int_field(payload, "ULRC").unwrap_or(0);
    let unat = TdfEncoder::find_int_field(payload, "UNAT").unwrap_or(0);
    let usta = TdfEncoder::find_int_field(payload, "USTA").unwrap_or(0);
    let uwan = TdfEncoder::find_int_field(payload, "UWAN")
        .map(|v| v as u32)
        .or_else(|| TdfEncoder::find_long_field(payload, "UWAN").map(|v| v as u32))
        .unwrap_or(0);

    crate::debug_println!(
        "\x1b[38;2;100;200;255m[CNC]\x1b[0m setClientMetrics UBFL={} USTA={} UNAT={} UFLG={} ULRC={} UWAN={:#010x} UDEV={}",
        ubfl, usta, unat, uflg, ulrc, uwan, udev
    );

    if uwan != 0 {
        crate::session::merge_network_snapshot(crate::session::NetworkSnapshot {
            exip_ip: Some(uwan),
            inip_ip: None,
            exip_port: None,
            inip_port: None,
            bps: None,
        });
    }

    Ok(Bytes::from(Vec::new()))
}

pub fn handle_util_set_client_state_28(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_messaging_send_message(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    let mut response = Vec::new();
    let mgid = crate::session::get_next_message_id();
    response.extend_from_slice(&TdfEncoder::encode_int("MGID", mgid as i32));
    response.extend_from_slice(&TdfEncoder::encode_list("MIDS", &[mgid as i32]));
    Ok(Bytes::from(response))
}

pub fn handle_user_sessions_command_1(payload: &[u8]) -> BlazeResult<Bytes> {
    if payload.is_empty() {
        return Ok(Bytes::from(Vec::new()));
    }
    Ok(Bytes::from(payload.to_vec()))
}

pub fn handle_user_sessions_update_hardware_flags(payload: &[u8]) -> BlazeResult<Bytes> {
    if let Some(hwfg) = TdfEncoder::find_int_field(payload, "HWFG") {
        crate::session::set_hwfg(hwfg as u32);
    }
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_user_sessions_lookup_user(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    let session = crate::session::get_user_session();
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_int("CNTX", 1016290622));
    response.extend_from_slice(&TdfEncoder::encode_int("ERRC", 0));
    let mut user = Vec::new();
    user.extend_from_slice(&TdfEncoder::encode_long("AID ", session.user_id as i64));
    user.extend_from_slice(&TdfEncoder::encode_string("NAME", &session.display_name));
    user.extend_from_slice(&TdfEncoder::encode_string("NASP", "cem_ea_id"));
    user.extend_from_slice(&TdfEncoder::encode_long("ID  ", session.persona_id as i64));
    response.extend_from_slice(&TdfEncoder::encode_struct("USER", &user));
    Ok(Bytes::from(response))
}

pub fn handle_user_sessions_lookup_users(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    let session = crate::session::get_user_session();
    let mut response = Vec::new();
    let mut ulst_entry = Vec::new();
    let mut edat = Vec::new();
    edat.extend_from_slice(&TdfEncoder::encode_string("BPS ", ""));
    edat.extend_from_slice(&TdfEncoder::encode_string("CTY ", ""));
    edat.extend_from_slice(&TdfEncoder::encode_int("CTYP", 0));
    ulst_entry.extend_from_slice(&TdfEncoder::encode_struct("EDAT", &edat));
    ulst_entry.extend_from_slice(&TdfEncoder::encode_int("FLGS", 0));
    let mut user = Vec::new();
    user.extend_from_slice(&TdfEncoder::encode_long("AID ", session.user_id as i64));
    user.extend_from_slice(&TdfEncoder::encode_string("NAME", &session.display_name));
    ulst_entry.extend_from_slice(&TdfEncoder::encode_struct("USER", &user));
    let tag = TdfEncoder::make_tag("ULST");
    response.extend_from_slice(&[tag[0], tag[1], tag[2], 0x04, 0x03, 0x01]);
    response.extend_from_slice(&ulst_entry);
    response.push(0x00);
    Ok(Bytes::from(response))
}

pub fn handle_user_sessions_update_network_info(payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::{merge_network_snapshot, NetworkSnapshot};

    let mut ips = TdfEncoder::find_all_u32_fields(payload, "IP  ");
    if ips.is_empty() {
        ips = TdfEncoder::scan_all_u32_fields(payload, "IP  ");
    }
    let mut ports = TdfEncoder::find_all_int_fields(payload, "PORT");
    if ports.is_empty() {
        ports = TdfEncoder::scan_all_int_fields(payload, "PORT");
    }
    let bps = TdfEncoder::find_string_field(payload, "BPS ")
        .or_else(|| TdfEncoder::find_string_field(payload, "BPS"))
        .or_else(|| TdfEncoder::scan_first_string_field(payload, "BPS "))
        .or_else(|| TdfEncoder::scan_first_string_field(payload, "BPS"))
        .filter(|s| !s.is_empty());
    let mut n = NetworkSnapshot::default();
    if ips.len() >= 2 {
        n.exip_ip = Some(ips[0]);
        n.inip_ip = Some(ips[1]);
    } else if ips.len() == 1 {
        n.exip_ip = Some(ips[0]);
    }
    if ports.len() >= 2 {
        n.exip_port = Some(ports[0]);
        n.inip_port = Some(ports[1]);
    } else if ports.len() == 1 {
        n.exip_port = Some(ports[0]);
    }
    n.bps = bps;
    merge_network_snapshot(n);
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_user_sessions_set_user_cross_platform_opt_in(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_user_sessions_command_60(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_0(payload: &[u8]) -> BlazeResult<Bytes> {
    if payload.len() >= 1 {
        Ok(Bytes::from(vec![payload[0]]))
    } else {
        Ok(Bytes::from(vec![0x09]))
    }
}

pub fn handle_stats_command_3840(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_10496(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_14080(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_16640(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_20224(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_22784(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_stats_command_28928(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

fn cnc_join_game_response(gid: i64) -> Bytes {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    response.extend_from_slice(&TdfEncoder::encode_int("JGS ", JGS_JOINED_GAME));
    Bytes::from(response)
}

/// matches **`handle_game_manager_command_16`** (GID + JGS + **`OCAL`**) and now also emits **`NTOP`**
/// so the client picks up the intended network topology (PEER vs DEDICATED).
fn cnc_join_game_response_with_ocal(gid: i64, gsid: Option<i64>) -> Bytes {
    let mut response = Vec::new();
    let server_label = gsid
        .filter(|&id| id > 0)
        .map(|id| id.to_string())
        .unwrap_or_default();
    // Blaze `JoinGameResponse` must lead with `GID`/`JGS` -- a leading `SRVR` string breaks the RTS
    response.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    response.extend_from_slice(&TdfEncoder::encode_int("JGS ", JGS_JOINED_GAME));
    response.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_DEFAULT));
    response.extend_from_slice(&TdfEncoder::encode_int("OCAL", 0));
    if let Some(server_id) = gsid.filter(|&id| id > 0) {
        response.extend_from_slice(&TdfEncoder::encode_long("GSID", server_id));
    }
    if !server_label.is_empty() {
        response.extend_from_slice(&TdfEncoder::encode_string("SRVR", &server_label));
    }
    Bytes::from(response)
}

/// `GameManager.joinGame` (0x0004::0x0009) -- `JoinGameResponse` with the requested or default game id.
pub fn handle_game_manager_join_game(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = cnc_extract_join_game_id(payload);
    let session = crate::session::get_user_session();
    let pid = if session.persona_id == 0 {
        1000_i64
    } else {
        session.persona_id as i64
    };
    let name = if session.display_name.is_empty() {
        "Player".to_string()
    } else {
        session.display_name.clone()
    };
    if !game_state::join_password_allowed(gid, pid) {
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m joinGame REJECTED gid={} pid={} (password required - verify via shell / ATTR _password)",
            gid,
            pid
        );
        return Err(crate::common::error::BlazeError::AuthorizationRequired);
    }
    if let Some(player) = game_state::ensure_client_player(gid, pid, &name) {
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m joinGame gid={} pid={} host={} (GameRoom lobby)",
            gid,
            pid,
            game_state::host_persona_for_gid(gid)
        );
        let _ = player;
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m joinGame gid={} pid={} (no game row)",
            gid,
            pid
        );
    }
    Ok(cnc_join_game_response(gid))
}

/// Shared GID extraction for CNC `joinGame` flow.
pub fn cnc_extract_join_game_id(payload: &[u8]) -> i64 {
    TdfEncoder::find_int_field(payload, "GID")
        .map(|v| v as i64)
        .or_else(|| {
            TdfEncoder::scan_all_u32_fields(payload, "GID")
                .first()
                .map(|&u| u as i64)
        })
        .filter(|&g| g > 0)
        .unwrap_or(1)
}

/// CNC dedicated reset (`CreateGameRequest` in / `JoinGameResponse` out). Also mapped at EA id `0x16`.
pub fn handle_game_manager_reset_dedicated_server(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = cnc_extract_reset_game_id(payload);
    let _ = game_state::adopt_host_lobby_pending_into(gid);
    game_state::adopt_host_lobby_pending_attrs_into(gid);
    if let Some(attr_level) = extract_attr_level(payload) {
        let pending = game_state::get_map_path(gid);
        if pending.is_empty() {
            tracing::warn!(
                target: "cnc",
                "[CNC] resetDedicated ATTR._level=\"{}\" but PENDING empty - adopting ATTR (lobby select-map missed)",
                attr_level
            );
            game_state::set_map_path(gid, &attr_level);
        } else if pending != attr_level {
            tracing::info!(
                target: "cnc",
                "[CNC] resetDedicated map: PENDING=\"{}\" ATTR=\"{}\" - inject will force PENDING",
                pending,
                attr_level
            );
        } else {
            tracing::info!(
                target: "cnc",
                "[CNC] resetDedicated map=\"{}\" (PENDING==ATTR)",
                pending
            );
        }
    } else {
        tracing::info!(
            target: "cnc",
            "[CNC] resetDedicated gid={} PENDING=\"{}\" (no ATTR._level on wire)",
            gid,
            game_state::get_map_path(gid)
        );
    }
    game_state::seed_from_reset(payload, gid);
    // Resolve the dedicated session id the reset will land on even if the pool assignment hasn't
    // been created yet (the encrypted-Fire2 path builds this reply before orchestrate runs), so the
    // reply carries GSID/SRVR and the shell `serverID` isn't "unknown".
    let gsid = dedicated_pool::host_for_gid(gid)
        .map(|d| d.blaze_session_id as i64)
        .or_else(|| dedicated_pool::peek_dedicated_for_gid(gid).map(|s| s as i64));
    Ok(cnc_join_game_response_with_ocal(gid, gsid))
}

/// Shared GID extraction for CNC `resetDedicatedServer` flow -- used by the request reply and by the
/// follow-up `NotifyGameSetup` async push so both reference the same id.
pub fn cnc_extract_reset_game_id(payload: &[u8]) -> i64 {
    TdfEncoder::find_int_field(payload, "RGID")
        .filter(|&g| g > 0)
        .map(|g| g as i64)
        .or_else(|| {
            TdfEncoder::scan_all_u32_fields(payload, "RGID")
                .first()
                .copied()
                .filter(|&u| u > 0)
                .map(|u| u as i64)
        })
        .unwrap_or(1)
}

/// `GameManager.finalizeGameCreation` -- CNC wire id **`0x000F`** (`UpdateGameSessionRequest`: `GID` + `XNNC`/`XSES` blobs).
pub fn handle_game_manager_finalize_game_creation(payload: &[u8]) -> BlazeResult<Bytes> {
    if let Some((gid, pid, blobs)) = game_state::apply_set_player_custom_data(payload) {
        for (key, value) in &blobs {
            crate::debug_println!(
                "\x1b[38;2;255;215;0m[CNC]\x1b[0m finalizeGameCreation gid={} pid={} {} ({} bytes)",
                gid,
                pid,
                key,
                value.len()
            );
        }
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;165;0m[CNC]\x1b[0m finalizeGameCreation: could not parse GID/XNNC/XSES ({} bytes)",
            payload.len()
        );
    }
    Ok(Bytes::from(Vec::new()))
}

/// `GameManager.setPlayerCustomData` -- CNC wire id **`0x0012`** (18).
pub fn handle_game_manager_set_player_custom_data(payload: &[u8]) -> BlazeResult<Bytes> {
    if let Some((gid, pid, blobs)) = game_state::apply_set_player_custom_data(payload) {
        for (key, value) in &blobs {
            crate::debug_println!(
                "\x1b[38;2;255;215;0m[CNC]\x1b[0m setPlayerCustomData gid={} pid={} {} ({} bytes)",
                gid,
                pid,
                key,
                value.len()
            );
        }
    }
    Ok(Bytes::from(Vec::new()))
}

/// `GameManager.meshEndpointsConnected` (**`0x0004::0x0041`**, RPC id 65) -- client reports mesh link up after `createdGameNetwork`.
pub fn handle_game_manager_mesh_endpoints_connected(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = TdfEncoder::find_long_field(payload, "GID ")
        .or_else(|| TdfEncoder::find_long_field(payload, "GID"))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .unwrap_or(0);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m meshEndpointsConnected gid={}",
        gid
    );
    Ok(Bytes::from(Vec::new()))
}

/// `GameManager.updateMeshConnection` (**`0x0004::0x001D`**, RPC id 29) -- the joining client reports it
/// has connected to the dedicated's endpoint. Reply is an empty ack; the important part is the follow-up
/// `NotifyGamePlayerStateChange(ACTIVE_CONNECTED)` (sent by the dispatcher) which fires the client's
/// `createGameNetworkCb` and lets the game loop proceed instead of stalling until the RPC times out.
pub fn handle_game_manager_update_mesh_connection(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = TdfEncoder::find_long_field(payload, "GID ")
        .or_else(|| TdfEncoder::find_long_field(payload, "GID"))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .unwrap_or(1);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m updateMeshConnection gid={} -> player ACTIVE_CONNECTED",
        gid
    );
    Ok(Bytes::from(Vec::new()))
}

/// CNC `GameManager.removePlayer` (**`0x0004::0x000B`** -- same numeric id as EA `startMatchmaking`).
pub fn handle_game_manager_remove_player(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = TdfEncoder::find_long_field(payload, "GID ")
        .or_else(|| TdfEncoder::find_long_field(payload, "GID"))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64))
        .unwrap_or(0);
    let pid = TdfEncoder::find_long_field(payload, "PID ")
        .or_else(|| TdfEncoder::find_long_field(payload, "PID"))
        .or_else(|| TdfEncoder::find_int_field(payload, "PID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "PID").map(|v| v as i64))
        .unwrap_or(0);
    let reason = TdfEncoder::find_int_field(payload, "REAS")
        .filter(|&r| r != PLAYER_REMOVED_REASON_PLAYER_KICKED)
        .unwrap_or(PLAYER_REMOVED_REASON_PLAYER_LEFT);

    if gid <= 0 {
        crate::debug_println!(
            "\x1b[38;2;255;165;0m[CNC]\x1b[0m removePlayer: could not parse GID ({} bytes)",
            payload.len()
        );
        return Ok(Bytes::from(Vec::new()));
    }

    let remaining_humans = if pid > 0 {
        game_state::remove_player_ex(gid, pid)
    } else {
        Some((game_state::human_player_count(gid), false))
    };

    if pid > 0 {
        fireframe::request_client_local_game_teardown(gid, pid, reason);
    }

    match remaining_humans {
        Some((0, _)) => {
            let _ = game_state::reclaim_after_empty_humans(gid);
            crate::debug_println!(
                "\x1b[38;2;255;215;0m[CNC]\x1b[0m removePlayer gid={} pid={} - empty humans; reclaim Idle + Standby",
                gid,
                pid
            );
        }
        Some((n, converted)) => {
            crate::debug_println!(
                "\x1b[38;2;255;215;0m[CNC]\x1b[0m removePlayer gid={} pid={} - humans remaining={}{}",
                gid,
                pid,
                n,
                if converted { " (converted?AI)" } else { "" }
            );
        }
        None => {
            dedicated_pool::release_gid(gid);
            crate::debug_println!(
                "\x1b[38;2;255;215;0m[CNC]\x1b[0m removePlayer gid={} pid={} - no game; pool released",
                gid,
                pid
            );
        }
    }

    Ok(Bytes::from(Vec::new()))
}

/// `GameManager.advanceGameState` (0x0004::0x0003) -- client requests state transition from pre-game to in-game.
/// Reply is empty (success). Server pushes `NotifyGameStateChange(InGame)` after reply.
pub fn handle_game_manager_advance_game_state(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = TdfEncoder::find_long_field(payload, "GID ")
        .or_else(|| TdfEncoder::find_long_field(payload, "GID"))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64))
        .unwrap_or(0);
    if gid > 0 {
        game_state::set_phase(gid, game_state::GamePhase::InGame);
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m advanceGameState gid={} ? InGame",
            gid
        );
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;165;0m[CNC]\x1b[0m advanceGameState: could not parse GID ({} bytes)",
            payload.len()
        );
    }
    Ok(Bytes::from(Vec::new()))
}

/// GameManager.setGameSettings (0x0004::0x0004) -- host updates game settings (map, mode, etc.) in lobby.
/// Reply is empty (success). Server pushes NotifyGameSettingsChange after reply.
pub fn handle_game_manager_set_game_settings(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = TdfEncoder::find_long_field(payload, "GID ")
        .or_else(|| TdfEncoder::find_long_field(payload, "GID"))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64))
        .unwrap_or(0);
    let gset = TdfEncoder::find_int_field(payload, "GSET");
    let gnam = TdfEncoder::find_string_field(payload, "GNAM");

    if gid > 0 {
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m setGameSettings gid={} gset={:?} gnam={:?}",
            gid, gset, gnam
        );
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;165;0m[CNC]\x1b[0m setGameSettings: could not parse GID ({} bytes)",
            payload.len()
        );
    }
    Ok(Bytes::from(Vec::new()))
}

/// GameManager.destroyGame (0x0004::0x0005) -- destroy game and release dedicated server.
pub fn handle_game_manager_destroy_game(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = TdfEncoder::find_long_field(payload, "GID ")
        .or_else(|| TdfEncoder::find_long_field(payload, "GID"))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64))
        .unwrap_or(0);
    if gid > 0 {
        game_state::destroy_game(gid);
        dedicated_pool::release_gid(gid);
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m destroyGame gid={} -- game removed, pool assignment released",
            gid
        );
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;165;0m[CNC]\x1b[0m destroyGame: could not parse GID ({} bytes)",
            payload.len()
        );
    }
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_command_7(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

/// `GameManager.setPlayerAttributes` (0x0004::0x0008).
pub fn handle_game_manager_set_player_attributes(payload: &[u8]) -> BlazeResult<Bytes> {
    if let Some((gid, pid, attrs)) = game_state::apply_set_player_attributes(payload) {
        for (key, value) in &attrs {
            crate::debug_println!(
                "\x1b[38;2;255;215;0m[CNC]\x1b[0m setPlayerAttributes gid={} pid={} {}={}",
                gid,
                pid,
                key,
                value
            );
        }
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;165;0m[CNC]\x1b[0m setPlayerAttributes: could not parse GID/PID/ATTR map ({} bytes)",
            payload.len()
        );
    }
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_command_10(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_command_16(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("GID ", 52136290991));
    response.extend_from_slice(&TdfEncoder::encode_int("JGS ", 0));
    response.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_DEFAULT));
    response.extend_from_slice(&TdfEncoder::encode_int("OCAL", 0));
    Ok(Bytes::from(response))
}

/// `GameManager.getGameListSnapshot` (0x0004::0x0064).
/// Game rows are **not** inline -- the client expects follow-up `NotifyGameListUpdate` (cmd 201).
pub fn handle_game_manager_get_game_list_snapshot(_payload: &[u8]) -> BlazeResult<Bytes> {
    let gids = game_state::all_game_gids();
    let game_count = gids.len() as u32;
    let list_id = game_state::alloc_browser_list_id();
    game_state::store_game_list_snapshot(list_id, gids);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m getGameListSnapshot list_id={} ngd={} gids={:?}",
        list_id,
        game_count,
        game_state::all_game_gids()
    );
    Ok(Bytes::from(game_state::build_get_game_list_response(
        list_id, game_count,
    )))
}

fn parse_gids_from_integer_list_field(payload: &[u8], field: &str) -> Vec<i64> {
    let tag = TdfEncoder::make_tag(field);
    let mut i = 0;
    while i + 6 <= payload.len() {
        if payload[i] == tag[0]
            && payload[i + 1] == tag[1]
            && payload[i + 2] == tag[2]
            && payload[i + 3] == 0x04
            && payload[i + 4] == 0x00
        {
            let rest = &payload[i + 5..];
            if let Ok((count, n)) = TdfEncoder::decode_varint(rest) {
                let mut gids = Vec::new();
                let mut pos = n;
                for _ in 0..count {
                    if pos >= rest.len() {
                        break;
                    }
                    if let Ok((gid, consumed)) = TdfEncoder::decode_varint(&rest[pos..]) {
                        if gid > 0 {
                            gids.push(gid as i64);
                        }
                        pos += consumed;
                    } else {
                        break;
                    }
                }
                if !gids.is_empty() {
                    return gids;
                }
            }
        }
        i += 1;
    }
    Vec::new()
}

fn parse_first_gid_from_gid_list(payload: &[u8]) -> Option<i64> {
    parse_gids_from_integer_list_field(payload, "GIDL")
        .into_iter()
        .next()
}

/// Parses `GetFullGameDataRequest` (`GIDL` / `PIDL` integer lists) or root `GID` scan.
fn parse_get_full_game_data_gids(payload: &[u8]) -> Vec<i64> {
    let mut gids = parse_gids_from_integer_list_field(payload, "GIDL");
    if gids.is_empty() {
        gids = parse_gids_from_integer_list_field(payload, "PIDL");
    }
    if gids.is_empty() {
        if let Some(gid) = parse_first_gid_from_gid_list(payload) {
            gids.push(gid);
        }
    }
    if gids.is_empty() {
        if payload.len() >= 7 && payload[3] == 0x04 && payload[4] == 0x00 {
            if let Ok((count, n)) = TdfEncoder::decode_varint(&payload[5..]) {
                let mut pos = 5 + n;
                for _ in 0..count {
                    if pos >= payload.len() {
                        break;
                    }
                    if let Ok((gid, consumed)) = TdfEncoder::decode_varint(&payload[pos..]) {
                        if gid > 0 {
                            gids.push(gid as i64);
                        }
                        pos += consumed;
                    } else {
                        break;
                    }
                }
            }
        }
    }
    if gids.is_empty() {
        if let Some(gid) = TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64) {
            if gid > 0 {
                gids.push(gid);
            }
        } else if let Some(&u) = TdfEncoder::scan_all_u32_fields(payload, "GID").first() {
            if u > 0 {
                gids.push(u as i64);
            }
        }
    }
    if gids.is_empty() {
        gids.push(1);
    }
    gids
}

/// `GameManager.listGames` (0x0004::0x000E) -- minimal `GLST` so the client does not RPC-timeout.
pub fn handle_game_manager_list_games(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut game = Vec::new();
    game.extend_from_slice(&TdfEncoder::encode_long("GID ", 1));
    game.extend_from_slice(&TdfEncoder::encode_string("GNAM", "Skirmish"));
    game.extend_from_slice(&TdfEncoder::encode_int("PCNT", 1));
    game.extend_from_slice(&TdfEncoder::encode_int("PCAP", 8));
    Ok(Bytes::from(encode_struct_list("GLST", &[game])))
}

fn parse_list_game_data_gid(payload: &[u8]) -> i64 {
    parse_first_gid_from_gid_list(payload)
        .or_else(|| {
            TdfEncoder::find_int_field(payload, "GID")
                .map(|v| v as i64)
                .filter(|&g| g > 0)
        })
        .or_else(|| {
            TdfEncoder::scan_all_u32_fields(payload, "GID")
                .first()
                .copied()
                .map(|u| u as i64)
                .filter(|&g| g > 0)
        })
        .unwrap_or(1)
}

/// `GameManager.listGameData` (0x0004::0x0022) -- `ListGameData::mGameRoster` as `PLST` (matches login roster shape).
pub fn handle_game_manager_list_game_data(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = parse_list_game_data_gid(payload);
    let players = game_state::plst_entries_for_gid(gid);
    let mut response = Vec::new();
    response.extend_from_slice(&encode_struct_list("PLST", &players));
    Ok(Bytes::from(response))
}

const GFGD_MGAMES_LIST_TAG: &str = "LGAM";

/// `GameManager.getFullGameData` (0x0004::0x0067 / 0x002C) -- `GetFullGameDataResponse::mGames`.
pub fn handle_game_manager_get_full_game_data(payload: &[u8]) -> BlazeResult<Bytes> {
    let gids = parse_get_full_game_data_gids(payload);
    for gid in &gids {
        game_state::ensure_game_stub(*gid);
    }
    let mut entries = Vec::with_capacity(gids.len());
    for gid in &gids {
        entries.push(build_list_game_data_entry(*gid)?);
    }
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m getFullGameData gids={:?} rows={}",
        gids,
        entries.len()
    );
    Ok(Bytes::from(encode_struct_list(GFGD_MGAMES_LIST_TAG, &entries)))
}

pub fn handle_game_manager_return_dedicated_server_to_pool(payload: &[u8]) -> BlazeResult<Bytes> {
    log_gmgr_payload_hex("returnDedicatedServerToPool", payload);
    Ok(Bytes::from(Vec::new()))
}

/// `GameManager.addQueuedPlayerToGame` (0x0004::0x0026 / RPC id 38).
pub fn handle_game_manager_add_queued_player_to_game(payload: &[u8]) -> BlazeResult<Bytes> {
    log_gmgr_payload_hex("addQueuedPlayerToGame", payload);
    let (gid, player) = game_state::add_queued_player(payload)?;
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m addQueuedPlayerToGame gid={} slot={} ai_pid={} name={}",
        gid,
        player.slot,
        player.persona_id,
        player.display_name
    );
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_register_dynamic_dedicated_server_creator(payload: &[u8]) -> BlazeResult<Bytes> {
    // PreAuthRequest/PreAuthResponse TDFs. An empty reply clears QOSS and leaves
    // QosManager with "PingSiteInfoByAliasMap was empty" / "No ping site configured".
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[CNC]\x1b[0m registerDynamicDedicatedServerCreator (pool creator registered; PreAuthResponse)"
    );
    handle_util_preauth(payload)
}

pub fn handle_game_manager_unregister_dynamic_dedicated_server_creator(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[CNC]\x1b[0m unregisterDynamicDedicatedServerCreator"
    );
    Ok(Bytes::from(Vec::new()))
}

fn log_gmgr_payload_hex(label: &str, payload: &[u8]) {
    if payload.is_empty() {
        crate::debug_println!("[CNC] {} payload: (empty)", label);
        return;
    }
    let hex: String = payload
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");
    crate::debug_println!(
        "[CNC] {} payload ({} bytes): {}",
        label,
        payload.len(),
        hex
    );
}

pub fn handle_game_manager_command_113(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    Ok(Bytes::from(Vec::new()))
}

pub fn build_user_sessions_user_updated_notification() -> BlazeResult<Bytes> {
    let (uid_u, _) = cnc_effective_identity();
    let uid = uid_u as i64;
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_int("FLGS", 3));
    response.extend_from_slice(&TdfEncoder::encode_long("ID  ", uid));
    Ok(Bytes::from(response))
}

pub fn build_user_sessions_user_authenticated_notification() -> BlazeResult<Bytes> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let (uid_u, display_name) = cnc_effective_identity();
    let uid = uid_u as i64;
    let is_dedicated = crate::session::session_module::current_blaze_session_id()
        .map(|sid| dedicated_pool::dedicated_identity_for_session(sid).is_some())
        .unwrap_or(false);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_int("ALOC", now as i32));
    response.extend_from_slice(&TdfEncoder::encode_long("BUID", uid));
    response.extend_from_slice(&TdfEncoder::encode_string("DSNM", &display_name));
    response.extend_from_slice(&TdfEncoder::encode_bool("FRST", false));
    response.extend_from_slice(&TdfEncoder::encode_string("KEY ", "SESSKY"));
    response.extend_from_slice(&TdfEncoder::encode_int("LAST", now as i32));
    response.extend_from_slice(&TdfEncoder::encode_long("LLOG", now));
    let mail = if is_dedicated { String::new() } else { get_user_session().email };
    response.extend_from_slice(&TdfEncoder::encode_string("MAIL", &mail));
    response.extend_from_slice(&TdfEncoder::encode_long("PID ", uid));
    response.extend_from_slice(&TdfEncoder::encode_int("PLAT", 4));
    response.extend_from_slice(&TdfEncoder::encode_long("UID ", uid));
    response.extend_from_slice(&TdfEncoder::encode_long("XREF", 0));
    Ok(Bytes::from(response))
}

pub fn build_user_sessions_user_added_notification() -> BlazeResult<Bytes> {
    let (uid_u, display_name) = cnc_effective_identity();
    let uid = uid_u as i64;

    let mut response = Vec::new();
    let data = encode_union_struct("ADDR", 2, |valu| {
        let mut exip = Vec::new();
        exip.extend_from_slice(&TdfEncoder::encode_int("IP  ", 0));
        exip.extend_from_slice(&TdfEncoder::encode_int("PORT", 0));
        valu.extend_from_slice(&TdfEncoder::encode_struct("EXIP", &exip));

        let mut inip = Vec::new();
        inip.extend_from_slice(&TdfEncoder::encode_int("IP  ", 0));
        inip.extend_from_slice(&TdfEncoder::encode_int("PORT", 0));
        valu.extend_from_slice(&TdfEncoder::encode_struct("INIP", &inip));
    });
    let mut data_struct = data.to_vec();
    data_struct.extend_from_slice(&TdfEncoder::encode_string("BPS ", ""));
    data_struct.extend_from_slice(&TdfEncoder::encode_string("CTY ", ""));
    data_struct.extend_from_slice(&TdfEncoder::encode_int("HWFG", 0));
    let mut qdat = Vec::new();
    qdat.extend_from_slice(&TdfEncoder::encode_int("DBPS", 0));
    qdat.extend_from_slice(&TdfEncoder::encode_int("NATT", 0));
    qdat.extend_from_slice(&TdfEncoder::encode_int("UBPS", 0));
    data_struct.extend_from_slice(&TdfEncoder::encode_struct("QDAT", &qdat));
    data_struct.extend_from_slice(&TdfEncoder::encode_long("UATT", 0));
    data_struct.extend_from_slice(&encode_struct_list("ULST", &[]));
    response.extend_from_slice(&TdfEncoder::encode_struct("DATA", &data_struct));

    let mut user = Vec::new();
    user.extend_from_slice(&TdfEncoder::encode_long("AID ", uid));
    user.extend_from_slice(&TdfEncoder::encode_int("ALOC", 0));
    user.extend_from_slice(&TdfEncoder::encode_long("EXID", uid));
    user.extend_from_slice(&TdfEncoder::encode_long("ID  ", uid));
    user.extend_from_slice(&TdfEncoder::encode_string("NAME", &display_name));
    user.extend_from_slice(&TdfEncoder::encode_long("ORIG", 0));
    response.extend_from_slice(&TdfEncoder::encode_struct("USER", &user));
    Ok(Bytes::from(response))
}

//   NEW_STATE=0, INITIALIZING=1, INACTIVE_VIRTUAL=2, PRE_GAME=130(0x82), IN_GAME=131(0x83),
//   POST_GAME=4, RESETABLE=7. NOTE: PRE_GAME/IN_GAME are NOT 1/2 -- 1 is INITIALIZING, 2 is
//   INACTIVE_VIRTUAL. Sending 1/2 leaves the game stuck in INITIALIZING and it never starts.
#[allow(dead_code)]
pub(crate) const GSTA_INITIALIZING: i32 = 1;
#[allow(dead_code)]
pub(crate) const GSTA_PRE_GAME: i32 = 130;
pub(crate) const GSTA_IN_GAME: i32 = 131;
#[allow(dead_code)]
const GSTA_POST_GAME: i32 = 4;
pub(crate) const GSTA_RESETABLE: i32 = 7;

/// `UUID` for `NotifyGameSetup`: use `CreateGameRequest` when present, else a fresh v4 string.
fn cnc_resolve_notify_game_uuid(request_payload: &[u8]) -> String {
    game_state::resolve_game_uuid(request_payload)
}

/// GameManager `NotifyGameStateChange` (`0x0004` / `0x64`): root `GID\0` + `GSTA` (BFP4FToolsWV / CNC launcher).
pub fn build_game_manager_notify_game_state_change(gid: i64, gsta: i32) -> BlazeResult<Bytes> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_int("GID\0", gid as i32));
    out.extend_from_slice(&TdfEncoder::encode_int("GSTA", gsta));
    Ok(Bytes::from(out))
}

pub const PLAYER_REMOVED_REASON_GAME_DESTROYED: i32 = 4;
pub const PLAYER_REMOVED_REASON_PLAYER_LEFT: i32 = 6;
pub const PLAYER_REMOVED_REASON_PLAYER_KICKED: i32 = 8;

pub fn build_game_manager_notify_player_removed(
    gid: i64,
    pid: i64,
    reason: i32,
) -> BlazeResult<Bytes> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_long("PID ", pid));
    out.extend_from_slice(&TdfEncoder::encode_int("REAS", reason));
    Ok(Bytes::from(out))
}

/// GameManager `NotifyGameAttribChange` (`0x0004` / cmd **80** / `0x50`).
pub fn build_game_manager_notify_game_attrib_change(
    gid: i64,
    attrs: &indexmap::IndexMap<String, String>,
) -> BlazeResult<Bytes> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("ATTR", attrs));
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    Ok(Bytes::from(out))
}

/// ACTIVE_CONNECTED=4. After the joining client reports its mesh connection (updateMeshConnection),
/// the server must flip it to ACTIVE_CONNECTED so `createGameNetworkCb` fires and the game proceeds.
pub const PLAYER_STATE_ACTIVE_CONNECTED: i32 = 4;

/// `GameManager.NotifyGamePlayerStateChange` (notification id **116**): tells the client a player's
/// connection state changed (`GID` + `PID` + `STAT`). Fields in alphabetical tag order.
pub fn build_game_manager_notify_game_player_state_change(
    gid: i64,
    pid: i64,
    state: i32,
) -> BlazeResult<Bytes> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_long("PID ", pid));
    out.extend_from_slice(&TdfEncoder::encode_int("STAT", state));
    Ok(Bytes::from(out))
}

/// Notify pooled `cnc.server.exe` to run `resetDedicatedServer` (cmd 220 / `NotifyCreateDynamicDedicatedServerGame`).
pub fn build_notify_create_dynamic_dedicated_server_game(
    gid: i64,
    create_request: &[u8],
) -> BlazeResult<Bytes> {
    let greq = inject_level_attr(gid, create_request).unwrap_or_else(|| create_request.to_vec());

    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_struct("GREQ", &greq));
    Ok(Bytes::from(out))
}

fn inject_level_attr(gid: i64, create_request: &[u8]) -> Option<Vec<u8>> {
    let map_path = game_state::get_map_path(gid);
    if map_path.is_empty() {
        return None;
    }

    let (_, _, start, total_len) = TdfEncoder::scan_root_level_fields(create_request)
        .into_iter()
        .find(|(tag, ty, _, _)| tag == "ATTR" && *ty == 0x05)?;
    let end = start.checked_add(total_len)?;
    if total_len < 4 || end > create_request.len() {
        return None;
    }

    let mut pairs =
        TdfEncoder::decode_string_string_map_untagged(&create_request[start + 4..end]).ok()?;
    if pairs.get("_level").map(String::as_str) == Some(map_path.as_str()) {
        return None; // already correct -- keep the echo verbatim
    }
    let prev = pairs
        .get("_level")
        .cloned()
        .unwrap_or_else(|| "<none>".into());
    tracing::info!(
        target: "cnc",
        "[CNC] inject _level gid={} \"{}\" ? \"{}\"",
        gid,
        prev,
        map_path
    );
    pairs.insert("_level".to_string(), map_path);

    let mut out = Vec::with_capacity(create_request.len() + 64);
    out.extend_from_slice(&create_request[..start]);
    out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("ATTR", &pairs));
    out.extend_from_slice(&create_request[end..]);
    Some(out)
}

fn extract_attr_level(create_request: &[u8]) -> Option<String> {
    let (_, _, start, total_len) = TdfEncoder::scan_root_level_fields(create_request)
        .into_iter()
        .find(|(tag, ty, _, _)| tag == "ATTR" && *ty == 0x05)?;
    let end = start.checked_add(total_len)?;
    if total_len < 4 || end > create_request.len() {
        return None;
    }
    let pairs =
        TdfEncoder::decode_string_string_map_untagged(&create_request[start + 4..end]).ok()?;
    pairs.get("_level").filter(|s| !s.is_empty()).cloned()
}

/// Blaze persona id used as host in CNC GameManager notifies (`ADMN`, `PROS`, **`PHID`**, etc.).
fn cnc_notify_host_persona_i32() -> i32 {
    let session = crate::session::get_user_session();
    let id = if session.persona_id == 0 {
        1000u64
    } else {
        session.persona_id
    };
    id.min(i32::MAX as u64) as i32
}

/// GameManager `NotifyGameSetup` (`0x0004` / `0x14`): pushed after successful reset/create so the client wires the game into `mGameMap`.
/// **`GAME.HNET`**: copied from the request only when it is already a root **`LIST`** of **`STRUCT`** rows
/// (`0x04` / item `0x03`); otherwise encoded like stock **`GameSetup`**: list of struct rows (**`EXIP`** / **`INIP`**).
pub fn build_game_manager_notify_game_setup(
    request_payload: &[u8],
    gid: i64,
) -> BlazeResult<Bytes> {
    let session = crate::session::get_user_session();
    let uid_i32 = cnc_notify_host_persona_i32();
    let uid = uid_i32 as i64;
    let dedicated = dedicated_pool::host_for_gid(gid);
    let topology_persona = dedicated.map(|d| d.persona_id).unwrap_or(uid);
    let _display_name = if session.display_name.is_empty() {
        "Player"
    } else {
        session.display_name.as_str()
    };

    // Echo create-request **`GNAM` / ATTR / VOIP / UUID`; **`GAME`** skeleton matches **`notify_game_setup_join`**.
    let gnam = TdfEncoder::find_string_field(request_payload, "GNAM")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Skirmish".to_string());
    let voip = TdfEncoder::find_int_field(request_payload, "VOIP").unwrap_or(0);
    // CNC `GameBase` / `NotifyGameSetup` uses the same topology as `resetDedicatedServer`: dedicated, not peer-hosted.
    let ntop_game = NTOP_CLIENT_SERVER_DEDICATED;
    let game_uuid = cnc_resolve_notify_game_uuid(request_payload);

    // HNET endpoints: pooled dedicated INIP when assigned; else `CreateGameRequest` / QoS / session EXIP.
    let ips = TdfEncoder::scan_all_int_fields(request_payload, "IP  ");
    let ports = TdfEncoder::scan_all_int_fields(request_payload, "PORT");
    let host_inip_ip = dedicated
        .map(|d| d.inip_ip)
        .filter(|&ip| ip != 0)
        .or_else(|| ips.get(1).copied())
        .unwrap_or(0);
    let host_inip_port_from_request = dedicated
        .map(|d| d.inip_port)
        .filter(|&p| p != 0)
        .or_else(|| ports.get(1).copied())
        .unwrap_or(0);
    let req_exip_ip = ips.first().copied().unwrap_or(0);
    let req_exip_port = ports.first().copied().unwrap_or(0);

    let host_exip_ip = dedicated
        .map(|d| d.exip_ip)
        .filter(|&ip| ip != 0)
        .or_else(|| {
            session
                .network_exip_ip
                .map(|u| u as i32)
                .filter(|&ip| ip != 0)
        })
        .or_else(|| {
            crate::session::peek_qos_observed_exip_ip()
                .map(|u| u as i32)
                .filter(|&ip| ip != 0)
        })
        .or_else(|| req_exip_ip.ne(&0).then_some(req_exip_ip))
        .unwrap_or(0);

    let host_inip_port = if host_inip_port_from_request != 0 {
        host_inip_port_from_request
    } else {
        CNC_TEST_DEDICATED_PORT
    };
    let host_exip_port = dedicated
        .map(|d| d.exip_port)
        .filter(|&p| p != 0)
        .or_else(|| req_exip_port.ne(&0).then_some(req_exip_port))
        .unwrap_or(host_inip_port);

    let gid_i32 = gid.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let _ = gid_i32;

    let build_endpoint = |ip: i32, port: i32| -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
        out.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
        out
    };

    // Derived from the dedicated address.
    let serverid = {
        let sid_ip = (if host_exip_ip != 0 { host_exip_ip } else { host_inip_ip }) as u32;
        if sid_ip != 0 {
            format!(
                "{}.{}.{}.{}",
                (sid_ip >> 24) & 0xFF,
                (sid_ip >> 16) & 0xFF,
                (sid_ip >> 8) & 0xFF,
                sid_ip & 0xFF
            )
        } else {
            "127.0.0.1".to_string()
        }
    };
    let mut attr_map =
        TdfEncoder::find_string_string_map_field(request_payload, "ATTR").unwrap_or_default();
    attr_map.insert("serverid".to_string(), serverid.clone());
    // Carry the lobby's chosen level in game attributes (informational for the client, which loads it
    // from local RtsSettings; authoritative for the dedicated spawn -- see build_dedicated_host_...).
    let map_path = crate::client::cnc::game_state::get_map_path(gid);
    if !map_path.is_empty() {
        attr_map.insert("_level".to_string(), map_path);
    }
    crate::client::cnc::game_state::apply_password_flag_to_attrs(gid, &mut attr_map);
    let mut dstr_map = indexmap::IndexMap::new();
    dstr_map.insert("serverid".to_string(), serverid.clone());
    crate::client::cnc::game_state::apply_password_secret_to_attrs(gid, &mut dstr_map);
    let mut matr_map = indexmap::IndexMap::new();
    matr_map.insert("serverid".to_string(), serverid);

    // HPID+HSLT host struct used by DHST / PHST / THST.
    let host_hpid = |persona: i64| -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(&TdfEncoder::encode_long("HPID", persona));
        h.extend_from_slice(&TdfEncoder::encode_int("HSLT", 0));
        h
    };

    // CRITICAL: Blaze heat2 requires fields in ASCENDING packed-tag order; the decoder silently skips
    // any field (and everything after it) that arrives out of order. Emit strictly in this order:
    // ADMN ATTR CAP CRIT DHST DSTR GID GNAM GSET GSID GSTA HNET NTOP PHST THST UUID VOIP XNNC XSES.
    let mut game = Vec::new();
    game.extend_from_slice(&TdfEncoder::encode_long_list("ADMN", &[uid]));
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("ATTR", &attr_map));
    game.extend_from_slice(&TdfEncoder::encode_long_list("CAP ", &[0x20, 0]));
    if let Some(raw) = TdfEncoder::extract_top_level_field_bytes(request_payload, "CRIT") {
        game.extend_from_slice(&raw);
    }
    game.extend_from_slice(&TdfEncoder::encode_struct("DHST", &host_hpid(topology_persona)));
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("DSTR", &dstr_map));
    game.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    game.extend_from_slice(&TdfEncoder::encode_string("GNAM", &gnam));
    if let Some(gset) = TdfEncoder::scan_all_int_fields(request_payload, "GSET")
        .first()
        .copied()
        .or_else(|| TdfEncoder::find_int_field(request_payload, "GSET"))
    {
        game.extend_from_slice(&TdfEncoder::encode_int("GSET", gset));
    }
    if let Some(d) = dedicated {
        game.extend_from_slice(&TdfEncoder::encode_long("GSID", d.blaze_session_id as i64));
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[CNC]\x1b[0m NotifyGameSetup gid={}: no pooled dedicated assignment (GSID omitted)",
            gid
        );
    }
    // Initial setup state = INITIALIZING (drives preInitGameNetwork -> mesh -> finalizeGameCreation,
    // then PRE_GAME / IN_GAME via advanceGameState).
    game.extend_from_slice(&TdfEncoder::encode_int("GSTA", GSTA_INITIALIZING));

    let mut hnet_row = Vec::new();
    hnet_row.extend_from_slice(&TdfEncoder::encode_struct(
        "EXIP",
        &build_endpoint(host_exip_ip, host_exip_port),
    ));
    hnet_row.extend_from_slice(&TdfEncoder::encode_struct(
        "INIP",
        &build_endpoint(host_inip_ip, host_inip_port),
    ));
    game.extend_from_slice(&encode_union_list("HNET", HNET_UNION_MEMBER_VALU, &[hnet_row]));
    // MATR (0xb61d32) sorts between HNET and NTOP.
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("MATR", &matr_map));
    game.extend_from_slice(&TdfEncoder::encode_int("NTOP", ntop_game));
    game.extend_from_slice(&TdfEncoder::encode_struct("PHST", &host_hpid(topology_persona)));
    game.extend_from_slice(&TdfEncoder::encode_struct("THST", &host_hpid(topology_persona)));
    game.extend_from_slice(&TdfEncoder::encode_string("UUID", &game_uuid));
    game.extend_from_slice(&TdfEncoder::encode_int("VOIP", voip));
    game.extend_from_slice(&TdfEncoder::encode_binary("XNNC", &[]));
    game.extend_from_slice(&TdfEncoder::encode_binary("XSES", &[]));

    game_state::set_replicated_wire_fields(gid, game.clone());

    let pros = game_state::pros_entries_for_gid(gid);
    game_state::set_pros_wire_fields(gid, pros.clone());

    let reas = encode_reas_reset_dedicated();

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_struct("GAME", &game));
    response.extend_from_slice(&encode_struct_list("PROS", &pros));
    response.extend_from_slice(&encode_struct_list("QUEU", &[]));
    response.extend_from_slice(&reas);
    Ok(Bytes::from(response))
}

/// Dedicated-host `NotifyGameSetup`: THST/PHST/ADMN = dedicated persona, HNET = dedicated bind, empty roster.
#[allow(clippy::too_many_arguments)]
pub fn build_dedicated_host_notify_game_setup(
    gid: i64,
    host_persona: i64,
    inip_ip: i32,
    inip_port: i32,
    exip_ip: i32,
    exip_port: i32,
    ded_session_id: u64,
    request_payload: &[u8],
) -> BlazeResult<Bytes> {
    let gnam = TdfEncoder::find_string_field(request_payload, "GNAM")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Skirmish".to_string());
    let voip = TdfEncoder::find_int_field(request_payload, "VOIP").unwrap_or(0);
    let game_uuid = cnc_resolve_notify_game_uuid(request_payload);

    let build_endpoint = |ip: i32, port: i32| -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
        out.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
        out
    };

    let host_hpid = |persona: i64| -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(&TdfEncoder::encode_long("HPID", persona));
        h.extend_from_slice(&TdfEncoder::encode_int("HSLT", 0));
        h
    };
    // Blaze packed-tag order: ADMN ATTR CAP DHST DSTR GID GNAM GSET GSID GSTA HNET MATR NTOP
    // PHST THST UUID VOIP XNNC XSES.
    //
    let serverid = {
        let sid_ip = (if exip_ip != 0 { exip_ip } else { inip_ip }) as u32;
        if sid_ip != 0 {
            format!(
                "{}.{}.{}.{}",
                (sid_ip >> 24) & 0xFF,
                (sid_ip >> 16) & 0xFF,
                (sid_ip >> 8) & 0xFF,
                sid_ip & 0xFF
            )
        } else {
            "127.0.0.1".to_string()
        }
    };
    let mut game = Vec::new();
    game.extend_from_slice(&TdfEncoder::encode_long_list("ADMN", &[host_persona]));
    // ATTR `_level` carries the lobby map when Blaze createGame omits it on the wire.
    let mut ded_attr =
        TdfEncoder::find_string_string_map_field(request_payload, "ATTR").unwrap_or_default();
    let ded_map_path = crate::client::cnc::game_state::get_map_path(gid);
    if !ded_map_path.is_empty() {
        ded_attr.insert("_level".to_string(), ded_map_path);
    }
    ded_attr.insert("serverid".to_string(), serverid.clone());
    crate::client::cnc::game_state::apply_password_secret_to_attrs(gid, &mut ded_attr);
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("ATTR", &ded_attr));
    game.extend_from_slice(&TdfEncoder::encode_long_list("CAP ", &[0x20, 0]));
    game.extend_from_slice(&TdfEncoder::encode_struct("DHST", &host_hpid(host_persona)));
    let mut dstr_map = indexmap::IndexMap::new();
    dstr_map.insert("serverid".to_string(), serverid.clone());
    crate::client::cnc::game_state::apply_password_secret_to_attrs(gid, &mut dstr_map);
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("DSTR", &dstr_map));
    game.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    game.extend_from_slice(&TdfEncoder::encode_string("GNAM", &gnam));
    if let Some(gset) = TdfEncoder::scan_all_int_fields(request_payload, "GSET")
        .first()
        .copied()
        .or_else(|| TdfEncoder::find_int_field(request_payload, "GSET"))
    {
        game.extend_from_slice(&TdfEncoder::encode_int("GSET", gset));
    }
    game.extend_from_slice(&TdfEncoder::encode_long("GSID", ded_session_id as i64));
    game.extend_from_slice(&TdfEncoder::encode_int("GSTA", GSTA_INITIALIZING));

    let mut hnet_row = Vec::new();
    hnet_row.extend_from_slice(&TdfEncoder::encode_struct(
        "EXIP",
        &build_endpoint(exip_ip, exip_port),
    ));
    hnet_row.extend_from_slice(&TdfEncoder::encode_struct(
        "INIP",
        &build_endpoint(inip_ip, inip_port),
    ));
    game.extend_from_slice(&encode_union_list("HNET", HNET_UNION_MEMBER_VALU, &[hnet_row]));

    let mut matr_map = indexmap::IndexMap::new();
    matr_map.insert("serverid".to_string(), serverid);
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("MATR", &matr_map));
    game.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_CLIENT_SERVER_DEDICATED));
    game.extend_from_slice(&TdfEncoder::encode_struct("PHST", &host_hpid(host_persona)));
    game.extend_from_slice(&TdfEncoder::encode_struct("THST", &host_hpid(host_persona)));
    game.extend_from_slice(&TdfEncoder::encode_string("UUID", &game_uuid));
    game.extend_from_slice(&TdfEncoder::encode_int("VOIP", voip));
    game.extend_from_slice(&TdfEncoder::encode_binary("XNNC", &[]));
    game.extend_from_slice(&TdfEncoder::encode_binary("XSES", &[]));

    // REAS member 0, DCTX=4 (HOST_INJECTION_SETUP_CONTEXT).
    let reas = encode_reas_dataless(4);

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_struct("GAME", &game));
    // Empty roster: dedicated is host, not a player.
    response.extend_from_slice(&encode_struct_list("PROS", &[]));
    response.extend_from_slice(&encode_struct_list("QUEU", &[]));
    response.extend_from_slice(&reas);
    Ok(Bytes::from(response))
}

/// Flat `ReplicatedGameData` field blob (no `GAME` struct wrapper).
fn build_replicated_game_data_fields(gid: i64) -> Vec<u8> {
    game_state::replicated_wire_fields(gid).unwrap_or_else(|| build_replicated_game_data_fields_fallback(gid))
}

fn build_replicated_game_data_fields_fallback(gid: i64) -> Vec<u8> {
    let session = crate::session::get_user_session();
    let uid_i32 = cnc_notify_host_persona_i32();
    let uid = uid_i32 as i64;
    let dedicated = dedicated_pool::host_for_gid(gid);

    let host_inip_ip = dedicated
        .map(|d| d.inip_ip)
        .filter(|&ip| ip != 0)
        .or_else(|| session.network_inip_ip.map(|u| u as i32))
        .unwrap_or(0);
    let host_inip_port = dedicated
        .map(|d| d.inip_port)
        .filter(|&p| p != 0)
        .or_else(|| session.network_inip_port.map(|u| u as i32).filter(|&p| p != 0))
        .unwrap_or(CNC_TEST_DEDICATED_PORT);
    let host_exip_ip = dedicated
        .map(|d| d.exip_ip)
        .filter(|&ip| ip != 0)
        .or_else(|| session.network_exip_ip.map(|u| u as i32))
        .unwrap_or(0);
    let host_exip_port = dedicated
        .map(|d| d.exip_port)
        .filter(|&p| p != 0)
        .or_else(|| session.network_exip_port.map(|u| u as i32).filter(|&p| p != 0))
        .unwrap_or(host_inip_port);
    let build_endpoint = |ip: i32, port: i32| -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
        out.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
        out
    };

    let gnam = game_state::game_name(gid);
    let game_uuid = game_state::game_uuid(gid);

    // (and DSTR for dedicated-server attribs). Keep ATTR too for general game-attribute consumers.
    // See build_game_manager_notify_game_setup / build_dedicated_host_notify_game_setup.
    let serverid = {
        let sid_ip = (if host_exip_ip != 0 { host_exip_ip } else { host_inip_ip }) as u32;
        if sid_ip != 0 {
            format!(
                "{}.{}.{}.{}",
                (sid_ip >> 24) & 0xFF,
                (sid_ip >> 16) & 0xFF,
                (sid_ip >> 8) & 0xFF,
                sid_ip & 0xFF
            )
        } else {
            "127.0.0.1".to_string()
        }
    };
    let mut attr = indexmap::IndexMap::new();
    attr.insert("PingSiteAlias".to_string(), "False".to_string());
    attr.insert("serverid".to_string(), serverid.clone());
    let mut dstr = indexmap::IndexMap::new();
    dstr.insert("serverid".to_string(), serverid.clone());
    let mut matr = indexmap::IndexMap::new();
    matr.insert("serverid".to_string(), serverid);

    let host_hpid = |persona: i64| -> Vec<u8> {
        let mut h = Vec::new();
        h.extend_from_slice(&TdfEncoder::encode_long("HPID", persona));
        h.extend_from_slice(&TdfEncoder::encode_int("HSLT", 0));
        h
    };
    // Ascending Blaze packed-tag order (heat2 decoder skips out-of-order fields):
    // ADMN ATTR CAP DHST DSTR GID GNAM GSTA HNET NTOP PHST THST UUID VOIP.
    let mut game = Vec::new();
    game.extend_from_slice(&TdfEncoder::encode_long_list("ADMN", &[uid]));
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("ATTR", &attr));
    game.extend_from_slice(&TdfEncoder::encode_long_list("CAP ", &[0x20, 0]));
    game.extend_from_slice(&TdfEncoder::encode_struct("DHST", &host_hpid(uid)));
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("DSTR", &dstr));
    game.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    game.extend_from_slice(&TdfEncoder::encode_string("GNAM", &gnam));
    game.extend_from_slice(&TdfEncoder::encode_int("GSTA", GSTA_RESETABLE));
    let mut hnet_row = Vec::new();
    hnet_row.extend_from_slice(&TdfEncoder::encode_struct(
        "EXIP",
        &build_endpoint(host_exip_ip, host_exip_port),
    ));
    hnet_row.extend_from_slice(&TdfEncoder::encode_struct(
        "INIP",
        &build_endpoint(host_inip_ip, host_inip_port),
    ));
    game.extend_from_slice(&encode_union_list("HNET", HNET_UNION_MEMBER_VALU, &[hnet_row]));
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("MATR", &matr));
    game.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_CLIENT_SERVER_DEDICATED));
    game.extend_from_slice(&TdfEncoder::encode_struct("PHST", &host_hpid(uid)));
    game.extend_from_slice(&TdfEncoder::encode_struct("THST", &host_hpid(uid)));
    game.extend_from_slice(&TdfEncoder::encode_string("UUID", &game_uuid));
    game.extend_from_slice(&TdfEncoder::encode_int("VOIP", 0));
    game
}

/// One `ListGameData` row: nested `GAME` (`ReplicatedGameData`) + `PROS` roster.
fn build_list_game_data_entry(gid: i64) -> BlazeResult<Vec<u8>> {
    let game = build_replicated_game_data_fields(gid);
    let pros = game_state::gfgd_roster_entries_for_gid(gid);
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_struct("GAME", &game));
    out.extend_from_slice(&encode_struct_list("PROS", &pros));
    Ok(out)
}

/// `NotifyGameSetup` body: nested `GAME` struct + `PROS` + `QUEU` (+ `REAS` added by caller).
pub fn build_game_manager_game_payload(gid: i64) -> BlazeResult<Bytes> {
    let game = build_replicated_game_data_fields(gid);
    game_state::set_replicated_wire_fields(gid, game.clone());
    let pros = game_state::pros_entries_for_gid(gid);
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_struct("GAME", &game));
    response.extend_from_slice(&encode_struct_list("PROS", &pros));
    response.extend_from_slice(&encode_struct_list("QUEU", &[]));
    Ok(Bytes::from(response))
}

/// Join-specific `NotifyGameSetup`: synthesize a stable dedicated-server payload.
/// We intentionally avoid copying arbitrary fields from `JoinGameRequest`/`JoinGameResponse`.
pub fn build_game_manager_notify_game_setup_join(gid: i64) -> BlazeResult<Bytes> {
    let mut response = build_game_manager_game_payload(gid)?.to_vec();
    response.extend_from_slice(&encode_reas_dataless_join());
    Ok(Bytes::from(response))
}

/// `Blaze::GameManager::NotifyPlatformHostInitialized` (component `0x0004`, command `0x47`).
/// Sent immediately after `NotifyGameSetup` so `GameManagerAPI` flips the platform-host state and
/// stops waiting for an injection notification on a peer-hosted game.
/// Wire: **`GID `**, **`HPID`** (long persona id), **`PHST`** (platform host slot id = 0).
/// Do not use **`PHID`** as INTEGER -- persona ids exceed single-byte varints and the client only consumes the first byte (`0`).
pub fn build_game_manager_notify_platform_host_initialized(gid: i64) -> BlazeResult<Bytes> {
    let gid = gid.clamp(i64::MIN, i64::MAX);
    let host = cnc_notify_host_persona_i32() as i64;
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    response.extend_from_slice(&TdfEncoder::encode_long("HPID", host));
    response.extend_from_slice(&TdfEncoder::encode_int("PHST", 0));
    Ok(Bytes::from(response))
}

/// `GameManager.NotifyPlayerJoinCompleted` (`0x0004` / `0x001E`) -- host join finished on dedicated reset.
pub fn build_game_manager_notify_player_join_completed(gid: i64) -> BlazeResult<Bytes> {
    game_state::mark_host_join_completed(gid);
    let player = game_state::host_player_for_gid(gid);
    Ok(Bytes::from(game_state::build_replicated_player(&player, gid)))
}

/// `GameManager.NotifyPlayerJoining` (`0x0004` / `0x0015`, cmd 21).
/// Sent to the dedicated HOST session so its GMGR adds the joining client to game `gid`'s roster.
/// same row schema as `NotifyGameSetup::PROS`). Without this the dedicated hosts the game but keeps
/// an EMPTY roster (`[GAME] dropping onNotifyPlayerCustomDataChanged for unknown local player`) and
pub fn build_game_manager_notify_player_joining(
    player: &game_state::CncPlayer,
    gid: i64,
) -> BlazeResult<Bytes> {
    let mut out = Vec::new();
    // TDF tag order: `GID ` (0x47..) before `PDAT` (0x50..).
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    let pdat = game_state::build_gfgd_pros_entry(player, gid);
    out.extend_from_slice(&TdfEncoder::encode_struct("PDAT", &pdat));
    Ok(Bytes::from(out))
}

/// `GameManager.NotifyJoiningPlayerInitiateConnections` (`0x0004` / `0x0016`).
/// Sent after `NotifyPlatformHostInitialized` so the client's `onNotifyJoiningPlayerInitiateConnections`
/// handler finds the game with its host address set and calls `preInitGameNetwork` / `fb_Blaze_connection_queue_incoming_fireframe`.
/// Payload layout matches `NotifyGameSetup`: `GAME` struct + `PROS` + `QUEU` + `REAS`.
pub fn build_game_manager_notify_joining_player_initiate_connections(gid: i64) -> BlazeResult<Bytes> {
    let mut response = build_game_manager_game_payload(gid)?.to_vec();
    response.extend_from_slice(&encode_reas_dataless_join());
    Ok(Bytes::from(response))
}

/// Emit `REAS = UNION{ DATALESS_CONTEXT(0): DCTX=JOIN }` for join-game notify path.
fn encode_reas_dataless_join() -> Bytes {
    encode_reas_dataless(1)
}

/// Emit `REAS = UNION{ member 0 = DatalessSetupContext: VALU{ DCTX } }`
fn encode_reas_dataless(dctx: i32) -> Bytes {
    encode_union_struct("REAS", 0, |body| {
        body.extend_from_slice(&TdfEncoder::encode_int("DCTX", dctx));
    })
}

/// Emit `REAS = UNION{ member 1 = ResetDedicatedServerSetupContext: VALU{ ERR=0 } }` (client reset path).
/// Dedicated host notify uses [`encode_reas_dataless`] with DCTX=4 instead.
fn encode_reas_reset_dedicated() -> Bytes {
    encode_union_struct("REAS", 1, |body| {
        body.extend_from_slice(&TdfEncoder::encode_int("ERR ", 0));
    })
}

fn encode_union_struct(
    union_tag: &str,
    member_index: u64,
    build_value_struct: impl FnOnce(&mut Vec<u8>),
) -> Bytes {
    let mut out = Vec::new();
    let tag = TdfEncoder::make_tag(union_tag);
    out.push(tag[0]);
    out.push(tag[1]);
    out.push(tag[2]);
    out.push(0x06);
    out.extend_from_slice(&TdfEncoder::encode_varint(member_index));

    let mut value_struct = Vec::new();
    build_value_struct(&mut value_struct);
    // Blaze union wire uses VALU for the active member payload.
    out.extend_from_slice(&TdfEncoder::encode_struct("VALU", &value_struct));
    Bytes::from(out)
}

fn encode_struct_list(tag: &str, structs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    let tag_encoded = TdfEncoder::make_tag(tag);
    out.push(tag_encoded[0]);
    out.push(tag_encoded[1]);
    out.push(tag_encoded[2]);
    out.push(0x4);
    out.push(0x3);
    out.extend_from_slice(&TdfEncoder::encode_varint(structs.len() as u64));
    for s in structs {
        out.extend_from_slice(s);
        out.push(0x00);
    }
    out
}

fn encode_union_list(tag: &str, member_byte: u8, structs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    let tag_encoded = TdfEncoder::make_tag(tag);
    out.push(tag_encoded[0]);
    out.push(tag_encoded[1]);
    out.push(tag_encoded[2]);
    out.push(0x4);
    out.push(0x3);
    out.extend_from_slice(&TdfEncoder::encode_varint(structs.len() as u64));
    for s in structs {
        out.push(member_byte);
        out.extend_from_slice(s);
        out.push(0x00);
    }
    out
}

const HNET_UNION_MEMBER_VALU: u8 = 0x02;

/// Legacy helper retained for tests only. CNC cmd `0x0070` is **`NotifyGameReset`**, not Starting --
/// do not push this on the live CNC client path (see `pushes_after_advance_game_state`).
#[cfg(test)]
pub fn build_game_manager_notify_game_starting(gid: i64) -> BlazeResult<Bytes> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_bool("STRT", true));
    Ok(Bytes::from(out))
}

/// `GameManager.NotifyGameSettingsChange` (`0x0004` / `0x006E` / cmd 110) -- sent after setGameSettings
/// to notify all players of updated game settings. Wire: GID (long) + GSET (int) + ATTR (map).
pub fn build_game_manager_notify_game_settings_change(gid: i64, gset: i32) -> BlazeResult<Bytes> {
    let game = game_state::get_game(gid);
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_int("GSET", gset));
    if let Some(ref g) = game {
        if !g.players.is_empty() {
            let host = g.host_persona;
            out.extend_from_slice(&TdfEncoder::encode_long("HOST", host));
        }
    }
    Ok(Bytes::from(out))
}

#[cfg(test)]
mod notify_game_setup_tests {
    use super::*;
    use crate::blaze::tdf::{TdfEncoder, TdfTreeParser};
    use indexmap::IndexMap;

    fn reset_test_games() {
        game_state::clear_all_games_for_test();
    }

    fn encode_union_list(tag: &str, items: &[Vec<u8>]) -> Vec<u8> {
        let tag_encoded = TdfEncoder::make_tag(tag);
        let mut out = Vec::new();
        out.push(tag_encoded[0]);
        out.push(tag_encoded[1]);
        out.push(tag_encoded[2]);
        out.push(0x04);
        out.push(0x06);
        out.extend_from_slice(&TdfEncoder::encode_varint(items.len() as u64));
        for item in items {
            out.extend_from_slice(item);
        }
        out
    }

    fn find_tag<'a>(nodes: &'a [crate::blaze::tdf::TdfTreeNode], want: &str) -> Option<&'a crate::blaze::tdf::TdfTreeNode> {
        for n in nodes {
            if n.tag == want {
                return Some(n);
            }
            if let Some(hit) = find_tag(&n.children, want) {
                return Some(hit);
            }
        }
        None
    }

    #[test]
    fn mesh_endpoints_connected_returns_empty_ok() {
        let mut req = Vec::new();
        req.extend_from_slice(&TdfEncoder::encode_int("GID ", 1));
        let rsp = handle_game_manager_mesh_endpoints_connected(&req).expect("handler");
        assert!(rsp.is_empty());
    }

    #[test]
    fn reset_join_game_response_leads_with_gid() {
        let rsp = super::cnc_join_game_response_with_ocal(1, Some(1));
        let gid_tag = TdfEncoder::make_tag("GID ");
        assert!(
            rsp.starts_with(&gid_tag),
            "JoinGameResponse must start with GID tag, got {:02x?}",
            &rsp[..4.min(rsp.len())]
        );
        assert_eq!(TdfEncoder::find_long_field(&rsp, "GID "), Some(1));
        assert_eq!(TdfEncoder::find_string_field(&rsp, "SRVR").as_deref(), Some("1"));
    }

    #[test]
    fn notify_setup_reas_parses_reset_dedicated_union() {
        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        let tree = TdfTreeParser::parse_packet(&payload).expect("parse");
        let reas = find_tag(&tree, "REAS").expect("REAS");
        assert!(
            reas.value_display.contains("1"),
            "REAS should decode active member 1, got {:?}",
            reas.value_display
        );
        assert!(
            find_tag(&reas.children, "ERR").is_some() || reas.value_display.contains("ERR"),
            "reset REAS should include ERR=0"
        );
        assert!(
            reas.value_display.contains("1"),
            "REAS reset body should be member 1 (ResetDedicatedServerSetupContext)"
        );
    }

    #[test]
    fn dedicated_host_notify_reas_is_host_injection_not_reset() {
        let payload = build_dedicated_host_notify_game_setup(
            700004,
            1000,
            0x7f000001,
            25200,
            0x7f000001,
            25200,
            42,
            &[],
        )
        .expect("encode");
        let tree = TdfTreeParser::parse_packet(&payload).expect("parse");
        let reas = find_tag(&tree, "REAS").expect("REAS");
        assert!(
            reas.value_display.contains("0") || reas.value_display.contains("dlsc"),
            "REAS should be dataless union member 0, got {:?}",
            reas.value_display
        );
        let dctx = find_tag(&reas.children, "DCTX").expect("DCTX");
        assert!(
            dctx.value_display.contains('4'),
            "dedicated host REAS DCTX must be 4 (HOST_INJECTION_SETUP_CONTEXT), got {:?}",
            dctx.value_display
        );
        let reas_tag = TdfEncoder::make_tag("REAS");
        let reset_needle: [u8; 5] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0x01];
        assert!(
            !payload.windows(reset_needle.len()).any(|w| w == reset_needle),
            "dedicated host REAS must not use reset member 1"
        );
    }

    #[test]
    fn set_player_custom_data_parses_captured_request_shape() {
        let wire: [u8; 15] = [
            0x9e, 0x99, 0x00, 0x00, 0x01, 0xe2, 0xeb, 0xa3, 0x02, 0x00, 0xe3, 0x39, 0x73, 0x02,
            0x00,
        ];
        let applied = game_state::apply_set_player_custom_data(&wire).expect("parse");
        assert_eq!(applied.0, 1);
        assert_eq!(applied.2.len(), 3);
        assert!(applied.2.contains_key("AuthToken"));
        assert!(applied.2.contains_key("XNNC"));
        assert!(applied.2.contains_key("XSES"));
        assert!(applied.2["XNNC"].is_empty());
        assert!(applied.2["XSES"].is_empty());
        let notify = game_state::build_notify_player_custom_data_change(
            applied.0,
            applied.1,
            &applied.2,
        );
        assert_eq!(TdfEncoder::find_long_field(&notify, "GID "), Some(1));
        let cdat = TdfEncoder::find_blob_field(&notify, "CDAT").expect("CDAT blob");
        assert_eq!(cdat, b"ABC123");
        assert!(TdfEncoder::find_string_string_map_field(&notify, "CDAT").is_none());
        let fields = TdfEncoder::scan_root_level_fields(&notify);
        let mut prev = 0u32;
        for (tag, type_byte, _, _) in &fields {
            let t = TdfEncoder::make_tag(tag.trim());
            let v = ((t[0] as u32) << 16) | ((t[1] as u32) << 8) | (t[2] as u32);
            assert!(v >= prev, "field {} out of packed-tag order", tag);
            if tag.trim() == "CDAT" {
                assert_eq!(*type_byte, 0x02, "CDAT must be BLOB");
            }
            prev = v;
        }
    }

    #[test]
    fn notify_player_custom_data_change_round_trips_auth_token() {
        let mut data = indexmap::IndexMap::new();
        data.insert("AuthToken".to_string(), b"ABC123".to_vec());
        let notify =
            game_state::build_notify_player_custom_data_change(1, 1_201_618_778, &data);
        let cdat = TdfEncoder::find_blob_field(&notify, "CDAT").expect("CDAT blob");
        assert_eq!(cdat, b"ABC123");
        assert!(TdfEncoder::find_string_string_map_field(&notify, "CDAT").is_none());
        assert_eq!(TdfEncoder::find_long_field(&notify, "GID "), Some(1));
        assert_eq!(
            TdfEncoder::find_long_field(&notify, "PID "),
            Some(1_201_618_778)
        );
        let tree = TdfTreeParser::parse_packet(&notify).expect("parse notify");
        let cdat_node = find_tag(&tree, "CDAT").expect("CDAT blob node");
        assert!(
            cdat_node.value_display.contains("ABC123")
                || cdat_node.value_display.to_lowercase().contains("blob")
                || !cdat_node.value_display.is_empty(),
            "CDAT blob should parse, got {:?}",
            cdat_node.value_display
        );
    }

    #[test]
    fn set_player_attributes_parses_captured_request_shape() {
        let wire: [u8; 36] = [
            0x87, 0x4d, 0x32, 0x05, 0x01, 0x01, 0x01, 0x09, 0x5f, 0x66, 0x61, 0x63, 0x74, 0x69,
            0x6f, 0x6e, 0x00, 0x04, 0x55, 0x53, 0x41, 0x00, 0x9e, 0x99, 0x00, 0x00, 0x01, 0xc2,
            0x99, 0x00, 0x00, 0x9a, 0xfd, 0xf9, 0xf9, 0x08,
        ];
        let applied = game_state::apply_set_player_attributes(&wire).expect("parse");
        assert_eq!(applied.0, 1);
        assert_eq!(applied.1, 1_201_618_778);
        assert_eq!(applied.2.get("_faction").map(String::as_str), Some("USA"));
    }

    #[test]
    fn notify_player_attrib_change_uses_packed_tag_order_with_authtoken() {
        let mut attrs = IndexMap::new();
        attrs.insert("AuthToken".to_string(), "ABC123".to_string());
        let notify =
            game_state::build_notify_player_attrib_change(1, 1_201_618_778, &attrs);
        let loaded = TdfEncoder::find_string_string_map_field(&notify, "ATTR").expect("ATTR");
        assert_eq!(loaded.get("AuthToken").map(String::as_str), Some("ABC123"));
        assert_eq!(TdfEncoder::find_long_field(&notify, "GID "), Some(1));
        assert_eq!(
            TdfEncoder::find_long_field(&notify, "PID "),
            Some(1_201_618_778)
        );
        let fields = TdfEncoder::scan_root_level_fields(&notify);
        let mut prev = 0u32;
        let mut tags = Vec::new();
        for (tag, _, _, _) in &fields {
            let t = TdfEncoder::make_tag(tag.trim());
            let v = ((t[0] as u32) << 16) | ((t[1] as u32) << 8) | (t[2] as u32);
            assert!(
                v >= prev,
                "field {} out of packed-tag order (saw {:06x} after {:06x})",
                tag,
                v,
                prev
            );
            prev = v;
            tags.push(tag.trim().to_string());
        }
        assert_eq!(tags.first().map(String::as_str), Some("ATTR"));
        assert!(notify[0] == 0x87 && notify[1] == 0x4d && notify[2] == 0x32);
    }

    #[test]
    fn notify_game_state_change_parses() {
        let payload = build_game_manager_notify_game_state_change(42, GSTA_RESETABLE).expect("encode");
        assert_eq!(TdfEncoder::find_int_field(&payload, "GID\0"), Some(42));
        assert_eq!(TdfEncoder::find_int_field(&payload, "GSTA"), Some(GSTA_RESETABLE));
        TdfTreeParser::parse_packet(&payload).expect("parse tree");
    }

    #[test]
    fn notify_player_removed_encodes_gid_pid_reas() {
        let payload = build_game_manager_notify_player_removed(
            1,
            1_201_618_778,
            PLAYER_REMOVED_REASON_PLAYER_LEFT,
        )
        .expect("encode");
        assert_eq!(TdfEncoder::find_long_field(&payload, "GID "), Some(1));
        assert_eq!(
            TdfEncoder::find_long_field(&payload, "PID "),
            Some(1_201_618_778)
        );
        assert_eq!(
            TdfEncoder::find_int_field(&payload, "REAS"),
            Some(PLAYER_REMOVED_REASON_PLAYER_LEFT)
        );
        assert_ne!(
            TdfEncoder::find_int_field(&payload, "REAS"),
            Some(PLAYER_REMOVED_REASON_PLAYER_KICKED),
            "PLAYER_KICKED triggers FrontEndTest on client"
        );
        TdfTreeParser::parse_packet(&payload).expect("parse tree");
    }

    #[test]
    fn notify_setup_nested_uuid_non_empty() {
        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        let u = TdfEncoder::find_string_field(&payload, "UUID").expect("UUID in GAME");
        assert!(u.len() >= 8 && u != ".", "{}", u);
    }

    #[test]
    fn notify_setup_hnet_before_xnnc_retail_field_order() {
        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        let hnet = TdfEncoder::make_tag("HNET");
        let xnnc = TdfEncoder::make_tag("XNNC");
        let hnet_pos = payload
            .windows(3)
            .position(|w| w == hnet)
            .expect("HNET tag in notify");
        let xnnc_pos = payload
            .windows(3)
            .position(|w| w == xnnc)
            .expect("XNNC tag in notify");
        assert!(
            hnet_pos < xnnc_pos,
            "retail ReplicatedGameData expects HNET@{hnet_pos} before XNNC@{xnnc_pos}"
        );
    }

    // Dedicated reset must use `rdsc` (member 1) so the client does not run create/finalize paths.
    #[test]
    fn notify_setup_reas_is_reset_dedicated_not_cancel_sentinel() {
        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        let reas_tag = TdfEncoder::make_tag("REAS");
        let cancel_needle: [u8; 6] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0xbf, 0x01];
        assert!(
            !payload.windows(cancel_needle.len()).any(|w| w == cancel_needle),
            "REAS must not carry union member 127 (INVALID_MEMBER) -- that is the cancel sentinel"
        );
        let rdsc_needle: [u8; 5] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0x01];
        assert!(
            payload.windows(rdsc_needle.len()).any(|w| w == rdsc_needle),
            "reset REAS must encode UNION member 1 (ResetDedicatedServerSetupContext)"
        );
        let dctx_needle: [u8; 5] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0x00];
        assert!(
            !payload.windows(dctx_needle.len()).any(|w| w == dctx_needle),
            "reset REAS must not use dataless DCTX=CREATE member 0"
        );
    }

    #[test]
    fn notify_setup_join_reas_is_dataless_not_cancel_sentinel() {
        let payload = build_game_manager_notify_game_setup_join(1).expect("encode");
        let reas_tag = TdfEncoder::make_tag("REAS");
        let cancel_needle: [u8; 6] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0xbf, 0x01];
        assert!(
            !payload.windows(cancel_needle.len()).any(|w| w == cancel_needle),
            "join REAS must not carry union member 127"
        );
        let join_needle: [u8; 5] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0x00];
        assert!(
            payload.windows(join_needle.len()).any(|w| w == join_needle),
            "join REAS must encode UNION member 0 (DATALESS_CONTEXT)"
        );
    }

    #[test]
    fn notify_create_dynamic_dedicated_server_game_encodes_gid_and_request() {
        let mut req = Vec::new();
        req.extend_from_slice(&TdfEncoder::encode_string("GNAM", "Skirmish"));
        req.extend_from_slice(&TdfEncoder::encode_int("GSET", 271));
        req.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_CLIENT_SERVER_DEDICATED));
        let payload =
            build_notify_create_dynamic_dedicated_server_game(42, &req).expect("encode");
        assert_eq!(TdfEncoder::find_long_field(&payload, "GID "), Some(42));
        assert!(TdfEncoder::find_string_field(&payload, "GNAM").as_deref() == Some("Skirmish"));
        let tree = TdfTreeParser::parse_packet(&payload).expect("parse");
        assert!(find_tag(&tree, "GREQ").is_some(), "cmd 220 must wrap request in GREQ");
    }

    #[test]
    fn notify_setup_echoes_gset_from_request() {
        let mut req = Vec::new();
        req.extend_from_slice(&TdfEncoder::encode_string("GNAM", "XEVRAC"));
        req.extend_from_slice(&TdfEncoder::encode_int("GSET", 271));
        assert_eq!(
            TdfEncoder::scan_all_int_fields(&req, "GSET").first().copied(),
            Some(271)
        );
        let payload = build_game_manager_notify_game_setup(&req, 1).expect("encode");
        assert_eq!(
            TdfEncoder::scan_all_int_fields(&payload, "GSET").first().copied(),
            Some(271)
        );
    }

    #[test]
    fn notify_setup_core_fields_decode() {
        let mut req = Vec::new();
        req.extend_from_slice(&TdfEncoder::encode_string("GNAM", "XEVRAC"));
        req.extend_from_slice(&TdfEncoder::encode_int("GSET", 271));
        let payload = build_game_manager_notify_game_setup(&req, 1).expect("encode");
        let tree = TdfTreeParser::parse_packet(&payload).expect("parse");
        assert!(find_tag(&tree, "GAME").is_some(), "GAME root missing");
        assert!(find_tag(&tree, "GNAM").is_some(), "GNAM missing from GAME");
        let mut needle = Vec::new();
        needle.extend_from_slice(&TdfEncoder::make_tag("GID "));
        needle.push(0x00);
        needle.extend_from_slice(&TdfEncoder::encode_varint(1u64));
        assert!(
            payload.windows(needle.len()).any(|w| w == needle.as_slice()),
            "nested GAME.GID must match JoinGameResponse: GID space + INTEGER + varint 1"
        );
    }

    #[test]
    fn notify_hnet_union_fallback_parses_in_tree() {
        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        let tree = TdfTreeParser::parse_packet(&payload).expect("parse");
        let hnet = find_tag(&tree, "HNET").expect("HNET field");
        assert!(!hnet.children.is_empty(), "HNET list empty");
    }

    /// serverid must ride in the INITIAL NotifyGameSetup (create/reset, join, and dedicated-host).
    #[test]
    fn notify_setup_carries_serverid_in_initial_attr() {
        reset_test_games();
        // TDF string-string map stores null-terminated key/value bytes; check both are present
        // and structurally valid (GAME struct parses) in create/reset, join, and dedicated-host.
        let has = |p: &[u8], needle: &[u8]| p.windows(needle.len()).any(|w| w == needle);
        let game_has_matr = |p: &[u8]| {
            let tree = TdfTreeParser::parse_packet(p).expect("parse");
            let game = find_tag(&tree, "GAME").expect("GAME");
            find_tag(&game.children, "MATR").is_some()
        };

        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        assert!(
            TdfTreeParser::parse_packet(&payload)
                .ok()
                .and_then(|t| find_tag(&t, "GAME").map(|_| ()))
                .is_some(),
            "create-path GAME struct must parse"
        );
        assert!(has(&payload, b"serverid\0"), "serverid key missing (create path)");
        assert!(has(&payload, b"127.0.0.1\0"), "serverid value missing (create path)");
        assert!(game_has_matr(&payload), "MATR missing (create path)");

        let jpayload = build_game_manager_notify_game_setup_join(1).expect("encode join");
        assert!(
            TdfTreeParser::parse_packet(&jpayload)
                .ok()
                .and_then(|t| find_tag(&t, "GAME").map(|_| ()))
                .is_some(),
            "join-path GAME struct must parse"
        );
        assert!(has(&jpayload, b"serverid\0"), "serverid key missing (join path)");
        assert!(has(&jpayload, b"127.0.0.1\0"), "serverid value missing (join path)");
        assert!(game_has_matr(&jpayload), "MATR missing (join path)");

        let dpayload = build_dedicated_host_notify_game_setup(
            700010,
            1000,
            0x7f000001,
            25200,
            0x7f000001,
            25200,
            42,
            &[],
        )
        .expect("encode dedicated host");
        assert!(
            TdfTreeParser::parse_packet(&dpayload)
                .ok()
                .and_then(|t| find_tag(&t, "GAME").map(|_| ()))
                .is_some(),
            "dedicated-host GAME struct must parse"
        );
        assert!(has(&dpayload, b"serverid\0"), "serverid key missing (dedicated host)");
        assert!(has(&dpayload, b"127.0.0.1\0"), "serverid value missing (dedicated host)");
        assert!(game_has_matr(&dpayload), "MATR missing (dedicated host) - Join needs mesh attributes");
    }

    /// AUDIT: recursively verify EVERY struct we emit (top-level + nested) lists its fields in
    /// strictly ascending packed-tag order -- the invariant the Blaze heat2 decoder relies on.
    #[test]
    fn all_response_packets_ascending_tag_order() {
        fn pack(tag: &str) -> u32 {
            let t = TdfEncoder::make_tag(tag);
            ((t[0] as u32) << 16) | ((t[1] as u32) << 8) | (t[2] as u32)
        }
        fn walk(nodes: &[crate::blaze::tdf::TdfTreeNode], enforce: bool, path: &str, fails: &mut Vec<String>) {
            if enforce {
                let mut prev = 0u32;
                let mut prev_tag = String::new();
                for n in nodes {
                    let v = pack(&n.tag);
                    if v < prev {
                        fails.push(format!(
                            "{path}: '{}' (0x{v:06x}) after '{prev_tag}' (0x{prev:06x})",
                            n.tag
                        ));
                    }
                    prev = v;
                    prev_tag = n.tag.clone();
                }
            }
            for n in nodes {
                // Struct children are tag-ordered fields; LIST/MAP/UNION children are items/entries.
                let child_enforce = n.value_type == "STRUCT";
                walk(&n.children, child_enforce, &format!("{path}/{}", n.tag), fails);
            }
        }

        reset_test_games();
        let mut fails: Vec<String> = Vec::new();
        let mut audit = |name: &str, r: BlazeResult<Bytes>| {
            if let Ok(bytes) = r {
                match TdfTreeParser::parse_packet(&bytes) {
                    Ok(tree) => walk(&tree, true, name, &mut fails),
                    Err(e) => fails.push(format!("{name}: parse error {e:?}")),
                }
            }
        };

        audit("get_server_instance", handle_redirector_get_server_instance(&[]));
        audit("preauth", handle_util_preauth(&[]));
        audit("post_auth", handle_util_post_auth(&[]));
        audit("telemetry", handle_util_get_telemetry_server(&[]));
        audit("login", handle_auth_login(&[]));
        audit("login_persona", handle_auth_login_persona(&[]));
        audit("update_network_info", handle_user_sessions_update_network_info(&[]));
        audit("user_added", build_user_sessions_user_added_notification());
        audit("user_authenticated", build_user_sessions_user_authenticated_notification());
        audit("reset_dedicated", handle_game_manager_reset_dedicated_server(&[]));
        audit("notify_game_setup", build_game_manager_notify_game_setup(&[], 1));
        audit("notify_game_setup_join", build_game_manager_notify_game_setup_join(1));
        audit("notify_initiate_connections", build_game_manager_notify_joining_player_initiate_connections(1));
        audit("notify_platform_host", build_game_manager_notify_platform_host_initialized(1));
        let joining_player = game_state::ensure_client_player(1, 1_201_618_778, "Player2")
            .expect("test game should accept joining player");
        audit(
            "notify_player_joining",
            build_game_manager_notify_player_joining(&joining_player, 1),
        );
        audit("notify_player_join_completed", build_game_manager_notify_player_join_completed(1));
        audit("notify_game_state_change", build_game_manager_notify_game_state_change(1, 130));
        audit("notify_game_player_state_change", build_game_manager_notify_game_player_state_change(1, 1201618778, 4));
        audit(
            "notify_player_removed",
            build_game_manager_notify_player_removed(1, 1201618778, PLAYER_REMOVED_REASON_PLAYER_LEFT),
        );
        audit(
            "notify_player_custom_data_change",
            Ok(Bytes::from(game_state::build_notify_player_custom_data_change(
                1,
                1_201_618_778,
                &indexmap::indexmap! {
                    "AuthToken".to_string() => game_state::auth_token_custom_data_blob(),
                },
            ))),
        );
        audit("get_full_game_data", handle_game_manager_get_full_game_data(&[]));
        audit(
            "dedicated_host_setup",
            build_dedicated_host_notify_game_setup(1, 1000, 0x7f000001, 25200, 0x7f000001, 25200, 42, &[]),
        );

        assert!(fails.is_empty(), "TDF field-order violations:\n{}", fails.join("\n"));
    }

    /// Blaze heat2 decoders skip out-of-order fields -- every GAME struct we emit MUST list its
    /// members in strictly ascending packed-tag order. Verify across all three GAME builders.
    #[test]
    fn game_struct_fields_ascending_tag_order() {
        // Unique gids + no reset_test_games(): avoid racing the shared game_state used by other tests.
        let pack = |tag: &str| -> u32 {
            let t = TdfEncoder::make_tag(tag);
            ((t[0] as u32) << 16) | ((t[1] as u32) << 8) | (t[2] as u32)
        };
        let check = |payload: &[u8], label: &str| {
            let tree = TdfTreeParser::parse_packet(payload).expect("parse");
            let game = find_tag(&tree, "GAME").expect("GAME struct");
            let mut prev = 0u32;
            let mut prev_tag = String::new();
            for child in &game.children {
                let v = pack(&child.tag);
                assert!(
                    v > prev,
                    "{label}: field '{}' (0x{v:06x}) is not after '{prev_tag}' (0x{prev:06x}) -- OUT OF ORDER",
                    child.tag
                );
                prev = v;
                prev_tag = child.tag.clone();
            }
        };
        check(&build_game_manager_notify_game_setup(&[], 700001).expect("create"), "notify_game_setup");
        check(&build_game_manager_notify_game_setup_join(700002).expect("join"), "join");
        check(
            &build_dedicated_host_notify_game_setup(700003, 1000, 0x7f000001, 25200, 0x7f000001, 25200, 42, &[])
                .expect("dedicated"),
            "dedicated_host",
        );
    }

    #[test]
    fn extract_hnet_after_other_root_fields() {
        let mut req = Vec::new();
        req.extend_from_slice(&TdfEncoder::encode_string("GNAM", "XEVRAC"));
        req.extend_from_slice(&TdfEncoder::encode_int("GSET", 271));
        let ep = |ip: i32, port: i32| {
            let mut v = Vec::new();
            v.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
            v.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
            v
        };
        let mut hnet_valu = Vec::new();
        hnet_valu.extend_from_slice(&TdfEncoder::encode_struct("EXIP", &ep(0, 0)));
        hnet_valu.extend_from_slice(&TdfEncoder::encode_struct("INIP", &ep(0x0a00_00e6, 3659)));
        let mut item = Vec::new();
        item.extend_from_slice(&TdfEncoder::encode_varint(2));
        item.extend_from_slice(&TdfEncoder::encode_struct("VALU", &hnet_valu));
        req.extend_from_slice(&encode_union_list("HNET", &[item]));

        let raw = TdfEncoder::extract_top_level_field_bytes(&req, "HNET").expect("HNET");
        assert_eq!(raw[3], 0x04, "HNET must be LIST");
        assert!(raw.len() > 12);
    }

    #[test]
    fn extract_hnet_after_attr_string_string_map() {
        let mut attr = IndexMap::new();
        attr.insert("PingSiteAlias".to_string(), "False".to_string());
        let mut req = Vec::new();
        req.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("ATTR", &attr));
        req.extend_from_slice(&TdfEncoder::encode_string("GNAM", "XEVRAC"));
        let ep = |ip: i32, port: i32| {
            let mut v = Vec::new();
            v.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
            v.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
            v
        };
        let mut hnet_valu = Vec::new();
        hnet_valu.extend_from_slice(&TdfEncoder::encode_struct("EXIP", &ep(0, 0)));
        hnet_valu.extend_from_slice(&TdfEncoder::encode_struct("INIP", &ep(0x0a00_00e6, 3659)));
        let mut item = Vec::new();
        item.extend_from_slice(&TdfEncoder::encode_varint(2));
        item.extend_from_slice(&TdfEncoder::encode_struct("VALU", &hnet_valu));
        req.extend_from_slice(&encode_union_list("HNET", &[item]));

        let raw = TdfEncoder::extract_top_level_field_bytes(&req, "HNET").expect("HNET after ATTR");
        assert_eq!(raw[3], 0x04);
    }

    #[test]
    fn notify_normalizes_union_request_hnet_to_struct_list() {
        let ep = |ip: i32, port: i32| {
            let mut v = Vec::new();
            v.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
            v.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
            v
        };
        let mut hnet_valu = Vec::new();
        hnet_valu.extend_from_slice(&TdfEncoder::encode_struct("EXIP", &ep(0, 0)));
        hnet_valu.extend_from_slice(&TdfEncoder::encode_struct("INIP", &ep(0x0a00_00e6, 3659)));
        let mut hnet_union_item = Vec::new();
        hnet_union_item.extend_from_slice(&TdfEncoder::encode_varint(2));
        hnet_union_item.extend_from_slice(&TdfEncoder::encode_struct("VALU", &hnet_valu));
        let req = encode_union_list("HNET", &[hnet_union_item]);

        let payload = build_game_manager_notify_game_setup(&req, 1).expect("encode");
        let tree = TdfTreeParser::parse_packet(&payload).expect("parse");
        let hnet = find_tag(&tree, "HNET").expect("HNET in GAME");
        assert!(!hnet.children.is_empty(), "HNET list empty");
        assert!(
            find_tag(&tree, "EXIP").is_some(),
            "union create request should yield struct-list HNET with EXIP"
        );
    }

    #[test]
    fn notify_platform_host_uses_hpid_long_not_phid_int() {
        let payload = build_game_manager_notify_platform_host_initialized(1).expect("notify");
        assert!(
            payload.windows(3).any(|w| w == TdfEncoder::make_tag("HPID")),
            "NotifyPlatformHostInitialized must use HPID (long persona), not PHID int"
        );
        assert!(
            !payload.windows(3).any(|w| w == TdfEncoder::make_tag("PHID")),
            "PHID int truncates persona varints -- client reads PHID=0 and misaligns TDF"
        );
    }

    #[test]
    fn get_full_game_data_nested_game_row_after_notify_setup() {
        reset_test_games();
        let mut req = Vec::new();
        req.extend_from_slice(&TdfEncoder::encode_string("GNAM", "XEVRAC"));
        req.extend_from_slice(&TdfEncoder::encode_int("GSET", 271));
        game_state::seed_from_reset(&req, 1);
        let _notify = build_game_manager_notify_game_setup(&req, 1).expect("notify");

        let mut gfgd_req = Vec::new();
        gfgd_req.extend_from_slice(&TdfEncoder::encode_long_list("GIDL", &[1_i64]));
        let resp = handle_game_manager_get_full_game_data(&gfgd_req).expect("gfgd");
        let tree = TdfTreeParser::parse_packet(&resp).expect("parse gfgd");

        let lgam = find_tag(&tree, "LGAM").expect("LGAM root");
        assert_eq!(lgam.children.len(), 1, "LGAM must have one row");

        let row = &lgam.children[0];
        let game = find_tag(&row.children, "GAME").expect("GAME struct in ListGameData row");
        assert!(
            find_tag(&game.children, "GNAM").is_some(),
            "GNAM missing in GAME"
        );
        assert!(
            find_tag(&game.children, "GID ")
                .or_else(|| find_tag(&game.children, "GID"))
                .is_some(),
            "GID missing in GAME"
        );
        // NTOP via the linear long-scanner: find_int_field uses skip_field which mis-skips
        // multi-entry string maps (now that ATTR carries both PingSiteAlias and serverid).
        let ntop = TdfEncoder::find_long_field(&resp, "NTOP")
            .map(|v| v as i32)
            .unwrap_or(-1);
        assert_eq!(ntop, NTOP_CLIENT_SERVER_DEDICATED, "NTOP must be dedicated");

        let gnam = TdfEncoder::find_string_field(&resp, "GNAM").unwrap_or_default();
        assert_eq!(gnam, "XEVRAC");

        let pros_tag = TdfEncoder::make_tag("PROS");
        assert!(resp.windows(3).any(|w| w == pros_tag));

        let gid = TdfEncoder::find_long_field(&resp, "GID ")
            .or_else(|| TdfEncoder::find_long_field(&resp, "GID"))
            .or_else(|| TdfEncoder::find_int_field(&resp, "GID ").map(|v| v as i64))
            .or_else(|| TdfEncoder::find_int_field(&resp, "GID").map(|v| v as i64));
        assert_eq!(gid, Some(1));
    }

    #[test]
    fn pros_entry_retail_field_order_preserves_pid_time_stat() {
        reset_test_games();
        let player = game_state::CncPlayer {
            persona_id: 1201618778,
            display_name: "Xevrac".to_string(),
            slot: 0,
            team: 1,
            is_ai: false,
            ready: false,
            attribs: indexmap::IndexMap::new(),
            custom_data: indexmap::IndexMap::new(),
            stat: 2,
        };
        let notify_row = game_state::build_notify_pros_entry(&player, 1);
        let gfgd_row = game_state::build_gfgd_pros_entry(&player, 1);
        for (label, row) in [("notify", notify_row.as_slice()), ("gfgd", gfgd_row.as_slice())] {
            assert_eq!(
                TdfEncoder::find_long_field(row, "EXID"),
                Some(1201618778),
                "{label} PROS EXID"
            );
            assert_eq!(
                TdfEncoder::find_long_field(row, "PID "),
                Some(1201618778),
                "{label} PROS PID union member 0"
            );
            assert_eq!(
                TdfEncoder::find_long_field(row, "GID "),
                Some(1),
                "{label} PROS GID"
            );
            assert_eq!(
                TdfEncoder::find_long_field(row, "TIME"),
                Some(1201618778),
                "{label} PROS TIME (persona @ +208)"
            );
            assert!(
                row.windows(3).any(|w| w == TdfEncoder::make_tag("SID ")),
                "{label} PROS SID tag"
            );
            assert_eq!(TdfEncoder::find_long_field(row, "STAT"), Some(2), "{label} PROS STAT");
            assert!(
                !row.windows(3).any(|w| w == TdfEncoder::make_tag("UID ")),
                "{label} PROS must not emit UID -- not in retail ReplicatedGamePlayer schema"
            );
        }
        assert!(
            !notify_row.windows(3).any(|w| w == TdfEncoder::make_tag("BLOB")),
            "notify PROS must not include BLOB -- client mis-parses and crashes"
        );
        for (label, row) in [("notify", notify_row.as_slice()), ("gfgd", gfgd_row.as_slice())] {
            let pnet_pos = row
                .windows(3)
                .position(|w| w == TdfEncoder::make_tag("PNET"))
                .expect("{label} PNET");
            let pid_pos = row
                .windows(3)
                .position(|w| w == TdfEncoder::make_tag("PID "))
                .expect("{label} PID");
            assert!(pid_pos < pnet_pos, "{label} PID must precede PNET on wire");
            assert_eq!(
                row.get(pnet_pos + 3).copied(),
                Some(0x06),
                "{label} PNET must be NetworkAddress UNION (0x06), not OBJECT_ID or member 127"
            );
        }
    }

    #[test]
    fn pros_entry_uses_space_padded_loc_pid_time_tags() {
        let player = game_state::CncPlayer {
            persona_id: 1201618778,
            display_name: "Xevrac".to_string(),
            slot: 0,
            team: 1,
            is_ai: false,
            ready: false,
            attribs: indexmap::IndexMap::new(),
            custom_data: indexmap::IndexMap::new(),
            stat: 2,
        };
        let row = game_state::build_pros_entry(&player, 1);
        assert!(row.windows(3).any(|w| w == TdfEncoder::make_tag("LOC ")));
        assert!(row.windows(3).any(|w| w == TdfEncoder::make_tag("PID ")));
        assert!(row.windows(3).any(|w| w == TdfEncoder::make_tag("TIME")));
    }
}
