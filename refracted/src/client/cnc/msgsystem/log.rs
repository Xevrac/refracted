//! Directional MessageSystem / SimuCloud log helpers.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{LazyLock, Mutex};

use tracing::{debug, info};

use crate::core::console::push_grpc_compact_upsert;

use super::messages::{
    frame_type_label, ALLOW_INPUT_CHANGE_TYPE_ID, CLIENT_FINISHED_LOAD_TYPE_ID,
    GENERAL_TAUNT_TYPE_ID, LOAD_MAP_TYPE_ID, PING_QUERY_TYPE_ID, PING_REPLY_TYPE_ID,
    REQUEST_RANDOM_GENERAL_TAUNT_TYPE_ID, REQUEST_SPECIFIC_GENERAL_TAUNT_TYPE_ID,
    START_GAME_TYPE_ID,
};
use super::LOG_TAG;

const SIM_TAG: &str = "\x1b[38;2;140;180;140m[SIM]\x1b[0m";
const ORCH_TAG: &str = "\x1b[38;2;140;180;220m[Orchestration]\x1b[0m";

#[derive(Clone, Copy, PartialEq, Eq)]
enum RelayDirection {
    ClientToRts,
    RtsToClient,
    RtsToServer,
    ServerToRts,
}

struct RelayLogCompactor {
    ping_counts: HashMap<String, u32>,
}

static COMPACTOR: LazyLock<Mutex<RelayLogCompactor>> = LazyLock::new(|| {
    Mutex::new(RelayLogCompactor {
        ping_counts: HashMap::new(),
    })
});

fn is_ping(type_id: u16) -> bool {
    type_id == PING_QUERY_TYPE_ID || type_id == PING_REPLY_TYPE_ID
}

fn is_handshake_milestone(type_id: u16) -> bool {
    matches!(
        type_id,
        0 | 1 | 12 | 27 | 28 | 92 | 100 | 180 | 186 | 200 | 201 | 215
    ) || type_id == LOAD_MAP_TYPE_ID
        || type_id == CLIENT_FINISHED_LOAD_TYPE_ID
        || type_id == START_GAME_TYPE_ID
        || type_id == ALLOW_INPUT_CHANGE_TYPE_ID
        || type_id == GENERAL_TAUNT_TYPE_ID
        || type_id == REQUEST_RANDOM_GENERAL_TAUNT_TYPE_ID
        || type_id == REQUEST_SPECIFIC_GENERAL_TAUNT_TYPE_ID
}

fn relay_milestone_line(direction: RelayDirection, frame_name: &str) -> String {
    match direction {
        RelayDirection::ClientToRts => format!("[{LOG_TAG}] Client → hub: {frame_name}"),
        RelayDirection::RtsToClient => format!("[{LOG_TAG}] Hub → client: {frame_name}"),
        RelayDirection::RtsToServer => format!("[{LOG_TAG}] Hub → dedicated: {frame_name}"),
        RelayDirection::ServerToRts => format!("[{LOG_TAG}] Dedicated → hub: {frame_name}"),
    }
}

fn relay_debug_line(
    direction: RelayDirection,
    peer: SocketAddr,
    type_id: u16,
    payload_len: usize,
    extra: &str,
) -> String {
    let name = frame_type_label(type_id);
    let suffix = if extra.is_empty() {
        String::new()
    } else {
        format!(" {extra}")
    };
    match direction {
        RelayDirection::ClientToRts => format!(
            "[{LOG_TAG}] {peer} client→hub typeId={type_id} len={payload_len} ({name}){suffix}"
        ),
        RelayDirection::RtsToClient => format!(
            "[{LOG_TAG}] {peer} hub→client typeId={type_id} len={payload_len} ({name}){suffix}"
        ),
        RelayDirection::RtsToServer => format!(
            "[{LOG_TAG}] {peer} hub→dedicated typeId={type_id} len={payload_len} ({name}){suffix}"
        ),
        RelayDirection::ServerToRts => format!(
            "[{LOG_TAG}] {peer} dedicated→hub typeId={type_id} len={payload_len} ({name}){suffix}"
        ),
    }
}

fn compact_key(direction: RelayDirection, type_id: u16) -> String {
    let dir = match direction {
        RelayDirection::ClientToRts => "C2R",
        RelayDirection::RtsToClient => "R2C",
        RelayDirection::RtsToServer => "R2S",
        RelayDirection::ServerToRts => "S2R",
    };
    format!("RTS|{dir}|{type_id}")
}

fn emit_compact_upsert(upsert_key: String, text: &str, count: u32) {
    if !crate::core::console::is_debug_logging_enabled() {
        return;
    }
    let ansi = if count <= 1 {
        text.to_string()
    } else {
        format!("{text} x{count}")
    };
    push_grpc_compact_upsert(upsert_key, &ansi);
}

pub fn flush_relay_log_compactor() {
    let mut guard = COMPACTOR.lock().expect("relay log compactor lock");
    guard.ping_counts.clear();
}

fn log_relay(
    direction: RelayDirection,
    peer: SocketAddr,
    type_id: u16,
    payload_len: usize,
    extra: &str,
) {
    let frame_name = frame_type_label(type_id);
    let debug_line = relay_debug_line(direction, peer, type_id, payload_len, extra);
    debug!("{debug_line}");

    if is_ping(type_id) {
        let key = compact_key(direction, type_id);
        let mut guard = COMPACTOR.lock().expect("relay log compactor lock");
        let count = guard
            .ping_counts
            .entry(key.clone())
            .and_modify(|c| *c = c.saturating_add(1))
            .or_insert(1);
        emit_compact_upsert(key, &debug_line, *count);
        return;
    }

    if is_handshake_milestone(type_id) {
        flush_relay_log_compactor();
        info!("{}", relay_milestone_line(direction, frame_name));
        if type_id == ALLOW_INPUT_CHANGE_TYPE_ID && extra.starts_with("payload=") {
            info!("[{LOG_TAG}] AllowInputChange wire {extra}");
        }
    }
}

pub fn log_rts_system(peer: SocketAddr, detail: &str) {
    if detail.contains("connected") || detail.contains('↔') {
        info!("[{LOG_TAG}] {peer} {detail}");
    } else {
        debug!("[{LOG_TAG}] {peer} {detail}");
    }
}

pub fn log_sim_milestone(detail: &str) {
    info!("{SIM_TAG} {detail}");
}

pub fn log_sim_debug(detail: &str) {
    debug!("{SIM_TAG} {detail}");
}

pub fn log_orch_milestone(detail: &str) {
    info!("{ORCH_TAG} {detail}");
}

pub fn log_orch_debug(detail: &str) {
    debug!("{ORCH_TAG} {detail}");
}

/// Back-compat: treat as a SimuCloud debug line.
pub fn log_sim(detail: &str) {
    log_sim_debug(detail);
}

pub fn log_client_to_rts(peer: SocketAddr, type_id: u16, payload_len: usize, extra: &str) {
    log_relay(
        RelayDirection::ClientToRts,
        peer,
        type_id,
        payload_len,
        extra,
    );
}

pub fn log_rts_to_client(peer: SocketAddr, type_id: u16, payload_len: usize, extra: &str) {
    log_relay(
        RelayDirection::RtsToClient,
        peer,
        type_id,
        payload_len,
        extra,
    );
}

pub fn log_rts_to_server(peer: SocketAddr, type_id: u16, payload_len: usize, extra: &str) {
    log_relay(
        RelayDirection::RtsToServer,
        peer,
        type_id,
        payload_len,
        extra,
    );
}

pub fn log_server_to_rts(peer: SocketAddr, type_id: u16, payload_len: usize, extra: &str) {
    log_relay(
        RelayDirection::ServerToRts,
        peer,
        type_id,
        payload_len,
        extra,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_frames_are_not_milestones() {
        assert!(!is_handshake_milestone(PING_QUERY_TYPE_ID));
        assert!(is_handshake_milestone(LOAD_MAP_TYPE_ID));
    }

    #[test]
    fn relay_milestone_line_is_readable() {
        let line = relay_milestone_line(RelayDirection::RtsToClient, "LoadMap");
        assert!(line.contains("LoadMap"));
        assert!(line.to_lowercase().contains("hub"));
    }
}
