//! CnC **FireFrame** (size-prefixed) notification envelopes and post-RPC push sequences.
//! The global Blaze server only gates on `current_game == "cnc"` and performs I/O; payloads live here.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::common::error::BlazeResult;

#[derive(Debug, Clone)]
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

/// Drop queued Blaze notifications for a dedicated session.
pub fn clear_pending_pushes(blaze_session_id: u64) {
    pending_pushes().lock().remove(&blaze_session_id);
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

pub fn pushes_after_reset_dedicated_server(
    client_session_id: u64,
    request: &[u8],
) -> BlazeResult<Vec<OutgoingPush>> {
    let gid = super::cnc_extract_reset_game_id(request);
    match super::dedicated_pool::orchestrate_client_reset(client_session_id, gid, request) {
        Some(dedicated_sid) => crate::debug_println!(
            "\x1b[38;2;100;200;255m[Dedicated pool]\x1b[0m queued cmd 220 NotifyCreate for dedicated session #{} (gid={})",
            dedicated_sid,
            gid
        ),
        None => crate::debug_println!(
            "\x1b[38;2;255;180;100m[Dedicated pool]\x1b[0m no idle pooled dedicated for resetDedicatedServer (gid={}) -- registerDynamicDedicatedServerCreator (0x96) required",
            gid
        ),
    }
    pushes_client_join_after_reset(request, gid)
}

pub fn pushes_rematch_teardown_before_reset_reply(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    if !super::game_state::blaze_pregame_already_pushed(gid) {
        return Ok(Vec::new());
    }
    crate::debug_println!(
        "\x1b[38;2;100;200;255m[CNC]\x1b[0m rematch teardown before resetDedicatedServer gid={}",
        gid
    );
    super::game_state::clear_blaze_join_and_push_flags(gid);

    let removed = super::build_game_manager_notify_game_removed(
        gid,
        super::GAME_REMOVAL_REASON_GAME_DESTROYED,
    )?;
    let wire_removed = notification_envelope(0x0004, 0x0010, &removed);
    let removed_pl = wire_removed.len();

    Ok(vec![OutgoingPush {
        wire: wire_removed,
        component: 0x0004,
        command: 0x0010,
        tdf_body: removed.to_vec(),
        blaze_send_label: "NotifyGameRemoved before resetDedicatedServer (rematch)",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGameRemoved Component=4, Command=16, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            removed_pl
        ),
    }])
}

pub fn pushes_client_join_after_reset(request: &[u8], gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    let promote_reset = super::game_state::blaze_join_setup_already_pushed(gid)
        || super::game_state::client_local_game_active(gid);
    // joinGame NotifyGameSetup already ran preInitGameNetwork + updateMeshConnection.
    // A second InitiateConnections re-adds the same Blaze Fiber (duplicate dispatchee).
    // Rematch NotifyGameRemoved clears mesh-live so InitiateConnections still runs.
    let skip_initiate = super::game_state::client_mesh_already_connected(gid);
    let setup_label = if promote_reset {
        "NotifyGameReset after resetDedicatedServer"
    } else {
        "NotifyGameSetup after resetDedicatedServer"
    };
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: {} + NotifyGameStateChange + NotifyPlatformHostInitialized + {} after resetDedicatedServer (gid={})",
        if promote_reset {
            "NotifyGameReset"
        } else {
            "NotifyGameSetup"
        },
        if skip_initiate {
            "NotifyPlayerJoinCompleted (mesh already live; skip InitiateConnections)"
        } else {
            "NotifyJoiningPlayerInitiateConnections"
        },
        gid
    );

    let (setup, setup_cmd, setup_log) = if promote_reset {
        let reset = super::build_game_manager_notify_game_reset(request, gid)?;
        let wire = notification_envelope(0x0004, 0x0070, &reset);
        let pl = wire.len();
        (
            reset,
            0x0070,
            format!(
                "[Blaze→Client] GameManager.NotifyGameReset Component=4, Command=112, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl
            ),
        )
    } else {
        let setup = super::build_game_manager_notify_game_setup(request, gid)?;
        let wire = notification_envelope(0x0004, 0x0014, &setup);
        let pl = wire.len();
        (
            setup,
            0x0014,
            format!(
                "[Blaze→Client] GameManager.NotifyGameSetup Component=4, Command=20, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl
            ),
        )
    };
    let wire_setup = notification_envelope(0x0004, setup_cmd, &setup);
    if promote_reset {
        super::game_state::clear_blaze_join_setup_pushed(gid);
    }

    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_INITIALIZING)?;
    let wire_gstate = notification_envelope(0x0004, 0x0064, &gstate);
    let pl_gstate = wire_gstate.len();

    let phost = super::build_game_manager_notify_platform_host_initialized(gid)?;
    let wire_phost = notification_envelope(0x0004, 0x0047, &phost);
    let pl_phost = wire_phost.len();

    let mut out = vec![
        OutgoingPush {
            wire: wire_setup,
            component: 0x0004,
            command: setup_cmd,
            tdf_body: setup.to_vec(),
            blaze_send_label: setup_label,
            info_log_line: setup_log,
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
    ];

    if skip_initiate {
        super::game_state::mark_orch_mesh_already_connected(gid);
        let mut join_done = pushes_player_join_completed(gid)?;
        for p in &mut join_done {
            p.blaze_send_label = "NotifyPlayerJoinCompleted after resetDedicatedServer (mesh already live)";
        }
        out.extend(join_done);
    } else {
        let initiate = super::build_game_manager_notify_joining_player_initiate_connections(gid)?;
        let wire_initiate = notification_envelope(0x0004, 0x0016, &initiate);
        let pl_initiate = wire_initiate.len();
        out.push(OutgoingPush {
            wire: wire_initiate,
            component: 0x0004,
            command: 0x0016,
            tdf_body: initiate.to_vec(),
            blaze_send_label: "NotifyJoiningPlayerInitiateConnections after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyJoiningPlayerInitiateConnections Component=4, Command=22, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_initiate
            ),
        });
    }

    Ok(out)
}

pub fn pushes_after_join_game(request: &[u8]) -> BlazeResult<Vec<OutgoingPush>> {
    pushes_after_join_game_lobby(request, false)
}

pub fn pushes_after_join_game_lobby(
    request: &[u8],
    defer_mesh: bool,
) -> BlazeResult<Vec<OutgoingPush>> {
    let gid = super::cnc_extract_join_game_id(request);
    let _ = super::game_state::try_mark_blaze_join_setup_pushed(gid);
    if let Some(sid) = crate::session::current_blaze_session_id() {
        if let Some(sess) = crate::session::blaze_sessions::get_session(sid) {
            if let Some(pid) = sess.persona_id.filter(|&p| p != 0) {
                crate::client::cnc::dedicated_pool::note_client_msgsys_route_from_current_session(
                    gid,
                    pid as i64,
                );
            }
        }
    }
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameSetup + NotifyGameStateChange{} after joinGame (gid={}){}",
        if defer_mesh {
            " (no platform host yet)"
        } else {
            " + NotifyPlatformHostInitialized"
        },
        gid,
        if defer_mesh {
            " [lobby — mesh deferred]"
        } else {
            ""
        }
    );

    let setup = super::build_game_manager_notify_game_setup_join(gid)?;
    let wire_setup = notification_envelope(0x0004, 0x0014, &setup);
    let pl0 = wire_setup.len();

    let gsta = if defer_mesh {
        super::GSTA_PRE_GAME
    } else {
        super::GSTA_INITIALIZING
    };
    let gstate = super::build_game_manager_notify_game_state_change(gid, gsta)?;
    let wire_gstate = notification_envelope(0x0004, 0x0064, &gstate);
    let pl1 = wire_gstate.len();

    let mut out = vec![
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
    ];

    // Lobby join defers mesh until Start Battle — do not tell the client it is platform host yet.
    if !defer_mesh {
        let phost = super::build_game_manager_notify_platform_host_initialized(gid)?;
        let wire_phost = notification_envelope(0x0004, 0x0047, &phost);
        let pl2 = wire_phost.len();
        out.push(OutgoingPush {
            wire: wire_phost,
            component: 0x0004,
            command: 0x0047,
            tdf_body: phost.to_vec(),
            blaze_send_label: "NotifyPlatformHostInitialized after joinGame",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyPlatformHostInitialized Component=4, Command=71, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl2
            ),
        });
    }

    if !defer_mesh {
        let initiate = super::build_game_manager_notify_joining_player_initiate_connections(gid)?;
        let wire_initiate = notification_envelope(0x0004, 0x0016, &initiate);
        let pl_initiate = wire_initiate.len();
        out.push(OutgoingPush {
            wire: wire_initiate,
            component: 0x0004,
            command: 0x0016,
            tdf_body: initiate.to_vec(),
            blaze_send_label: "NotifyJoiningPlayerInitiateConnections after joinGame",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyJoiningPlayerInitiateConnections Component=4, Command=22, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_initiate
            ),
        });
        // NotifyPlayerJoinCompleted waits for updateMeshConnection (see pushes_player_join_completed).
    }

    Ok(out)
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

/// Seed AuthToken before GameReady.
///
pub fn pushes_auth_token_custom_data(gid: i64, pid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    let mut blobs = indexmap::IndexMap::new();
    blobs.insert(
        "AuthToken".to_string(),
        super::game_state::auth_token_custom_data_blob(),
    );
    let mut out = pushes_after_set_player_custom_data(gid, pid, &blobs)?;
    let mut attrs = indexmap::IndexMap::new();
    attrs.insert(
        "AuthToken".to_string(),
        super::game_state::CNC_AUTH_TOKEN.to_string(),
    );
    out.extend(pushes_after_set_player_attributes(gid, pid, &attrs)?);
    Ok(out)
}

pub fn pushes_after_set_player_custom_data(
    gid: i64,
    pid: i64,
    data: &indexmap::IndexMap<String, Vec<u8>>,
) -> BlazeResult<Vec<OutgoingPush>> {
    let body = super::game_state::build_notify_player_custom_data_change(gid, pid, data);
    let cmd = 0x005Fu16;
    let wire = notification_envelope(0x0004, cmd, &body);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: cmd,
        tdf_body: body,
        blaze_send_label: "NotifyPlayerCustomDataChange",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyPlayerCustomDataChange Component=4, Command=95, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

/// After `advanceGameState`: `NotifyGameStateChange(IN_GAME)`.
pub fn pushes_after_advance_game_state(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameStateChange(InGame) after advanceGameState (gid={})",
        gid
    );

    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_IN_GAME)?;
    let wire_gstate = notification_envelope(0x0004, 0x0064, &gstate);
    let pl_gstate = wire_gstate.len();

    Ok(vec![OutgoingPush {
        wire: wire_gstate,
        component: 0x0004,
        command: 0x0064,
        tdf_body: gstate.to_vec(),
        blaze_send_label: "NotifyGameStateChange after advanceGameState",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl_gstate
        ),
    }])
}

/// After `updateMeshConnection`: `NotifyGamePlayerStateChange(ACTIVE_CONNECTED)`.
pub fn pushes_after_update_mesh_connection(gid: i64, pid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGamePlayerStateChange(ACTIVE_CONNECTED) after updateMeshConnection (gid={}, pid={})",
        gid,
        pid
    );
    let body = super::build_game_manager_notify_game_player_state_change(
        gid,
        pid,
        super::PLAYER_STATE_ACTIVE_CONNECTED,
    )?;
    let wire = notification_envelope(0x0004, 116, &body);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: 116,
        tdf_body: body.to_vec(),
        blaze_send_label: "NotifyGamePlayerStateChange(ACTIVE_CONNECTED) after updateMeshConnection",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGamePlayerStateChange Component=4, Command=116, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

pub fn pushes_player_join_completed(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    let body = super::build_game_manager_notify_player_join_completed(gid)?;
    let wire = notification_envelope(0x0004, 0x001E, &body);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: 0x001E,
        tdf_body: body.to_vec(),
        blaze_send_label: "NotifyPlayerJoinCompleted after updateMeshConnection",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyPlayerJoinCompleted Component=4, Command=30, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

pub fn pushes_host_state_advance_for_client(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: host finalizeGameCreation -- defer PRE_GAME/IN_GAME until after GameReady (gid={})",
        gid
    );
    let _ = gid;
    Ok(Vec::new())
}

pub fn pushes_advance_game_to_ingame(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: TEST advanceGameState PRE_GAME(130)->IN_GAME(131) after mesh connected (gid={})",
        gid
    );
    let mut out = Vec::with_capacity(2);
    for (gsta, label) in [
        (super::GSTA_PRE_GAME, "NotifyGameStateChange(PRE_GAME) [test host advance]"),
        (super::GSTA_IN_GAME, "NotifyGameStateChange(IN_GAME) [test host advance]"),
    ] {
        let gstate = super::build_game_manager_notify_game_state_change(gid, gsta)?;
        let wire = notification_envelope(0x0004, 0x0064, &gstate);
        let pl = wire.len();
        out.push(OutgoingPush {
            wire,
            component: 0x0004,
            command: 0x0064,
            tdf_body: gstate.to_vec(),
            blaze_send_label: label,
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl
            ),
        });
    }
    Ok(out)
}

/// `NotifyGameAttribChange` with GameReady (+ CnCGameId).
/// The dedicated needs the same notify so CNCLive can publish.
pub fn pushes_game_ready_attrib(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameAttribChange(GameReady) (gid={})",
        gid
    );
    let mut attrs = indexmap::IndexMap::new();
    let serverid = super::dedicated_pool::host_for_gid(gid)
        .map(|h| {
            let ip = (if h.exip_ip != 0 { h.exip_ip } else { h.inip_ip }) as u32;
            format!("{}.{}.{}.{}", (ip >> 24) & 0xFF, (ip >> 16) & 0xFF, (ip >> 8) & 0xFF, ip & 0xFF)
        })
        .filter(|s| s != "0.0.0.0")
        .unwrap_or_else(|| "127.0.0.1".to_string());
    attrs.insert("CnCGameId".to_string(), gid.to_string());
    attrs.insert("serverid".to_string(), serverid);
    attrs.insert("GameReady".to_string(), "1".to_string());
    super::game_state::apply_password_flag_to_attrs(gid, &mut attrs);
    let body = super::build_game_manager_notify_game_attrib_change(gid, &attrs)?;
    let wire = notification_envelope(0x0004, 0x0050, &body);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: 0x0050,
        tdf_body: body.to_vec(),
        blaze_send_label: "NotifyGameAttribChange(GameReady) -> client engine connect",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGameAttribChange Component=4, Command=80, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

pub fn pushes_password_attrib(
    gid: i64,
    protected: bool,
    secret: Option<&str>,
) -> BlazeResult<Vec<OutgoingPush>> {
    let mut attrs = indexmap::IndexMap::new();
    let with_secret = secret.filter(|s| !s.is_empty()).is_some();
    if protected {
        attrs.insert(super::game_state::ATTR_PASSWORD_FLAG.to_string(), "1".to_string());
        if let Some(s) = secret.filter(|s| !s.is_empty()) {
            attrs.insert(super::game_state::ATTR_PASSWORD_SECRET.to_string(), s.to_string());
        }
    } else {
        attrs.insert(super::game_state::ATTR_PASSWORD_FLAG.to_string(), "0".to_string());
        attrs.insert(super::game_state::ATTR_PASSWORD_SECRET.to_string(), String::new());
    }
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameAttribChange(password) gid={} protected={} secret={}",
        gid,
        protected,
        with_secret
    );
    let body = super::build_game_manager_notify_game_attrib_change(gid, &attrs)?;
    let wire = notification_envelope(0x0004, 0x0050, &body);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: 0x0050,
        tdf_body: body.to_vec(),
        blaze_send_label: "NotifyGameAttribChange",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGameAttribChange Component=4, Command=80, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

pub fn pushes_notify_player_removed(gid: i64, pid: i64, reason: i32) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyPlayerRemoved gid={} pid={} reas={}",
        gid,
        pid,
        reason
    );
    let body = super::build_game_manager_notify_player_removed(gid, pid, reason)?;
    let wire = notification_envelope(0x0004, 0x0028, &body);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: 0x0028,
        tdf_body: body.to_vec(),
        blaze_send_label: "NotifyPlayerRemoved",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyPlayerRemoved Component=4, Command=40, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

pub fn request_client_local_game_teardown(gid: i64, pid: i64, reason: i32) {
    if gid <= 0 || pid <= 0 {
        return;
    }
    let sid = super::game_state::blaze_session_for_persona(pid)
        .or_else(|| super::game_state::client_session_for_gid(gid));
    let Some(sid) = sid else {
        crate::debug_println!(
            "\x1b[38;2;255;165;0m[CNC]\x1b[0m NotifyPlayerRemoved skipped — no client session (gid={} pid={})",
            gid,
            pid
        );
        return;
    };
    match pushes_notify_player_removed(gid, pid, reason) {
        Ok(pushes) if !pushes.is_empty() => {
            enqueue_pending_pushes(sid, pushes);
            let _ = crate::blaze::server::inject_bus::broadcast(Vec::new());
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[CNC]\x1b[0m queued NotifyPlayerRemoved → client #{} gid={} pid={} reas={}",
                sid,
                gid,
                pid,
                reason
            );
        }
        Ok(_) => {}
        Err(e) => {
            crate::debug_println!(
                "\x1b[38;2;255;165;0m[CNC]\x1b[0m NotifyPlayerRemoved encode failed: {}",
                e
            );
        }
    }
}

pub fn pushes_dedicated_reclaim_idle(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    // Removed before RESETABLE so the local game is destroyed first.
    let removed = super::build_game_manager_notify_game_removed(
        gid,
        super::GAME_REMOVAL_REASON_GAME_DESTROYED,
    )?;
    let wire_removed = notification_envelope(0x0004, 0x0010, &removed);
    let removed_pl = wire_removed.len();

    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_RESETABLE)?;
    let wire_resetable = notification_envelope(0x0004, 0x0064, &gstate);
    let resetable_pl = wire_resetable.len();

    Ok(vec![
        OutgoingPush {
            wire: wire_removed,
            component: 0x0004,
            command: 0x0010,
            tdf_body: removed.to_vec(),
            blaze_send_label: "NotifyGameRemoved dedicated reclaim",
            info_log_line: format!(
                "[Blaze→Server] GameManager.NotifyGameRemoved Component=4, Command=16, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                removed_pl
            ),
        },
        OutgoingPush {
            wire: wire_resetable,
            component: 0x0004,
            command: 0x0064,
            tdf_body: gstate.to_vec(),
            blaze_send_label: "NotifyGameStateChange(RESETABLE) dedicated reclaim",
            info_log_line: format!(
                "[Blaze→Server] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                resetable_pl
            ),
        },
    ])
}

/// After mesh is ready: `NotifyGameStateChange(PRE_GAME)`.
pub fn pushes_after_finalize_game_creation(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameStateChange(PRE_GAME) after finalizeGameCreation (gid={})",
        gid
    );
    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_PRE_GAME)?;
    let wire = notification_envelope(0x0004, 0x0064, &gstate);
    let pl = wire.len();
    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: 0x0064,
        tdf_body: gstate.to_vec(),
        blaze_send_label: "NotifyGameStateChange(PRE_GAME) after finalizeGameCreation",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGameStateChange Component=4, Command=100, Size={}, MsgType=NOTIFICATION, MsgNum=0",
            pl
        ),
    }])
}

pub fn pushes_after_set_game_settings(gid: i64, gset: i32) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameSettingsChange after setGameSettings (gid={}, gset={})",
        gid, gset
    );

    let body = super::build_game_manager_notify_game_settings_change(gid, gset)?;
    let wire = notification_envelope(0x0004, 0x006E, &body);
    let pl = wire.len();

    Ok(vec![OutgoingPush {
        wire,
        component: 0x0004,
        command: 0x006E,
        tdf_body: body.to_vec(),
        blaze_send_label: "NotifyGameSettingsChange after setGameSettings",
        info_log_line: format!(
            "[Blaze→Client] GameManager.NotifyGameSettingsChange Component=4, Command=110, Size={}, MsgType=NOTIFICATION, MsgNum=0",
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
