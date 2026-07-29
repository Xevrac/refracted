//! Offline / unit-test helpers for Victory Client MessageSystem join wire shapes.
//!
//! Production ServerHost lives in Prism (`prism.cnc.network.dll` / `DedicatedJoinBootstrap`).
//! Refracted only MITMs that session; these encoders are for dump parity tests.

use super::messages::{DEFAULT_PERSONA_ID, DEFAULT_PLAYER_HANDLE};

#[derive(Clone, Debug)]
pub struct JoinContext {
    pub map_path: String,
    pub player_handle: u32,
    pub persona: u64,
    pub faction: u32,
    pub team: u32,
}

impl Default for JoinContext {
    fn default() -> Self {
        Self {
            map_path: super::super::game_state::DEFAULT_MAP_PATH.to_string(),
            player_handle: DEFAULT_PLAYER_HANDLE,
            persona: DEFAULT_PERSONA_ID,
            faction: 0,
            team: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::messages::{
        encode_server_hello_frame, encode_server_ready_to_start_frame,
    };
    use super::super::negotiation::{SERVER_HELLO, SERVER_READY_TO_START};

    #[test]
    fn join_context_defaults_are_retail_shaped() {
        let ctx = JoinContext::default();
        assert_eq!(ctx.player_handle, 1);
        assert_eq!(ctx.persona, DEFAULT_PERSONA_ID);
        assert!(!ctx.map_path.is_empty());
    }

    #[test]
    fn bootstrap_frames_match_dumps() {
        assert_eq!(
            encode_server_hello_frame(1, DEFAULT_PERSONA_ID, 0, 0, 0),
            SERVER_HELLO
        );
        assert_eq!(encode_server_ready_to_start_frame(), SERVER_READY_TO_START);
    }
}
