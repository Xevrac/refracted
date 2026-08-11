//! MessageSystem bootstrap control messages (Client channel join path).
//!
//! Wire shapes match retail `PlayerMessages` (`Rts.CnC.Messages.Client.*`) and Prism
//! `DedicatedJoinBootstrap`: ClientHello → ServerHello / ServerReadyToStart / LoadMap →
//! ClientFinishedLoad → StartGame → AllowInputChange(true).
//! LoadMap drives RTS join load via native `RtsGameClient_onMessage` case 61 (MapId →
//! RtsSettings + camera). Playable HUD (`GameInput` OnAttach) requires Client state
//! Ingame (10) so RtsUI can load DefaultUILayout / bottomBarUI. AllowInputChange then
//! hides the LoadingScreen child that OnAttach cached.

use super::wire::{SimpleFrame, WireReader, WireWriter};

pub const PROTOCOL_TYPE_ID_MAPPING_TYPE_ID: u16 = 0;
pub const PROTOCOL_VERSION_TYPE_ID: u16 = 1;
pub const ALLOW_INPUT_CHANGE_TYPE_ID: u16 = 12;
pub const GENERAL_TAUNT_TYPE_ID: u16 = 92;
pub const CLIENT_FINISHED_LOAD_TYPE_ID: u16 = 27;
pub const CLIENT_HELLO_TYPE_ID: u16 = 28;
pub const LOAD_MAP_TYPE_ID: u16 = 100;
pub const PING_QUERY_TYPE_ID: u16 = 106;
pub const PING_REPLY_TYPE_ID: u16 = 107;
pub const REQUEST_RANDOM_GENERAL_TAUNT_TYPE_ID: u16 = 180;
pub const REQUEST_SPECIFIC_GENERAL_TAUNT_TYPE_ID: u16 = 186;
pub const SERVER_HELLO_TYPE_ID: u16 = 200;
pub const SERVER_READY_TO_START_TYPE_ID: u16 = 201;
pub const START_GAME_TYPE_ID: u16 = 215;

pub const DEFAULT_PLAYER_HANDLE: u32 = 1;
pub const DEFAULT_PERSONA_ID: u64 = 1_201_618_778;

pub fn frame_type_label(type_id: u16) -> &'static str {
    match type_id {
        0 => "ProtocolTypeIdMapping",
        1 => "ProtocolVersion",
        12 => "AllowInputChange",
        27 => "ClientFinishedLoad",
        28 => "ClientHello",
        92 => "GeneralTaunt",
        100 => "LoadMap",
        106 => "PingQuery",
        107 => "PingReply",
        180 => "RequestRandomGeneralTaunt",
        186 => "RequestSpecificGeneralTaunt",
        200 => "ServerHello",
        201 => "ServerReadyToStart",
        215 => "StartGame",
        _ => "Unknown",
    }
}

pub fn encode_server_hello(
    player_handle: u32,
    persona: u64,
    faction: u32,
    team: u32,
    general_id: u32,
) -> SimpleFrame {
    let allegiance = if team > 0 { 1.0_f32 } else { 0.0_f32 };
    let mut w = WireWriter::new();
    w.write_ref_array_u32(&[player_handle]);
    w.write_ref_array_u64(&[persona]);
    w.write_ref_array_u32(&[0]); // PlayerType
    w.write_ref_array_f32(&[allegiance]); // AllegianceLevel
    w.write_ref_array_u32(&[faction]);
    w.write_ref_array_u32(&[general_id]); // General
    w.write_ref_array_u32(&[team]);
    SimpleFrame {
        type_id: SERVER_HELLO_TYPE_ID,
        payload: w.into_bytes(),
    }
}

pub fn encode_server_hello_frame(
    player_handle: u32,
    persona: u64,
    faction: u32,
    team: u32,
    general_id: u32,
) -> Vec<u8> {
    encode_server_hello(player_handle, persona, faction, team, general_id).write()
}

pub fn encode_server_ready_to_start() -> SimpleFrame {
    SimpleFrame {
        type_id: SERVER_READY_TO_START_TYPE_ID,
        payload: Vec::new(),
    }
}

pub fn encode_server_ready_to_start_frame() -> Vec<u8> {
    encode_server_ready_to_start().write()
}

pub fn encode_load_map(player_handle: u32, map_id: &str) -> SimpleFrame {
    let mut w = WireWriter::new();
    w.write_u32(player_handle);
    w.write_string(Some(map_id));
    w.write_string(Some(""));
    w.write_string(Some(""));
    w.write_f32(0.0);
    w.write_f32(0.0);
    w.write_f32(0.0);
    // Quaternion.Identity
    w.write_f32(0.0);
    w.write_f32(0.0);
    w.write_f32(0.0);
    w.write_f32(1.0);
    SimpleFrame {
        type_id: LOAD_MAP_TYPE_ID,
        payload: w.into_bytes(),
    }
}

pub fn encode_load_map_frame(player_handle: u32, map_id: &str) -> Vec<u8> {
    encode_load_map(player_handle, map_id).write()
}

pub fn decode_load_map_id(payload: &[u8]) -> Option<String> {
    let mut r = WireReader::new(payload);
    let _handle = r.read_u32().ok()?;
    r.read_string().ok()?.filter(|s| !s.is_empty())
}

pub fn encode_ping_reply(client_start_time: u32, server_time: u32) -> SimpleFrame {
    let mut w = WireWriter::new();
    w.write_u32(client_start_time);
    w.write_u32(server_time);
    SimpleFrame {
        type_id: PING_REPLY_TYPE_ID,
        payload: w.into_bytes(),
    }
}

pub fn encode_ping_reply_frame(client_start_time: u32, server_time: u32) -> Vec<u8> {
    encode_ping_reply(client_start_time, server_time).write()
}

pub fn decode_ping_query(payload: &[u8]) -> Option<u32> {
    let mut r = WireReader::new(payload);
    r.read_u32().ok()
}

pub fn decode_client_hello(payload: &[u8]) -> Option<u64> {
    let mut r = WireReader::new(payload);
    r.read_u64().ok()
}

pub fn encode_start_game(player_id: u32, faction: u32) -> SimpleFrame {
    let mut w = WireWriter::new();
    w.write_u32(faction);
    w.write_ref_array_u32(&[player_id]);
    w.write_ref_array_u32(&[0]);
    w.write_ref_array_f32(&[0.0]);
    SimpleFrame {
        type_id: START_GAME_TYPE_ID,
        payload: w.into_bytes(),
    }
}

pub fn encode_start_game_frame(player_id: u32, faction: u32) -> Vec<u8> {
    encode_start_game(player_id, faction).write()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::negotiation::read_frame_dump;
    use super::super::wire::SimpleFrame;

    const ALPHA_TUTORIAL: &str = "levels/SP/Alpha_Tutorial/Alpha_Tutorial";

    #[test]
    fn load_map_matches_retail_dump() {
        let Some(expected) = read_frame_dump("load_map.bin") else {
            eprintln!("skip dump-parity: frames/load_map.bin not present");
            return;
        };
        let encoded = encode_load_map_frame(1, ALPHA_TUTORIAL);
        assert_eq!(encoded, expected, "LoadMap encoder must match retail dump");
    }

    #[test]
    fn server_hello_matches_retail_dump() {
        let Some(expected) = read_frame_dump("server_hello.bin") else {
            eprintln!("skip dump-parity: frames/server_hello.bin not present");
            return;
        };
        let encoded = encode_server_hello_frame(1, DEFAULT_PERSONA_ID, 0, 0, 0);
        assert_eq!(encoded, expected, "ServerHello encoder must match retail dump");
    }

    #[test]
    fn server_ready_matches_retail_dump() {
        let encoded = encode_server_ready_to_start_frame();
        assert_eq!(encoded.len(), 6, "ServerReadyToStart must be empty-payload type 201");
        assert_eq!(&encoded[0..2], &SERVER_READY_TO_START_TYPE_ID.to_le_bytes());
        let Some(expected) = read_frame_dump("server_ready.bin") else {
            eprintln!("skip dump-parity: frames/server_ready.bin not present");
            return;
        };
        assert_eq!(encoded, expected);
    }

    #[test]
    fn load_map_payload_len_scales_with_path() {
        let short = encode_load_map(1, "levels/SP/Foo/Foo");
        let long = encode_load_map(1, "levels/SP/Some_Long_Map_Name/Some_Long_Map_Name");
        assert!(long.payload.len() > short.payload.len());
        let (decoded, _) = SimpleFrame::try_read(&long.write()).unwrap().unwrap();
        assert_eq!(decoded.type_id, LOAD_MAP_TYPE_ID);
        assert_eq!(
            decode_load_map_id(&decoded.payload).as_deref(),
            Some("levels/SP/Some_Long_Map_Name/Some_Long_Map_Name")
        );
    }

    #[test]
    fn ping_reply_round_trip_header() {
        let frame = encode_ping_reply(12345, 67890);
        assert_eq!(frame.type_id, PING_REPLY_TYPE_ID);
        assert_eq!(frame.payload.len(), 8);
        assert_eq!(u32::from_le_bytes(frame.payload[0..4].try_into().unwrap()), 12345);
        assert_eq!(u32::from_le_bytes(frame.payload[4..8].try_into().unwrap()), 67890);
    }

    #[test]
    fn client_hello_decodes_persona() {
        let mut w = super::super::wire::WireWriter::new();
        w.write_u64(DEFAULT_PERSONA_ID);
        assert_eq!(decode_client_hello(&w.into_bytes()), Some(DEFAULT_PERSONA_ID));
    }

    #[test]
    fn start_game_has_expected_type_and_arrays() {
        let frame = encode_start_game(1, 0);
        assert_eq!(frame.type_id, START_GAME_TYPE_ID);
        assert!(frame.payload.len() >= 16);
        assert_eq!(u32::from_le_bytes(frame.payload[0..4].try_into().unwrap()), 0);
    }
}
