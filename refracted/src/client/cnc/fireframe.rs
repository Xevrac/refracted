//! CnC **FireFrame** (size-prefixed) notification envelopes and post-RPC push sequences.
//! The global Blaze server only gates on `current_game == "cnc"` and performs I/O; payloads live here.

use crate::common::error::BlazeResult;

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

pub fn pushes_after_reset_dedicated_server(request: &[u8]) -> BlazeResult<Vec<OutgoingPush>> {
    let gid = super::cnc_extract_reset_game_id(request);
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameStateChange + NotifyGameSetup + NotifyPlatformHostInitialized after resetDedicatedServer (gid={})",
        gid
    );

    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_RESETABLE)?;
    let wire_gstate = notification_envelope(0x0004, 0x0064, &gstate);
    let pl0 = wire_gstate.len();

    let setup = super::build_game_manager_notify_game_setup(request, gid)?;
    let wire_setup = notification_envelope(0x0004, 0x0014, &setup);
    let pl1 = wire_setup.len();

    let phost = super::build_game_manager_notify_platform_host_initialized(gid)?;
    let wire_phost = notification_envelope(0x0004, 0x0047, &phost);
    let pl2 = wire_phost.len();

    Ok(vec![
        OutgoingPush {
            wire: wire_gstate,
            component: 0x0004,
            command: 0x0064,
            tdf_body: gstate.to_vec(),
            blaze_send_label: "NotifyGameStateChange after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl0
            ),
        },
        OutgoingPush {
            wire: wire_setup,
            component: 0x0004,
            command: 0x0014,
            tdf_body: setup.to_vec(),
            blaze_send_label: "NotifyGameSetup after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyGameSetup Component=4, Command=20, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl1
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
                pl2
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
