//! SimuCloud orchestrator (`PublicSimuCloudChannel` → `CreateGame`).

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Instant};

use super::log::{log_sim_debug, log_sim_milestone};
use super::wire::{Envelope, SimpleFrame, WireReader, WireWriter};

/// Dedicated SimuCloud host listen port (`SimuCloudChannelHost` in prism.cnc.network.dll).
pub const SIMUCLOUD_PORT: u16 = 18388;

const PROTOCOL_TYPE_ID_MAPPING_TYPE_ID: u16 = 0;
const PROTOCOL_VERSION_TYPE_ID: u16 = super::negotiation::PROTOCOL_VERSION_TYPE_ID;

/// One roster entry (mirrors `Rts.CnC.Messages.GameSetup.PlayerInfo`).
#[derive(Clone, Debug)]
pub struct PlayerInfo {
    pub player_id: u64,
    pub reconnect: bool,
    pub faction: i32,
    pub general_id: u32,
    pub team: i32,
    pub start_point: i32,
    pub difficulty: i32,
    pub is_ai: bool,
    pub allegiance_levels: Vec<f32>,
    pub skill_tree_unlocks: Vec<u32>,
    pub consumable_player_power: u32,
    /// Native skill-tree enable
    pub enable_skill_tree: bool,
}

pub const CREATE_GAME_OPTIONS_NONE: u32 = 0;
pub const CREATE_GAME_OPTIONS_ALLOW_RECONNECT: u32 = 1;
/// Do not put this on `PlayerInfo` — native deserializer has `EnableSkillTree` only.
pub const CREATE_GAME_OPTIONS_ENABLE_TECH_TREE: u32 = 0x20;
/// Generals rank / kill XP (`PlayerExperienceChange`).
pub const CREATE_GAME_OPTIONS_ENABLE_SPECIAL_ABILITIES: u32 = 0x40;
/// Oil as a second currency. Default off; derricks pay gold like Generals.
pub const CREATE_GAME_OPTIONS_ENABLE_OIL_ECONOMY: u32 = 0x80;
/// NS Resource Centers do not deplete (remaining stays). Default off.
pub const CREATE_GAME_OPTIONS_INFINITE_RESOURCE_CENTERS: u32 = 0x100;

fn create_game_options(
    enable_tech_tree: bool,
    enable_special_abilities: bool,
    enable_oil_economy: bool,
    infinite_resource_centers: bool,
) -> u32 {
    let mut options = CREATE_GAME_OPTIONS_ALLOW_RECONNECT;
    if enable_tech_tree {
        options |= CREATE_GAME_OPTIONS_ENABLE_TECH_TREE;
    }
    if enable_special_abilities {
        options |= CREATE_GAME_OPTIONS_ENABLE_SPECIAL_ABILITIES;
    }
    if enable_oil_economy {
        options |= CREATE_GAME_OPTIONS_ENABLE_OIL_ECONOMY;
    }
    if infinite_resource_centers {
        options |= CREATE_GAME_OPTIONS_INFINITE_RESOURCE_CENTERS;
    }
    options
}

const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const CONNECT_BUDGET: Duration = Duration::from_secs(12);
const READ_TIMEOUT: Duration = Duration::from_secs(5);
/// Cold dedicated load can exceed 45s; wait up to this for GameReady.
const GAME_READY_WAIT: Duration = Duration::from_secs(120);

/// Split a map path into `(MapName, DirPath)` for `SimuCloud.CreateGame`.
pub fn split_map_path(full: &str) -> (String, String) {
    let normalized = full.replace('\\', "/");
    if let Some((dir, leaf)) = normalized.rsplit_once('/') {
        let dir_path = if dir.is_empty() {
            String::new()
        } else {
            format!("{dir}/")
        };
        (leaf.to_string(), dir_path)
    } else {
        (normalized, String::new())
    }
}

fn protocol_version_frame() -> SimpleFrame {
    super::negotiation::encode_protocol_version()
}

/// Serialize `SimuCloud.CreateGame` payload (type id comes from negotiated PTM).
pub fn encode_create_game_payload(
    game_id: &[u8; 16],
    map_name: &str,
    dir_path: &str,
    info: &[PlayerInfo],
    options: u32,
) -> Vec<u8> {
    let mut w = WireWriter::new();
    w.write_string(Some(map_name));
    w.write_string(Some(dir_path));
    write_player_info_array(&mut w, info);
    w.write_bytes(game_id);
    w.write_rts_enum_i32(options as i32);
    w.into_bytes()
}

pub fn encode_create_game_envelope(
    type_id: u16,
    game_id: &[u8; 16],
    map_name: &str,
    dir_path: &str,
    info: &[PlayerInfo],
    options: u32,
) -> Envelope {
    Envelope {
        sender: Vec::new(),
        receiver: Vec::new(),
        type_id,
        payload: encode_create_game_payload(game_id, map_name, dir_path, info, options),
    }
}

fn write_player_info_array(w: &mut WireWriter, info: &[PlayerInfo]) {
    w.write_ref_array_len(info.len());
    for p in info {
        write_player_info(w, p);
    }
}

fn write_player_info(w: &mut WireWriter, p: &PlayerInfo) {
    w.write_u64(p.player_id);
    w.write_bool(p.reconnect);
    w.write_i32(p.faction);
    w.write_u32(p.general_id);
    w.write_i32(p.team);
    w.write_i32(p.start_point);
    w.write_i32(p.difficulty);
    w.write_bool(p.is_ai);
    w.write_ref_array_f32(&p.allegiance_levels);
    w.write_ref_array_u32(&p.skill_tree_unlocks);
    w.write_u32(p.consumable_player_power);
    w.write_bool(p.enable_skill_tree);
}

/// Parse `ProtocolTypeIdMapping` and find the type id for a message name suffix (e.g. `CreateGame`).
pub fn parse_type_id_for_suffix(ptm_payload: &[u8], suffix: &str) -> Option<u16> {
    if let Some(id) = scan_retail_ptm_type_id(ptm_payload, suffix) {
        return Some(id);
    }

    // Simple (id, string) table used by hand-built test fixtures.
    let mut r = WireReader::new(ptm_payload);
    if r.read_u8().ok()? != 1 {
        return None;
    }
    let count = r.read_var_i32().ok()? as usize;
    for _ in 0..count {
        let id = r.read_u16().ok()?;
        let name = match r.read_string() {
            Ok(Some(s)) => s,
            Ok(None) => continue,
            Err(_) => return None,
        };
        if simucloud_name_matches_suffix(&name, suffix) {
            return Some(id);
        }
    }
    None
}

fn simucloud_name_matches_suffix(name: &str, suffix: &str) -> bool {
    let marker = format!("SimuCloud.{suffix}");
    let Some(pos) = name.find(&marker) else {
        return false;
    };
    match name.as_bytes().get(pos + marker.len()) {
        None | Some(b',') | Some(b'+') => true,
        _ => false,
    }
}

/// `ProtocolTypeIdMapping` embeds each type as `01 {id} 00 01 {ref} 01` before the name.
fn scan_retail_ptm_type_id(ptm_payload: &[u8], suffix: &str) -> Option<u16> {
    for prefix in ["Rts.CnC.Messages.SimuCloud.", "SimuCloud."] {
        let marker = format!("{prefix}{suffix}");
        let needle = marker.as_bytes();
        let mut search_from = 0;
        while search_from < ptm_payload.len() {
            let Some(rel) = ptm_payload[search_from..]
                .windows(needle.len())
                .position(|window| window == needle)
            else {
                break;
            };
            let pos = search_from + rel;
            search_from = pos + 1;
            if !retail_ptm_name_boundary(ptm_payload, pos, needle.len()) {
                continue;
            }
            if pos >= 6
                && ptm_payload[pos - 6] == 1
                && ptm_payload[pos - 4] == 0
                && ptm_payload[pos - 3] == 1
                && ptm_payload[pos - 1] == 1
            {
                return Some(ptm_payload[pos - 5] as u16);
            }
        }
    }
    None
}

fn retail_ptm_name_boundary(ptm_payload: &[u8], pos: usize, marker_len: usize) -> bool {
    match ptm_payload.get(pos + marker_len) {
        None | Some(b',') | Some(b'+') => true,
        _ => false,
    }
}

fn uuid_to_guid_bytes(uuid: &str) -> [u8; 16] {
    let hex: String = uuid.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() == 32 {
        let mut out = [0u8; 16];
        for i in 0..16 {
            if let Ok(byte) = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16) {
                out[i] = byte;
            }
        }
        return out;
    }
    [0u8; 16]
}

/// Faction codes: None=0, USA=1, APA=2, ESC=3, GLA=4.
pub fn parse_faction_code(raw: &str) -> Option<i32> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(n) = s.parse::<i32>() {
        return match n {
            0..=4 => Some(n),
            _ => None,
        };
    }
    match s.to_ascii_uppercase().as_str() {
        "NONE" => Some(0),
        "USA" => Some(1),
        // APA is the Asia/Pacific faction; lobby/legacy aliases.
        "APA" | "CHINA" | "CHI" => Some(2),
        // ESC is European; lobby UI historically labeled it "EU".
        "ESC" | "EU" => Some(3),
        "GLA" => Some(4),
        _ => None,
    }
}

fn faction_from_player(attribs: &indexmap::IndexMap<String, String>, is_ai: bool) -> i32 {
    if let Some(raw) = attribs.get("_faction") {
        if let Some(id) = parse_faction_code(raw) {
            return id;
        }
    }
    // Empty ATTR (common when shell attr sync is skipped) must not encode as None=0.
    // Alpha default is APA (faction=2); USA has no generals in this build.
    if is_ai {
        4 // GLA
    } else {
        2 // APA
    }
}

fn i32_attr(attribs: &indexmap::IndexMap<String, String>, key: &str) -> Option<i32> {
    attribs.get(key).and_then(|s| s.trim().parse::<i32>().ok())
}

fn u32_attr(attribs: &indexmap::IndexMap<String, String>, key: &str) -> Option<u32> {
    attribs.get(key).and_then(|s| s.trim().parse::<u32>().ok())
}

#[cfg(test)]
mod general_id_parse_tests {
    use super::u32_attr;
    use indexmap::IndexMap;

    #[test]
    fn parses_rts_general_server_id_hash() {
        // APA_ClassicGeneral ServerId / GeneralsToLoad GeneralID
        let mut m = IndexMap::new();
        m.insert("_general".into(), "2914080600".into());
        assert_eq!(u32_attr(&m, "_general"), Some(2914080600));
        // APA_AtomicGeneral exceeds i32::MAX — must not use i32 parse
        m.insert("_general".into(), "3919700239".into());
        assert_eq!(u32_attr(&m, "_general"), Some(3919700239));
    }
}

/// Allegiance[17]: 0 at index 0; +1.0 if i == team else -1.0.
fn allegiance_for_team(team: i32) -> Vec<f32> {
    let mut levels = vec![0.0_f32; 17];
    for i in 1..17 {
        levels[i] = if i as i32 == team { 1.0 } else { -1.0 };
    }
    levels
}

fn roster_from_game(game: &super::super::game_state::CncGame) -> Vec<PlayerInfo> {
    let gid = game.gid;
    game.players
        .iter()
        .map(|p| {
            let team = i32_attr(&p.attribs, "_team").unwrap_or(p.team).max(1);
            let start_point = super::super::game_state::effective_startpoint_for_player(gid, p)
                .max(0);
            let start_point = if start_point > 0 {
                start_point
            } else {
                i32_attr(&p.attribs, "_startpoint")
                    .filter(|&s| s > 0)
                    .unwrap_or(p.slot + 1)
            };
            let difficulty = i32_attr(&p.attribs, "_difficulty").unwrap_or(0);
            let general_id = u32_attr(&p.attribs, "_general").unwrap_or(0);
            let general_id = if general_id != 0 {
                general_id
            } else {
                // Mirror game_state ensure_general_attr — never ship CreateGame with general=0.
                let faction = p
                    .attribs
                    .get("_faction")
                    .map(String::as_str)
                    .unwrap_or("APA");
                match faction.trim().to_ascii_uppercase().as_str() {
                    "ESC" | "EU" => 232716472,
                    "GLA" => 580378690,
                    _ => 2914080600, // APA Classic (also USA / missing)
                }
            };
            let consumable = u32_attr(&p.attribs, "_consumable")
                .or_else(|| u32_attr(&p.attribs, "_consumableplayerpower"))
                .unwrap_or(0);
            PlayerInfo {
                player_id: p.persona_id as u64,
                reconnect: false,
                faction: faction_from_player(&p.attribs, p.is_ai),
                general_id,
                team,
                start_point,
                difficulty,
                is_ai: p.is_ai,
                allegiance_levels: allegiance_for_team(team),
                skill_tree_unlocks: Vec::new(),
                consumable_player_power: consumable,
                enable_skill_tree: true,
            }
        })
        .collect()
}

async fn read_simple_frame(stream: &mut TcpStream) -> std::io::Result<SimpleFrame> {
    let mut header = [0u8; 6];
    timeout(READ_TIMEOUT, stream.read_exact(&mut header))
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "frame header timeout"))??;
    let payload_len = u32::from_le_bytes([header[2], header[3], header[4], header[5]]) as usize;
    let mut payload = vec![0u8; payload_len];
    if payload_len > 0 {
        timeout(READ_TIMEOUT, stream.read_exact(&mut payload))
            .await
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::TimedOut, "frame payload timeout")
            })??;
    }
    Ok(SimpleFrame {
        type_id: u16::from_le_bytes([header[0], header[1]]),
        payload,
    })
}

async fn write_simple_frame(stream: &mut TcpStream, frame: &SimpleFrame) -> std::io::Result<()> {
    stream.write_all(&frame.write()).await
}

async fn negotiate_type_ids(stream: &mut TcpStream) -> std::io::Result<(u16, u16, u16)> {
    let pv = protocol_version_frame();
    log_sim_debug(&format!(
        "TX ProtocolVersion typeId={} payloadLen={}",
        pv.type_id,
        pv.payload.len()
    ));
    write_simple_frame(stream, &pv).await?;

    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        let frame = match read_simple_frame(stream).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                log_sim_debug(
                    "Dedicated closed during protocol negotiation (check SimuCloud host and protocol version)",
                );
                return Err(std::io::Error::new(
                    e.kind(),
                    "SimuCloud closed during ProtocolVersion/PTM negotiation",
                ));
            }
            Err(e) => return Err(e),
        };
        let payload_preview = if frame.payload.len() >= 4 {
            format!(
                "ver={} hex={:02x?}",
                u32::from_le_bytes(frame.payload[0..4].try_into().unwrap_or([0; 4])),
                &frame.payload[..frame.payload.len().min(8)]
            )
        } else {
            format!("hex={:02x?}", frame.payload)
        };
        log_sim_debug(&format!(
            "RX typeId={} payloadLen={} {payload_preview}",
            frame.type_id,
            frame.payload.len()
        ));
        if frame.type_id == PROTOCOL_VERSION_TYPE_ID {
            log_sim_debug("Host sent ProtocolVersion during negotiation");
            continue;
        }
        if frame.type_id == PROTOCOL_TYPE_ID_MAPPING_TYPE_ID {
            log_sim_debug(&format!("PTM received ({} bytes)", frame.payload.len()));
            let create = parse_type_id_for_suffix(&frame.payload, "CreateGame").ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "PTM received but CreateGame type id not found",
                )
            })?;
            let ready = parse_type_id_for_suffix(&frame.payload, "GameReady").unwrap_or(0);
            let failure =
                parse_type_id_for_suffix(&frame.payload, "CreateGameFailure").unwrap_or(0);
            return Ok((create, ready, failure));
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "timed out waiting for ProtocolTypeIdMapping",
    ))
}

/// Connect to the dedicated SimuCloud host, send `CreateGame`, read `GameReady`.
pub async fn orchestrate_create_game(gid: i64) -> std::io::Result<()> {
    super::super::game_state::clear_match_connection_lost(gid);
    super::super::game_state::seed_from_join(gid);
    super::super::game_state::resolve_startpoints_before_create(gid);
    log_sim_debug(&format!("CreateGame roster ready gid={gid}"));
    let game = match super::super::game_state::get_game(gid) {
        Some(g) => g,
        None => {
            log_sim_debug(&format!("No game state for gid={gid}; skipping CreateGame"));
            return Ok(());
        }
    };

    let map_path = {
        let p = super::super::game_state::get_map_path(gid);
        if p.is_empty() {
            super::super::game_state::active_map_path()
        } else {
            p
        }
    };
    let (map_name, dir_path) = split_map_path(&map_path);
    let roster = roster_from_game(&game);
    if roster.is_empty() {
        log_sim_debug(&format!("Empty roster for gid={gid}; skipping CreateGame"));
        return Ok(());
    }
    // CreateGame already has resolved picks; clear lobby attrs so rematch does not reuse them.
    super::super::game_state::flush_lobby_startpoints(gid);

    let game_id = uuid_to_guid_bytes(&game.uuid);
    let upstream = crate::client::cnc::dedicated_pool::simucloud_upstream_for_gid(gid);
    let deadline = Instant::now() + CONNECT_BUDGET;
    let mut stream = None;
    while Instant::now() < deadline {
        match timeout(CONNECT_TIMEOUT, TcpStream::connect(upstream)).await {
            Ok(Ok(s)) => {
                stream = Some(s);
                break;
            }
            Ok(Err(e)) => {
                log_sim_debug(&format!("Connect to {upstream} failed ({e}); retrying"));
            }
            Err(_) => {}
        }
        sleep(Duration::from_millis(200)).await;
    }

    let mut stream = match stream {
        Some(s) => s,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!(
                    "SimuCloud host not listening on {upstream} within {:?} \
                     -- deploy prism.cnc.network.dll (with SimuCloudChannelHost) next to \
                     cnc.server.exe; dedicated RuntimeLog should show \
                     [.NET] co-starting SimuCloud and [RTS/simucloud] host listening",
                    CONNECT_BUDGET
                ),
            ));
        }
    };

    log_sim_milestone(&format!(
        "Starting match setup -- map \"{map_name}\", {} player(s), specialAbilities={} techTree={} oilEconomy={} infiniteResources={}",
        roster.len(),
        game.enable_special_abilities,
        game.enable_tech_tree,
        game.enable_oil_economy,
        game.enable_infinite_resource_centers
    ));
    for p in &roster {
        log_sim_debug(&format!(
            "CreateGame roster pid={} faction={} team={} startPoint={} ai={}",
            p.player_id, p.faction, p.team, p.start_point, p.is_ai
        ));
    }
    log_sim_debug(&format!("Connected to dedicated orchestrator at {upstream}"));

    let (create_type_id, game_ready_type_id, failure_type_id) =
        if let Ok(override_id) = std::env::var("REFRACTED_SIMUCLOUD_CREATE_TYPE_ID") {
            let id = override_id.trim().parse::<u16>().map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid REFRACTED_SIMUCLOUD_CREATE_TYPE_ID",
                )
            })?;
            (id, 0, 0)
        } else {
            negotiate_type_ids(&mut stream).await?
        };

    log_sim_debug(&format!(
        "Protocol map: CreateGame={create_type_id} GameReady={game_ready_type_id} Failure={failure_type_id}"
    ));

    // SimuCloud uses SimpleFrames (same as ProtocolVersion/PTM), not routing Envelopes.
    let create_frame = SimpleFrame {
        type_id: create_type_id,
        payload: encode_create_game_payload(
            &game_id,
            &map_name,
            &dir_path,
            &roster,
            create_game_options(
                game.enable_tech_tree,
                game.enable_special_abilities,
                game.enable_oil_economy,
                game.enable_infinite_resource_centers,
            ),
        ),
    };
    write_simple_frame(&mut stream, &create_frame).await?;

    // Dedicated waits for ServerLevel load before CreateGame. Keep polling until GAME_READY_WAIT.
    const REPLY_READ_TIMEOUT: Duration = Duration::from_secs(15);
    let reply_deadline = Instant::now() + GAME_READY_WAIT;
    let mut idle_log_at = Instant::now();
    while Instant::now() < reply_deadline {
        let remaining = reply_deadline.saturating_duration_since(Instant::now());
        let chunk = REPLY_READ_TIMEOUT.min(remaining);
        let frame = {
            let mut header = [0u8; 6];
            match timeout(chunk, stream.read_exact(&mut header)).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    return Err(std::io::Error::new(
                        e.kind(),
                        format!("SimuCloud closed while waiting for GameReady: {e}"),
                    ));
                }
                Err(_) => {
                    if idle_log_at.elapsed() >= Duration::from_secs(15) {
                        log_sim_debug(&format!(
                            "Still waiting for GameReady from dedicated (~{}s left)...",
                            remaining.as_secs()
                        ));
                        idle_log_at = Instant::now();
                    }
                    continue;
                }
            }
            let payload_len =
                u32::from_le_bytes([header[2], header[3], header[4], header[5]]) as usize;
            let mut payload = vec![0u8; payload_len];
            if payload_len > 0 {
                let payload_wait = REPLY_READ_TIMEOUT.min(
                    reply_deadline.saturating_duration_since(Instant::now()),
                );
                match timeout(payload_wait, stream.read_exact(&mut payload)).await {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => {
                        return Err(std::io::Error::new(
                            e.kind(),
                            format!("SimuCloud closed while reading GameReady payload: {e}"),
                        ));
                    }
                    Err(_) => continue,
                }
            }
            SimpleFrame {
                type_id: u16::from_le_bytes([header[0], header[1]]),
                payload,
            }
        };
        log_sim_debug(&format!(
            "Dedicated reply typeId={} ({} bytes)",
            frame.type_id,
            frame.payload.len()
        ));
        if game_ready_type_id != 0 && frame.type_id == game_ready_type_id {
            super::log::flush_relay_log_compactor();
            log_sim_milestone(&format!("Match ready on dedicated (game {gid})"));
            return Ok(());
        }
        if failure_type_id != 0 && frame.type_id == failure_type_id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "dedicated replied CreateGameFailure",
            ));
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "timed out waiting for GameReady",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_player(id: u64, faction: i32, team: i32) -> PlayerInfo {
        PlayerInfo {
            player_id: id,
            reconnect: false,
            faction,
            general_id: 0,
            team,
            start_point: 1,
            difficulty: 0,
            is_ai: false,
            allegiance_levels: vec![],
            skill_tree_unlocks: vec![],
            consumable_player_power: 0,
            enable_skill_tree: false,
        }
    }

    #[test]
    fn faction_codes_match_native_enum() {
        assert_eq!(parse_faction_code("USA"), Some(1));
        assert_eq!(parse_faction_code("usa"), Some(1));
        assert_eq!(parse_faction_code("APA"), Some(2));
        assert_eq!(parse_faction_code("China"), Some(2));
        assert_eq!(parse_faction_code("ESC"), Some(3));
        assert_eq!(parse_faction_code("EU"), Some(3));
        assert_eq!(parse_faction_code("GLA"), Some(4));
        assert_eq!(parse_faction_code("None"), Some(0));
        assert_eq!(parse_faction_code("4"), Some(4));
        assert_eq!(parse_faction_code("bogus"), None);
    }

    #[test]
    fn allegiance_marks_team_slot() {
        let a = allegiance_for_team(2);
        assert_eq!(a.len(), 17);
        assert_eq!(a[0], 0.0);
        assert_eq!(a[1], -1.0);
        assert_eq!(a[2], 1.0);
        assert_eq!(a[3], -1.0);
    }

    #[test]
    fn split_map_path_splits_parent_dir() {
        let (map, dir) = split_map_path("levels/SP/Alpha_Tutorial/Alpha_Tutorial");
        assert_eq!(map, "Alpha_Tutorial");
        assert_eq!(dir, "levels/SP/Alpha_Tutorial/");
    }

    #[test]
    fn create_game_encodes_map_first() {
        let gid = [0u8; 16];
        let payload = encode_create_game_payload(
            &gid,
            "levels/SP/Alpha_Tutorial/Alpha_Tutorial",
            "levels/SP/Alpha_Tutorial",
            &[sample_player(1201618778, 1, 0)],
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT,
        );
        assert!(payload.len() > 40);
    }

    #[test]
    fn create_game_options_ors_unused_lobby_bits() {
        assert_eq!(
            create_game_options(false, false, false, false),
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT
        );
        assert_eq!(
            create_game_options(true, false, false, false),
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT | CREATE_GAME_OPTIONS_ENABLE_TECH_TREE
        );
        assert_eq!(
            create_game_options(false, true, false, false),
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT | CREATE_GAME_OPTIONS_ENABLE_SPECIAL_ABILITIES
        );
        assert_eq!(
            create_game_options(false, false, true, false),
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT | CREATE_GAME_OPTIONS_ENABLE_OIL_ECONOMY
        );
        assert_eq!(
            create_game_options(true, true, true, false),
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT
                | CREATE_GAME_OPTIONS_ENABLE_TECH_TREE
                | CREATE_GAME_OPTIONS_ENABLE_SPECIAL_ABILITIES
                | CREATE_GAME_OPTIONS_ENABLE_OIL_ECONOMY
        );
        assert_eq!(
            create_game_options(false, false, false, true),
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT
                | CREATE_GAME_OPTIONS_INFINITE_RESOURCE_CENTERS
        );
    }

    #[test]
    fn create_game_options_enum_uses_rts_typecode() {
        let gid = [0u8; 16];
        let both = encode_create_game_payload(
            &gid,
            "Oasis",
            "Levels/MP/",
            &[sample_player(1201618778, 1, 0)],
            create_game_options(true, true, true, false),
        );
        let off = encode_create_game_payload(
            &gid,
            "Oasis",
            "Levels/MP/",
            &[sample_player(1201618778, 1, 0)],
            create_game_options(false, false, false, false),
        );
        assert_eq!(&both[both.len() - 5..], &[9, 0xE1, 0, 0, 0]);
        assert_eq!(&off[off.len() - 5..], &[9, 0x01, 0, 0, 0]);
    }

    #[test]
    fn split_map_path_matches_retail_splitpath() {
        let (map, dir) = split_map_path("Levels/SP/Alpha_Tutorial/Alpha_Tutorial");
        assert_eq!(map, "Alpha_Tutorial");
        assert_eq!(dir, "Levels/SP/Alpha_Tutorial/");
    }

    #[test]
    fn create_game_envelope_wraps_payload() {
        let gid = [0u8; 16];
        let env = encode_create_game_envelope(
            9,
            &gid,
            "Alpha_Tutorial",
            "levels/SP/Alpha_Tutorial/",
            &[sample_player(1201618778, 1, 0)],
            CREATE_GAME_OPTIONS_ALLOW_RECONNECT,
        );
        assert_eq!(env.type_id, 9);
        assert!(env.sender.is_empty());
        assert!(env.receiver.is_empty());
        let bytes = env.write();
        assert!(bytes.len() > env.payload.len() + 12);
    }

    #[test]
    fn parse_ptm_finds_create_game_suffix() {
        let mut w = WireWriter::new();
        w.write_u8(1);
        w.write_var_i32(2);
        w.write_u16(7);
        w.write_string(Some("Rts.CnC.Messages.SimuCloud.AddPlayers"));
        w.write_u16(9);
        w.write_string(Some("Rts.CnC.Messages.SimuCloud.CreateGame"));
        let id = parse_type_id_for_suffix(&w.into_bytes(), "CreateGame").unwrap();
        assert_eq!(id, 9);
    }

    #[test]
    fn parse_retail_simucloud_ptm_finds_create_game() {
        let Some(ptm) = super::super::negotiation::read_frame_dump("simucloud_ptm.bin") else {
            eprintln!("skip dump-parity: frames/simucloud_ptm.bin not present");
            return;
        };
        let payload_len = u32::from_le_bytes([ptm[2], ptm[3], ptm[4], ptm[5]]) as usize;
        let payload = &ptm[6..6 + payload_len];
        let create = parse_type_id_for_suffix(payload, "CreateGame").expect("CreateGame in simucloud_ptm.bin");
        assert_eq!(create, 2, "SimuCloud CreateGame typeId");
        let ready = parse_type_id_for_suffix(payload, "GameReady").expect("GameReady in simucloud_ptm.bin");
        assert_eq!(ready, 4);
    }

    #[test]
    fn parse_ptm_skips_null_type_name_entries() {
        let mut w = WireWriter::new();
        w.write_u8(1);
        w.write_var_i32(2);
        w.write_u16(99);
        w.write_u8(0); // null type name (retail PTM includes placeholder rows)
        w.write_u16(9);
        w.write_string(Some("Rts.CnC.Messages.SimuCloud.CreateGame"));
        let id = parse_type_id_for_suffix(&w.into_bytes(), "CreateGame").unwrap();
        assert_eq!(id, 9);
    }

    #[test]
    fn protocol_version_frame_matches_retail_client() {
        let frame = protocol_version_frame();
        assert_eq!(frame.type_id, PROTOCOL_VERSION_TYPE_ID);
        // Retail ClientNetWrapper sends payloadLen=7 (version + empty auth ref, not null).
        assert_eq!(frame.payload.len(), 7);
        let version = u32::from_le_bytes(frame.payload[0..4].try_into().unwrap());
        assert_eq!(
            version, 2,
            "ClientChannelDescriptor metadata <Version> is 2"
        );
    }
}
