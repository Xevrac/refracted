//! Command & Conquer 
//!
//! Wire dispatch from [`crate::client`] and Blaze/HTTP handlers once this title is implemented.

pub mod dedicated_pool;
pub mod fireframe;
pub mod game_state;

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

// Blaze GameManager `NTOP` (NetworkTopology) constants — values verified
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

/// Blaze `preAuth` **QOSS** — without **LTPS** ping sites the CNC client logs *No ping site configured* and can stall after auth.
fn cnc_encode_preauth_qoss_field() -> Vec<u8> {
    let qos_ports = crate::common::game::current_service_ports();
    let mut qoss_struct = Vec::new();
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("CQFR", 300_000_000));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("CQRR", 0));
    let mut ltps_map = indexmap::IndexMap::new();
    let mut region_struct = Vec::new();
    region_struct.extend_from_slice(&TdfEncoder::encode_string("PSA ", "127.0.0.1"));
    region_struct.extend_from_slice(&TdfEncoder::encode_int("PSP ", qos_ports.qos_data as i32));
    // Use a real region id from the Labs table; non-standard keys may confuse strict QoS parsers.
    ltps_map.insert("aws-syd".to_string(), region_struct);
    qoss_struct.extend_from_slice(&TdfEncoder::encode_string_struct_map_ordered("LTPS", &ltps_map));
    let mut qcnf_struct = Vec::new();
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_int("DPSP", qos_ports.qos_data as i32));
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_string(
        "QCA ",
        "qoscoordinator.gameservices.ea.com",
    ));
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_int("QCP ", qos_ports.qos_alt as i32));
    qcnf_struct.extend_from_slice(&TdfEncoder::encode_string("QPR ", "cnc-community-qos"));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_struct("QCNF", &qcnf_struct));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("SQRR", 15_000_000));
    qoss_struct.extend_from_slice(&TdfEncoder::encode_int("VERS", 1));
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
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "cfg" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// CNC probe HTTP routes (`/cnc/probe-dump`, `/cnc/probe-add-ai`).
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
    if base == "cnc/probe-add-ai" && (is_post || is_get) {
        let _ = body;
        return Some(handle_probe_add_ai(query));
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

fn query_param_i64(query: Option<&str>, key: &str, default: i64) -> i64 {
    query
        .and_then(|q| {
            q.split('&').find_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                if k == key {
                    percent_decode_plus(v).parse().ok()
                } else {
                    None
                }
            })
        })
        .unwrap_or(default)
}

/// Dev probe: add AI to game state and queue Blaze join notifies on the active client session.
fn handle_probe_add_ai(query: Option<&str>) -> HttpResponse {
    use crate::common::error::BlazeError;

    let gid = query_param_i64(query, "gid", 1);
    let result = probe_add_queued_player_to_game(gid);
    match result {
        Ok((gid, ai_pid, name, sessions)) => {
            let body = serde_json::json!({
                "ok": true,
                "gid": gid,
                "ai_pid": ai_pid,
                "name": name,
                "blaze_sessions": sessions,
            });
            HttpResponse::new(
                200,
                "application/json",
                body.to_string().into_bytes(),
            )
        }
        Err(e) => {
            let (status, msg) = match &e {
                BlazeError::InvalidPacket(m) => (400, m.clone()),
                _ => (500, e.to_string()),
            };
            let body = serde_json::json!({ "ok": false, "error": msg });
            HttpResponse::new(status, "application/json", body.to_string().into_bytes())
        }
    }
}

/// Same roster + notify path as GMGR `addQueuedPlayerToGame`, triggered from debug probe HTTP.
pub fn probe_add_queued_player_to_game(gid: i64) -> BlazeResult<(i64, i64, String, usize)> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    let (gid, player) = game_state::add_queued_player(&payload)?;
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m probe-add-ai gid={} slot={} ai_pid={} name={}",
        gid,
        player.slot,
        player.persona_id,
        player.display_name
    );
    let pushes = fireframe::pushes_after_add_queued_player(gid, &player)?;
    let session_ids: Vec<u64> = crate::session::blaze_sessions::list_sessions()
        .into_iter()
        .filter(|s| s.authenticated)
        .filter(|s| {
            s.clnt
                .as_deref()
                .map(|c| c.contains("RtsBlaze"))
                .unwrap_or(true)
        })
        .map(|s| s.id)
        .collect();
    if session_ids.is_empty() {
        return Err(crate::common::error::BlazeError::InvalidPacket(
            "no authenticated Blaze client session — log in first".into(),
        ));
    }
    for sid in &session_ids {
        fireframe::enqueue_pending_pushes(*sid, pushes.clone());
    }
    Ok((
        gid,
        player.persona_id,
        player.display_name,
        session_ids.len(),
    ))
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

pub fn try_handle_http_request(method: &str, path: &str) -> Option<HttpResponse> {
    let is_head = method == "HEAD";
    if method != "GET" && !is_head {
        return None;
    }

    let clean = path.split('?').next().unwrap_or(path);
    let request_rel = if let Some(rest) = clean.strip_prefix("/cnc/data/") {
        rest
    } else {
        clean.trim_start_matches('/')
    };

    let rel = sanitize_relative_request_path(request_rel)?;
    let root = cnc_data_runtime_dir();
    let full = root.join(&rel);

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
            return Some(HttpResponse::new(200, ct, body));
        }
    }

    Some(HttpResponse::new(404, "text/plain", b"Not Found".to_vec()))
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
    let script = format!(
        "<script>window.__CNC_PROFILE={};</script>",
        json
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
    response.extend_from_slice(&encode_union_struct("ADDR", 0, "VALU", |valu| {
        valu.extend_from_slice(&TdfEncoder::encode_string("HOST", host));
        valu.extend_from_slice(&TdfEncoder::encode_int("IP\0\0", ip));
        valu.extend_from_slice(&TdfEncoder::encode_int("PORT", ports.blaze_main as i32));
    }));
    // 0 = plain TCP on blaze_main (TLS to 127.0.0.1 often yields SDK disconnect / RPC stall).
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
        (0x0009, 0x16) => Some(Err(crate::common::error::BlazeError::AuthorizationRequired)),
        (0x0009, 0x1c) => Some(handle_util_set_client_state_28(payload)),
        (0x0001, 0x0a) => Some(handle_auth_login(payload)),
        (0x0001, 0x28) => Some(handle_auth_login(payload)),
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
        (0x0004, 0x03) => Some(handle_game_manager_command_3(payload)),
        (0x0004, 0x05) => Some(handle_game_manager_command_5(payload)),
        (0x0004, 0x07) => Some(handle_game_manager_command_7(payload)),
        (0x0004, 0x09) => Some(handle_game_manager_join_game(payload)),
        (0x0004, 0x08) => Some(handle_game_manager_set_player_attributes(payload)),
        (0x0004, 0x0b) => Some(handle_game_manager_remove_player(payload)),
        (0x0004, 0x0d) => Some(handle_game_manager_finalize_game_creation(payload)),
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
        // CNC Blaze 3.19.4: dedicated reset uses 0x0019; official table lists reset at 0x16 — both return JoinGameResponse.
        (0x0004, 0x16) => Some(handle_game_manager_reset_dedicated_server(payload)),
        (0x0004, 0x19) => Some(handle_game_manager_reset_dedicated_server(payload)),
        (0x0004, 0x71) => Some(handle_game_manager_command_113(payload)),
        // RedirectorComponent::getServerInstance
        (0x0005, 0x0001) => Some(handle_redirector_get_server_instance(payload)),
        // UtilComponent::preAuth
        (0x0009, 0x0007) => Some(handle_util_preauth(payload)),
        // Blaze::Rooms — hub assigns component id at runtime (~`0x7800` segment). Extend when discovery captures `(id,opcode)`.
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

    let mut pss = Vec::new();
    pss.extend_from_slice(&TdfEncoder::encode_string("ADRS", "127.0.0.1"));
    pss.extend_from_slice(&TdfEncoder::encode_string("PJID", "123071"));
    pss.extend_from_slice(&TdfEncoder::encode_int("PORT", 80));
    pss.extend_from_slice(&TdfEncoder::encode_int("RPRT", 9));
    pss.extend_from_slice(&TdfEncoder::encode_int("TIID", 0));
    pss.extend_from_slice(&TdfEncoder::encode_struct("CSIG", &[]));
    pss.extend_from_slice(&TdfEncoder::encode_object_id_list("OIDS", &[]));
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

pub fn handle_auth_login(payload: &[u8]) -> BlazeResult<Bytes> {
    if let Some(mail) = TdfEncoder::find_string_field(payload, "MAIL") {
        if !mail.is_empty() {
            let mut s = get_user_session();
            s.email = mail.clone();
            set_user_session(s);
        }
    }

    let session = crate::session::get_user_session();
    let uid = if session.persona_id == 0 {
        1000
    } else {
        session.persona_id as i64
    };
    let display_name = if session.display_name.is_empty() {
        "Player"
    } else {
        session.display_name.as_str()
    };
    let session_key = crate::client::labs::payload_auth::blaze_session_key(
        session.user_id as i64,
        session.persona_id as i64,
    );

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_bool("ANON", false));
    response.extend_from_slice(&TdfEncoder::encode_bool("NTOS", false));
    response.extend_from_slice(&TdfEncoder::encode_string("PCTK", ""));

    let mut profile_struct = Vec::new();
    profile_struct.extend_from_slice(&TdfEncoder::encode_string("DSNM", display_name));
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
    set_user_session(session.clone());

    let uid = session.persona_id as i64;
    let display_name = if session.display_name.is_empty() {
        "Player"
    } else {
        session.display_name.as_str()
    };
    let mail = session.email.as_str();
    let session_key = crate::client::labs::payload_auth::blaze_session_key(
        session.user_id as i64,
        session.persona_id as i64,
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("BUID", uid));
    response.extend_from_slice(&TdfEncoder::encode_bool("FRST", false));
    response.extend_from_slice(&TdfEncoder::encode_string("KEY ", &session_key));
    response.extend_from_slice(&TdfEncoder::encode_long("LLOG", now));
    response.extend_from_slice(&TdfEncoder::encode_string("MAIL", mail));

    let mut pdtl = Vec::new();
    pdtl.extend_from_slice(&TdfEncoder::encode_string("DSNM", display_name));
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

/// `JoinGameResponse` variant used after **`resetDedicatedServer`** / CNC create (`sub_A4DAE0` → `sub_A4BB60`):
/// matches **`handle_game_manager_command_16`** (GID + JGS + **`OCAL`**) and now also emits **`NTOP`**
/// so the client picks up the intended network topology (PEER vs DEDICATED).
fn cnc_join_game_response_with_ocal(gid: i64) -> Bytes {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    response.extend_from_slice(&TdfEncoder::encode_int("JGS ", JGS_JOINED_GAME));
    response.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_DEFAULT));
    response.extend_from_slice(&TdfEncoder::encode_int("OCAL", 0));
    Bytes::from(response)
}

/// `GameManager.joinGame` (0x0004::0x0009) — `JoinGameResponse` with the requested or default game id.
pub fn handle_game_manager_join_game(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = cnc_extract_join_game_id(payload);
    let session = crate::session::get_user_session();
    let pid = if session.persona_id == 0 {
        1000_i64
    } else {
        session.persona_id as i64
    };
    if !game_state::is_player_in_game(gid, pid) {
        game_state::seed_from_join(gid);
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
///
/// `blazeCreateGame` drives **`0x0004::0x0019`** with **`CreateGameRequest`** (GNAM, GSET, HNET, …). Reply must match
/// the **`JoinGameResponse`** shape the RTS client unpacks after **`sub_A4BB60`** (include **`OCAL`**).
pub fn handle_game_manager_reset_dedicated_server(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = cnc_extract_reset_game_id(payload);
    game_state::seed_from_reset(payload, gid);
    Ok(cnc_join_game_response_with_ocal(gid))
}

/// Shared GID extraction for CNC `resetDedicatedServer` flow — used by the request reply and by the
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

/// `GameManager.finalizeGameCreation` (**`0x0004::0x000D`**) — often follows create / reset on the wire; unhandled RPCs yield long waits.
pub fn handle_game_manager_finalize_game_creation(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

/// CNC `GameManager.removePlayer` (**`0x0004::0x000B`** — same numeric id as EA `startMatchmaking`).
pub fn handle_game_manager_remove_player(_payload: &[u8]) -> BlazeResult<Bytes> {
    Ok(Bytes::from(Vec::new()))
}

pub fn handle_game_manager_command_3(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_string("HOST", "203.129.23.162"));
    response.extend_from_slice(&TdfEncoder::encode_int("PORT", 65535));
    response.extend_from_slice(&TdfEncoder::encode_string("REGION", "aws-syd"));
    Ok(Bytes::from(response))
}

pub fn handle_game_manager_command_5(payload: &[u8]) -> BlazeResult<Bytes> {
    let _ = payload;
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
///
/// Per BlazeSDK `gamebrowser.tdf`: reply is `GetGameListResponse` (`glid`, `maxf`, `ngd`, …).
/// Game rows are **not** inline — the client expects follow-up `NotifyGameListUpdate` (cmd 201).
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

/// `GameManager.listGames` (0x0004::0x000E) — minimal `GLST` so the client does not RPC-timeout.
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

/// `GameManager.listGameData` (0x0004::0x0022) — `ListGameData::mGameRoster` as `PLST` (matches login roster shape).
pub fn handle_game_manager_list_game_data(payload: &[u8]) -> BlazeResult<Bytes> {
    let gid = parse_list_game_data_gid(payload);
    let players = game_state::plst_entries_for_gid(gid);
    let mut response = Vec::new();
    response.extend_from_slice(&encode_struct_list("PLST", &players));
    Ok(Bytes::from(response))
}

/// Wire tag for `GetFullGameDataResponse::mGames` (client SDK field `LGAM`).
const GFGD_MGAMES_LIST_TAG: &str = "LGAM";

/// `GameManager.getFullGameData` (0x0004::0x0067 / 0x002C) — `GetFullGameDataResponse::mGames`.
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
    let _ = payload;
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[CNC]\x1b[0m registerDynamicDedicatedServerCreator (pool creator registered)"
    );
    Ok(Bytes::from(Vec::new()))
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
    let session = crate::session::get_user_session();
    let uid = if session.persona_id == 0 { 1000 } else { session.persona_id as i64 };
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_int("FLGS", 3));
    response.extend_from_slice(&TdfEncoder::encode_long("ID  ", uid));
    Ok(Bytes::from(response))
}

pub fn build_user_sessions_user_authenticated_notification() -> BlazeResult<Bytes> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let session = crate::session::get_user_session();
    let uid = if session.persona_id == 0 { 1000 } else { session.persona_id as i64 };
    let display_name = if session.display_name.is_empty() {
        "Player"
    } else {
        session.display_name.as_str()
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_int("ALOC", now as i32));
    response.extend_from_slice(&TdfEncoder::encode_long("BUID", uid));
    response.extend_from_slice(&TdfEncoder::encode_string("DSNM", display_name));
    response.extend_from_slice(&TdfEncoder::encode_bool("FRST", false));
    response.extend_from_slice(&TdfEncoder::encode_string("KEY ", "SESSKY"));
    response.extend_from_slice(&TdfEncoder::encode_int("LAST", now as i32));
    response.extend_from_slice(&TdfEncoder::encode_long("LLOG", now));
    let mail = session.email.as_str();
    response.extend_from_slice(&TdfEncoder::encode_string("MAIL", if mail.is_empty() { "" } else { mail }));
    response.extend_from_slice(&TdfEncoder::encode_long("PID ", uid));
    response.extend_from_slice(&TdfEncoder::encode_int("PLAT", 4));
    response.extend_from_slice(&TdfEncoder::encode_long("UID ", uid));
    response.extend_from_slice(&TdfEncoder::encode_long("XREF", 0));
    Ok(Bytes::from(response))
}

pub fn build_user_sessions_user_added_notification() -> BlazeResult<Bytes> {
    let session = crate::session::get_user_session();
    let uid = if session.persona_id == 0 { 1000 } else { session.persona_id as i64 };
    let display_name = if session.display_name.is_empty() {
        "Player"
    } else {
        session.display_name.as_str()
    };

    let mut response = Vec::new();
    let data = encode_union_struct("ADDR", 2, "VALU", |valu| {
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
    user.extend_from_slice(&TdfEncoder::encode_string("NAME", display_name));
    user.extend_from_slice(&TdfEncoder::encode_long("ORIG", 0));
    response.extend_from_slice(&TdfEncoder::encode_struct("USER", &user));
    Ok(Bytes::from(response))
}

// GameManager `GSTA`: 1=PRE_GAME, 2=IN_GAME, 4=POST_GAME, 7=RESETABLE (used after resetDedicatedServer).
#[allow(dead_code)]
const GSTA_PRE_GAME: i32 = 1;
pub(crate) const GSTA_RESETABLE: i32 = 7;

/// `UUID` for `NotifyGameSetup`: use `CreateGameRequest` when present, else a fresh v4 string.
fn cnc_resolve_notify_game_uuid(request_payload: &[u8]) -> String {
    game_state::resolve_game_uuid(request_payload)
}

/// GameManager `NotifyGameStateChange` (`0x0004` / `0x64`): root `GID\0` + `GSTA` (BFP4FToolsWV / CNC launcher).
/// Wire command matches client→server `getGameListSnapshot`; payload is the two-field notify shape.
pub fn build_game_manager_notify_game_state_change(gid: i64, gsta: i32) -> BlazeResult<Bytes> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_int("GID\0", gid as i32));
    out.extend_from_slice(&TdfEncoder::encode_int("GSTA", gsta));
    Ok(Bytes::from(out))
}

/// `ReplicatedGameData::mPlatformHostState` — host persona so GMGR stops treating the game as unhosted.
fn append_replicated_platform_host(out: &mut Vec<u8>, host_persona: i64) {
    let mut phst = Vec::new();
    phst.extend_from_slice(&TdfEncoder::encode_long("HPID", host_persona));
    phst.extend_from_slice(&TdfEncoder::encode_int("HSLT", 0));
    out.extend_from_slice(&TdfEncoder::encode_struct("PHST", &phst));
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
/// CNC / BFP4FToolsWV also labels this path “NotifyServerGameSetup”; same opcode and `GAME` payload.
///
/// **`GAME.HNET`**: copied from the request only when it is already a root **`LIST`** of **`STRUCT`** rows
/// (`0x04` / item `0x03`); otherwise encoded like stock **`GameSetup`**: list of struct rows (**`EXIP`** / **`INIP`**).
pub fn build_game_manager_notify_game_setup(
    request_payload: &[u8],
    gid: i64,
) -> BlazeResult<Bytes> {
    let session = crate::session::get_user_session();
    let uid_i32 = cnc_notify_host_persona_i32();
    let uid = uid_i32 as i64;
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

    // HNET endpoints: INIP from `CreateGameRequest`; EXIP from `updateNetworkInfo`, else our QoS
    // listener's observed peer address (`record_qos_observed_client_endpoint`), else the request's
    // first IP pair, else 0. EXIP port: session / request, else mirror INIP (game) port.
    let ips = TdfEncoder::scan_all_int_fields(request_payload, "IP  ");
    let ports = TdfEncoder::scan_all_int_fields(request_payload, "PORT");
    let host_inip_ip = ips.get(1).copied().unwrap_or(0);
    let host_inip_port_from_request = ports.get(1).copied().unwrap_or(0);
    let req_exip_ip = ips.first().copied().unwrap_or(0);
    let req_exip_port = ports.first().copied().unwrap_or(0);

    let host_exip_ip = session
        .network_exip_ip
        .map(|u| u as i32)
        .filter(|&ip| ip != 0)
        .or_else(|| {
            crate::session::peek_qos_observed_exip_ip()
                .map(|u| u as i32)
                .filter(|&ip| ip != 0)
        })
        .or_else(|| req_exip_ip.ne(&0).then_some(req_exip_ip))
        .unwrap_or(0);

    // Mirror the request's INIP/EXIP ports verbatim; only fall back to the dev port when the request
    // actually sent 0 (or wasn't an IpPairAddress union). Hardcoding CNC_TEST_DEDICATED_PORT (25200)
    // for both meant the client never saw the real listener (3659) it advertised in CreateGameRequest.
    let host_inip_port = if host_inip_port_from_request != 0 {
        host_inip_port_from_request
    } else {
        CNC_TEST_DEDICATED_PORT
    };
    let host_exip_port = if req_exip_port != 0 {
        req_exip_port
    } else {
        host_inip_port
    };

    let gid_i32 = gid.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let _ = gid_i32;

    let build_endpoint = |ip: i32, port: i32| -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
        out.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
        out
    };

    let mut game = Vec::new();
    game.extend_from_slice(&TdfEncoder::encode_long_list("ADMN", &[uid]));
    if let Some(raw) = TdfEncoder::extract_top_level_field_bytes(request_payload, "ATTR") {
        game.extend_from_slice(&raw);
    } else {
        game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
            "ATTR",
            &indexmap::IndexMap::new(),
        ));
    }
    game.extend_from_slice(&TdfEncoder::encode_long_list("CAP ", &[0x20, 0]));
    if let Some(raw) = TdfEncoder::extract_top_level_field_bytes(request_payload, "CRIT") {
        game.extend_from_slice(&raw);
    }
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

    game.extend_from_slice(&TdfEncoder::encode_int("NTOP", ntop_game));
    append_replicated_platform_host(&mut game, uid);
    game.extend_from_slice(&TdfEncoder::encode_string("UUID", &game_uuid));
    game.extend_from_slice(&TdfEncoder::encode_int("VOIP", voip));

    game_state::set_replicated_wire_fields(gid, game.clone());

    let pros = game_state::pros_entries_for_gid(gid);
    game_state::set_pros_wire_fields(gid, pros.clone());

    let reas = encode_reas_dataless();

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_struct("GAME", &game));
    response.extend_from_slice(&encode_struct_list("PROS", &pros));
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

    let host_inip_ip = session
        .network_inip_ip
        .map(|u| u as i32)
        .unwrap_or(0);
    let host_inip_port = session
        .network_inip_port
        .map(|u| u as i32)
        .filter(|&p| p != 0)
        .unwrap_or(CNC_TEST_DEDICATED_PORT);
    let host_exip_ip = session
        .network_exip_ip
        .map(|u| u as i32)
        .unwrap_or(0);
    let host_exip_port = session
        .network_exip_port
        .map(|u| u as i32)
        .filter(|&p| p != 0)
        .unwrap_or(host_inip_port);
    let build_endpoint = |ip: i32, port: i32| -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
        out.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
        out
    };

    let gnam = game_state::game_name(gid);
    let game_uuid = game_state::game_uuid(gid);

    let mut attr = indexmap::IndexMap::new();
    attr.insert("PingSiteAlias".to_string(), "False".to_string());

    let mut game = Vec::new();
    game.extend_from_slice(&TdfEncoder::encode_long_list("ADMN", &[uid]));
    game.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered("ATTR", &attr));
    game.extend_from_slice(&TdfEncoder::encode_long_list("CAP ", &[0x20, 0]));
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
    game.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_CLIENT_SERVER_DEDICATED));
    append_replicated_platform_host(&mut game, uid);
    game.extend_from_slice(&TdfEncoder::encode_string("UUID", &game_uuid));
    game.extend_from_slice(&TdfEncoder::encode_int("VOIP", 0));
    game
}

/// One `ListGameData` row: flat `ReplicatedGameData` fields + `PROS` roster.
/// The CNC client binds list-item fields directly onto `ListGameData` (not a nested `GAME` struct).
fn build_list_game_data_entry(gid: i64) -> BlazeResult<Vec<u8>> {
    let game = build_replicated_game_data_fields(gid);
    let pros = game_state::pros_entries_for_gid(gid);
    let mut out = game;
    out.extend_from_slice(&encode_struct_list("PROS", &pros));
    Ok(out)
}

/// `NotifyGameSetup` body: nested `GAME` struct + `PROS` + `QUEU` (+ `REAS` added by caller).
pub fn build_game_manager_game_payload(gid: i64) -> BlazeResult<Bytes> {
    let game = build_replicated_game_data_fields_fallback(gid);
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
    response.extend_from_slice(&encode_reas_dataless());
    Ok(Bytes::from(response))
}

/// `Blaze::GameManager::NotifyPlatformHostInitialized` (component `0x0004`, command `0x47`).
///
/// Sent immediately after `NotifyGameSetup` so `GameManagerAPI` flips the platform-host state and
/// stops waiting for an injection notification on a peer-hosted game.
///
/// Wire: **`GID `**, **`HPID`** (long persona id), **`PHST`** (platform host slot id = 0).
/// Do not use **`PHID`** as INTEGER — persona ids exceed single-byte varints and the client only consumes the first byte (`0`).
pub fn build_game_manager_notify_platform_host_initialized(gid: i64) -> BlazeResult<Bytes> {
    let gid = gid.clamp(i64::MIN, i64::MAX);
    let host = cnc_notify_host_persona_i32() as i64;
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    response.extend_from_slice(&TdfEncoder::encode_long("HPID", host));
    response.extend_from_slice(&TdfEncoder::encode_int("PHST", 0));
    Ok(Bytes::from(response))
}

/// `GameManager.NotifyPlayerJoinCompleted` (`0x0004` / `0x001E`) — host join finished on dedicated reset.
pub fn build_game_manager_notify_player_join_completed(gid: i64) -> BlazeResult<Bytes> {
    game_state::mark_host_join_completed(gid);
    let player = game_state::host_player_for_gid(gid);
    Ok(Bytes::from(game_state::build_replicated_player(&player, gid)))
}

/// Emit `REAS = UNION{ DATALESS_CONTEXT(0): DCTX }` for the reset/create success path.
/// Wire: `REAS(3) + 0x06(UNION) + 0x00(member 0) + DCTX(3) + 0x00(INTEGER) + varint(0)`.
fn encode_reas_dataless() -> Bytes {
    let reas_tag = TdfEncoder::make_tag("REAS");
    let mut out = Vec::new();
    out.push(reas_tag[0]);
    out.push(reas_tag[1]);
    out.push(reas_tag[2]);
    out.push(0x06);
    out.push(0x00);
    out.extend_from_slice(&TdfEncoder::encode_int("DCTX", 0));
    Bytes::from(out)
}

fn encode_union_struct(
    union_tag: &str,
    member_index: u64,
    value_tag: &str,
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
    out.extend_from_slice(&TdfEncoder::encode_struct(value_tag, &value_struct));
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

#[cfg(test)]
mod notify_game_setup_tests {
    use super::*;
    use crate::blaze::tdf::{TdfEncoder, TdfTreeParser};
    use indexmap::IndexMap;

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
    fn notify_game_state_change_parses() {
        let payload = build_game_manager_notify_game_state_change(42, GSTA_RESETABLE).expect("encode");
        assert_eq!(TdfEncoder::find_int_field(&payload, "GID\0"), Some(42));
        assert_eq!(TdfEncoder::find_int_field(&payload, "GSTA"), Some(GSTA_RESETABLE));
        TdfTreeParser::parse_packet(&payload).expect("parse tree");
    }

    #[test]
    fn notify_setup_nested_uuid_non_empty() {
        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        let u = TdfEncoder::find_string_field(&payload, "UUID").expect("UUID in GAME");
        assert!(u.len() >= 8 && u != ".", "{}", u);
    }

    // Regression: REAS=127 is `INVALID_MEMBER` → client cancels the freshly built game
    // ("canceled or timed out locally"). Reset path must use DATALESS_CONTEXT (member 0)
    // carrying DCTX so `onNotifyGameSetup` binds the game and accepts followup notifies.
    #[test]
    fn notify_setup_reas_is_dataless_not_cancel_sentinel() {
        let payload = build_game_manager_notify_game_setup(&[], 1).expect("encode");
        let reas_tag = TdfEncoder::make_tag("REAS");
        let cancel_needle: [u8; 6] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0xbf, 0x01];
        assert!(
            !payload.windows(cancel_needle.len()).any(|w| w == cancel_needle),
            "REAS must not carry union member 127 (INVALID_MEMBER) — that is the cancel sentinel"
        );
        let dctx_needle: [u8; 5] = [reas_tag[0], reas_tag[1], reas_tag[2], 0x06, 0x00];
        assert!(
            payload.windows(dctx_needle.len()).any(|w| w == dctx_needle),
            "REAS must encode UNION member 0 (DATALESS_CONTEXT)"
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
            "PHID int truncates persona varints — client reads PHID=0 and misaligns TDF"
        );
    }

    #[test]
    fn get_full_game_data_flat_row_after_notify_setup() {
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
        assert!(
            find_tag(&row.children, "GNAM").is_some(),
            "GNAM missing in flat LGAM row"
        );
        assert!(
            find_tag(&row.children, "GID ")
                .or_else(|| find_tag(&row.children, "GID"))
                .is_some(),
            "GID missing in flat LGAM row"
        );
        assert!(
            find_tag(&tree, "GAME").is_none(),
            "ListGameData row must not wrap ReplicatedGameData in GAME"
        );

        let gnam = TdfEncoder::find_string_field(&resp, "GNAM").unwrap_or_default();
        assert_eq!(gnam, "XEVRAC");

        let pros_tag = TdfEncoder::make_tag("PROS");
        assert!(resp.windows(3).any(|w| w == pros_tag));
    }

    #[test]
    fn pros_entry_uses_space_padded_loc_pid_uid_tags() {
        let player = game_state::CncPlayer {
            persona_id: 1201618778,
            display_name: "Xevrac".to_string(),
            slot: 0,
            team: 1,
            is_ai: false,
            attribs: indexmap::IndexMap::new(),
            stat: 2,
        };
        let row = game_state::build_pros_entry(&player, 1);
        assert!(row.windows(3).any(|w| w == TdfEncoder::make_tag("LOC ")));
        assert!(row.windows(3).any(|w| w == TdfEncoder::make_tag("PID ")));
        assert!(row.windows(3).any(|w| w == TdfEncoder::make_tag("UID ")));
    }
}
