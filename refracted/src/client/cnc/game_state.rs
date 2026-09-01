//! In-memory CNC lobby/game roster shared by GMGR replies and notify payloads.

use indexmap::IndexMap;
use parking_lot::Mutex;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::blaze::tdf::TdfEncoder;
use crate::common::error::{BlazeError, BlazeResult};
use crate::session::get_user_session;

const PROS_STAT_ACTIVE_CONNECTING: i32 = 2;
const STAS_IN_GAME: i32 = 2;
pub const ATTR_PASSWORD_FLAG: &str = "_password";
pub const ATTR_PASSWORD_SECRET: &str = "_spw";

static GAMES: OnceLock<Mutex<HashMap<i64, CncGame>>> = OnceLock::new();
static LAST_ADD_QUEUED: OnceLock<Mutex<Option<(i64, CncPlayer)>>> = OnceLock::new();
static LAST_ATTR_CHANGE: OnceLock<Mutex<Option<(i64, i64, IndexMap<String, String>)>>> =
    OnceLock::new();
static LAST_CUSTOM_DATA_CHANGE: OnceLock<Mutex<Option<(i64, i64, IndexMap<String, Vec<u8>>)>>> =
    OnceLock::new();
static NEXT_BROWSER_LIST_ID: AtomicI64 = AtomicI64::new(1);
static LAST_GAME_LIST_SNAPSHOT: OnceLock<Mutex<Option<(i64, Vec<i64>)>>> = OnceLock::new();
#[derive(Clone, Debug, Default)]
struct PendingMapInfo {
    path: String,
    start_count: i32,
}

static PENDING_MAPS: OnceLock<Mutex<HashMap<i64, PendingMapInfo>>> = OnceLock::new();
static PENDING_PLAYER_ATTRS: OnceLock<Mutex<HashMap<i64, HashMap<i64, IndexMap<String, String>>>>> =
    OnceLock::new();
static BLAZE_PREGAME_PUSHED: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static BLAZE_JOIN_SETUP_PUSHED: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static BLAZE_INGAME_PUSHED: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static GAME_READY_PUSHED: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
/// Gids whose mesh is already live.
static CLIENT_MESH_LIVE: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static SERVER_LOST_GIDS: OnceLock<Mutex<HashMap<i64, u64>>> = OnceLock::new();
/// pid → (gid, blaze session) for the lost-connection modal.
static LOST_PERSONAS: OnceLock<Mutex<HashMap<i64, (i64, u64)>>> = OnceLock::new();
static JOIN_PASSWORD_AUTH: OnceLock<Mutex<HashMap<(i64, i64), Instant>>> = OnceLock::new();
const JOIN_PASSWORD_AUTH_TTL: Duration = Duration::from_secs(120);

#[derive(Clone)]
struct LobbyChatLine {
    user: String,
    text: String,
}

static LOBBY_CHAT: OnceLock<Mutex<HashMap<i64, Vec<LobbyChatLine>>>> = OnceLock::new();

fn lobby_chat() -> &'static Mutex<HashMap<i64, Vec<LobbyChatLine>>> {
    LOBBY_CHAT.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn lobby_chat_json(gid: i64) -> serde_json::Value {
    let lines = lobby_chat()
        .lock()
        .get(&gid)
        .cloned()
        .unwrap_or_default();
    serde_json::json!({
        "ok": true,
        "gid": gid,
        "messages": lines.iter().map(|l| serde_json::json!({
            "user": l.user,
            "text": l.text,
        })).collect::<Vec<_>>(),
    })
}

pub fn lobby_chat_push(gid: i64, user: &str, text: &str) -> serde_json::Value {
    let user: String = user.chars().take(32).collect();
    let text: String = text.chars().take(200).collect();
    if text.trim().is_empty() {
        return lobby_chat_json(gid);
    }
    {
        let mut m = lobby_chat().lock();
        let list = m.entry(gid).or_default();
        list.push(LobbyChatLine { user, text });
        if list.len() > 80 {
            let extra = list.len() - 80;
            list.drain(0..extra);
        }
    }
    lobby_chat_json(gid)
}

fn server_lost_gids() -> &'static Mutex<HashMap<i64, u64>> {
    SERVER_LOST_GIDS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lost_personas() -> &'static Mutex<HashMap<i64, (i64, u64)>> {
    LOST_PERSONAS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn note_persona_lost(pid: i64, gid: i64) {
    if pid <= 0 {
        return;
    }
    let sid = blaze_session_for_persona(pid).unwrap_or(0);
    lost_personas().lock().insert(pid, (gid.max(0), sid));
}

fn mark_humans_lost(gid: i64) {
    if gid <= 0 {
        return;
    }
    let pids: Vec<i64> = games()
        .lock()
        .get(&gid)
        .map(|g| {
            g.players
                .iter()
                .filter(|p| !p.is_ai && p.persona_id > 0)
                .map(|p| p.persona_id)
                .collect()
        })
        .unwrap_or_default();
    let mut m = lost_personas().lock();
    for pid in pids {
        let sid = blaze_session_for_persona(pid).unwrap_or(0);
        m.insert(pid, (gid, sid));
    }
}

fn persona_is_lost(pid: i64, gid: i64) -> bool {
    if pid <= 0 {
        return false;
    }
    let mut m = lost_personas().lock();
    match m.get(&pid).copied() {
        Some((lost_gid, sid)) => {
            if sid == 0 || !sessions_for_persona(pid).contains(&sid) {
                m.remove(&pid);
                return false;
            }
            gid <= 0 || lost_gid <= 0 || lost_gid == gid
        }
        None => false,
    }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn note_server_lost(gid: i64) {
    if gid <= 0 {
        return;
    }
    server_lost_gids().lock().insert(gid, now_unix_secs());
}

pub fn signal_lost_game_server(gid: i64) {
    let gid = resolve_active_match_gid(gid);
    if gid > 0 {
        note_server_lost(gid);
        mark_humans_lost(gid);
    }
    kick_clients_on_match_lost(
        gid,
        crate::client::cnc::PLAYER_REMOVED_REASON_GAME_DESTROYED,
    );
}

/// POST `/cnc/lost-game-server`. Marks that persona, or the match's humans if pid is 0.
pub fn notify_shell_match_lost(gid: i64, pid: i64) {
    if pid > 0 {
        note_persona_lost(pid, gid);
        if gid > 0 {
            note_server_lost(gid);
        }
        return;
    }
    if gid > 0 {
        note_server_lost(gid);
        mark_humans_lost(gid);
    }
}

/// Resolve gid when the request sent 0.
fn resolve_active_match_gid(hint: i64) -> i64 {
    if hint > 0 {
        return hint;
    }
    if let Some(gid) = orchestration()
        .lock()
        .iter()
        .filter(|(_, o)| o.client_session_id > 0)
        .map(|(gid, _)| *gid)
        .max()
    {
        return gid;
    }
    games()
        .lock()
        .iter()
        .filter(|(_, g)| !g.is_standby && g.phase == GamePhase::InGame)
        .map(|(gid, _)| *gid)
        .max()
        .unwrap_or(0)
}

/// Notify players so they leave the match.
fn kick_clients_on_match_lost(gid: i64, reason: i32) {
    let gid = resolve_active_match_gid(gid);
    let humans: Vec<i64> = if gid > 0 {
        games()
            .lock()
            .get(&gid)
            .map(|g| {
                g.players
                    .iter()
                    .filter(|p| !p.is_ai)
                    .map(|p| p.persona_id)
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let fallback_pid = {
        let s = crate::session::get_user_session();
        if s.persona_id == 0 {
            0
        } else {
            s.persona_id as i64
        }
    };
    if gid <= 0 {
        return;
    }
    let targets: Vec<i64> = if humans.is_empty() {
        if fallback_pid > 0 {
            vec![fallback_pid]
        } else {
            Vec::new()
        }
    } else {
        humans
    };
    for pid in targets {
        crate::client::cnc::fireframe::request_client_local_game_teardown(gid, pid, reason);
    }
}

/// Clear disconnect flags for one match. Other games stay.
pub fn clear_match_connection_lost(gid: i64) {
    if gid <= 0 {
        return;
    }
    server_lost_gids().lock().remove(&gid);
    lost_personas()
        .lock()
        .retain(|_, (lost_gid, _)| *lost_gid != gid);
}

pub fn clear_persona_match_lost(pid: i64) {
    if pid <= 0 {
        return;
    }
    lost_personas().lock().remove(&pid);
}

/// pid required when gid is 0.
pub fn match_connection_status_json(gid: i64, pid: i64) -> serde_json::Value {
    let persona_lost = persona_is_lost(pid, gid);
    let match_lost = gid > 0 && server_lost_gids().lock().contains_key(&gid);
    let server_lost = if pid > 0 {
        persona_lost
    } else {
        match_lost
    };
    let lost = server_lost;
    serde_json::json!({
        "lost": lost,
        "gid": gid,
        "pid": pid,
        "serverLost": server_lost,
        "clientLost": persona_lost,
        "shellLost": persona_lost,
    })
}

fn blaze_pregame_pushed() -> &'static Mutex<HashSet<i64>> {
    BLAZE_PREGAME_PUSHED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn blaze_join_setup_pushed() -> &'static Mutex<HashSet<i64>> {
    BLAZE_JOIN_SETUP_PUSHED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn blaze_ingame_pushed() -> &'static Mutex<HashSet<i64>> {
    BLAZE_INGAME_PUSHED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn game_ready_pushed() -> &'static Mutex<HashSet<i64>> {
    GAME_READY_PUSHED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn client_mesh_live() -> &'static Mutex<HashSet<i64>> {
    CLIENT_MESH_LIVE.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn clear_blaze_push_flags(gid: i64) {
    clear_blaze_join_and_push_flags(gid);
    clear_orchestration(gid);
}

/// Clears post-join one-shots; keeps join-setup until reset.
pub fn clear_blaze_one_shot_flags(gid: i64) {
    blaze_pregame_pushed().lock().remove(&gid);
    blaze_ingame_pushed().lock().remove(&gid);
    game_ready_pushed().lock().remove(&gid);
}

pub fn clear_blaze_join_and_push_flags(gid: i64) {
    clear_blaze_one_shot_flags(gid);
    blaze_join_setup_pushed().lock().remove(&gid);
    client_mesh_live().lock().remove(&gid);
}

/// True after updateMeshConnection for this gid.
pub fn client_mesh_already_connected(gid: i64) -> bool {
    client_mesh_live().lock().contains(&gid)
}

pub fn note_client_mesh_connected(gid: i64) {
    client_mesh_live().lock().insert(gid);
}

/// Keep GameReady unblocked when skipping a second InitiateConnections.
pub fn mark_orch_mesh_already_connected(gid: i64) {
    orchestration().lock().entry(gid).and_modify(|e| {
        e.mesh_active_connected = true;
    });
}

#[derive(Debug, Clone)]
struct GidOrchestration {
    client_session_id: u64,
    dedicated_session_id: Option<u64>,
    dedicated_host_ready: bool,
    join_pushes_released: bool,
    mesh_active_connected: bool,
    simucloud_match_ready: bool,
    pending_mesh_pid: Option<i64>,
    pending_game_ready_pid: Option<i64>,
    deferred_join_pushes: Option<Vec<super::fireframe::OutgoingPush>>,
}

pub enum MeshUpdateResult {
    DeferredUntilHostReady,
    Push(Vec<super::fireframe::OutgoingPush>),
}

fn mesh_active_connected_pushes(gid: i64, pid: i64) -> Option<Vec<super::fireframe::OutgoingPush>> {
    Some(super::fireframe::pushes_after_update_mesh_connection(gid, pid).ok()?)
}

fn game_ready_and_state_advance_pushes(
    gid: i64,
) -> Option<Vec<super::fireframe::OutgoingPush>> {
    if games().lock().get(&gid).map(|g| g.is_standby).unwrap_or(true) {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    if try_mark_game_ready_pushed(gid) {
        out.extend(super::fireframe::pushes_game_ready_attrib(gid).ok()?);
        enqueue_game_ready_to_dedicated(gid);
        if try_mark_blaze_pregame_pushed(gid) {
            out.extend(super::fireframe::pushes_advance_game_to_ingame(gid).ok()?);
            let _ = try_mark_blaze_ingame_pushed(gid);
            // Blaze GSTA IN_GAME is the session notify, not RTS/match start.
            // Keep CncGame phase PreGame (browser "Lobby") until Start Battle.
        }
    }
    Some(out)
}

fn mesh_active_connected_and_game_ready_pushes(
    gid: i64,
    pid: i64,
) -> Option<Vec<super::fireframe::OutgoingPush>> {
    let mut out = mesh_active_connected_pushes(gid, pid).unwrap_or_default();

    let (orchestrating, sim_ready) = {
        let m = orchestration().lock();
        match m.get(&gid) {
            Some(o) => (true, o.simucloud_match_ready),
            None => (false, true),
        }
    };
    if orchestrating && !sim_ready {
        orchestration().lock().entry(gid).and_modify(|e| {
            e.mesh_active_connected = true;
            e.pending_game_ready_pid = Some(pid);
        });
        super::msgsystem::log::log_orch_debug(&format!(
            "GameReady deferred until SimuCloud match ready (game {gid})"
        ));
        return Some(out);
    }

    out.extend(game_ready_and_state_advance_pushes(gid).unwrap_or_default());
    Some(out)
}

/// Push AuthToken for the joining client, then GameReady, to the dedicated.
fn enqueue_game_ready_to_dedicated(gid: i64) {
    if games().lock().get(&gid).map(|g| g.is_standby).unwrap_or(true) {
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[CNC]\x1b[0m GameReady mirror skipped: game standby/reclaimed (gid={})",
            gid
        );
        return;
    }
    let dedicated_sid = orchestration()
        .lock()
        .get(&gid)
        .and_then(|o| o.dedicated_session_id)
        .or_else(|| super::dedicated_pool::peek_dedicated_for_gid(gid));
    let Some(dedicated_sid) = dedicated_sid else {
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[CNC]\x1b[0m GameReady mirror skipped: no dedicated session (gid={})",
            gid
        );
        return;
    };

    let host_pid = super::dedicated_pool::host_for_gid(gid)
        .map(|h| h.persona_id)
        .unwrap_or(0);
    let client_pid = super::dedicated_pool::client_session_for_gid(gid)
        .and_then(|sid| crate::session::blaze_sessions::get_session(sid))
        .and_then(|s| s.persona_id)
        .map(|p| p as i64)
        .filter(|p| *p != 0)
        .or_else(|| {
            players_for_gid(gid)
                .into_iter()
                .map(|p| p.persona_id)
                .find(|pid| *pid != 0 && *pid != host_pid)
        })
        .unwrap_or(0);

    let mut ded_pushes = Vec::new();
    if client_pid != 0 {
        if let Ok(auth) = super::fireframe::pushes_auth_token_custom_data(gid, client_pid) {
            for mut p in auth {
                p.blaze_send_label = "NotifyPlayerAttrib/CDAT AuthToken -> dedicated joining client";
                p.info_log_line = p.info_log_line.replace("[Blaze→Client]", "[Blaze→Server]");
                ded_pushes.push(p);
            }
        }
    } else {
        crate::debug_println!(
            "\x1b[38;2;255;180;100m[CNC]\x1b[0m GameReady mirror: no joining client pid for AuthToken (gid={})",
            gid
        );
    }
    if let Ok(ready) = super::fireframe::pushes_game_ready_attrib(gid) {
        for mut p in ready {
            p.blaze_send_label = "NotifyGameAttribChange(GameReady) -> dedicated RtsBlaze publish";
            p.info_log_line = format!(
                "[Blaze→Server] GameManager.NotifyGameAttribChange(GameReady) Component=4, Command=80 (dedicated publish path)"
            );
            ded_pushes.push(p);
        }
    }

    if ded_pushes.is_empty() {
        return;
    }

    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: mirroring GameReady (+ client AuthToken pid={}) to dedicated session #{} (gid={})",
        client_pid,
        dedicated_sid,
        gid
    );
    super::fireframe::enqueue_pending_pushes(dedicated_sid, ded_pushes);
}

static ORCHESTRATION: OnceLock<Mutex<HashMap<i64, GidOrchestration>>> = OnceLock::new();

fn orchestration() -> &'static Mutex<HashMap<i64, GidOrchestration>> {
    ORCHESTRATION.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clear_orchestration(gid: i64) {
    orchestration().lock().remove(&gid);
}

pub fn has_orchestration(gid: i64) -> bool {
    orchestration().lock().contains_key(&gid)
}

pub fn has_deferred_join_pushes(gid: i64) -> bool {
    orchestration()
        .lock()
        .get(&gid)
        .and_then(|o| o.deferred_join_pushes.as_ref())
        .is_some()
}

/// Called from `orchestrate_client_reset` when a pooled dedicated is assigned.
pub fn begin_reset_orchestration(gid: i64, client_session_id: u64, dedicated_session_id: u64) {
    clear_blaze_one_shot_flags(gid);
    orchestration().lock().insert(
        gid,
        GidOrchestration {
            client_session_id,
            dedicated_session_id: Some(dedicated_session_id),
            dedicated_host_ready: false,
            join_pushes_released: false,
            mesh_active_connected: false,
            simucloud_match_ready: false,
            pending_mesh_pid: None,
            pending_game_ready_pid: None,
            deferred_join_pushes: None,
        },
    );
}

fn finish_client_join_release(
    gid: i64,
    client_sid: u64,
    mut out: Vec<super::fireframe::OutgoingPush>,
    pending_mesh_pid: Option<i64>,
) -> (u64, Vec<super::fireframe::OutgoingPush>) {
    if let Ok(host_pushes) = super::fireframe::pushes_host_state_advance_for_client(gid) {
        out.extend(host_pushes);
    }
    if let Some(pid) = pending_mesh_pid {
        orchestration().lock().entry(gid).and_modify(|e| {
            e.mesh_active_connected = true;
        });
        if let Some(pushes) = mesh_active_connected_and_game_ready_pushes(gid, pid) {
            out.extend(pushes);
        }
    } else {
        // Mesh is UDP CANA to EnginePeer. Sim is MsgSys TCP, so the client
        // often never reports updateMeshConnection — send ACTIVE_CONNECTED after InitiateConnections.
        let pid = resolve_joining_client_pid(gid, client_sid);
        schedule_synthetic_client_mesh_if_needed(gid, client_sid, pid);
    }
    (client_sid, out)
}

fn resolve_joining_client_pid(gid: i64, client_sid: u64) -> i64 {
    crate::session::blaze_sessions::get_session(client_sid)
        .and_then(|s| s.persona_id)
        .map(|p| p as i64)
        .filter(|p| *p != 0)
        .or_else(|| {
            let s = crate::session::get_user_session();
            (s.persona_id != 0).then_some(s.persona_id as i64)
        })
        .or_else(|| {
            players_for_gid(gid)
                .into_iter()
                .map(|p| p.persona_id)
                .find(|p| *p != 0)
        })
        .unwrap_or(1000)
}

/// Send `NotifyGamePlayerStateChange(ACTIVE_CONNECTED)` if the client never reports mesh.
fn schedule_synthetic_client_mesh_if_needed(gid: i64, client_sid: u64, pid: i64) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let already = orchestration()
            .lock()
            .get(&gid)
            .map(|o| o.mesh_active_connected)
            .unwrap_or(true);
        if already {
            return;
        }
        super::msgsystem::log::log_orch_milestone(&format!(
            "Synthesizing ACTIVE_CONNECTED (no client updateMeshConnection; MsgSys TCP sim) game {gid} pid={pid}"
        ));
        match on_client_mesh_update(gid, pid) {
            MeshUpdateResult::DeferredUntilHostReady => {
                super::msgsystem::log::log_orch_debug(&format!(
                    "Synthetic mesh held until host ready (game {gid})"
                ));
            }
            MeshUpdateResult::Push(pushes) => {
                if !pushes.is_empty() {
                    super::fireframe::enqueue_pending_pushes(client_sid, pushes);
                    let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
                }
            }
        }
    });
}

/// Store client join notifies until the dedicated host completes setup (finalize + advance).
pub fn defer_client_join_pushes(
    gid: i64,
    client_session_id: u64,
    pushes: Vec<super::fireframe::OutgoingPush>,
) -> Option<(u64, Vec<super::fireframe::OutgoingPush>)> {
    let mut m = orchestration().lock();
    let entry = m.entry(gid).or_insert_with(|| GidOrchestration {
        client_session_id,
        dedicated_session_id: super::dedicated_pool::peek_dedicated_for_gid(gid),
        dedicated_host_ready: false,
        join_pushes_released: false,
        mesh_active_connected: false,
        simucloud_match_ready: false,
        pending_mesh_pid: None,
        pending_game_ready_pid: None,
        deferred_join_pushes: None,
    });
    entry.client_session_id = client_session_id;
    if entry.join_pushes_released {
        return None;
    }
    if entry.deferred_join_pushes.is_some() {
        return None;
    }
    if entry.dedicated_host_ready {
        entry.join_pushes_released = true;
        let client_sid = entry.client_session_id;
        let pending_mesh_pid = entry.pending_mesh_pid.take();
        drop(m);
        super::msgsystem::log::log_orch_debug(&format!(
            "Client join notifies flush on defer (host already ready, game {gid})"
        ));
        return Some(finish_client_join_release(
            gid,
            client_sid,
            pushes,
            pending_mesh_pid,
        ));
    }
    entry.deferred_join_pushes = Some(pushes);
    None
}

pub fn is_dedicated_host_ready(gid: i64) -> bool {
    orchestration()
        .lock()
        .get(&gid)
        .map(|o| o.dedicated_host_ready)
        .unwrap_or(false)
}

pub fn client_session_for_gid(gid: i64) -> Option<u64> {
    orchestration()
        .lock()
        .get(&gid)
        .map(|o| o.client_session_id)
        .or_else(|| super::dedicated_pool::client_session_for_gid(gid))
}

/// Dedicated host finished `finalizeGameCreation`. Release deferred client join +
/// push host state advance notifies to the client session.
pub fn complete_dedicated_host_setup(
    gid: i64,
) -> Option<(u64, Vec<super::fireframe::OutgoingPush>)> {
    let mut m = orchestration().lock();
    let entry = m.get_mut(&gid)?;
    if entry.join_pushes_released {
        return None;
    }
    entry.dedicated_host_ready = true;
    let Some(deferred) = entry.deferred_join_pushes.take() else {
        super::msgsystem::log::log_orch_debug(&format!(
            "Host ready; waiting for deferred client join notifies (game {gid})"
        ));
        return None;
    };
    entry.join_pushes_released = true;
    let client_sid = entry.client_session_id;
    let pending_mesh_pid = entry.pending_mesh_pid.take();
    drop(m);

    Some(finish_client_join_release(
        gid,
        client_sid,
        deferred,
        pending_mesh_pid,
    ))
}

/// Client reported mesh via `updateMeshConnection`. Defer until host setup when orchestrating reset.
pub fn on_client_mesh_update(gid: i64, pid: i64) -> MeshUpdateResult {
    note_client_mesh_connected(gid);
    let mut m = orchestration().lock();
    let Some(entry) = m.get_mut(&gid) else {
        drop(m);
        return MeshUpdateResult::Push(
            mesh_active_connected_and_game_ready_pushes(gid, pid).unwrap_or_default(),
        );
    };
    if !entry.join_pushes_released {
        entry.pending_mesh_pid = Some(pid);
        super::msgsystem::log::log_orch_debug(&format!(
            "Mesh update deferred until client join notifies flush (game {gid})"
        ));
        return MeshUpdateResult::DeferredUntilHostReady;
    }
    entry.mesh_active_connected = true;
    drop(m);
    let mut out = mesh_active_connected_and_game_ready_pushes(gid, pid).unwrap_or_default();
    // JoinCompleted after InitiateConnections, not on the same tick.
    if let Ok(join_done) = super::fireframe::pushes_player_join_completed(gid) {
        if let Some(connected_at) = out
            .iter()
            .position(|p| p.component == 0x0004 && p.command == 116)
        {
            let insert_at = connected_at + 1;
            out.splice(insert_at..insert_at, join_done);
        } else {
            out.extend(join_done);
        }
    }
    MeshUpdateResult::Push(out)
}

pub fn on_cmd220_delivered_to_dedicated(gid: i64) {
    super::msgsystem::log::log_orch_milestone(&format!(
        "Dedicated received match assignment -- starting orchestration (game {gid})"
    ));
    let _ = orchestration().lock().entry(gid).or_insert_with(|| GidOrchestration {
        client_session_id: super::dedicated_pool::client_session_for_gid(gid).unwrap_or(0),
        dedicated_session_id: super::dedicated_pool::peek_dedicated_for_gid(gid),
        dedicated_host_ready: false,
        join_pushes_released: false,
        mesh_active_connected: false,
        simucloud_match_ready: false,
        pending_mesh_pid: None,
        pending_game_ready_pid: None,
        deferred_join_pushes: None,
    });

    let gid_spawn = gid;
    tokio::spawn(async move {
        let dedicated_sid = super::dedicated_pool::peek_dedicated_for_gid(gid_spawn);
        let map_path = super::game_state::get_map_path(gid_spawn);
        let mut delay_ms = 500u64;
        if let Some(sid) = dedicated_sid {
            delay_ms += super::dedicated_pool::recycle_create_game_extra_delay_ms(sid, &map_path);
        }
        if delay_ms > 500 {
            super::msgsystem::log::log_orch_debug(&format!(
                "CreateGame deferred {delay_ms}ms (recycled dedicated map switch, game {gid_spawn})"
            ));
        }
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        match super::msgsystem::simucloud::orchestrate_create_game(gid_spawn).await {
            Ok(()) => {
                super::msgsystem::log::log_orch_milestone(&format!(
                    "Match orchestration complete (game {gid_spawn})"
                ));
                if let Some((client_sid, pushes)) = on_simucloud_match_ready(gid_spawn) {
                    if !pushes.is_empty() {
                        super::fireframe::enqueue_pending_pushes(client_sid, pushes);
                        let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
                    }
                }
            }
            Err(e) => super::msgsystem::log::log_orch_milestone(&format!(
                "Match orchestration failed (game {gid_spawn}): {e} \
                 — Blaze GameReady held; client stays on lobby until retry"
            )),
        }
    });
}

pub fn simucloud_ready_gids() -> Vec<i64> {
    orchestration()
        .lock()
        .iter()
        .filter(|(_, o)| o.simucloud_match_ready)
        .map(|(&gid, _)| gid)
        .collect()
}

pub fn on_simucloud_match_ready(gid: i64) -> Option<(u64, Vec<super::fireframe::OutgoingPush>)> {
    let mut m = orchestration().lock();
    let entry = m.get_mut(&gid)?;
    entry.simucloud_match_ready = true;
    let client_sid = entry.client_session_id;
    let pid = entry
        .pending_game_ready_pid
        .take()
        .or(entry.pending_mesh_pid);
    let mesh_done = entry.mesh_active_connected || pid.is_some();
    drop(m);

    if !mesh_done {
        super::msgsystem::log::log_orch_debug(&format!(
            "SimuCloud ready; waiting for client mesh before GameReady (game {gid})"
        ));
        // If the client never reports updateMeshConnection, send ACTIVE_CONNECTED now.
        let pid = resolve_joining_client_pid(gid, client_sid);
        schedule_synthetic_client_mesh_if_needed(gid, client_sid, pid);
        return Some((client_sid, Vec::new()));
    }

    let mut out = Vec::new();
    if let Some(pid) = pid {
        if !orchestration()
            .lock()
            .get(&gid)
            .map(|o| o.mesh_active_connected)
            .unwrap_or(false)
        {
            out.extend(mesh_active_connected_pushes(gid, pid).unwrap_or_default());
        }
    }
    out.extend(game_ready_and_state_advance_pushes(gid).unwrap_or_default());
    Some((client_sid, out))
}

/// Returns `true` the first time we push PRE_GAME for this gid.
pub fn try_mark_blaze_pregame_pushed(gid: i64) -> bool {
    blaze_pregame_pushed().lock().insert(gid)
}

pub fn blaze_pregame_already_pushed(gid: i64) -> bool {
    blaze_pregame_pushed().lock().contains(&gid)
}

/// True after joinGame `NotifyGameSetup`.
pub fn clear_blaze_join_setup_pushed(gid: i64) {
    blaze_join_setup_pushed().lock().remove(&gid);
}

pub fn try_mark_blaze_join_setup_pushed(gid: i64) -> bool {
    blaze_join_setup_pushed().lock().insert(gid)
}

pub fn blaze_join_setup_already_pushed(gid: i64) -> bool {
    blaze_join_setup_pushed().lock().contains(&gid)
}

pub fn client_local_game_active(gid: i64) -> bool {
    games()
        .lock()
        .get(&gid)
        .is_some_and(|g| !g.is_standby)
}

/// Returns `true` the first time we push IN_GAME for this gid.
pub fn try_mark_blaze_ingame_pushed(gid: i64) -> bool {
    blaze_ingame_pushed().lock().insert(gid)
}

pub fn blaze_ingame_already_pushed(gid: i64) -> bool {
    blaze_ingame_pushed().lock().contains(&gid)
}

/// Returns `true` the first time we push GameReady for this gid.
pub fn try_mark_game_ready_pushed(gid: i64) -> bool {
    game_ready_pushed().lock().insert(gid)
}

/// Default map when the lobby has not chosen one.
pub const DEFAULT_MAP_PATH: &str = "Levels/SP/Alpha_Tutorial/Alpha_Tutorial";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamePhase {
    PreGame = 1,
    InGame = 2,
    PostGame = 4,
    Resetable = 7,
}

impl GamePhase {
    pub fn as_gsta(self) -> i32 {
        self as i32
    }

    pub fn label(self) -> &'static str {
        match self {
            GamePhase::PreGame => "Lobby",
            GamePhase::InGame => "InGame",
            GamePhase::PostGame => "PostGame",
            GamePhase::Resetable => "Resetable",
        }
    }
}

const NTOP_DEDICATED: i32 = 1;
const FIT_SCORE_DEFAULT: i32 = 100;

const AI_PERSONA_MIN: i64 = 9_000_000_000;
const AI_PERSONA_MAX: i64 = 9_800_000_000;

fn next_ai_persona_id() -> i64 {
    // Callers already hold `games()`; locking here deadlocks.
    rand::thread_rng().gen_range(AI_PERSONA_MIN..AI_PERSONA_MAX)
}

fn games() -> &'static Mutex<HashMap<i64, CncGame>> {
    GAMES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
pub fn clear_all_games_for_test() {
    games().lock().clear();
}

#[derive(Clone, Debug)]
pub struct CncPlayer {
    pub persona_id: i64,
    pub display_name: String,
    pub slot: i32,
    pub team: i32,
    pub is_ai: bool,
    pub ready: bool,
    pub attribs: IndexMap<String, String>,
    pub custom_data: IndexMap<String, Vec<u8>>,
    pub stat: i32,
}

#[derive(Clone, Debug)]
pub struct CncGame {
    pub gid: i64,
    pub name: String,
    pub host_persona: i64,
    pub max_players: i32,
    pub players: Vec<CncPlayer>,
    pub uuid: String,
    pub phase: GamePhase,
    pub map_path: String,
    /// Start slots for this map (`select-map`); 0 = infer from roster.
    pub start_count: i32,
    pub dedicated_session_id: Option<u64>,
    pub is_standby: bool,
    pub password: String,
    /// Special abilities / rank XP. Default on.
    pub enable_special_abilities: bool,
    /// In-match building TechTree. Default on.
    pub enable_tech_tree: bool,
    /// Oil as a second currency. Default off.
    pub enable_oil_economy: bool,
    /// Resource centers do not deplete. Default off.
    pub enable_infinite_resource_centers: bool,
    /// Unlock entire faction roster when Tech Tree is on. Default off.
    pub enable_unlock_full_faction_roster: bool,
    /// Flat `ReplicatedGameData` wire bytes last sent in `NotifyGameSetup` / `getFullGameData`.
    replicated_wire: Option<Vec<u8>>,
    /// `PROS` roster rows last sent in `NotifyGameSetup` (reused for `getFullGameData`).
    pros_wire: Option<Vec<Vec<u8>>>,
}

fn join_password_auth() -> &'static Mutex<HashMap<(i64, i64), Instant>> {
    JOIN_PASSWORD_AUTH.get_or_init(|| Mutex::new(HashMap::new()))
}

fn purge_expired_join_auth(map: &mut HashMap<(i64, i64), Instant>) {
    let now = Instant::now();
    map.retain(|_, until| *until > now);
}

fn clear_join_auth_for_gid(gid: i64) {
    let mut map = join_password_auth().lock();
    map.retain(|(g, _), _| *g != gid);
}

pub fn is_password_protected(gid: i64) -> bool {
    games()
        .lock()
        .get(&gid)
        .map(|g| !g.password.is_empty())
        .unwrap_or(false)
}

pub fn set_game_password(gid: i64, persona_id: i64, password: &str) -> serde_json::Value {
    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return serde_json::json!({ "ok": false, "error": "game not found", "gid": gid });
    };
    if persona_id > 0 && game.host_persona > 0 && persona_id != game.host_persona {
        return serde_json::json!({
            "ok": false,
            "error": "host only",
            "gid": gid,
            "admin": game.host_persona,
        });
    }
    let trimmed = password.trim();
    game.password = if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    };
    let protected = !game.password.is_empty();
    let secret = game.password.clone();
    let host = game.host_persona;
    let dedicated_sid = game.dedicated_session_id;
    if !protected {
        drop(m);
        clear_join_auth_for_gid(gid);
    } else {
        drop(m);
    }
    publish_password_attribs(gid, protected, if protected { Some(secret.as_str()) } else { None }, host, dedicated_sid, persona_id);
    serde_json::json!({
        "ok": true,
        "gid": gid,
        "passwordProtected": protected,
        "attr": ATTR_PASSWORD_FLAG,
    })
}

/// Host match-progression flags. Defaults on when the game row is created.
pub fn set_match_options(
    gid: i64,
    persona_id: i64,
    enable_special_abilities: Option<bool>,
    enable_tech_tree: Option<bool>,
    enable_oil_economy: Option<bool>,
    enable_infinite_resource_centers: Option<bool>,
    enable_unlock_full_faction_roster: Option<bool>,
) -> serde_json::Value {
    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return serde_json::json!({ "ok": false, "error": "game not found", "gid": gid });
    };
    if persona_id > 0 && game.host_persona > 0 && persona_id != game.host_persona {
        return serde_json::json!({
            "ok": false,
            "error": "host only",
            "gid": gid,
            "admin": game.host_persona,
        });
    }
    if let Some(v) = enable_special_abilities {
        game.enable_special_abilities = v;
    }
    if let Some(v) = enable_tech_tree {
        game.enable_tech_tree = v;
    }
    let _ = enable_oil_economy;
    game.enable_oil_economy = false;
    if let Some(v) = enable_infinite_resource_centers {
        game.enable_infinite_resource_centers = v;
    }
    if let Some(v) = enable_unlock_full_faction_roster {
        game.enable_unlock_full_faction_roster = v;
    }
    serde_json::json!({
        "ok": true,
        "gid": gid,
        "enableSpecialAbilities": game.enable_special_abilities,
        "enableTechTree": game.enable_tech_tree,
        "enableOilEconomy": game.enable_oil_economy,
        "enableInfiniteResourceCenters": game.enable_infinite_resource_centers,
        "enableUnlockFullFactionRoster": game.enable_unlock_full_faction_roster,
    })
}

pub fn match_options(gid: i64) -> (bool, bool, bool, bool, bool) {
    games()
        .lock()
        .get(&gid)
        .map(|g| {
            (
                g.enable_special_abilities,
                g.enable_tech_tree,
                g.enable_oil_economy,
                g.enable_infinite_resource_centers,
                g.enable_unlock_full_faction_roster,
            )
        })
        .unwrap_or((true, true, false, false, false))
}

/// Copy host lobby progression flags onto a dedicated gid (same pattern as pending map).
pub fn adopt_host_lobby_match_options_into(dedicated_gid: i64) {
    let host = host_persona();
    let source = {
        let m = games().lock();
        m.iter()
            .find(|(g, game)| **g != dedicated_gid && game.host_persona == host)
            .map(|(_, game)| {
                (
                    game.enable_special_abilities,
                    game.enable_tech_tree,
                    game.enable_oil_economy,
                    game.enable_infinite_resource_centers,
                    game.enable_unlock_full_faction_roster,
                )
            })
    };
    let Some((special, tech, oil, infinite, full_roster)) = source else {
        return;
    };
    if let Some(game) = games().lock().get_mut(&dedicated_gid) {
        game.enable_special_abilities = special;
        game.enable_tech_tree = tech;
        game.enable_oil_economy = false;
        game.enable_infinite_resource_centers = infinite;
        game.enable_unlock_full_faction_roster = full_roster;
        let _ = oil;
    }
}

pub fn verify_game_password(gid: i64, persona_id: i64, password: &str) -> serde_json::Value {
    let (expected, dedicated_sid, host) = {
        let m = games().lock();
        match m.get(&gid) {
            Some(g) => (g.password.clone(), g.dedicated_session_id, g.host_persona),
            None => {
                return serde_json::json!({ "ok": false, "error": "game not found", "gid": gid });
            }
        }
    };
    if expected.is_empty() {
        return serde_json::json!({
            "ok": true,
            "gid": gid,
            "passwordProtected": false,
            "authorized": true,
        });
    }
    if password.trim() != expected {
        return serde_json::json!({
            "ok": false,
            "error": "wrong password",
            "gid": gid,
            "passwordProtected": true,
        });
    }
    if persona_id > 0 {
        let mut map = join_password_auth().lock();
        purge_expired_join_auth(&mut map);
        map.insert((gid, persona_id), Instant::now() + JOIN_PASSWORD_AUTH_TTL);
        push_password_secret_to_persona(gid, persona_id, &expected);
    }
    let _ = (dedicated_sid, host);
    serde_json::json!({
        "ok": true,
        "gid": gid,
        "passwordProtected": true,
        "authorized": true,
    })
}

fn sessions_for_persona(persona_id: i64) -> Vec<u64> {
    if persona_id <= 0 {
        return Vec::new();
    }
    crate::session::blaze_sessions::list_sessions()
        .into_iter()
        .filter(|s| s.persona_id == Some(persona_id as u64))
        .map(|s| s.id)
        .collect()
}

pub fn blaze_session_for_persona(persona_id: i64) -> Option<u64> {
    sessions_for_persona(persona_id).into_iter().next()
}

fn push_password_secret_to_persona(gid: i64, persona_id: i64, secret: &str) {
    let Ok(pushes) = super::fireframe::pushes_password_attrib(gid, true, Some(secret)) else {
        return;
    };
    for sid in sessions_for_persona(persona_id) {
        super::fireframe::enqueue_pending_pushes(sid, pushes.clone());
    }
    let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
}

fn publish_password_attribs(
    gid: i64,
    protected: bool,
    secret: Option<&str>,
    host_persona: i64,
    dedicated_sid: Option<u64>,
    setter_persona: i64,
) {
    if let Ok(flag_pushes) = super::fireframe::pushes_password_attrib(gid, protected, None) {
        let human_pids: Vec<i64> = players_for_gid(gid)
            .into_iter()
            .filter(|p| !p.is_ai)
            .map(|p| p.persona_id)
            .collect();
        let mut sids = std::collections::HashSet::new();
        for pid in human_pids {
            for sid in sessions_for_persona(pid) {
                sids.insert(sid);
            }
        }
        for pid in [host_persona, setter_persona] {
            for sid in sessions_for_persona(pid) {
                sids.insert(sid);
            }
        }
        for sid in sids {
            super::fireframe::enqueue_pending_pushes(sid, flag_pushes.clone());
        }
    }

    if let Some(secret) = secret.filter(|s| !s.is_empty()) {
        if let Ok(secret_pushes) = super::fireframe::pushes_password_attrib(gid, true, Some(secret)) {
            if let Some(dsid) = dedicated_sid {
                let mut dedicated = secret_pushes.clone();
                for p in &mut dedicated {
                    p.info_log_line = p
                        .info_log_line
                        .replace("[Blaze→Client]", "[Blaze→Server]");
                }
                super::fireframe::enqueue_pending_pushes(dsid, dedicated);
            }
            for pid in [host_persona, setter_persona] {
                for sid in sessions_for_persona(pid) {
                    super::fireframe::enqueue_pending_pushes(sid, secret_pushes.clone());
                }
            }
        }
    } else if !protected {
        if let (Some(dsid), Ok(mut clear_pushes)) = (
            dedicated_sid,
            super::fireframe::pushes_password_attrib(gid, false, None),
        ) {
            for p in &mut clear_pushes {
                p.info_log_line = p
                    .info_log_line
                    .replace("[Blaze→Client]", "[Blaze→Server]");
            }
            super::fireframe::enqueue_pending_pushes(dsid, clear_pushes);
        }
    }
    let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
}

pub fn apply_password_flag_to_attrs(gid: i64, attrs: &mut IndexMap<String, String>) {
    if is_password_protected(gid) {
        attrs.insert(ATTR_PASSWORD_FLAG.to_string(), "1".to_string());
    } else {
        attrs.shift_remove(ATTR_PASSWORD_FLAG);
    }
}

pub fn apply_password_secret_to_attrs(gid: i64, attrs: &mut IndexMap<String, String>) {
    let secret = games()
        .lock()
        .get(&gid)
        .map(|g| g.password.clone())
        .unwrap_or_default();
    if secret.is_empty() {
        attrs.shift_remove(ATTR_PASSWORD_SECRET);
        attrs.shift_remove(ATTR_PASSWORD_FLAG);
    } else {
        attrs.insert(ATTR_PASSWORD_FLAG.to_string(), "1".to_string());
        attrs.insert(ATTR_PASSWORD_SECRET.to_string(), secret);
    }
}

fn has_join_password_auth(gid: i64, persona_id: i64) -> bool {
    if persona_id <= 0 {
        return false;
    }
    let mut map = join_password_auth().lock();
    purge_expired_join_auth(&mut map);
    map.get(&(gid, persona_id))
        .map(|until| *until > Instant::now())
        .unwrap_or(false)
}

pub fn join_password_allowed(gid: i64, persona_id: i64) -> bool {
    let m = games().lock();
    let Some(game) = m.get(&gid) else {
        return true;
    };
    if game.password.is_empty() {
        return true;
    }
    if game.players.iter().any(|p| !p.is_ai && p.persona_id == persona_id) {
        return true;
    }
    let humans = game.players.iter().filter(|p| !p.is_ai).count();
    if humans == 0 || game.host_persona == 0 {
        return true;
    }
    drop(m);
    has_join_password_auth(gid, persona_id)
}

pub fn dedicated_session_for_gid(gid: i64) -> Option<u64> {
    games()
        .lock()
        .get(&gid)
        .and_then(|g| g.dedicated_session_id)
}

pub fn ensure_standby_game(gid: i64, hostname: &str, dedicated_session_id: u64) {
    let mut m = games().lock();
    if let Some(game) = m.get_mut(&gid) {
        game.name = hostname.to_string();
        game.dedicated_session_id = Some(dedicated_session_id);
        game.is_standby = true;
        game.phase = GamePhase::PreGame;
        if game.map_path.is_empty() {
            game.map_path = String::new();
        }
        return;
    }
    m.insert(
        gid,
        CncGame {
            gid,
            name: hostname.to_string(),
            host_persona: 0,
            max_players: 8,
            players: Vec::new(),
            uuid: new_uuid_v4_string(),
            phase: GamePhase::PreGame,
            map_path: String::new(),
            start_count: 0,
            dedicated_session_id: Some(dedicated_session_id),
            is_standby: true,
            password: String::new(),
            enable_special_abilities: true,
            enable_tech_tree: true,
            enable_oil_economy: false,
            enable_infinite_resource_centers: false,
            enable_unlock_full_faction_roster: false,
            replicated_wire: None,
            pros_wire: None,
        },
    );
}

pub fn reset_standby_after_pool_return(gid: i64) {
    let dedicated_sid = games()
        .lock()
        .get(&gid)
        .and_then(|g| g.dedicated_session_id);
    let restore_name = dedicated_sid.and_then(|sid| {
        crate::client::cnc::dedicated_pool::get_entry(sid)
            .map(|e| crate::client::cnc::dedicated_pool::browser_server_name(&e))
    });

    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return;
    };
    if !game.is_standby {
        return;
    }
    game.players.clear();
    game.host_persona = 0;
    game.phase = GamePhase::PreGame;
    game.map_path.clear();
    game.start_count = 0;
    game.password.clear();
    game.enable_special_abilities = true;
    game.enable_tech_tree = true;
    game.enable_oil_economy = false;
    game.enable_infinite_resource_centers = false;
    game.enable_unlock_full_faction_roster = false;
    game.replicated_wire = None;
    game.pros_wire = None;
    if let Some(name) = restore_name {
        game.name = name;
    }
    clear_blaze_push_flags(gid);
    drop(m);
    clear_join_auth_for_gid(gid);
}

pub fn force_standby_reset(gid: i64) {
    {
        let mut m = games().lock();
        if let Some(game) = m.get_mut(&gid) {
            game.is_standby = true;
        } else {
            return;
        }
    }
    reset_standby_after_pool_return(gid);
}

fn host_persona() -> i64 {
    let session = get_user_session();
    if session.persona_id == 0 {
        1000
    } else {
        session.persona_id as i64
    }
}

fn host_display_name() -> String {
    let session = get_user_session();
    if session.display_name.is_empty() {
        "Player".to_string()
    } else {
        session.display_name.clone()
    }
}

fn new_uuid_v4_string() -> String {
    let mut b = [0u8; 16];
    rand::thread_rng().fill(&mut b);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13],
        b[14], b[15],
    )
}

fn sane_uuid(s: &str) -> bool {
    !s.is_empty() && s != "." && s.len() >= 8
}

fn resolve_uuid(request_payload: &[u8]) -> String {
    TdfEncoder::find_string_field(request_payload, "UUID")
        .filter(|s| sane_uuid(s))
        .or_else(|| {
            TdfEncoder::scan_first_string_field(request_payload, "UUID").filter(|s| sane_uuid(s))
        })
        .unwrap_or_else(new_uuid_v4_string)
}

pub fn set_replicated_wire_fields(gid: i64, fields: Vec<u8>) {
    if let Some(game) = games().lock().get_mut(&gid) {
        game.replicated_wire = Some(fields);
    }
}

pub fn set_pros_wire_fields(gid: i64, entries: Vec<Vec<u8>>) {
    if let Some(game) = games().lock().get_mut(&gid) {
        game.pros_wire = Some(entries);
    }
}

pub fn replicated_wire_fields(gid: i64) -> Option<Vec<u8>> {
    games().lock().get(&gid).and_then(|g| g.replicated_wire.clone())
}

pub fn pros_wire_fields(gid: i64) -> Option<Vec<Vec<u8>>> {
    games().lock().get(&gid).and_then(|g| g.pros_wire.clone())
}

/// Ensure a stub game row exists when the client calls `getFullGameData` before reset/create.
pub fn ensure_game_stub(gid: i64) {
    if games().lock().contains_key(&gid) {
        return;
    }
    seed_from_reset(&[], gid);
}

/// APA_ClassicGeneral ServerId (StaticData/Generals + FactionAPA GeneralsToLoad).
const DEFAULT_GENERAL_APA_CLASSIC: &str = "2914080600";
const DEFAULT_GENERAL_APA_TUTORIAL: &str = "497011786";
const DEFAULT_GENERAL_EU_TUTORIAL: &str = "3463861546";
const DEFAULT_GENERAL_EU_CLASSIC: &str = "232716472";
/// GLA_ClassicGeneral ServerId — default AI opponent.
const DEFAULT_GENERAL_GLA_CLASSIC: &str = "580378690";
const DEFAULT_GENERAL_GLA_TUTORIAL: &str = "145977592";

fn is_alpha_tutorial_map(map_path: &str) -> bool {
    map_path
        .to_ascii_lowercase()
        .contains("alpha_tutorial")
}

/// Default Blaze player ATTR for a human slot (CreateGame / PROS).
/// MsgSys ServerHello also needs `_general` = RtsGeneral.ServerId (HashId).
fn default_human_attribs(slot: i32, team: i32) -> IndexMap<String, String> {
    default_human_attribs_for_map("", slot, team)
}

fn default_human_attribs_for_map(
    map_path: &str,
    _slot: i32,
    team: i32,
) -> IndexMap<String, String> {
    let (faction, general) = if is_alpha_tutorial_map(map_path) {
        ("EU", DEFAULT_GENERAL_EU_TUTORIAL)
    } else {
        ("APA", DEFAULT_GENERAL_APA_CLASSIC)
    };
    let mut attribs = IndexMap::new();
    attribs.insert("_faction".to_string(), faction.to_string());
    attribs.insert("_isai".to_string(), "0".to_string());
    attribs.insert("_team".to_string(), team.max(1).to_string());
    attribs.insert("_startpoint".to_string(), "0".to_string());
    attribs.insert("_general".to_string(), general.to_string());
    attribs
}

fn pending_player_attrs() -> &'static Mutex<HashMap<i64, HashMap<i64, IndexMap<String, String>>>> {
    PENDING_PLAYER_ATTRS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn apply_attr_to_player(player: &mut CncPlayer, key: &str, value: &str) {
    player.attribs.insert(key.to_string(), value.to_string());
    match key {
        "_team" => {
            if let Ok(t) = value.parse::<i32>() {
                player.team = t.max(1);
            }
        }
        "_startpoint" => {
            if let Ok(s) = value.parse::<i32>() {
                // 0 = lobby random (`?`); do not remap Blaze slot.
                if s > 0 {
                    player.slot = (s - 1).max(0);
                }
            }
        }
        "_isai" => player.is_ai = value == "1" || value.eq_ignore_ascii_case("true"),
        _ => {}
    }
}

fn attrs_mark_ai(attrs: &IndexMap<String, String>) -> bool {
    attrs
        .get("_isai")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn startpoint_from_attrs(attrs: &IndexMap<String, String>) -> i32 {
    attrs
        .get("_startpoint")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0)
}

/// Live roster plus pending lobby overlays.
pub fn effective_startpoint_for_player(gid: i64, player: &CncPlayer) -> i32 {
    let live = startpoint_from_attrs(&player.attribs);
    if live > 0 {
        return live;
    }
    let pending = pending_player_attrs().lock();
    let Some(by_pid) = pending.get(&gid) else {
        return live;
    };
    for pid in [0i64, player.persona_id] {
        if pid == 0 && player.is_ai {
            continue;
        }
        if let Some(attrs) = by_pid.get(&pid) {
            let sp = startpoint_from_attrs(attrs);
            if sp > 0 {
                return sp;
            }
        }
    }
    live
}

fn infer_startpoint_capacity(
    pending: Option<&HashMap<i64, IndexMap<String, String>>>,
    game: &CncGame,
) -> i32 {
    let mut max_id = 0i32;
    for player in &game.players {
        let sp = startpoint_from_attrs(&player.attribs);
        if sp > 0 {
            max_id = max_id.max(sp);
        }
    }
    if let Some(by_pid) = pending {
        for attrs in by_pid.values() {
            let sp = startpoint_from_attrs(attrs);
            if sp > 0 {
                max_id = max_id.max(sp);
            }
        }
    }
    max_id.max(game.players.len() as i32).max(1)
}

fn pending_start_count(gid: i64) -> i32 {
    PENDING_MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .get(&gid)
        .map(|info| info.start_count)
        .unwrap_or(0)
}

fn startpoint_capacity_for_game(
    gid: i64,
    pending: Option<&HashMap<i64, IndexMap<String, String>>>,
    game: &CncGame,
) -> i32 {
    if game.start_count > 0 {
        return game.start_count;
    }
    let pending_count = pending_start_count(gid);
    if pending_count > 0 {
        return pending_count;
    }
    infer_startpoint_capacity(pending, game)
}

/// Resolve lobby `?` and duplicate picks to unique start ids before CreateGame.
pub fn resolve_startpoints_before_create(gid: i64) {
    let pending_snapshot = pending_player_attrs().lock().get(&gid).cloned();
    let map_path = get_map_path(gid);

    let mut games_guard = games().lock();
    let Some(game) = games_guard.get_mut(&gid) else {
        return;
    };
    let capacity = startpoint_capacity_for_game(
        gid,
        pending_snapshot.as_ref(),
        game,
    );

    let mut effective: Vec<i32> = Vec::with_capacity(game.players.len());
    for player in &game.players {
        let mut sp = startpoint_from_attrs(&player.attribs);
        if sp <= 0 {
            if let Some(ref pending) = pending_snapshot {
                for pid in [0i64, player.persona_id] {
                    if pid == 0 && player.is_ai {
                        continue;
                    }
                    if let Some(attrs) = pending.get(&pid) {
                        let psp = startpoint_from_attrs(attrs);
                        if psp > 0 {
                            sp = psp;
                            break;
                        }
                    }
                }
            }
        }
        effective.push(sp);
    }

    let mut used: HashSet<i32> = HashSet::new();
    let mut needs_pick: Vec<usize> = Vec::new();
    for (idx, sp) in effective.iter().enumerate() {
        if *sp > 0 && used.insert(*sp) {
            continue;
        }
        needs_pick.push(idx);
    }

    let mut rng = rand::thread_rng();
    let mut free: Vec<i32> = (1..=capacity).filter(|id| !used.contains(id)).collect();
    for i in (1..free.len()).rev() {
        let j = rng.gen_range(0..=i);
        free.swap(i, j);
    }

    for (pick_idx, player_idx) in needs_pick.iter().enumerate() {
        let sp = free
            .get(pick_idx)
            .copied()
            .or_else(|| (1..=capacity).find(|id| !used.contains(id)))
            .unwrap_or(1);
        used.insert(sp);
        effective[*player_idx] = sp;
    }

    let picks: Vec<i32> = effective.clone();
    for (idx, sp) in effective.into_iter().enumerate() {
        if let Some(player) = game.players.get_mut(idx) {
            let sp = if sp > 0 {
                sp
            } else {
                effective_startpoint_for_player(gid, player).max(1)
            };
            player
                .attribs
                .insert("_startpoint".to_string(), sp.to_string());
            player.slot = (sp - 1).max(0);
        }
    }

    tracing::info!(
        target: "cnc",
        "[CNC] startpoints resolved gid={gid} map=\"{map_path}\" capacity={capacity} picks={picks:?}"
    );
}

/// Clear lobby `_startpoint` picks after CreateGame has snapshotted the roster.
pub fn flush_lobby_startpoints(gid: i64) {
    {
        let mut m = games().lock();
        if let Some(game) = m.get_mut(&gid) {
            for player in &mut game.players {
                player
                    .attribs
                    .insert("_startpoint".to_string(), "0".to_string());
            }
        }
    }
    {
        let mut pending = pending_player_attrs().lock();
        if let Some(by_pid) = pending.get_mut(&gid) {
            for attrs in by_pid.values_mut() {
                if attrs.contains_key("_startpoint") {
                    attrs.insert("_startpoint".to_string(), "0".to_string());
                }
            }
        }
    }
    tracing::info!(
        target: "cnc",
        "[CNC] flushed lobby startpoints gid={gid}"
    );
}

/// Clamp lobby startpoint picks to `1..=capacity`.
pub fn clamp_lobby_startpoints(gid: i64, capacity: i32) {
    if capacity <= 0 {
        return;
    }
    let mut changed = false;
    {
        let mut m = games().lock();
        if let Some(game) = m.get_mut(&gid) {
            for player in &mut game.players {
                let sp = startpoint_from_attrs(&player.attribs);
                if sp > capacity {
                    player
                        .attribs
                        .insert("_startpoint".to_string(), "0".to_string());
                    changed = true;
                }
            }
        }
    }
    {
        let mut pending = pending_player_attrs().lock();
        if let Some(by_pid) = pending.get_mut(&gid) {
            for attrs in by_pid.values_mut() {
                let sp = startpoint_from_attrs(attrs);
                if sp > capacity {
                    attrs.insert("_startpoint".to_string(), "0".to_string());
                    changed = true;
                }
            }
        }
    }
    if changed {
        tracing::info!(
            target: "cnc",
            "[CNC] clamped lobby startpoints gid={gid} capacity={capacity}"
        );
    }
}

/// Lobby AI slots used to POST `pid=0`. Negative ids never collide with a Blaze persona.
fn synthetic_ai_persona(startpoint: i32) -> i64 {
    let sp = if startpoint > 0 { startpoint as i64 } else { 1 };
    -(1000 + sp)
}

fn strip_poisoned_host_ai_pending(gid: i64) {
    let mut pending = pending_player_attrs().lock();
    if let Some(by_pid) = pending.get_mut(&gid) {
        if let Some(slot) = by_pid.get_mut(&0) {
            if attrs_mark_ai(slot) {
                slot.shift_remove("_isai");
            }
        }
    }
}

fn merge_pending_into_player(gid: i64, player: &mut CncPlayer, map_path: &str) {
    // Overlay pid=0 (pre-auth host) then exact persona. Do not copy `_isai=1` onto a human.
    let overlays = {
        let pending = pending_player_attrs().lock();
        let mut layers = Vec::new();
        if let Some(by_pid) = pending.get(&gid) {
            if let Some(a) = by_pid.get(&0) {
                if !attrs_mark_ai(a) || player.is_ai {
                    layers.push(a.clone());
                }
            }
            if player.persona_id != 0 {
                if let Some(a) = by_pid.get(&player.persona_id) {
                    layers.push(a.clone());
                }
            }
        }
        layers
    };
    for attrs in &overlays {
        for (k, v) in attrs {
            apply_attr_to_player(player, k, v);
        }
    }
    ensure_general_attr(player, map_path);
    if !overlays.is_empty() {
        crate::debug_println!(
            "[CNC] merge pending attrs gid={} pid={} faction={:?} general={:?}",
            gid,
            player.persona_id,
            player.attribs.get("_faction"),
            player.attribs.get("_general")
        );
    }
}

fn ensure_general_attr(player: &mut CncPlayer, map_path: &str) {
    let map = if map_path.is_empty() {
        DEFAULT_MAP_PATH
    } else {
        map_path
    };
    let faction = player
        .attribs
        .get("_faction")
        .map(|s| s.as_str())
        .unwrap_or("APA");
    if is_alpha_tutorial_map(&map) {
        apply_attr_to_player(player, "_general", tutorial_general_for_faction(faction));
        return;
    }

    let gen_missing = player
        .attribs
        .get("_general")
        .map(|s| s.trim().is_empty() || s.trim() == "0")
        .unwrap_or(true);
    if !gen_missing {
        return;
    }
    if matches!(
        faction.trim().to_ascii_uppercase().as_str(),
        "USA" | "NONE" | "" | "0"
    ) {
        apply_attr_to_player(player, "_faction", "APA");
        apply_attr_to_player(player, "_general", DEFAULT_GENERAL_APA_CLASSIC);
    } else {
        apply_attr_to_player(player, "_general", default_general_for_faction(faction));
    }
}

fn write_pending_player_attrs(gid: i64, persona_id: i64, attrs: &IndexMap<String, String>) {
    let mut pending = pending_player_attrs().lock();
    let by_pid = pending.entry(gid).or_default();
    let slot = by_pid.entry(persona_id).or_default();
    for (k, v) in attrs {
        slot.insert(k.clone(), v.clone());
    }
}

fn apply_pending_attrs_to_live_game(gid: i64, persona_id: i64, attrs: &IndexMap<String, String>) {
    let map = get_map_path(gid);
    let map_path = if map.is_empty() {
        DEFAULT_MAP_PATH
    } else {
        map.as_str()
    };
    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return;
    };
    if attrs_mark_ai(attrs) {
        let startpoint = startpoint_from_attrs(attrs);
        let mut pid = if persona_id < 0 {
            persona_id
        } else if persona_id > 0 && persona_id != game.host_persona {
            persona_id
        } else {
            synthetic_ai_persona(startpoint)
        };
        if pid == game.host_persona || pid == 0 {
            pid = synthetic_ai_persona(startpoint);
        }
        if let Some(idx) = game.players.iter().position(|p| p.persona_id == pid) {
            if let Some(player) = game.players.get_mut(idx) {
                if player.persona_id != game.host_persona || player.is_ai {
                    for (k, v) in attrs {
                        apply_attr_to_player(player, k, v);
                    }
                    ensure_general_attr(player, map_path);
                    return;
                }
            }
        }
        let team = attrs
            .get("_team")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(2)
            .max(1);
        let slot = if startpoint > 0 {
            startpoint - 1
        } else {
            game.players.len() as i32
        };
        let faction = attrs
            .get("_faction")
            .cloned()
            .unwrap_or_else(|| "GLA".to_string());
        let mut player = CncPlayer {
            persona_id: pid,
            display_name: "AI".to_string(),
            slot,
            team,
            is_ai: true,
            ready: true,
            attribs: default_ai_attribs(slot, team, &faction),
            custom_data: IndexMap::new(),
            stat: PROS_STAT_ACTIVE_CONNECTING,
        };
        for (k, v) in attrs {
            apply_attr_to_player(&mut player, k, v);
        }
        ensure_general_attr(&mut player, map_path);
        game.players.push(player);
        return;
    }
    let targets: Vec<usize> = if persona_id == 0 {
        game.players
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                !p.is_ai && (p.persona_id == game.host_persona || game.host_persona == 0)
            })
            .map(|(i, _)| i)
            .take(1)
            .collect()
    } else {
        game.players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.persona_id == persona_id && !p.is_ai)
            .map(|(i, _)| i)
            .collect()
    };
    for i in targets {
        if let Some(player) = game.players.get_mut(i) {
            for (k, v) in attrs {
                if k == "_isai" {
                    continue;
                }
                apply_attr_to_player(player, k, v);
            }
            ensure_general_attr(player, map_path);
        }
    }
}

fn reapply_all_pending_attrs(gid: i64) {
    let by_pid = pending_player_attrs()
        .lock()
        .get(&gid)
        .cloned()
        .unwrap_or_default();
    for (pid, attrs) in by_pid {
        apply_pending_attrs_to_live_game(gid, pid, &attrs);
    }
}

/// Record lobby player attrs. `persona_id == 0` is the host unless `_isai=1`.
pub fn set_pending_player_attrs(gid: i64, mut persona_id: i64, attrs: IndexMap<String, String>) {
    if attrs.is_empty() {
        return;
    }
    if attrs_mark_ai(&attrs) && persona_id == 0 {
        persona_id = synthetic_ai_persona(startpoint_from_attrs(&attrs));
        strip_poisoned_host_ai_pending(gid);
    }
    write_pending_player_attrs(gid, persona_id, &attrs);
    apply_pending_attrs_to_live_game(gid, persona_id, &attrs);
}

pub fn adopt_host_lobby_pending_attrs_into(dedicated_gid: i64) {
    let host = host_persona();
    let lobby_gids: Vec<i64> = games()
        .lock()
        .iter()
        .filter(|(g, game)| **g != dedicated_gid && game.host_persona == host)
        .map(|(g, _)| *g)
        .collect();
    let adopted: Option<(i64, HashMap<i64, IndexMap<String, String>>)> = {
        let pending = pending_player_attrs().lock();
        let dedicated_empty = pending
            .get(&dedicated_gid)
            .map(|by_pid| by_pid.is_empty())
            .unwrap_or(true);
        if !dedicated_empty {
            return;
        }
        lobby_gids.into_iter().find_map(|lobby_gid| {
            pending.get(&lobby_gid).and_then(|by_pid| {
                if by_pid.is_empty() {
                    None
                } else {
                    Some((lobby_gid, by_pid.clone()))
                }
            })
        })
    };
    let Some((lobby_gid, by_pid)) = adopted else {
        return;
    };
    {
        let mut pending = pending_player_attrs().lock();
        pending.insert(dedicated_gid, by_pid.clone());
    }
    for (pid, attrs) in &by_pid {
        apply_pending_attrs_to_live_game(dedicated_gid, *pid, attrs);
    }
    tracing::info!(
        target: "cnc",
        "[CNC] player-attrs adopted lobby gid={} → dedicated gid={} pids={}",
        lobby_gid,
        dedicated_gid,
        by_pid.len()
    );
}

/// Snapshot used by `/cnc/player-probe` — validates map + per-player lobby/CreateGame fields.
pub fn player_data_probe(gid: i64) -> serde_json::Value {
    let map = get_map_path(gid);
    let pending = pending_player_attrs()
        .lock()
        .get(&gid)
        .cloned()
        .unwrap_or_default();
    let game = get_game(gid);
    let mut issues: Vec<String> = Vec::new();
    if map.is_empty() {
        issues.push("map_path empty (lobby /cnc/select-map not applied)".into());
    }
    let mut players_json = Vec::new();
    if let Some(ref g) = game {
        for p in &g.players {
            let faction = p
                .attribs
                .get("_faction")
                .cloned()
                .unwrap_or_else(|| "(missing)".into());
            let team = p
                .attribs
                .get("_team")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(p.team);
            let start = p
                .attribs
                .get("_startpoint")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(p.slot + 1);
            let general = p
                .attribs
                .get("_general")
                .cloned()
                .unwrap_or_else(|| "0".into());
            let is_ai = p
                .attribs
                .get("_isai")
                .map(|s| s == "1")
                .unwrap_or(p.is_ai);
            let mut ok = true;
            let mut notes = Vec::new();
            if faction == "(missing)" || faction.is_empty() || faction == "None" || faction == "0"
            {
                ok = false;
                notes.push("faction missing/None");
                issues.push(format!("player {} faction invalid", p.persona_id));
            }
            if team < 1 {
                ok = false;
                notes.push("team < 1");
                issues.push(format!("player {} team={}", p.persona_id, team));
            }
            if start < 0 {
                ok = false;
                notes.push("startpoint < 0");
                issues.push(format!("player {} startpoint={}", p.persona_id, start));
            } else if start == 0 {
                notes.push("startpoint random");
            }
            players_json.push(serde_json::json!({
                "persona_id": p.persona_id,
                "name": p.display_name,
                "slot": p.slot,
                "team": team,
                "startpoint": start,
                "faction": faction,
                "general": general,
                "is_ai": is_ai,
                "stat": p.stat,
                "spawned_hint": p.stat >= PROS_STAT_ACTIVE_CONNECTING,
                "ok": ok,
                "notes": notes,
            }));
        }
    } else {
        issues.push("no CncGame yet (attrs pending only)".into());
    }
    let pending_json: Vec<_> = pending
        .iter()
        .map(|(pid, attrs)| {
            serde_json::json!({
                "persona_id": pid,
                "attrs": attrs,
            })
        })
        .collect();
    let (enable_special_abilities, enable_tech_tree, enable_oil_economy, enable_infinite_resource_centers, enable_unlock_full_faction_roster) = game
        .as_ref()
        .map(|g| {
            (
                g.enable_special_abilities,
                g.enable_tech_tree,
                g.enable_oil_economy,
                g.enable_infinite_resource_centers,
                g.enable_unlock_full_faction_roster,
            )
        })
        .unwrap_or((true, true, false, false, false));
    serde_json::json!({
        "ok": issues.is_empty() && game.is_some(),
        "gid": gid,
        "map_path": map,
        "phase": format!("{:?}", get_phase(gid)),
        "enableSpecialAbilities": enable_special_abilities,
        "enableTechTree": enable_tech_tree,
        "enableOilEconomy": enable_oil_economy,
        "enableInfiniteResourceCenters": enable_infinite_resource_centers,
        "enableUnlockFullFactionRoster": enable_unlock_full_faction_roster,
        "player_count": players_json.len(),
        "players": players_json,
        "pending_attrs": pending_json,
        "issues": issues,
    })
}

fn tutorial_general_for_faction(faction: &str) -> &'static str {
    match faction.trim().to_ascii_uppercase().as_str() {
        "APA" | "CHINA" | "CHI" => DEFAULT_GENERAL_APA_TUTORIAL,
        "ESC" | "EU" => DEFAULT_GENERAL_EU_TUTORIAL,
        "GLA" => DEFAULT_GENERAL_GLA_TUTORIAL,
        _ => DEFAULT_GENERAL_EU_TUTORIAL,
    }
}

fn default_general_for_faction(faction: &str) -> &'static str {
    match faction.trim().to_ascii_uppercase().as_str() {
        "APA" | "CHINA" | "CHI" => DEFAULT_GENERAL_APA_CLASSIC,
        "ESC" | "EU" => DEFAULT_GENERAL_EU_CLASSIC,
        "GLA" => DEFAULT_GENERAL_GLA_CLASSIC,
        _ => {
            let map = get_map_path(resolve_host_reset_gid());
            if is_alpha_tutorial_map(&map) || (map.is_empty() && is_alpha_tutorial_map(DEFAULT_MAP_PATH))
            {
                DEFAULT_GENERAL_EU_TUTORIAL
            } else {
                DEFAULT_GENERAL_APA_CLASSIC
            }
        }
    }
}

fn default_ai_attribs(slot: i32, team: i32, faction: &str) -> IndexMap<String, String> {
    let mut attribs = IndexMap::new();
    attribs.insert("_isai".to_string(), "1".to_string());
    attribs.insert("_faction".to_string(), faction.to_string());
    attribs.insert("_team".to_string(), team.max(1).to_string());
    attribs.insert("_startpoint".to_string(), (slot + 1).max(1).to_string());
    attribs.insert(
        "_general".to_string(),
        default_general_for_faction(faction).to_string(),
    );
    attribs
}

pub fn seed_from_reset(request_payload: &[u8], gid: i64) {
    clear_blaze_one_shot_flags(gid);
    let gnam = TdfEncoder::find_string_field(request_payload, "GNAM")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Skirmish".to_string());
    let host = games()
        .lock()
        .get(&gid)
        .map(|g| g.host_persona)
        .filter(|&h| h > 0)
        .unwrap_or_else(host_persona);
    let host_name = host_display_name();
    let uuid = resolve_uuid(request_payload);
    let map_path = get_map_path(gid);
    let map_for_defaults = if map_path.is_empty() {
        DEFAULT_MAP_PATH
    } else {
        map_path.as_str()
    };
    let mut host_player = CncPlayer {
        persona_id: host,
        display_name: host_name,
        slot: 0,
        team: 1,
        is_ai: false,
        ready: true,
        attribs: default_human_attribs_for_map(map_for_defaults, 0, 1),
        custom_data: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    merge_pending_into_player(gid, &mut host_player, map_for_defaults);
    let (dedicated_session_id, password, enable_special_abilities, enable_tech_tree, enable_oil_economy, enable_infinite_resource_centers, enable_unlock_full_faction_roster) = games()
        .lock()
        .get(&gid)
        .map(|g| {
            (
                g.dedicated_session_id,
                g.password.clone(),
                g.enable_special_abilities,
                g.enable_tech_tree,
                g.enable_oil_economy,
                g.enable_infinite_resource_centers,
                g.enable_unlock_full_faction_roster,
            )
        })
        .unwrap_or((None, String::new(), true, true, true, false, false));
    let game = CncGame {
        gid,
        name: gnam,
        host_persona: host,
        max_players: 8,
        players: vec![host_player],
        uuid,
        phase: GamePhase::Resetable,
        map_path,
        start_count: pending_start_count(gid),
        dedicated_session_id,
        is_standby: false,
        password,
        enable_special_abilities,
        enable_tech_tree,
        enable_oil_economy,
        enable_infinite_resource_centers,
        enable_unlock_full_faction_roster,
        replicated_wire: None,
        pros_wire: None,
    };
    games().lock().insert(gid, game);
    reapply_all_pending_attrs(gid);
    destroy_orphan_host_lobbies(Some(gid));
}

pub fn seed_from_join(gid: i64) {
    if games().lock().contains_key(&gid) {
        return;
    }
    let map_path = get_map_path(gid);
    let map_for_defaults = if map_path.is_empty() {
        DEFAULT_MAP_PATH
    } else {
        map_path.as_str()
    };
    let host = host_persona();
    let host_name = host_display_name();
    let mut host_player = CncPlayer {
        persona_id: host,
        display_name: host_name,
        slot: 0,
        team: 1,
        is_ai: false,
        ready: true,
        attribs: default_human_attribs_for_map(map_for_defaults, 0, 1),
        custom_data: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    merge_pending_into_player(gid, &mut host_player, map_for_defaults);
    let mut m = games().lock();
    if m.contains_key(&gid) {
        return;
    }
    m.insert(
        gid,
        CncGame {
            gid,
            name: "Skirmish".to_string(),
            host_persona: host,
            max_players: 8,
            players: vec![host_player],
            uuid: new_uuid_v4_string(),
            phase: GamePhase::Resetable,
            map_path,
            start_count: pending_start_count(gid),
            dedicated_session_id: None,
            is_standby: false,
            password: String::new(),
            enable_special_abilities: true,
            enable_tech_tree: true,
            enable_oil_economy: false,
            enable_infinite_resource_centers: false,
            enable_unlock_full_faction_roster: false,
            replicated_wire: None,
            pros_wire: None,
        },
    );
}

pub fn get_game(gid: i64) -> Option<CncGame> {
    games().lock().get(&gid).cloned()
}

pub fn get_phase(gid: i64) -> GamePhase {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.phase)
        .unwrap_or(GamePhase::Resetable)
}

pub fn set_phase(gid: i64, phase: GamePhase) {
    if let Some(game) = games().lock().get_mut(&gid) {
        game.phase = phase;
    }
}

pub fn destroy_game(gid: i64) {
    games().lock().remove(&gid);
    clear_pending_map(gid);
    clear_blaze_push_flags(gid);
}

pub fn destroy_games_for_dedicated(dedicated_session_id: u64) {
    let gids: Vec<i64> = {
        let m = games().lock();
        m.iter()
            .filter(|(_, g)| g.dedicated_session_id == Some(dedicated_session_id))
            .map(|(gid, _)| *gid)
            .collect()
    };
    for gid in gids {
        signal_lost_game_server(gid);
        destroy_game(gid);
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m destroyed game gid={} (dedicated session #{} gone; clients should kick)",
            gid,
            dedicated_session_id
        );
    }
}

pub fn destroy_orphan_host_lobbies(keep_gid: Option<i64>) {
    let orphans: Vec<i64> = {
        let m = games().lock();
        m.iter()
            .filter(|(gid, g)| {
                if keep_gid == Some(**gid) {
                    return false;
                }
                match g.dedicated_session_id {
                    None => true,
                    Some(sid) => crate::client::cnc::dedicated_pool::get_entry(sid).is_none(),
                }
            })
            .map(|(gid, _)| *gid)
            .collect()
    };
    for gid in orphans {
        destroy_game(gid);
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m destroyed orphan host lobby gid={} (not dedicated-backed)",
            gid
        );
    }
}

pub fn remove_player(gid: i64, persona_id: i64) -> Option<usize> {
    remove_player_ex(gid, persona_id).map(|(humans, _)| humans)
}

pub fn remove_player_ex(gid: i64, persona_id: i64) -> Option<(usize, bool)> {
    let (humans, converted) = {
        let mut map = games().lock();
        let game = map.get_mut(&gid)?;

        let human_count = game.players.iter().filter(|p| !p.is_ai).count();
        let leaving_human = game
            .players
            .iter()
            .find(|p| p.persona_id == persona_id && !p.is_ai)
            .cloned();

        if let Some(leaving) = leaving_human {
            if human_count > 1 {
                let slot = leaving.slot;
                let team = leaving.team.max(1);
                let faction = leaving
                    .attribs
                    .get("_faction")
                    .cloned()
                    .unwrap_or_else(|| "APA".to_string());
                let general = leaving.attribs.get("_general").cloned();
                let ai_pid = next_ai_persona_id();
                if let Some(p) = game.players.iter_mut().find(|p| p.persona_id == persona_id) {
                    p.is_ai = true;
                    p.ready = true;
                    p.persona_id = ai_pid;
                    p.display_name = format!("AI_{}", slot + 1);
                    p.attribs.insert("_isai".to_string(), "1".to_string());
                    if let Some(g) = general {
                        if !g.is_empty() && g != "0" {
                            p.attribs.insert("_general".to_string(), g);
                        }
                    }
                    if !p.attribs.contains_key("_faction") {
                        p.attribs.insert("_faction".to_string(), faction);
                    }
                    p.attribs
                        .insert("_team".to_string(), team.to_string());
                    p.attribs
                        .insert("_startpoint".to_string(), (slot + 1).max(1).to_string());
                }
                if game.host_persona == persona_id {
                    game.host_persona = game
                        .players
                        .iter()
                        .find(|p| !p.is_ai)
                        .map(|p| p.persona_id)
                        .unwrap_or(0);
                }
                let remaining = game.players.iter().filter(|p| !p.is_ai).count();
                crate::debug_println!(
                    "\x1b[38;2;255;215;0m[CNC]\x1b[0m human→AI gid={} left_pid={} ai_pid={} humans_remaining={}",
                    gid,
                    persona_id,
                    ai_pid,
                    remaining
                );
                (remaining, true)
            } else {
                game.players.retain(|p| p.persona_id != persona_id);
                if game.host_persona == persona_id {
                    game.host_persona = game
                        .players
                        .iter()
                        .find(|p| !p.is_ai)
                        .map(|p| p.persona_id)
                        .unwrap_or(0);
                }
                let remaining = game.players.iter().filter(|p| !p.is_ai).count();
                (remaining, false)
            }
        } else {
            game.players.retain(|p| p.persona_id != persona_id);
            if game.host_persona == persona_id {
                game.host_persona = game
                    .players
                    .iter()
                    .find(|p| !p.is_ai)
                    .map(|p| p.persona_id)
                    .unwrap_or(0);
            }
            let remaining = game.players.iter().filter(|p| !p.is_ai).count();
            (remaining, false)
        }
    };
    if converted {
        refresh_pros_wire_for_gid(gid);
    }
    Some((humans, converted))
}

pub fn is_standby_game(gid: i64) -> bool {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.is_standby)
        .unwrap_or(false)
}

pub fn dedicated_session_id_for_gid(gid: i64) -> Option<u64> {
    games()
        .lock()
        .get(&gid)
        .and_then(|g| g.dedicated_session_id)
}

pub fn reclaim_after_empty_humans(gid: i64) -> serde_json::Value {
    if crate::client::cnc::dedicated_pool::reclaim_notify_recent_for_gid(gid) {
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[CNC]\x1b[0m reclaim after empty humans skipped — notify already sent (gid={})",
            gid
        );
        return serde_json::json!({
            "ok": true,
            "gid": gid,
            "humans": 0,
            "standbyReset": true,
            "reclaimCoalesced": true,
        });
    }
    let dedicated_sid = crate::client::cnc::dedicated_pool::reclaim_gid_to_idle_pool(gid);
    {
        let mut m = games().lock();
        if let Some(game) = m.get_mut(&gid) {
            game.is_standby = true;
            if game.dedicated_session_id.is_none() {
                game.dedicated_session_id = dedicated_sid;
            }
        }
    }
    if !games().lock().contains_key(&gid) {
        if let Some(sid) = dedicated_sid {
            if let Some(entry) = crate::client::cnc::dedicated_pool::get_entry(sid) {
                let hostname = entry
                    .server_hostname
                    .clone()
                    .unwrap_or_else(|| format!("DED{sid}"));
                ensure_standby_game(gid, &hostname, sid);
            }
        }
    }
    reset_standby_after_pool_return(gid);

    destroy_orphan_host_lobbies(Some(gid));

    if let Some(sid) = dedicated_sid {
        crate::client::cnc::dedicated_pool::request_dedicated_level_unload(sid, gid);
    }

    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m reclaim after empty humans gid={} dedicated={:?} → Recycling then Standby",
        gid,
        dedicated_sid
    );

    serde_json::json!({
        "ok": true,
        "gid": gid,
        "humans": 0,
        "standbyReset": true,
        "poolIdle": dedicated_sid.is_some(),
        "dedicated": dedicated_sid,
        "admin": 0,
    })
}

pub fn leave_gameroom(gid: i64, persona_id: i64) -> serde_json::Value {
    leave_gameroom_ex(gid, persona_id, false)
}

pub fn purge_persona_from_lobbies(persona_id: i64) {
    if persona_id <= 0 {
        return;
    }
    let gids: Vec<i64> = {
        let m = games().lock();
        m.iter()
            .filter(|(_, g)| {
                g.players
                    .iter()
                    .any(|p| !p.is_ai && p.persona_id == persona_id)
            })
            .map(|(gid, _)| *gid)
            .collect()
    };
    for gid in gids {
        let _ = leave_gameroom_ex(gid, persona_id, false);
        crate::debug_println!(
            "\x1b[38;2;255;215;0m[CNC]\x1b[0m purged persona {} from lobby gid={}",
            persona_id,
            gid
        );
    }
}

pub fn clear_lobby_humans(gid: i64) {
    let was_standby = is_standby_game(gid);
    {
        let mut m = games().lock();
        if let Some(game) = m.get_mut(&gid) {
            game.players.retain(|p| p.is_ai);
            game.host_persona = 0;
        } else {
            return;
        }
    }
    if was_standby {
        reset_standby_after_pool_return(gid);
    }
}

pub fn leave_gameroom_ex(gid: i64, persona_id: i64, force_clear: bool) -> serde_json::Value {
    let session_pid = {
        let s = get_user_session();
        if s.persona_id == 0 {
            0
        } else {
            s.persona_id as i64
        }
    };
    let humans_snapshot: Vec<i64> = {
        let m = games().lock();
        let Some(game) = m.get(&gid) else {
            return serde_json::json!({
                "ok": false,
                "error": "no game",
                "gid": gid,
            });
        };
        game.players
            .iter()
            .filter(|p| !p.is_ai)
            .map(|p| p.persona_id)
            .collect()
    };
    if humans_snapshot.is_empty() {
        if session_pid > 0 {
            crate::client::cnc::fireframe::request_client_local_game_teardown(
                gid,
                session_pid,
                crate::client::cnc::PLAYER_REMOVED_REASON_PLAYER_LEFT,
            );
        }
        return reclaim_after_empty_humans(gid);
    }

    let resolved_pid = if persona_id > 0 && humans_snapshot.iter().any(|&h| h == persona_id) {
        persona_id
    } else if session_pid > 0 && humans_snapshot.iter().any(|&h| h == session_pid) {
        session_pid
    } else if humans_snapshot.len() == 1 {
        humans_snapshot[0]
    } else if force_clear {
        0
    } else if persona_id > 0 {
        persona_id
    } else if session_pid > 0 {
        session_pid
    } else {
        return serde_json::json!({
            "ok": false,
            "error": "pid required",
            "gid": gid,
            "humans": humans_snapshot.len(),
        });
    };

    let was_standby = is_standby_game(gid);
    let (remaining, converted) = if force_clear && resolved_pid == 0 {
        let mut m = games().lock();
        if let Some(game) = m.get_mut(&gid) {
            game.players.retain(|p| p.is_ai);
            game.host_persona = 0;
            (
                Some(game.players.iter().filter(|p| !p.is_ai).count()),
                false,
            )
        } else {
            (None, false)
        }
    } else {
        match remove_player_ex(gid, resolved_pid) {
            Some((n, c)) => (Some(n), c),
            None => (None, false),
        }
    };
    match remaining {
        Some(0) if was_standby => {
            let teardown_pid = if resolved_pid > 0 {
                resolved_pid
            } else {
                session_pid
            };
            if teardown_pid > 0 {
                crate::client::cnc::fireframe::request_client_local_game_teardown(
                    gid,
                    teardown_pid,
                    crate::client::cnc::PLAYER_REMOVED_REASON_PLAYER_LEFT,
                );
            }
            let mut body = reclaim_after_empty_humans(gid);
            if let Some(obj) = body.as_object_mut() {
                obj.insert("pid".into(), serde_json::json!(resolved_pid));
            }
            body
        }
        Some(0) => {
            let teardown_pid = if resolved_pid > 0 {
                resolved_pid
            } else {
                session_pid
            };
            if teardown_pid > 0 {
                crate::client::cnc::fireframe::request_client_local_game_teardown(
                    gid,
                    teardown_pid,
                    crate::client::cnc::PLAYER_REMOVED_REASON_PLAYER_LEFT,
                );
            }
            let mut body = reclaim_after_empty_humans(gid);
            if let Some(obj) = body.as_object_mut() {
                obj.insert("pid".into(), serde_json::json!(resolved_pid));
            }
            body
        }
        Some(n) => {
            let admin = games()
                .lock()
                .get(&gid)
                .map(|g| g.host_persona)
                .unwrap_or(0);
            serde_json::json!({
                "ok": true,
                "gid": gid,
                "pid": resolved_pid,
                "humans": n,
                "admin": admin,
                "convertedToAi": converted,
            })
        }
        None => serde_json::json!({
            "ok": false,
            "error": "no game",
            "gid": gid,
            "pid": resolved_pid,
        }),
    }
}

pub fn set_player_ready(gid: i64, persona_id: i64, ready: bool) -> bool {
    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return false;
    };
    let Some(player) = game.players.iter_mut().find(|p| p.persona_id == persona_id) else {
        return false;
    };
    if player.is_ai {
        return false;
    }
    player.ready = ready;
    true
}

pub fn all_humans_ready(gid: i64) -> bool {
    let m = games().lock();
    let Some(game) = m.get(&gid) else {
        return false;
    };
    let host = game.host_persona;
    let humans: Vec<_> = game.players.iter().filter(|p| !p.is_ai).collect();
    !humans.is_empty()
        && humans
            .iter()
            .all(|p| p.ready || (host != 0 && p.persona_id == host))
}

pub fn lobby_roster_json(gid: i64) -> serde_json::Value {
    let m = games().lock();
    let Some(game) = m.get(&gid) else {
        let server_lost = server_lost_gids().lock().contains_key(&gid);
        return serde_json::json!({
            "ok": false,
            "gid": gid,
            "players": [],
            "serverLost": server_lost,
            "message": if server_lost {
                "Server connection lost."
            } else {
                ""
            },
        });
    };
    if let Some(sid) = game.dedicated_session_id {
        if crate::client::cnc::dedicated_pool::get_entry(sid).is_none() {
            drop(m);
            note_server_lost(gid);
            destroy_game(gid);
            return serde_json::json!({
                "ok": false,
                "gid": gid,
                "players": [],
                "serverLost": true,
                "message": "Server connection lost.",
            });
        }
    }
    let host = game.host_persona;
    let humans: Vec<_> = game.players.iter().filter(|p| !p.is_ai).collect();
    let all_ready = !humans.is_empty()
        && humans
            .iter()
            .all(|p| p.ready || (host != 0 && p.persona_id == host));
    let players: Vec<_> = game
        .players
        .iter()
        .map(|p| {
            let is_host = p.persona_id == host && host != 0;
            let startpoint = effective_startpoint_for_player(gid, p);
            serde_json::json!({
                "pid": p.persona_id,
                "name": p.display_name,
                "slot": p.slot,
                "team": p.team,
                "startpoint": startpoint,
                "isAi": p.is_ai,
                "ready": p.ready || p.is_ai || is_host,
                "isHost": is_host,
            })
        })
        .collect();
    serde_json::json!({
        "ok": true,
        "gid": gid,
        "admin": game.host_persona,
        "isStandby": game.is_standby,
        "passwordProtected": !game.password.is_empty(),
        "enableSpecialAbilities": game.enable_special_abilities,
        "enableTechTree": game.enable_tech_tree,
        "enableOilEconomy": game.enable_oil_economy,
        "enableInfiniteResourceCenters": game.enable_infinite_resource_centers,
        "enableUnlockFullFactionRoster": game.enable_unlock_full_faction_roster,
        "allReady": all_ready,
        "players": players,
        "serverLost": false,
    })
}

/// GID for resetDedicatedServer when the wire omits RGID (pool lobby 10xxx).
pub fn resolve_host_reset_gid() -> i64 {
    let host = host_persona();
    let best_pool_lobby: Option<i64> = {
        let m = games().lock();
        let mut best: Option<(i64, u8)> = None;
        for (gid, g) in m.iter() {
            if *gid < 10_000 {
                continue;
            }
            let owned = g.host_persona == host || (g.is_standby && g.host_persona == 0);
            if !owned {
                continue;
            }
            let score = u8::from(!g.map_path.is_empty()) * 4
                + u8::from(!g.players.is_empty()) * 2
                + u8::from(g.is_standby);
            if best.map(|(_, s)| score > s).unwrap_or(true) {
                best = Some((*gid, score));
            }
        }
        best.map(|(gid, _)| gid)
    };
    if let Some(gid) = best_pool_lobby {
        return gid;
    }
    if let Some(gid) = crate::client::cnc::dedicated_pool::registered_pool_gid() {
        return gid;
    }
    1
}

fn write_pending_map(gid: i64, map_path: &str, start_count: i32) {
    PENDING_MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .insert(
            gid,
            PendingMapInfo {
                path: map_path.to_string(),
                start_count,
            },
        );
    if let Some(game) = games().lock().get_mut(&gid) {
        game.map_path = map_path.to_string();
        if start_count > 0 {
            game.start_count = start_count;
        }
    }
}

pub fn set_map_path(gid: i64, map_path: &str) {
    set_map_selection(gid, map_path, 0);
}

pub fn set_map_selection(gid: i64, map_path: &str, start_count: i32) {
    write_pending_map(gid, map_path, start_count);
    clamp_lobby_startpoints(gid, start_count);
}

pub fn adopt_host_lobby_pending_into(dedicated_gid: i64) -> Option<String> {
    if !get_map_path(dedicated_gid).is_empty() {
        return None;
    }
    let host = host_persona();
    let lobby_gids: Vec<i64> = games()
        .lock()
        .iter()
        .filter(|(g, game)| **g != dedicated_gid && game.host_persona == host)
        .map(|(g, _)| *g)
        .collect();
    let path = {
        let pending = PENDING_MAPS
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock();
        lobby_gids.into_iter().find_map(|lobby_gid| {
            pending
                .get(&lobby_gid)
                .filter(|info| !info.path.is_empty())
                .cloned()
                .map(|info| (lobby_gid, info))
        })
    };
    let Some((lobby_gid, info)) = path else {
        return None;
    };
    write_pending_map(dedicated_gid, &info.path, info.start_count);
    tracing::info!(
        target: "cnc",
        "[CNC] adopted lobby PENDING gid={} → dedicated gid={} path=\"{}\" start_count={}",
        lobby_gid,
        dedicated_gid,
        info.path,
        info.start_count
    );
    Some(info.path)
}

pub fn clear_pending_map(gid: i64) {
    let removed = PENDING_MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .remove(&gid);
    if let Some(prev) = removed {
        tracing::info!(
            target: "cnc",
            "[CNC] clear pending map gid={} was=\"{}\" start_count={}",
            gid,
            prev.path,
            prev.start_count
        );
    }
}

pub fn get_map_path(gid: i64) -> String {
    let pending = PENDING_MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .get(&gid)
        .map(|info| info.path.clone())
        .unwrap_or_default();
    if !pending.is_empty() {
        return pending;
    }
    games()
        .lock()
        .get(&gid)
        .map(|g| g.map_path.clone())
        .unwrap_or_default()
}

/// Map path for the active lobby session when gid is not known yet.
pub fn active_map_path() -> String {
    let path = get_map_path(1);
    if !path.is_empty() {
        return path;
    }
    DEFAULT_MAP_PATH.to_string()
}

pub fn is_player_in_game(gid: i64, persona_id: i64) -> bool {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.players.iter().any(|p| p.persona_id == persona_id))
        .unwrap_or(false)
}

pub fn player_count(gid: i64) -> i32 {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.players.len() as i32)
        .unwrap_or(1)
}

pub fn human_player_count(gid: i64) -> usize {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.players.iter().filter(|p| !p.is_ai).count())
        .unwrap_or(0)
}

pub fn game_name(gid: i64) -> String {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Skirmish".to_string())
}

pub fn resolve_game_uuid(request_payload: &[u8]) -> String {
    resolve_uuid(request_payload)
}

pub fn game_uuid(gid: i64) -> String {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.uuid.clone())
        .unwrap_or_else(new_uuid_v4_string)
}

fn parse_add_queued_gid(payload: &[u8]) -> i64 {
    TdfEncoder::find_int_field(payload, "GID")
        .map(|v| v as i64)
        .or_else(|| {
            TdfEncoder::scan_all_u32_fields(payload, "GID")
                .first()
                .copied()
                .map(|u| u as i64)
        })
        .filter(|&g| g > 0)
        .unwrap_or(1)
}

fn next_free_slot(players: &[CncPlayer]) -> i32 {
    let used: std::collections::HashSet<i32> = players.iter().map(|p| p.slot).collect();
    for slot in 0..8 {
        if !used.contains(&slot) {
            return slot;
        }
    }
    players.len() as i32
}

pub fn add_queued_player(payload: &[u8]) -> BlazeResult<(i64, CncPlayer)> {
    let gid = parse_add_queued_gid(payload);
    seed_from_join(gid);
    let map = get_map_path(gid);
    let map_for_defaults = if map.is_empty() {
        DEFAULT_MAP_PATH
    } else {
        map.as_str()
    };

    let slot = TdfEncoder::find_int_field(payload, "SLOT")
        .or_else(|| TdfEncoder::find_int_field(payload, "SLOT"))
        .filter(|&s| s >= 0 && s < 8);

    let mut m = games().lock();
    let game = m
        .get_mut(&gid)
        .ok_or_else(|| BlazeError::InvalidPacket("missing game".into()))?;

    let slot = slot.unwrap_or_else(|| next_free_slot(&game.players));
    let ai_id = next_ai_persona_id();
    let ai_name = format!("AI_{}", slot + 1);

    let attribs = default_ai_attribs(slot, 2, "GLA");

    let mut player = CncPlayer {
        persona_id: ai_id,
        display_name: ai_name,
        slot,
        team: 2,
        is_ai: true,
        ready: true,
        attribs,
        custom_data: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    ensure_general_attr(&mut player, map_for_defaults);
    game.players.push(player.clone());
    refresh_pros_wire_for_gid(gid);
    *LAST_ADD_QUEUED
        .get_or_init(|| Mutex::new(None))
        .lock() = Some((gid, player.clone()));
    Ok((gid, player))
}

pub fn take_last_add_queued() -> Option<(i64, CncPlayer)> {
    LAST_ADD_QUEUED
        .get_or_init(|| Mutex::new(None))
        .lock()
        .take()
}

/// Add the joining client to the roster if missing.
pub fn ensure_client_player(gid: i64, persona_id: i64, display_name: &str) -> Option<CncPlayer> {
    let is_standby = games().lock().get(&gid).map(|g| g.is_standby).unwrap_or(false);
    if !is_standby {
        seed_from_join(gid);
    }
    let map_for_merge = {
        let map = get_map_path(gid);
        if map.is_empty() {
            DEFAULT_MAP_PATH.to_string()
        } else {
            map
        }
    };
    let player = {
        let mut m = games().lock();
        let game = m.get_mut(&gid)?;
        if let Some(existing_idx) = game
            .players
            .iter()
            .position(|p| p.persona_id == persona_id)
        {
            merge_pending_into_player(
                gid,
                &mut game.players[existing_idx],
                &map_for_merge,
            );
            if game.host_persona == 0 {
                game.host_persona = persona_id;
            }
            if game.host_persona == persona_id {
                game.players[existing_idx].ready = true;
            }
            return Some(game.players[existing_idx].clone());
        }
        if game.host_persona == 0 {
            game.host_persona = persona_id;
        }
        let is_host = game.host_persona == persona_id;
        let slot = next_free_slot(&game.players);
        let map_path = if !game.map_path.is_empty() {
            game.map_path.clone()
        } else {
            String::new()
        };
        drop(m);
        let map_for_defaults = if !map_path.is_empty() {
            map_path
        } else {
            let pending = get_map_path(gid);
            if pending.is_empty() {
                DEFAULT_MAP_PATH.to_string()
            } else {
                pending
            }
        };
        let mut player = CncPlayer {
            persona_id,
            display_name: if display_name.is_empty() {
                format!("Player{}", slot + 1)
            } else {
                display_name.to_string()
            },
            slot,
            team: 1,
            is_ai: false,
            ready: is_host,
            attribs: default_human_attribs_for_map(&map_for_defaults, slot, 1),
            custom_data: IndexMap::new(),
            stat: PROS_STAT_ACTIVE_CONNECTING,
        };
        merge_pending_into_player(gid, &mut player, &map_for_defaults);
        let mut m = games().lock();
        let game = m.get_mut(&gid)?;
        if game.players.iter().any(|p| p.persona_id == persona_id) {
            return game
                .players
                .iter()
                .find(|p| p.persona_id == persona_id)
                .cloned();
        }
        game.players.push(player.clone());
        player
    };
    // refresh_pros_wire_for_gid locks GAMES itself; release the mutation lock first.
    refresh_pros_wire_for_gid(gid);
    Some(player)
}

pub fn set_player_attribute(gid: i64, persona_id: i64, key: &str, value: &str) -> bool {
    let unchanged = {
        let m = games().lock();
        m.get(&gid)
            .and_then(|g| g.players.iter().find(|p| p.persona_id == persona_id))
            .and_then(|p| p.attribs.get(key))
            .map(|s| s.as_str() == value)
            .unwrap_or(false)
    };
    let mut attrs = IndexMap::new();
    attrs.insert(key.to_string(), value.to_string());
    set_pending_player_attrs(gid, persona_id, attrs);
    !unchanged
}

pub fn parse_set_player_attributes(payload: &[u8]) -> Option<(i64, i64, String, String)> {
    let applied = apply_set_player_attributes(payload)?;
    let (k, v) = applied.2.iter().next()?;
    Some((applied.0, applied.1, k.clone(), v.clone()))
}

fn parse_gm_gid_pid(payload: &[u8]) -> Option<(i64, i64)> {
    let gid = ["GID ", "GID"]
        .iter()
        .find_map(|tag| TdfEncoder::find_long_field(payload, tag))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64))?;
    let pid = ["PID ", "PID"]
        .iter()
        .find_map(|tag| TdfEncoder::find_long_field(payload, tag))
        .or_else(|| TdfEncoder::find_int_field(payload, "PID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "PID").map(|v| v as i64))?;
    Some((gid, pid))
}

pub fn apply_set_player_attributes(
    payload: &[u8],
) -> Option<(i64, i64, IndexMap<String, String>)> {
    let (gid, pid) = parse_gm_gid_pid(payload)?;
    let attrs = TdfEncoder::find_string_string_map_field(payload, "ATTR")?;
    if attrs.is_empty() {
        return None;
    }
    seed_from_join(gid);
    for (key, value) in &attrs {
        set_player_attribute(gid, pid, key, value);
    }
    // Always echo requested attrs -- client retries until `NotifyPlayerAttribChange` arrives.
    *LAST_ATTR_CHANGE
        .get_or_init(|| Mutex::new(None))
        .lock() = Some((gid, pid, attrs.clone()));
    Some((gid, pid, attrs))
}

pub fn take_last_attr_change() -> Option<(i64, i64, IndexMap<String, String>)> {
    LAST_ATTR_CHANGE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .take()
}

pub fn build_notify_player_attrib_change(
    gid: i64,
    pid: i64,
    attribs: &IndexMap<String, String>,
) -> Vec<u8> {
    // Packed-tag order ATTR, GID, PID so ATTR is visited.
    fn packed_tag(tag: &str) -> u32 {
        let t = TdfEncoder::make_tag(tag);
        ((t[0] as u32) << 16) | ((t[1] as u32) << 8) | (t[2] as u32)
    }
    let mut fields: Vec<(u32, Vec<u8>)> = Vec::new();
    fields.push((
        packed_tag("ATTR"),
        TdfEncoder::encode_string_string_map_ordered("ATTR", attribs).to_vec(),
    ));
    fields.push((
        packed_tag("GID "),
        TdfEncoder::encode_long("GID ", gid).to_vec(),
    ));
    fields.push((
        packed_tag("PID "),
        TdfEncoder::encode_long("PID ", pid).to_vec(),
    ));
    fields.sort_by_key(|(tag, _)| *tag);
    let mut out = Vec::new();
    for (_, bytes) in fields {
        out.extend_from_slice(&bytes);
    }
    out
}

/// Matches `RtsClientSettings.EAGenericAuthToken` / dedicated express login (`ABC123`).
pub const CNC_AUTH_TOKEN: &str = "ABC123";

const CUSTOM_DATA_NOTIFY_ORDER: &[&str] = &["AuthToken", "XNNC", "XSES"];

pub fn auth_token_custom_data_blob() -> Vec<u8> {
    CNC_AUTH_TOKEN.as_bytes().to_vec()
}

pub fn ensure_auth_token_in_custom_data(blobs: &mut IndexMap<String, Vec<u8>>) {
    blobs
        .entry("AuthToken".to_string())
        .or_insert_with(auth_token_custom_data_blob);
}

fn parse_gm_gid(payload: &[u8]) -> Option<i64> {
    ["GID ", "GID"]
        .iter()
        .find_map(|tag| TdfEncoder::find_long_field(payload, tag))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64))
}

fn resolve_custom_data_pid(payload: &[u8]) -> i64 {
    ["PID ", "PID"]
        .iter()
        .find_map(|tag| TdfEncoder::find_long_field(payload, tag))
        .or_else(|| TdfEncoder::find_int_field(payload, "PID ").map(|v| v as i64))
        .or_else(|| TdfEncoder::find_int_field(payload, "PID").map(|v| v as i64))
        .unwrap_or_else(host_persona)
}

fn parse_custom_data_blobs(payload: &[u8]) -> IndexMap<String, Vec<u8>> {
    if let Some(map) = TdfEncoder::find_string_blob_map_field(payload, "CDAT") {
        return map;
    }
    let mut blobs = IndexMap::new();
    for (tag, type_byte, _, _) in TdfEncoder::scan_root_level_fields(payload) {
        let key = tag.trim();
        if key == "GID" || key == "PID" || key == "CDAT" {
            continue;
        }
        if type_byte != 0x2 {
            continue;
        }
        if let Some(value) = TdfEncoder::find_blob_field(payload, key) {
            blobs.insert(key.to_string(), value);
        }
    }
    blobs
}

pub fn set_player_custom_data(
    gid: i64,
    persona_id: i64,
    data: &IndexMap<String, Vec<u8>>,
) -> bool {
    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return false;
    };
    let Some(player) = game.players.iter_mut().find(|p| p.persona_id == persona_id) else {
        return false;
    };
    for (key, value) in data {
        player.custom_data.insert(key.clone(), value.clone());
    }
    true
}

pub fn apply_set_player_custom_data(
    payload: &[u8],
) -> Option<(i64, i64, IndexMap<String, Vec<u8>>)> {
    let gid = parse_gm_gid(payload)?;
    let pid = resolve_custom_data_pid(payload);
    let mut blobs = parse_custom_data_blobs(payload);
    ensure_auth_token_in_custom_data(&mut blobs);
    seed_from_join(gid);
    set_player_custom_data(gid, pid, &blobs);
    *LAST_CUSTOM_DATA_CHANGE
        .get_or_init(|| Mutex::new(None))
        .lock() = Some((gid, pid, blobs.clone()));
    Some((gid, pid, blobs))
}

pub fn take_last_custom_data_change() -> Option<(i64, i64, IndexMap<String, Vec<u8>>)> {
    LAST_CUSTOM_DATA_CHANGE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .take()
}

pub fn build_notify_player_custom_data_change(
    gid: i64,
    pid: i64,
    data: &IndexMap<String, Vec<u8>>,
) -> Vec<u8> {
    let mut blobs = IndexMap::new();
    for key in CUSTOM_DATA_NOTIFY_ORDER {
        if let Some(value) = data.get(*key) {
            blobs.insert(key.to_string(), value.clone());
        }
    }
    for (key, value) in data {
        if !blobs.contains_key(key) {
            blobs.insert(key.clone(), value.clone());
        }
    }
    ensure_auth_token_in_custom_data(&mut blobs);
    // Encode AuthToken as a blob, not MAP, so GID/PID still visit.
    let cdat_bytes = blobs
        .get("AuthToken")
        .cloned()
        .unwrap_or_else(auth_token_custom_data_blob);
    fn packed_tag(tag: &str) -> u32 {
        let t = TdfEncoder::make_tag(tag);
        ((t[0] as u32) << 16) | ((t[1] as u32) << 8) | (t[2] as u32)
    }
    let mut fields: Vec<(u32, Vec<u8>)> = Vec::new();
    fields.push((
        packed_tag("CDAT"),
        TdfEncoder::encode_binary("CDAT", &cdat_bytes).to_vec(),
    ));
    fields.push((
        packed_tag("GID "),
        TdfEncoder::encode_long("GID ", gid).to_vec(),
    ));
    fields.push((
        packed_tag("PID "),
        TdfEncoder::encode_long("PID ", pid).to_vec(),
    ));
    fields.sort_by_key(|(tag, _)| *tag);
    let mut out = Vec::new();
    for (_, bytes) in fields {
        out.extend_from_slice(&bytes);
    }
    out
}

fn encode_empty_pnet_network_address() -> Vec<u8> {
    let mut out = Vec::new();
    let tag = TdfEncoder::make_tag("PNET");
    out.push(tag[0]);
    out.push(tag[1]);
    out.push(tag[2]);
    out.push(0x06);
    out.extend_from_slice(&TdfEncoder::encode_varint(2));
    let endpoint = |ip: i32, port: i32| {
        let mut ep = Vec::new();
        ep.extend_from_slice(&TdfEncoder::encode_int("IP  ", ip));
        ep.extend_from_slice(&TdfEncoder::encode_int("PORT", port));
        ep
    };
    // Ascending packed-tag order: EXIP (0x978a70) before INIP (0xa6ea70).
    let mut valu = Vec::new();
    valu.extend_from_slice(&TdfEncoder::encode_struct("EXIP", &endpoint(0, 0)));
    valu.extend_from_slice(&TdfEncoder::encode_struct("INIP", &endpoint(0, 0)));
    out.extend_from_slice(&TdfEncoder::encode_struct("VALU", &valu));
    out
}

fn append_pros_core_fields(out: &mut Vec<u8>, player: &CncPlayer, gid: i64, gfgd: bool) {
    out.extend_from_slice(&TdfEncoder::encode_long("EXID", player.persona_id));
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_int("LOC ", 0));
    out.extend_from_slice(&TdfEncoder::encode_string("NAME", &player.display_name));
    if !player.attribs.is_empty() {
        out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
            "PATT",
            &player.attribs,
        ));
    }
    out.extend_from_slice(&TdfEncoder::encode_long("PID ", player.persona_id));
    out.extend_from_slice(&encode_empty_pnet_network_address());
    out.extend_from_slice(&TdfEncoder::encode_int("SID ", 255));
    out.extend_from_slice(&TdfEncoder::encode_int("SLOT", player.slot));
    out.extend_from_slice(&TdfEncoder::encode_int("STAT", player.stat));
    out.extend_from_slice(&TdfEncoder::encode_int("TIDX", 0xFFFF));
    out.extend_from_slice(&TdfEncoder::encode_time("TIME", player.persona_id));
    if gfgd {
        out.extend_from_slice(&TdfEncoder::encode_object_id("UGID", 0, 0, 0));
    }
}

/// `NotifyGameSetup::PROS` — minimal row.
pub fn build_notify_pros_entry(player: &CncPlayer, gid: i64) -> Vec<u8> {
    let mut out = Vec::new();
    append_pros_core_fields(&mut out, player, gid, false);
    out
}

/// `getFullGameData::PROS` — extended row (`PNET` after `PID`).
pub fn build_gfgd_pros_entry(player: &CncPlayer, gid: i64) -> Vec<u8> {
    let mut out = Vec::new();
    append_pros_core_fields(&mut out, player, gid, true);
    out
}

/// Back-compat alias used by tests.
pub fn build_pros_entry(player: &CncPlayer, gid: i64) -> Vec<u8> {
    build_notify_pros_entry(player, gid)
}

fn pros_entries_from_game(game: &CncGame) -> Vec<Vec<u8>> {
    game.players
        .iter()
        .map(|p| build_notify_pros_entry(p, game.gid))
        .collect()
}

pub fn refresh_pros_wire_for_gid(gid: i64) {
    let pros = games()
        .lock()
        .get(&gid)
        .map(pros_entries_from_game)
        .unwrap_or_default();
    if !pros.is_empty() {
        set_pros_wire_fields(gid, pros);
    }
}

pub fn build_plst_entry(player: &CncPlayer) -> Vec<u8> {
    let pid_i32 = player.persona_id.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_string("DSNM", &player.display_name));
    out.extend_from_slice(&TdfEncoder::encode_int("LAST", 0));
    out.extend_from_slice(&TdfEncoder::encode_long("PID ", player.persona_id));
    out.extend_from_slice(&TdfEncoder::encode_int("PLAT", 0));
    out.extend_from_slice(&TdfEncoder::encode_int("STAS", STAS_IN_GAME));
    out.extend_from_slice(&TdfEncoder::encode_long("XREF", 0));
    let _ = pid_i32;
    out
}

pub fn build_replicated_player(player: &CncPlayer, gid: i64) -> Vec<u8> {
    let gid_i32 = gid.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let pid_i32 = player.persona_id.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_int("EXID", pid_i32));
    out.extend_from_slice(&TdfEncoder::encode_int("GID ", gid_i32));
    out.extend_from_slice(&TdfEncoder::encode_int("LOC ", 0));
    out.extend_from_slice(&TdfEncoder::encode_string("NAME", &player.display_name));
    out.extend_from_slice(&TdfEncoder::encode_int("PID ", pid_i32));
    out.extend_from_slice(&TdfEncoder::encode_int("SLOT", player.slot));
    out.extend_from_slice(&TdfEncoder::encode_int("STAT", player.stat));
    out.extend_from_slice(&TdfEncoder::encode_int("TIDX", player.team.max(0)));
    out.extend_from_slice(&TdfEncoder::encode_int("UID ", pid_i32));
    if !player.attribs.is_empty() {
        out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
            "ATTR",
            &player.attribs,
        ));
    }
    out
}

pub fn pros_entries_for_gid(gid: i64) -> Vec<Vec<u8>> {
    if let Some(cached) = pros_wire_fields(gid) {
        return cached;
    }
    games()
        .lock()
        .get(&gid)
        .map(|g| g.players.iter().map(|p| build_notify_pros_entry(p, gid)).collect())
        .unwrap_or_else(|| {
            let host = host_persona();
            let p = CncPlayer {
                persona_id: host,
                display_name: host_display_name(),
                slot: 0,
                team: 1,
                is_ai: false,
            ready: false,
                attribs: default_human_attribs(0, 1),
                custom_data: IndexMap::new(),
                stat: PROS_STAT_ACTIVE_CONNECTING,
            };
            vec![build_notify_pros_entry(&p, gid)]
        })
}

/// `ListGameData::mGameRoster` for `getFullGameData` -- extended rows (not cached notify wire).
pub fn gfgd_roster_entries_for_gid(gid: i64) -> Vec<Vec<u8>> {
    games()
        .lock()
        .get(&gid)
        .map(|g| {
            g.players
                .iter()
                .map(|p| build_gfgd_pros_entry(p, gid))
                .collect()
        })
        .unwrap_or_else(|| {
            let host = host_persona();
            let p = CncPlayer {
                persona_id: host,
                display_name: host_display_name(),
                slot: 0,
                team: 1,
                is_ai: false,
            ready: false,
                attribs: default_human_attribs(0, 1),
                custom_data: IndexMap::new(),
                stat: PROS_STAT_ACTIVE_CONNECTING,
            };
            vec![build_gfgd_pros_entry(&p, gid)]
        })
}

pub fn all_game_gids() -> Vec<i64> {
    destroy_orphan_host_lobbies(None);
    let mut gids: Vec<i64> = {
        let m = games().lock();
        m.iter()
            .filter(|(_, g)| {
                let Some(sid) = g.dedicated_session_id else {
                    return false;
                };
                crate::client::cnc::dedicated_pool::get_entry(sid)
                    .map(|e| e.creator_registered)
                    .unwrap_or(false)
            })
            .map(|(gid, _)| *gid)
            .collect()
    };
    gids.sort_unstable();
    gids
}

pub fn browser_game_list_json() -> serde_json::Value {
    {
        let orphan_sids: Vec<u64> = {
            let map = games().lock();
            map.values()
                .filter_map(|g| g.dedicated_session_id)
                .filter(|sid| crate::client::cnc::dedicated_pool::get_entry(*sid).is_none())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect()
        };
        for sid in orphan_sids {
            destroy_games_for_dedicated(sid);
        }
    }

    let map = games().lock();
    let mut rows = Vec::new();
    let mut listed_sessions: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for gid in {
        let mut g: Vec<i64> = map.keys().copied().collect();
        g.sort_unstable();
        g
    } {
        let Some(game) = map.get(&gid) else {
            continue;
        };
        let dedicated_session_id = game.dedicated_session_id;
        let Some(sid) = dedicated_session_id else {
            continue;
        };
        let alive = crate::client::cnc::dedicated_pool::get_entry(sid)
            .map(|e| e.creator_registered)
            .unwrap_or(false);
        if !alive {
            continue;
        }
        if !listed_sessions.insert(sid) {
            continue;
        }
        let humans = game.players.iter().filter(|p| !p.is_ai).count();
        let total = game.players.len();
        let map_leaf = game
            .map_path
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or(if game.is_standby { "Standby" } else { "Unknown" });
        let pool_entry = crate::client::cnc::dedicated_pool::get_entry(sid);
        let assignable = pool_entry
            .as_ref()
            .map(|e| crate::client::cnc::dedicated_pool::is_assignable(e))
            .unwrap_or(false);
        // Only true unload. InUse is !assignable but the match is live — do not label Recycling.
        let recycling = pool_entry
            .as_ref()
            .map(|e| {
                e.creator_registered
                    && matches!(
                        e.state,
                        crate::client::cnc::dedicated_pool::DedicatedPoolState::Recycling
                    )
            })
            .unwrap_or(false);
        let display_name = if game.is_standby || humans == 0 {
            pool_entry
                .as_ref()
                .map(|e| crate::client::cnc::dedicated_pool::browser_server_name(e))
                .unwrap_or_else(|| game.name.clone())
        } else {
            game.name.clone()
        };
        // Standby / reclaim rows must not report PreGame as browser "Lobby".
        let (phase_label, joinable) = if recycling {
            ("Recycling", false)
        } else if game.is_standby || humans == 0 {
            ("Standby", assignable)
        } else {
            (
                game.phase.label(),
                matches!(
                    game.phase,
                    GamePhase::PreGame | GamePhase::Resetable | GamePhase::InGame
                ),
            )
        };
        let ping_host = pool_entry.as_ref().map(|e| {
            e.peer
                .parse::<std::net::SocketAddr>()
                .map(|sa| sa.ip().to_string())
                .unwrap_or_else(|_| e.peer.clone())
        });
        rows.push(serde_json::json!({
            "gid": gid,
            "name": display_name,
            "map": map_leaf,
            "mapPath": game.map_path,
            "players": total,
            "humans": humans,
            "maxPlayers": game.max_players,
            "admin": game.host_persona,
            "phase": phase_label,
            "phaseCode": game.phase.as_gsta(),
            "kind": if game.is_standby { "standby" } else { "game" },
            "joinable": joinable,
            "dedicatedSessionId": dedicated_session_id,
            "pingMs": serde_json::Value::Null,
            "pingHost": ping_host,
            "pingPort": pool_entry
                .as_ref()
                .map(|e| crate::client::cnc::dedicated_pool::msg_sys_port(e))
                .unwrap_or(crate::client::cnc::dedicated_pool::DEDICATED_PING_TCP_PORT),
            "isStandby": game.is_standby,
            "passwordProtected": !game.password.is_empty(),
            "passwordAttr": "_password",
        }));
    }
    drop(map);

    let listed_standby_sessions: std::collections::HashSet<u64> = listed_sessions;
    for entry in crate::client::cnc::dedicated_pool::list_entries() {
        if listed_standby_sessions.contains(&entry.blaze_session_id) {
            continue;
        }
        let show = entry.creator_registered
            && matches!(
                entry.state,
                crate::client::cnc::dedicated_pool::DedicatedPoolState::Idle
                    | crate::client::cnc::dedicated_pool::DedicatedPoolState::CreatorRegistered
                    | crate::client::cnc::dedicated_pool::DedicatedPoolState::Recycling
            );
        if !show {
            continue;
        }
        let recycling = matches!(
            entry.state,
            crate::client::cnc::dedicated_pool::DedicatedPoolState::Recycling
        );
        let name = crate::client::cnc::dedicated_pool::browser_server_name(&entry);
        let map_leaf = "Standby";
        let gid = entry.current_gid.unwrap_or(0);
        let ping_host = entry
            .peer
            .parse::<std::net::SocketAddr>()
            .map(|sa| sa.ip().to_string())
            .ok();
        rows.push(serde_json::json!({
            "gid": gid,
            "name": name,
            "map": map_leaf,
            "mapPath": entry.current_map.clone().unwrap_or_default(),
            "players": 0,
            "humans": 0,
            "maxPlayers": 8,
            "admin": 0,
            "phase": if recycling { "Recycling" } else { "Standby" },
            "phaseCode": 0,
            "kind": "standby",
            "joinable": !recycling && gid > 0,
            "dedicatedSessionId": entry.blaze_session_id,
            "pingMs": serde_json::Value::Null,
            "pingHost": ping_host,
            "pingPort": crate::client::cnc::dedicated_pool::msg_sys_port(&entry),
            "isStandby": true,
            "passwordProtected": false,
        }));
    }

    serde_json::json!({ "ok": true, "games": rows, "count": rows.len() })
}

pub fn players_for_gid(gid: i64) -> Vec<CncPlayer> {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.players.clone())
        .unwrap_or_default()
}

/// Live match gids for this human persona (MsgSys hub routing).
pub fn gids_for_human_persona(persona_id: i64) -> Vec<i64> {
    if persona_id <= 0 {
        return Vec::new();
    }
    games()
        .lock()
        .iter()
        .filter_map(|(&gid, g)| {
            g.players
                .iter()
                .any(|p| !p.is_ai && p.persona_id == persona_id)
                .then_some(gid)
        })
        .collect()
}

/// Dedicated reset host: move local player out of `ACTIVE_CONNECTING` after platform-host init.
pub fn mark_host_join_completed(gid: i64) {
    let host = host_persona();
    let mut games = games().lock();
    let Some(game) = games.get_mut(&gid) else {
        return;
    };
    for player in &mut game.players {
        if player.persona_id == host || player.persona_id == game.host_persona {
            player.stat = PROS_STAT_ACTIVE_CONNECTING;
            player.slot = 0;
        }
    }
    game.pros_wire = Some(pros_entries_from_game(game));
}

pub fn host_player_for_gid(gid: i64) -> CncPlayer {
    let host = host_persona();
    games()
        .lock()
        .get(&gid)
        .and_then(|g| {
            g.players
                .iter()
                .find(|p| p.persona_id == g.host_persona || p.persona_id == host)
                .cloned()
        })
        .unwrap_or_else(|| CncPlayer {
            persona_id: host,
            display_name: host_display_name(),
            slot: 0,
            team: 1,
            is_ai: false,
            ready: false,
            attribs: default_human_attribs(0, 1),
            custom_data: IndexMap::new(),
            stat: PROS_STAT_ACTIVE_CONNECTING,
        })
}

pub fn host_persona_for_gid(gid: i64) -> i64 {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.host_persona)
        .unwrap_or_else(host_persona)
}

pub fn ai_players_for_gid(gid: i64) -> Vec<CncPlayer> {
    players_for_gid(gid)
        .into_iter()
        .filter(|p| p.is_ai)
        .collect()
}

pub fn alloc_browser_list_id() -> i64 {
    NEXT_BROWSER_LIST_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn store_game_list_snapshot(list_id: i64, gids: Vec<i64>) {
    *LAST_GAME_LIST_SNAPSHOT
        .get_or_init(|| Mutex::new(None))
        .lock() = Some((list_id, gids));
}

pub fn take_last_game_list_snapshot() -> Option<(i64, Vec<i64>)> {
    LAST_GAME_LIST_SNAPSHOT
        .get_or_init(|| Mutex::new(None))
        .lock()
        .take()
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

fn slot_capacities_vector(tag: &str, public_participants: u16) -> Vec<u8> {
    TdfEncoder::encode_list(
        tag,
        &[
            public_participants as i32,
            0,
            0,
            0,
        ],
    )
    .to_vec()
}

pub fn build_game_browser_player_data(player: &CncPlayer) -> Vec<u8> {
    let pid_i32 = player.persona_id.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_int("EXID", pid_i32));
    out.extend_from_slice(&TdfEncoder::encode_long("PID ", player.persona_id));
    out.extend_from_slice(&TdfEncoder::encode_string("NAME", &player.display_name));
    out.extend_from_slice(&TdfEncoder::encode_int("TIDX", player.team.max(0)));
    out.extend_from_slice(&TdfEncoder::encode_int("STAT", player.stat));
    if !player.attribs.is_empty() {
        out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
            "PATT",
            &player.attribs,
        ));
    }
    out
}

pub fn build_game_browser_game_data(gid: i64) -> Option<Vec<u8>> {
    let game = get_game(gid)?;
    let host = game.host_persona;
    let pcnt = game.players.len() as u16;
    let pcap = game.max_players as u16;

    let roster: Vec<Vec<u8>> = game
        .players
        .iter()
        .map(build_game_browser_player_data)
        .collect();

    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_string("GNAM", &game.name));
    out.extend_from_slice(&slot_capacities_vector("CAP ", pcap));
    out.extend_from_slice(&slot_capacities_vector("PCNT", pcnt));
    out.extend_from_slice(&TdfEncoder::encode_int("GSTA", game.phase.as_gsta()));
    out.extend_from_slice(&TdfEncoder::encode_long("HOST", host));
    out.extend_from_slice(&TdfEncoder::encode_int("NTOP", NTOP_DEDICATED));
    out.extend_from_slice(&encode_struct_list("ROST", &roster));
    out.extend_from_slice(&TdfEncoder::encode_long_list("ADMN", &[host]));
    Some(out)
}

fn build_game_browser_match_data(game_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_struct("GAM ", game_data));
    out.extend_from_slice(&TdfEncoder::encode_int("FIT ", 0));
    out
}

/// Blaze `GetGameListResponse` -- metadata only; games follow in `NotifyGameListUpdate`.
pub fn build_get_game_list_response(list_id: i64, game_count: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("glid", list_id));
    out.extend_from_slice(&TdfEncoder::encode_int("maxf", FIT_SCORE_DEFAULT));
    out.extend_from_slice(&TdfEncoder::encode_int("ngd", game_count as i32));
    out.extend_from_slice(&TdfEncoder::encode_bool("gmlt", true));
    out
}

/// Blaze `NotifyGameListUpdate` -- populates a snapshot/subscription list (`cmd` 201 / 0xC9).
pub fn build_notify_game_list_update(list_id: i64, gids: &[i64], is_final: bool) -> Vec<u8> {
    let mut match_entries = Vec::new();
    for &gid in gids {
        if let Some(game_data) = build_game_browser_game_data(gid) {
            match_entries.push(build_game_browser_match_data(&game_data));
        }
    }

    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("glid", list_id));
    out.extend_from_slice(&TdfEncoder::encode_int("done", if is_final { 1 } else { 0 }));
    out.extend_from_slice(&TdfEncoder::encode_long_list("remv", &[]));
    out.extend_from_slice(&encode_struct_list("updt", &match_entries));
    out
}

pub fn plst_entries_for_gid(gid: i64) -> Vec<Vec<u8>> {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.players.iter().map(build_plst_entry).collect())
        .unwrap_or_else(|| {
            let host = host_persona();
            let p = CncPlayer {
                persona_id: host,
                display_name: host_display_name(),
                slot: 0,
                team: 1,
                is_ai: false,
            ready: false,
                attribs: default_human_attribs(0, 1),
                custom_data: IndexMap::new(),
                stat: PROS_STAT_ACTIVE_CONNECTING,
            };
            vec![build_plst_entry(&p)]
        })
}
