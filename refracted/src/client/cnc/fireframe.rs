//! CnC **FireFrame** (size-prefixed) notification envelopes and post-RPC push sequences.
//! The global Blaze server only gates on `current_game == "cnc"` and performs I/O; payloads live here.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::common::error::BlazeResult;

#[derive(Clone)]
pub struct OutgoingPush {
    pub wire: Vec<u8>,
    pub component: u16,
    pub command: u16,
    pub tdf_body: Vec<u8>,
    pub blaze_send_label: &'static str,
    pub info_log_line: String,
}

pub fn notification_envelope(component: u16, command: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + payload.len());
    out.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    out.extend_from_slice(&component.to_be_bytes());
    out.extend_from_slice(&command.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0x2000u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(payload);
    out
}

static PENDING_PUSHES: OnceLock<Mutex<HashMap<u64, Vec<OutgoingPush>>>> = OnceLock::new();

fn pending_pushes() -> &'static Mutex<HashMap<u64, Vec<OutgoingPush>>> {
    PENDING_PUSHES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn take_pending_pushes(blaze_session_id: u64) -> Vec<OutgoingPush> {
    pending_pushes()
        .lock()
        .remove(&blaze_session_id)
        .unwrap_or_default()
}

pub fn enqueue_pending_pushes(blaze_session_id: u64, pushes: Vec<OutgoingPush>) {
    if pushes.is_empty() {
        return;
    }
    pending_pushes()
        .lock()
        .entry(blaze_session_id)
        .or_default()
        .extend(pushes);
}

pub fn pushes_after_reset_dedicated_server(request: &[u8]) -> BlazeResult<Vec<OutgoingPush>> {
    let gid = super::cnc_extract_reset_game_id(request);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameSetup + NotifyGameStateChange + NotifyPlatformHostInitialized after resetDedicatedServer (gid={})",
        gid
    );

    // Same order as `joinGame`: **NotifyGameSetup first** so `mGameMap` has the game before
    // `NotifyGameStateChange` / platform-host notifies (avoids "unknown local game" GMGR warnings).
    let setup = super::build_game_manager_notify_game_setup(request, gid)?;
    let wire_setup = notification_envelope(0x0004, 0x0014, &setup);
    let pl_setup = wire_setup.len();

    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_RESETABLE)?;
    let wire_gstate = notification_envelope(0x0004, 0x0064, &gstate);
    let pl_gstate = wire_gstate.len();

    let phost = super::build_game_manager_notify_platform_host_initialized(gid)?;
    let wire_phost = notification_envelope(0x0004, 0x0047, &phost);
    let pl_phost = wire_phost.len();

    let join_done = super::build_game_manager_notify_player_join_completed(gid)?;
    let wire_join_done = notification_envelope(0x0004, 0x001E, &join_done);
    let pl_join_done = wire_join_done.len();

    Ok(vec![
        OutgoingPush {
            wire: wire_setup,
            component: 0x0004,
            command: 0x0014,
            tdf_body: setup.to_vec(),
            blaze_send_label: "NotifyGameSetup after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyGameSetup Component=4, Command=20, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_setup
            ),
        },
        OutgoingPush {
            wire: wire_gstate,
            component: 0x0004,
            command: 0x0064,
            tdf_body: gstate.to_vec(),
            blaze_send_label: "NotifyGameStateChange after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_gstate
            ),
        },
        OutgoingPush {
            wire: wire_phost,
            component: 0x0004,
            command: 0x0047,
            tdf_body: phost.to_vec(),
            blaze_send_label: "NotifyPlatformHostInitialized after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyPlatformHostInitialized Component=4, Command=71, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_phost
            ),
        },
        OutgoingPush {
            wire: wire_join_done,
            component: 0x0004,
            command: 0x001E,
            tdf_body: join_done.to_vec(),
            blaze_send_label: "NotifyPlayerJoinCompleted after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyPlayerJoinCompleted Component=4, Command=30, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_join_done
            ),
        },
    ])
}

pub fn pushes_after_join_game(request: &[u8]) -> BlazeResult<Vec<OutgoingPush>> {
    let gid = super::cnc_extract_join_game_id(request);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameStateChange + NotifyGameSetup + NotifyPlatformHostInitialized after joinGame (gid={})",
        gid
    );

    let setup = super::build_game_manager_notify_game_setup_join(gid)?;
    let wire_setup = notification_envelope(0x0004, 0x0014, &setup);
    let pl0 = wire_setup.len();

    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_RESETABLE)?;
    let wire_gstate = notification_envelope(0x0004, 0x0064, &gstate);
    let pl1 = wire_gstate.len();

    let phost = super::build_game_manager_notify_platform_host_initialized(gid)?;
    let wire_phost = notification_envelope(0x0004, 0x0047, &phost);
    let pl2 = wire_phost.len();

    let join_done = super::build_game_manager_notify_player_join_completed(gid)?;
    let wire_join_done = notification_envelope(0x0004, 0x001E, &join_done);
    let pl3 = wire_join_done.len();

    Ok(vec![
        OutgoingPush {
            wire: wire_setup,
            component: 0x0004,
            command: 0x0014,
            tdf_body: setup.to_vec(),
            blaze_send_label: "NotifyGameSetup after joinGame",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyGameSetup Component=4, Command=20, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl0
            ),
        },
        OutgoingPush {
            wire: wire_gstate,
            component: 0x0004,
            command: 0x0064,
            tdf_body: gstate.to_vec(),
            blaze_send_label: "NotifyGameStateChange after joinGame",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl1
            ),
        },
        OutgoingPush {
            wire: wire_phost,
            component: 0x0004,
            command: 0x0047,
            tdf_body: phost.to_vec(),
            blaze_send_label: "NotifyPlatformHostInitialized after joinGame",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyPlatformHostInitialized Component=4, Command=71, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl2
            ),
        },
        OutgoingPush {
            wire: wire_join_done,
            component: 0x0004,
            command: 0x001E,
            tdf_body: join_done.to_vec(),
            blaze_send_label: "NotifyPlayerJoinCompleted after joinGame",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyPlayerJoinCompleted Component=4, Command=30, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl3
            ),
        },
    ])
}

pub fn pushes_after_login_persona() -> BlazeResult<Vec<OutgoingPush>> {
    let notifications = [
        (
            0x0002u16,
            super::build_user_sessions_user_added_notification()?,
            "UserSessions.UserAdded",
        ),
        (
            0x0005u16,
            super::build_user_sessions_user_updated_notification()?,
            "UserSessions.UserUpdated",
        ),
        (
            0x0008u16,
            super::build_user_sessions_user_authenticated_notification()?,
            "UserSessions.UserAuthenticated",
        ),
    ];

    let mut out = Vec::with_capacity(3);
    for (cmd, payload, name) in notifications {
        let wire = notification_envelope(0x7802, cmd, &payload);
        let pl = wire.len();
        out.push(OutgoingPush {
            wire,
            component: 0x7802,
            command: cmd,
            tdf_body: payload.to_vec(),
            blaze_send_label: name,
            info_log_line: format!(
                "[Blaze→Client] {} Component=30722, Command={}, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                name, cmd, pl
            ),
        });
    }
    Ok(out)
}

pub fn pushes_after_add_queued_player(gid: i64, player: &super::game_state::CncPlayer) -> BlazeResult<Vec<OutgoingPush>> {
    use super::game_state;

    let body = game_state::build_replicated_player(player, gid);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: player join notifies after addQueuedPlayerToGame (gid={}, slot={})",
        gid,
        player.slot
    );

    let notifies: [(u16, &'static str); 3] = [
        (0x0017, "NotifyPlayerJoiningQueue"),
        (0x0015, "NotifyPlayerJoining"),
        (0x001E, "NotifyPlayerJoinCompleted"),
    ];

    let mut out = Vec::with_capacity(3);
    for (cmd, label) in notifies {
        let wire = notification_envelope(0x0004, cmd, &body);
        let pl = wire.len();
        out.push(OutgoingPush {
            wire,
            component: 0x0004,
            command: cmd,
            tdf_body: body.clone(),
            blaze_send_label: label,
            info_log_line: format!(
                "[Blaze→Client] GameManager.{} Component=4, Command={}, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                label, cmd, pl
            ),
        });
    }
    Ok(out)
}

pub fn pushes_after_set_player_attributes(
    gid: i64,
    pid: i64,
    attribs: &indexmap::IndexMap<String, String>,
) -> BlazeResult<Vec<OutgoingPush>> {
    let body = super::game_state::build_notify_player_attrib_change(gid, pid, attribs);
    let cmd = 0x005Au16;
    let wire = notification_envelope(0x0004, cmd, &body);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: cmd,
        tdf_body: body,
        blaze_send_label: "NotifyPlayerAttribChange",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyPlayerAttribChange Component=4, Command=90, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

pub fn pushes_after_get_game_list_snapshot() -> BlazeResult<Vec<OutgoingPush>> {
    let Some((list_id, gids)) = super::game_state::take_last_game_list_snapshot() else {
        return Ok(Vec::new());
    };
    if gids.is_empty() {
        return Ok(Vec::new());
    }
    let body = super::game_state::build_notify_game_list_update(list_id, &gids, true);
    let cmd = 201u16;
    let wire = notification_envelope(0x0004, cmd, &body);
    let pl = wire.len();
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameListUpdate list_id={} games={}",
        list_id,
        gids.len()
    );
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: cmd,
        tdf_body: body,
        blaze_send_label: "NotifyGameListUpdate after getGameListSnapshot",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGameListUpdate Component=4, Command=201, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}
