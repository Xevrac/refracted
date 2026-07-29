//! In-memory CNC lobby/game roster shared by GMGR replies and notify payloads.

use indexmap::IndexMap;
use parking_lot::Mutex;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::OnceLock;

use crate::blaze::tdf::TdfEncoder;
use crate::common::error::{BlazeError, BlazeResult};
use crate::session::get_user_session;

const PROS_STAT_ACTIVE_CONNECTING: i32 = 2;
const STAS_IN_GAME: i32 = 2;

static GAMES: OnceLock<Mutex<HashMap<i64, CncGame>>> = OnceLock::new();
static LAST_ADD_QUEUED: OnceLock<Mutex<Option<(i64, CncPlayer)>>> = OnceLock::new();
static LAST_ATTR_CHANGE: OnceLock<Mutex<Option<(i64, i64, IndexMap<String, String>)>>> =
    OnceLock::new();
static LAST_CUSTOM_DATA_CHANGE: OnceLock<Mutex<Option<(i64, i64, IndexMap<String, Vec<u8>>)>>> =
    OnceLock::new();
static NEXT_BROWSER_LIST_ID: AtomicI64 = AtomicI64::new(1);
static LAST_GAME_LIST_SNAPSHOT: OnceLock<Mutex<Option<(i64, Vec<i64>)>>> = OnceLock::new();
/// Map paths chosen in the lobby before the game object exists (the createGame race). Keyed by gid;
/// consulted by `get_map_path` as a fallback so a map selected at Start Battle survives until the
/// game is created and its data (dedicated spawn) is built.
static PENDING_MAPS: OnceLock<Mutex<HashMap<i64, String>>> = OnceLock::new();
/// Lobby player attrs (`_faction` / `_team` / `_startpoint` / `_general` / …) posted via
/// `/cnc/player-attrs` before Blaze createGame. `seed_from_reset` rebuilds the roster from
/// defaults; these survive that wipe the same way `PENDING_MAPS` survives for the level path.
/// Outer key = gid; inner key = persona_id (`0` = host placeholder before session pid is known).
static PENDING_PLAYER_ATTRS: OnceLock<Mutex<HashMap<i64, HashMap<i64, IndexMap<String, String>>>>> =
    OnceLock::new();
/// Per-gid Blaze notify guards -- prevent IN_GAME → PRE_GAME regressions when mesh/finalize fire twice.
static BLAZE_PREGAME_PUSHED: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static BLAZE_INGAME_PUSHED: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
static GAME_READY_PUSHED: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();

fn blaze_pregame_pushed() -> &'static Mutex<HashSet<i64>> {
    BLAZE_PREGAME_PUSHED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn blaze_ingame_pushed() -> &'static Mutex<HashSet<i64>> {
    BLAZE_INGAME_PUSHED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn game_ready_pushed() -> &'static Mutex<HashSet<i64>> {
    GAME_READY_PUSHED.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn clear_blaze_push_flags(gid: i64) {
    blaze_pregame_pushed().lock().remove(&gid);
    blaze_ingame_pushed().lock().remove(&gid);
    game_ready_pushed().lock().remove(&gid);
    clear_orchestration(gid);
}

/// Per-gid join orchestration: client join notifies wait until the dedicated host finishes cmd 220 setup.
#[derive(Debug, Clone)]
struct GidOrchestration {
    client_session_id: u64,
    dedicated_session_id: Option<u64>,
    dedicated_host_ready: bool,
    mesh_active_connected: bool,
    /// Client sent `updateMeshConnection` before host setup finished -- flush after deferred join.
    pending_mesh_pid: Option<i64>,
    deferred_join_pushes: Option<Vec<super::fireframe::OutgoingPush>>,
}

/// Result of a client `updateMeshConnection` during reset orchestration.
pub enum MeshUpdateResult {
    /// Host not ready yet; ACTIVE_CONNECTED + GameReady will follow deferred join notifies.
    DeferredUntilHostReady,
    /// Pushes to send now (ACTIVE_CONNECTED, and GameReady when host is ready).
    Push(Vec<super::fireframe::OutgoingPush>),
}

fn mesh_active_connected_and_game_ready_pushes(
    gid: i64,
    pid: i64,
) -> Option<Vec<super::fireframe::OutgoingPush>> {
    let mut out = Vec::new();
    out.extend(super::fireframe::pushes_after_update_mesh_connection(gid, pid).ok()?);
    if try_mark_game_ready_pushed(gid) {
        out.extend(super::fireframe::pushes_game_ready_attrib(gid).ok()?);
        // Mirror GameReady to the dedicated Blaze session. Client onGameAttributeUpdated
        // (sub_1204D70) already publishes RtsBlazeJoinGameMessage for the join/connect chain;
        // the dedicated must receive the same NotifyGameAttribChange so its CNCLive listener
        // publishes into RtsServer::handleMessage (0xA739D0) — currently silent without this.
        enqueue_game_ready_to_dedicated(gid);
        // Native SDK progresses gameplay on GameState notifications; once mesh is ACTIVE_CONNECTED
        // and GameReady has been applied, synthesize PRE_GAME -> IN_GAME exactly once.
        if try_mark_blaze_pregame_pushed(gid) {
            out.extend(super::fireframe::pushes_advance_game_to_ingame(gid).ok()?);
            let _ = try_mark_blaze_ingame_pushed(gid);
            set_phase(gid, GamePhase::InGame);
        }
    }
    Some(out)
}

/// Push AuthToken (joining client) + GameReady attrib change to the pooled dedicated session.
///
/// `onGameAttributeUpdated` Join path reads AuthToken from the Game's player object. On dedicated
/// host-injection that player is the *joining client* (external roster entry), not the host
/// persona — AuthToken aimed at the host is dropped as "unknown local player".
fn enqueue_game_ready_to_dedicated(gid: i64) {
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

/// Called from `orchestrate_client_reset` when a pooled dedicated is assigned.
pub fn begin_reset_orchestration(gid: i64, client_session_id: u64, dedicated_session_id: u64) {
    orchestration().lock().insert(
        gid,
        GidOrchestration {
            client_session_id,
            dedicated_session_id: Some(dedicated_session_id),
            dedicated_host_ready: false,
            mesh_active_connected: false,
            pending_mesh_pid: None,
            deferred_join_pushes: None,
        },
    );
}

/// Store client join notifies until the dedicated host completes setup (finalize + advance).
pub fn defer_client_join_pushes(gid: i64, client_session_id: u64, pushes: Vec<super::fireframe::OutgoingPush>) {
    let mut m = orchestration().lock();
    let entry = m.entry(gid).or_insert_with(|| GidOrchestration {
        client_session_id,
        dedicated_session_id: super::dedicated_pool::peek_dedicated_for_gid(gid),
        dedicated_host_ready: false,
        mesh_active_connected: false,
        pending_mesh_pid: None,
        deferred_join_pushes: None,
    });
    entry.client_session_id = client_session_id;
    entry.deferred_join_pushes = Some(pushes);
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
    if entry.dedicated_host_ready {
        return None;
    }
    entry.dedicated_host_ready = true;
    let client_sid = entry.client_session_id;
    let pending_mesh_pid = entry.pending_mesh_pid.take();
    let mut out = entry.deferred_join_pushes.take().unwrap_or_default();
    drop(m);

    if let Ok(host_pushes) = super::fireframe::pushes_host_state_advance_for_client(gid) {
        out.extend(host_pushes);
    }
    // ACTIVE_CONNECTED + GameReady must follow NotifyGameSetup -- never from an early mesh
    // that raced the deferred join batch. Stay INITIALIZING until after GameReady/engine connect.
    if let Some(pid) = pending_mesh_pid {
        orchestration().lock().entry(gid).and_modify(|e| {
            e.mesh_active_connected = true;
        });
        if let Some(pushes) = mesh_active_connected_and_game_ready_pushes(gid, pid) {
            out.extend(pushes);
        }
    }
    Some((client_sid, out))
}

/// Client reported mesh via `updateMeshConnection`. Defer until host setup when orchestrating reset.
pub fn on_client_mesh_update(gid: i64, pid: i64) -> MeshUpdateResult {
    let mut m = orchestration().lock();
    let Some(entry) = m.get_mut(&gid) else {
        drop(m);
        return MeshUpdateResult::Push(
            mesh_active_connected_and_game_ready_pushes(gid, pid).unwrap_or_default(),
        );
    };
    if !entry.dedicated_host_ready {
        entry.pending_mesh_pid = Some(pid);
        super::msgsystem::log::log_orch_debug(&format!(
            "Mesh update deferred until host is ready (game {gid})"
        ));
        return MeshUpdateResult::DeferredUntilHostReady;
    }
    entry.mesh_active_connected = true;
    drop(m);
    MeshUpdateResult::Push(
        mesh_active_connected_and_game_ready_pushes(gid, pid).unwrap_or_default(),
    )
}

pub fn on_cmd220_delivered_to_dedicated(gid: i64) {
    super::msgsystem::log::log_orch_milestone(&format!(
        "Dedicated received match assignment -- starting orchestration (game {gid})"
    ));
    let _ = orchestration().lock().entry(gid).or_insert_with(|| GidOrchestration {
        client_session_id: super::dedicated_pool::client_session_for_gid(gid).unwrap_or(0),
        dedicated_session_id: super::dedicated_pool::peek_dedicated_for_gid(gid),
        dedicated_host_ready: false,
        mesh_active_connected: false,
        pending_mesh_pid: None,
        deferred_join_pushes: None,
    });

    let gid_spawn = gid;
    tokio::spawn(async move {
        // Brief delay so dedicated MsgSysHost + SimuCloud listener can come up after cmd 220.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        match super::msgsystem::simucloud::orchestrate_create_game(gid_spawn).await {
            Ok(()) => super::msgsystem::log::log_orch_milestone(&format!(
                "Match orchestration complete (game {gid_spawn})"
            )),
            Err(e) => super::msgsystem::log::log_orch_milestone(&format!(
                "Match orchestration failed (game {gid_spawn}): {e}"
            )),
        }
    });
}

/// Returns `true` the first time we push PRE_GAME for this gid.
pub fn try_mark_blaze_pregame_pushed(gid: i64) -> bool {
    blaze_pregame_pushed().lock().insert(gid)
}

pub fn blaze_pregame_already_pushed(gid: i64) -> bool {
    blaze_pregame_pushed().lock().contains(&gid)
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

/// Retail default when the lobby has not chosen a map yet.
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
            GamePhase::PreGame => "PreGame",
            GamePhase::InGame => "InGame",
            GamePhase::PostGame => "PostGame",
            GamePhase::Resetable => "Resetable",
        }
    }
}

const GSTA_RESETABLE: i32 = 0x07;
const NTOP_DEDICATED: i32 = 1;
const FIT_SCORE_DEFAULT: i32 = 100;

const AI_PERSONA_MIN: i64 = 9_000_000_000;
const AI_PERSONA_MAX: i64 = 9_800_000_000;

fn next_ai_persona_id() -> i64 {
    let mut rng = rand::thread_rng();
    loop {
        let id = rng.gen_range(AI_PERSONA_MIN..AI_PERSONA_MAX);
        let in_use = games()
            .lock()
            .values()
            .any(|g| g.players.iter().any(|p| p.persona_id == id));
        if !in_use {
            return id;
        }
    }
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
    /// Flat `ReplicatedGameData` wire bytes last sent in `NotifyGameSetup` / `getFullGameData`.
    replicated_wire: Option<Vec<u8>>,
    /// `PROS` roster rows last sent in `NotifyGameSetup` (reused for `getFullGameData`).
    pros_wire: Option<Vec<Vec<u8>>>,
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
/// GLA_ClassicGeneral ServerId — default AI opponent.
const DEFAULT_GENERAL_GLA_CLASSIC: &str = "580378690";

/// Default Blaze player ATTR for a human slot (CreateGame / PROS).
/// Native GameConfigs / `sub_A52D00` reads `_id`/`_team`/`_startpoint`/`_isai`/`_faction`;
/// MsgSys ServerHello also needs `_general` = RtsGeneral.ServerId (HashId).
/// Alpha has no USA generals — default APA Classic so CreateGame is never general=0.
fn default_human_attribs(slot: i32, team: i32) -> IndexMap<String, String> {
    let mut attribs = IndexMap::new();
    attribs.insert("_faction".to_string(), "APA".to_string());
    attribs.insert("_isai".to_string(), "0".to_string());
    attribs.insert("_team".to_string(), team.max(1).to_string());
    attribs.insert("_startpoint".to_string(), (slot + 1).max(1).to_string());
    attribs.insert("_general".to_string(), DEFAULT_GENERAL_APA_CLASSIC.to_string());
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
                player.slot = (s - 1).max(0);
            }
        }
        "_isai" => player.is_ai = value == "1" || value.eq_ignore_ascii_case("true"),
        _ => {}
    }
}

fn merge_pending_into_player(gid: i64, player: &mut CncPlayer) {
    // Merge pid=0 (pre-auth host) then exact persona so late persona sync wins keys.
    let overlays = {
        let pending = pending_player_attrs().lock();
        let mut layers = Vec::new();
        if let Some(by_pid) = pending.get(&gid) {
            if let Some(a) = by_pid.get(&0) {
                layers.push(a.clone());
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
    ensure_general_attr(player);
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

fn ensure_general_attr(player: &mut CncPlayer) {
    let gen_missing = player
        .attribs
        .get("_general")
        .map(|s| s.trim().is_empty() || s.trim() == "0")
        .unwrap_or(true);
    if !gen_missing {
        return;
    }
    let faction = player
        .attribs
        .get("_faction")
        .map(|s| s.as_str())
        .unwrap_or("APA");
    // Alpha: USA has PlayerData but no StaticData/Generals — coerce to APA Classic.
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

/// Record lobby-selected player attrs for a gid/persona. Survives `seed_from_reset` and is
/// applied into live roster rows when present. `persona_id == 0` targets the host placeholder.
pub fn set_pending_player_attrs(gid: i64, persona_id: i64, attrs: IndexMap<String, String>) {
    if attrs.is_empty() {
        return;
    }
    {
        let mut pending = pending_player_attrs().lock();
        let by_pid = pending.entry(gid).or_default();
        let slot = by_pid.entry(persona_id).or_default();
        for (k, v) in &attrs {
            slot.insert(k.clone(), v.clone());
        }
    }
    // Live game: apply immediately (also covers post-create tweaks).
    // Do not hold PENDING while locking GAMES (seed holds GAMES → PENDING).
    let mut m = games().lock();
    if let Some(game) = m.get_mut(&gid) {
        let targets: Vec<usize> = if persona_id == 0 {
            game.players
                .iter()
                .enumerate()
                .filter(|(_, p)| p.persona_id == game.host_persona || !p.is_ai)
                .map(|(i, _)| i)
                .take(1)
                .collect()
        } else {
            game.players
                .iter()
                .enumerate()
                .filter(|(_, p)| p.persona_id == persona_id)
                .map(|(i, _)| i)
                .collect()
        };
        for i in targets {
            if let Some(player) = game.players.get_mut(i) {
                for (k, v) in &attrs {
                    apply_attr_to_player(player, k, v);
                }
            }
        }
    }
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
            if start < 1 {
                ok = false;
                notes.push("startpoint < 1");
                issues.push(format!("player {} startpoint={}", p.persona_id, start));
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
    serde_json::json!({
        "ok": issues.is_empty() && game.is_some(),
        "gid": gid,
        "map_path": map,
        "phase": format!("{:?}", get_phase(gid)),
        "player_count": players_json.len(),
        "players": players_json,
        "pending_attrs": pending_json,
        "issues": issues,
    })
}

/// Default Blaze player ATTR for an AI slot.
fn default_general_for_faction(faction: &str) -> &'static str {
    match faction.trim().to_ascii_uppercase().as_str() {
        "APA" | "CHINA" | "CHI" => DEFAULT_GENERAL_APA_CLASSIC,
        "ESC" | "EU" => "232716472", // EU_ClassicGeneral
        "GLA" => DEFAULT_GENERAL_GLA_CLASSIC,
        // USA / unknown: Alpha has no USA generals — fall back to APA Classic.
        _ => DEFAULT_GENERAL_APA_CLASSIC,
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
    clear_blaze_push_flags(gid);
    let gnam = TdfEncoder::find_string_field(request_payload, "GNAM")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Skirmish".to_string());
    let host = host_persona();
    let host_name = host_display_name();
    let uuid = resolve_uuid(request_payload);
    let mut host_player = CncPlayer {
        persona_id: host,
        display_name: host_name,
        slot: 0,
        team: 1,
        is_ai: false,
        attribs: default_human_attribs(0, 1),
        custom_data: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    merge_pending_into_player(gid, &mut host_player);
    let map_path = get_map_path(gid);
    let game = CncGame {
        gid,
        name: gnam,
        host_persona: host,
        max_players: 8,
        players: vec![host_player],
        uuid,
        phase: GamePhase::Resetable,
        map_path,
        replicated_wire: None,
        pros_wire: None,
    };
    games().lock().insert(gid, game);
}

pub fn seed_from_join(gid: i64) {
    if games().lock().contains_key(&gid) {
        return;
    }
    let map_path = get_map_path(gid);
    let mut m = games().lock();
    if m.contains_key(&gid) {
        return;
    }
    let host = host_persona();
    let host_name = host_display_name();
    let mut host_player = CncPlayer {
        persona_id: host,
        display_name: host_name,
        slot: 0,
        team: 1,
        is_ai: false,
        attribs: default_human_attribs(0, 1),
        custom_data: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    merge_pending_into_player(gid, &mut host_player);
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
    clear_blaze_push_flags(gid);
}

pub fn set_map_path(gid: i64, map_path: &str) {
    PENDING_MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .insert(gid, map_path.to_string());
    if let Some(game) = games().lock().get_mut(&gid) {
        game.map_path = map_path.to_string();
    }
}

pub fn get_map_path(gid: i64) -> String {
    let existing = games()
        .lock()
        .get(&gid)
        .map(|g| g.map_path.clone())
        .unwrap_or_default();
    if !existing.is_empty() {
        return existing;
    }
    PENDING_MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .get(&gid)
        .cloned()
        .unwrap_or_default()
}

/// Map path for the active lobby session -- used when the MessageSystem client connects before we
/// know which gid they belong to (retail skirmish uses gid 1).
pub fn active_map_path() -> String {
    let path = get_map_path(1);
    if !path.is_empty() {
        return path;
    }
    let pending = PENDING_MAPS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock();
    if let Some(p) = pending.values().find(|p| !p.is_empty()) {
        return p.clone();
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

    let player = CncPlayer {
        persona_id: ai_id,
        display_name: ai_name,
        slot,
        team: 2,
        is_ai: true,
        attribs,
        custom_data: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
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

/// Add the joining **client** (non-AI) to the game roster if not already present, and return the
/// roster entry. Mirrors [`add_queued_player`] but for a real player driving `NotifyPlayerJoining`
/// -- the game is seeded with the host only, so without this the client is never in the roster.
pub fn ensure_client_player(gid: i64, persona_id: i64, display_name: &str) -> Option<CncPlayer> {
    seed_from_join(gid);
    let player = {
        let mut m = games().lock();
        let game = m.get_mut(&gid)?;
        if let Some(existing) = game
            .players
            .iter()
            .find(|p| p.persona_id == persona_id)
            .cloned()
        {
            return Some(existing);
        }
        let slot = next_free_slot(&game.players);
        let player = CncPlayer {
            persona_id,
            display_name: if display_name.is_empty() {
                format!("Player{}", slot + 1)
            } else {
                display_name.to_string()
            },
            slot,
            team: 1,
            is_ai: false,
            attribs: default_human_attribs(slot, 1),
            custom_data: IndexMap::new(),
            stat: PROS_STAT_ACTIVE_CONNECTING,
        };
        game.players.push(player.clone());
        player
    };
    // refresh_pros_wire_for_gid locks GAMES itself; release the mutation lock first.
    refresh_pros_wire_for_gid(gid);
    Some(player)
}

pub fn set_player_attribute(gid: i64, persona_id: i64, key: &str, value: &str) -> bool {
    // Always stash so createGame / seed_from_reset cannot wipe lobby choices.
    {
        let mut pending = pending_player_attrs().lock();
        pending
            .entry(gid)
            .or_default()
            .entry(persona_id)
            .or_default()
            .insert(key.to_string(), value.to_string());
    }
    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return true;
    };
    let Some(player) = game.players.iter_mut().find(|p| p.persona_id == persona_id) else {
        return true;
    };
    if player.attribs.get(key).map(String::as_str) == Some(value) {
        return false;
    }
    apply_attr_to_player(player, key, value);
    true
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
    // Packed-tag ascending order (ATTR < GID < PID) -- same visitation rule as CDAT.
    // Emitting GID/PID first leaves ATTR unvisited → client logs ATTR=[] and AuthToken stays null.
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
    // BlazeSDK NotifyPlayerCustomDataChange: CDAT is `blob mCustomData` (wire type 0x02).
    // Encoding it as MAP (0x05) breaks visitation so GID/PID stay 0 → "unknown game(0)".
    // Prefer AuthToken bytes as the blob body (CNC join uses player ATTR for the string).
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
    // `NetworkAddress` union member 2 = `IpPairAddress` (see Blaze `networkaddress.tdf`).
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

/// `NotifyGameSetup::PROS` -- minimal row (no `BLOB` / empty `PATT`; persona on `TIME` @ +208).
pub fn build_notify_pros_entry(player: &CncPlayer, gid: i64) -> Vec<u8> {
    let mut out = Vec::new();
    append_pros_core_fields(&mut out, player, gid, false);
    out
}

/// `getFullGameData::PROS` -- extended retail row (`PNET` NetworkAddress union after `PID`).
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
                attribs: default_human_attribs(0, 1),
                custom_data: IndexMap::new(),
                stat: PROS_STAT_ACTIVE_CONNECTING,
            };
            vec![build_gfgd_pros_entry(&p, gid)]
        })
}

pub fn all_game_gids() -> Vec<i64> {
    let mut gids: Vec<i64> = games().lock().keys().copied().collect();
    gids.sort_unstable();
    gids
}

pub fn players_for_gid(gid: i64) -> Vec<CncPlayer> {
    games()
        .lock()
        .get(&gid)
        .map(|g| g.players.clone())
        .unwrap_or_default()
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
    out.extend_from_slice(&TdfEncoder::encode_int("GSTA", GSTA_RESETABLE));
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
                attribs: default_human_attribs(0, 1),
                custom_data: IndexMap::new(),
                stat: PROS_STAT_ACTIVE_CONNECTING,
            };
            vec![build_plst_entry(&p)]
        })
}
