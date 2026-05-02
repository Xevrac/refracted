//! Per-title (client) behaviour: ports, Blaze fetchClientConfig maps, capture replay, build detection.
//!
//! Add a submodule under `client/` for each supported game id (see [`crate::common::game`]) and
//! register dispatch in [`hint_build_profile_from_text`] and Blaze handlers as needed.

pub mod cnc;
pub mod labs;
pub mod profile;

pub use profile::{aggregated_required_ports, ServiceFlags};

use bytes::Bytes;
use crate::common::build_profile::BuildProfile;
use crate::common::error::BlazeResult;
use crate::common::game::get_current_game_id;

pub fn hint_build_profile_from_text(text: &str) -> BuildProfile {
    match get_current_game_id().as_str() {
        "bf-labs" => labs::labs_hint_build_profile_from_text(text),
        _ => BuildProfile::Unknown,
    }
}

pub fn handle_redirector_get_server_instance(payload: &[u8]) -> Option<BlazeResult<Bytes>> {
    match get_current_game_id().as_str() {
        "cnc" => Some(cnc::handle_redirector_get_server_instance(payload)),
        _ => None,
    }
}

pub fn handle_util_preauth(payload: &[u8]) -> Option<BlazeResult<Bytes>> {
    match get_current_game_id().as_str() {
        "cnc" => Some(cnc::handle_util_preauth(payload)),
        _ => None,
    }
}

pub fn handle_packet_fields(
    component: u16,
    command: u16,
    payload: &[u8],
) -> Option<BlazeResult<Bytes>> {
    match get_current_game_id().as_str() {
        "cnc" => cnc::handle_packet_fields(component, command, payload),
        "bf-labs" => labs::handle_packet_fields(component, command, payload),
        _ => None,
    }
}
