pub mod build_detection;
pub mod capture;
pub mod constants;
pub mod data;
pub mod labs_client_config;
pub mod overrides;
pub mod payload_auth;
mod payload_game_manager;
mod payload_messaging;
mod payload_stats;
mod payload_util;

pub use build_detection::labs_hint_build_profile_from_text;
pub use capture::{normalize_url, parse_raw_response, try_load_captured_response};
pub use constants::LABS_SESSION_OBJECT_ID;
pub use data::photon_js_runtime_dir;
pub use labs_client_config::fetch_client_config_conf_map;

use bytes::Bytes;
use crate::blaze::tdf::TdfEncoder;
use crate::common::error::BlazeResult;

pub fn handle_packet_fields(
    component: u16,
    command: u16,
    payload: &[u8],
) -> Option<BlazeResult<Bytes>> {
    match (component, command) {
        (0x0009, 0x02) => Some(payload_util::handle_util_ping(payload)),
        (0x0009, 0x07) => Some(payload_util::handle_util_preauth(payload)),
        (0x0009, 0x01) => Some(payload_util::handle_util_fetch_client_config(payload)),
        (0x0009, 0x08) => Some(payload_util::handle_util_post_auth(payload)),
        (0x0009, 0x05) => Some(payload_util::handle_util_get_telemetry_server(payload)),
        (0x0009, 0x09) => Some(payload_util::handle_util_set_client_state(payload)),
        (0x0009, 0x16) => Some(Err(crate::common::error::BlazeError::AuthorizationRequired)),
        (0x0009, 0x1c) => Some(payload_util::handle_util_set_client_state_28(payload)),
        (0x0001, 0x0a) => Some(payload_auth::handle_auth_login(payload)),
        (0x0001, 0x28) => Some(payload_auth::handle_auth_login(payload)),
        (0x0001, 0x46) => Some(payload_auth::handle_auth_logout(payload)),
        (0x000F, 0x01) => Some(payload_messaging::handle_messaging_send_message(payload)),
        (0x7802, 0x01) => Some(handle_user_sessions_command_1(payload)),
        (0x7802, 0x08) => Some(handle_user_sessions_update_hardware_flags(payload)),
        (0x7802, 0x0c) => Some(handle_user_sessions_lookup_user(payload)),
        (0x7802, 0x0d) => Some(handle_user_sessions_lookup_users(payload)),
        (0x7802, 0x14) => Some(handle_user_sessions_update_network_info(payload)),
        (0x7802, 0x0b) => Some(handle_user_sessions_set_user_cross_platform_opt_in(payload)),
        (0x7802, 0x15) => Some(handle_user_sessions_lookup_users(payload)),
        (0x7802, 0x3c) => Some(handle_user_sessions_command_60(payload)),
        (0x0007, 0x00) => Some(payload_stats::handle_stats_command_0(payload)),
        (0x0007, 0xf00) => Some(payload_stats::handle_stats_command_3840(payload)),
        (0x0007, 0x2900) => Some(payload_stats::handle_stats_command_10496(payload)),
        (0x0007, 0x3700) => Some(payload_stats::handle_stats_command_14080(payload)),
        (0x0007, 0x4100) => Some(payload_stats::handle_stats_command_16640(payload)),
        (0x0007, 0x4f00) => Some(payload_stats::handle_stats_command_20224(payload)),
        (0x0007, 0x5900) => Some(payload_stats::handle_stats_command_22784(payload)),
        (0x0007, 0x7100) => Some(payload_stats::handle_stats_command_28928(payload)),
        (0x0004, 0x03) => Some(payload_game_manager::handle_game_manager_command_3(payload)),
        (0x0004, 0x05) => Some(payload_game_manager::handle_game_manager_command_5(payload)),
        (0x0004, 0x07) => Some(payload_game_manager::handle_game_manager_command_7(payload)),
        (0x0004, 0x0a) => Some(payload_game_manager::handle_game_manager_command_10(payload)),
        (0x0004, 0x10) => Some(payload_game_manager::handle_game_manager_command_16(payload)),
        (0x0004, 0x11) => Some(payload_game_manager::handle_game_manager_return_dedicated_server_to_pool(payload)),
        (0x0004, 0x71) => Some(payload_game_manager::handle_game_manager_command_113(payload)),
        _ => None,
    }
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
    // Minimal one-item struct list encoding
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
