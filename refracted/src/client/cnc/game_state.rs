//! In-memory CNC lobby/game roster shared by GMGR replies and notify payloads.

use indexmap::IndexMap;
use parking_lot::Mutex;
use rand::Rng;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::OnceLock;

use crate::blaze::tdf::TdfEncoder;
use crate::common::error::{BlazeError, BlazeResult};
use crate::session::get_user_session;

const PROS_STAT_ACTIVE_CONNECTING: i32 = 2;
const PROS_STAT_ACTIVE: i32 = 0;
const STAS_IN_GAME: i32 = 2;

static GAMES: OnceLock<Mutex<HashMap<i64, CncGame>>> = OnceLock::new();
static LAST_ADD_QUEUED: OnceLock<Mutex<Option<(i64, CncPlayer)>>> = OnceLock::new();
static LAST_ATTR_CHANGE: OnceLock<Mutex<Option<(i64, i64, IndexMap<String, String>)>>> =
    OnceLock::new();
static NEXT_BROWSER_LIST_ID: AtomicI64 = AtomicI64::new(1);
static LAST_GAME_LIST_SNAPSHOT: OnceLock<Mutex<Option<(i64, Vec<i64>)>>> = OnceLock::new();

const GSTA_RESETABLE: i32 = 0x07;
const NTOP_DEDICATED: i32 = 1;
const PLAYER_STATE_ACTIVE_CONNECTING: i32 = 2;
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

#[derive(Clone, Debug)]
pub struct CncPlayer {
    pub persona_id: i64,
    pub display_name: String,
    pub slot: i32,
    pub team: i32,
    pub is_ai: bool,
    pub attribs: IndexMap<String, String>,
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

pub fn seed_from_reset(request_payload: &[u8], gid: i64) {
    let gnam = TdfEncoder::find_string_field(request_payload, "GNAM")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Skirmish".to_string());
    let host = host_persona();
    let host_name = host_display_name();
    let uuid = resolve_uuid(request_payload);
    let host_player = CncPlayer {
        persona_id: host,
        display_name: host_name,
        slot: 0,
        team: 1,
        is_ai: false,
        attribs: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    let game = CncGame {
        gid,
        name: gnam,
        host_persona: host,
        max_players: 8,
        players: vec![host_player],
        uuid,
        replicated_wire: None,
        pros_wire: None,
    };
    games().lock().insert(gid, game);
}

pub fn seed_from_join(gid: i64) {
    let mut m = games().lock();
    if m.contains_key(&gid) {
        return;
    }
    let host = host_persona();
    let host_name = host_display_name();
    let host_player = CncPlayer {
        persona_id: host,
        display_name: host_name,
        slot: 0,
        team: 1,
        is_ai: false,
        attribs: IndexMap::new(),
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    m.insert(
        gid,
        CncGame {
            gid,
            name: "Skirmish".to_string(),
            host_persona: host,
            max_players: 8,
            players: vec![host_player],
            uuid: new_uuid_v4_string(),
            replicated_wire: None,
        pros_wire: None,
        },
    );
}

pub fn get_game(gid: i64) -> Option<CncGame> {
    games().lock().get(&gid).cloned()
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

    let mut attribs = IndexMap::new();
    attribs.insert("_isai".to_string(), "1".to_string());
    attribs.insert("_faction".to_string(), "GLA".to_string());
    attribs.insert("_startpoint".to_string(), format!("{}", slot + 1));
    attribs.insert("_team".to_string(), "2".to_string());

    let player = CncPlayer {
        persona_id: ai_id,
        display_name: ai_name,
        slot,
        team: 2,
        is_ai: true,
        attribs,
        stat: PROS_STAT_ACTIVE_CONNECTING,
    };
    game.players.push(player.clone());
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

pub fn set_player_attribute(gid: i64, persona_id: i64, key: &str, value: &str) -> bool {
    let mut m = games().lock();
    let Some(game) = m.get_mut(&gid) else {
        return false;
    };
    let Some(player) = game.players.iter_mut().find(|p| p.persona_id == persona_id) else {
        return false;
    };
    if player.attribs.get(key).map(String::as_str) == Some(value) {
        return false;
    }
    player.attribs.insert(key.to_string(), value.to_string());
    match key {
        "_team" => {
            if let Ok(t) = value.parse::<i32>() {
                player.team = t;
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
    true
}

pub fn parse_set_player_attributes(payload: &[u8]) -> Option<(i64, i64, String, String)> {
    let applied = apply_set_player_attributes(payload)?;
    let (k, v) = applied.2.iter().next()?;
    Some((applied.0, applied.1, k.clone(), v.clone()))
}

pub fn apply_set_player_attributes(
    payload: &[u8],
) -> Option<(i64, i64, IndexMap<String, String>)> {
    let gid = TdfEncoder::find_int_field(payload, "GID").map(|v| v as i64)?;
    let pid = TdfEncoder::find_int_field(payload, "PID").map(|v| v as i64)?;
    let attrs = TdfEncoder::find_string_string_map_field(payload, "ATTR")?;
    if attrs.is_empty() {
        return None;
    }
    seed_from_join(gid);
    let mut changed = IndexMap::new();
    for (key, value) in &attrs {
        if set_player_attribute(gid, pid, key, value) {
            changed.insert(key.clone(), value.clone());
        }
    }
    *LAST_ATTR_CHANGE
        .get_or_init(|| Mutex::new(None))
        .lock() = if changed.is_empty() {
        None
    } else {
        Some((gid, pid, changed))
    };
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
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("GID ", gid));
    out.extend_from_slice(&TdfEncoder::encode_long("PID ", pid));
    out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
        "ATTR",
        attribs,
    ));
    out
}

pub fn build_pros_entry(player: &CncPlayer, gid: i64) -> Vec<u8> {
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
    out.extend_from_slice(&TdfEncoder::encode_int("TIDX", 0xFFFF));
    out.extend_from_slice(&TdfEncoder::encode_int("UID ", pid_i32));
    if !player.attribs.is_empty() {
        out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
            "ATTR",
            &player.attribs,
        ));
    }
    out
}

fn encode_empty_pnet_union() -> Vec<u8> {
    let mut out = Vec::new();
    let tag = TdfEncoder::make_tag("PNET");
    out.push(tag[0]);
    out.push(tag[1]);
    out.push(tag[2]);
    out.push(0x06);
    out.extend_from_slice(&TdfEncoder::encode_varint(127));
    out
}

/// Full `ReplicatedGamePlayer` row for `ListGameData::mGameRoster` (`getFullGameData` path).
pub fn build_gfgd_pros_entry(player: &CncPlayer, gid: i64) -> Vec<u8> {
    let gid_i32 = gid.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let pid = player.persona_id;
    let pid_i32 = pid.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_int("EXID", pid_i32));
    out.extend_from_slice(&TdfEncoder::encode_int("GID ", gid_i32));
    out.extend_from_slice(&TdfEncoder::encode_int("LOC ", 0));
    out.extend_from_slice(&TdfEncoder::encode_string("NAME", &player.display_name));
    out.extend_from_slice(&encode_empty_pnet_union());
    out.extend_from_slice(&TdfEncoder::encode_long("PID ", pid));
    out.extend_from_slice(&TdfEncoder::encode_int("SID ", 255));
    out.extend_from_slice(&TdfEncoder::encode_int("SLOT", player.slot));
    out.extend_from_slice(&TdfEncoder::encode_int("STAT", player.stat));
    out.extend_from_slice(&TdfEncoder::encode_int("TIDX", 0xFFFF));
    out.extend_from_slice(&TdfEncoder::encode_int("TIME", 0));
    out.extend_from_slice(&TdfEncoder::encode_object_id("UGID", 0, 0, 0));
    out.extend_from_slice(&TdfEncoder::encode_long("UID ", pid));
    if !player.attribs.is_empty() {
        out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
            "PATT",
            &player.attribs,
        ));
    }
    out
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
        .map(|g| g.players.iter().map(|p| build_pros_entry(p, gid)).collect())
        .unwrap_or_else(|| {
            let host = host_persona();
            let p = CncPlayer {
                persona_id: host,
                display_name: host_display_name(),
                slot: 0,
                team: 1,
                is_ai: false,
                attribs: IndexMap::new(),
                stat: PROS_STAT_ACTIVE_CONNECTING,
            };
            vec![build_pros_entry(&p, gid)]
        })
}

/// `ListGameData::mGameRoster` for `getFullGameData` — full retail player rows (not notify `PROS`).
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
                attribs: IndexMap::new(),
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
            player.stat = PROS_STAT_ACTIVE;
            player.slot = 0;
        }
    }
    game.pros_wire = None;
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
            attribs: IndexMap::new(),
            stat: PROS_STAT_ACTIVE,
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
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("pid", player.persona_id));
    out.extend_from_slice(&TdfEncoder::encode_string("name", &player.display_name));
    out.extend_from_slice(&TdfEncoder::encode_int("tidx", player.team.max(0)));
    out.extend_from_slice(&TdfEncoder::encode_int("stat", PLAYER_STATE_ACTIVE_CONNECTING));
    if !player.attribs.is_empty() {
        out.extend_from_slice(&TdfEncoder::encode_string_string_map_ordered(
            "patt",
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
    out.extend_from_slice(&TdfEncoder::encode_long("gid", gid));
    out.extend_from_slice(&TdfEncoder::encode_string("gnam", &game.name));
    out.extend_from_slice(&slot_capacities_vector("cap", pcap));
    out.extend_from_slice(&slot_capacities_vector("pcnt", pcnt));
    out.extend_from_slice(&TdfEncoder::encode_int("gsta", GSTA_RESETABLE));
    out.extend_from_slice(&TdfEncoder::encode_long("host", host));
    out.extend_from_slice(&TdfEncoder::encode_int("ntop", NTOP_DEDICATED));
    out.extend_from_slice(&encode_struct_list("rost", &roster));
    out.extend_from_slice(&TdfEncoder::encode_long_list("admn", &[host]));
    Some(out)
}

fn build_game_browser_match_data(game_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_struct("gam", game_data));
    out.extend_from_slice(&TdfEncoder::encode_int("fit", FIT_SCORE_DEFAULT));
    out
}

/// Blaze `GetGameListResponse` — metadata only; games follow in `NotifyGameListUpdate`.
pub fn build_get_game_list_response(list_id: i64, game_count: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&TdfEncoder::encode_long("glid", list_id));
    out.extend_from_slice(&TdfEncoder::encode_int("maxf", FIT_SCORE_DEFAULT));
    out.extend_from_slice(&TdfEncoder::encode_int("ngd", game_count as i32));
    out.extend_from_slice(&TdfEncoder::encode_bool("gmlt", true));
    out
}

/// Blaze `NotifyGameListUpdate` — populates a snapshot/subscription list (`cmd` 201 / 0xC9).
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
                attribs: IndexMap::new(),
                stat: PROS_STAT_ACTIVE_CONNECTING,
            };
            vec![build_plst_entry(&p)]
        })
}
