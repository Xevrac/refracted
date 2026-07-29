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

/// Client-side join notifies after `resetDedicatedServer` (no dedicated orchestration).
pub fn pushes_client_join_after_reset(request: &[u8], gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameSetup + NotifyGameStateChange + NotifyPlatformHostInitialized + NotifyJoiningPlayerInitiateConnections after resetDedicatedServer (gid={})",
        gid
    );
    // Same order as `joinGame`: **NotifyGameSetup first** so `mGameMap` has the game before
    // `NotifyGameStateChange` / platform-host notifies (avoids "unknown local game" GMGR warnings).
    let setup = super::build_game_manager_notify_game_setup(request, gid)?;
    let wire_setup = notification_envelope(0x0004, 0x0014, &setup);
    let pl_setup = wire_setup.len();

    // Match the GAME.GSTA in NotifyGameSetup (INITIALIZING) -- the game is advanced to PRE_GAME only
    // after the client completes finalizeGameCreation (network mesh ready).
    let gstate = super::build_game_manager_notify_game_state_change(gid, super::GSTA_INITIALIZING)?;
    let wire_gstate = notification_envelope(0x0004, 0x0064, &gstate);
    let pl_gstate = wire_gstate.len();

    let phost = super::build_game_manager_notify_platform_host_initialized(gid)?;
    let wire_phost = notification_envelope(0x0004, 0x0047, &phost);
    let pl_phost = wire_phost.len();

    let initiate = super::build_game_manager_notify_joining_player_initiate_connections(gid)?;
    let wire_initiate = notification_envelope(0x0004, 0x0016, &initiate);
    let pl_initiate = wire_initiate.len();

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
            wire: wire_initiate,
            component: 0x0004,
            command: 0x0016,
            tdf_body: initiate.to_vec(),
            blaze_send_label: "NotifyJoiningPlayerInitiateConnections after resetDedicatedServer",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyJoiningPlayerInitiateConnections Component=4, Command=22, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_initiate
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

    let initiate = super::build_game_manager_notify_joining_player_initiate_connections(gid)?;
    let wire_initiate = notification_envelope(0x0004, 0x0016, &initiate);
    let pl_initiate = wire_initiate.len();

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
            wire: wire_initiate,
            component: 0x0004,
            command: 0x0016,
            tdf_body: initiate.to_vec(),
            blaze_send_label: "NotifyJoiningPlayerInitiateConnections after joinGame",
            info_log_line: format!(
                "[Blaze→Client] GameManager.NotifyJoiningPlayerInitiateConnections Component=4, Command=22, Size={}, MsgType=NOTIFICATION, MsgNum=0",
                pl_initiate
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

/// Seed AuthToken before GameReady.
///
/// CNC GameReady (`sub_1204D70` → `sub_12A6650`) reads **player attributes** `"AuthToken"`
/// (string map). `NotifyPlayerCustomDataChange.CDAT` is a BlazeSDK **blob** -- keep that
/// notify shape correct, and also publish the attribute the join path actually looks up.
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

/// After a real `advanceGameState` RPC -- only `NotifyGameStateChange(IN_GAME)`.
///
/// On CNC, Blaze `0x0070` is **`NotifyGameReset`**, not “Starting”. Emitting GID/STRT on
/// that command decodes as empty `ReplicatedGameData` → client drops `unknown game(0)`.
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

/// After the joining client reports its mesh connection (`updateMeshConnection`), flip its player to
/// ACTIVE_CONNECTED via `NotifyGamePlayerStateChange` (id 116) so `createGameNetworkCb` fires and the
/// game loop stops stalling (otherwise: 120s idle-starve → RPC timeout → disconnect).
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

/// Dedicated host finished setup -- do **not** force PRE_GAME/IN_GAME yet.
///
/// Working CNC path stays `INITIALIZING` through mesh + `GameReady` → engine connect /
/// MessageSystem; Blaze state advances after that. Early PRE_GAME plus fake Reset (`0x70`)
/// is the BF3-port regression that broke ClientConnect.
pub fn pushes_host_state_advance_for_client(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: host finalizeGameCreation -- defer PRE_GAME/IN_GAME until after GameReady (gid={})",
        gid
    );
    let _ = gid;
    Ok(Vec::new())
}

/// TEST (round 47): once the joining client reaches ACTIVE_CONNECTED it sits waiting for the
/// game HOST to advance the state. In production the dedicated drives finalizeGameCreation +
/// advanceGameState(PRE_GAME) + advanceGameState(IN_GAME) itself (see BF3 dedicated dump). Our
/// dedicated captures cmd 220 but never drives it, so the client stalls at ACTIVE_CONNECTED.
/// As a diagnostic, synthesize the host's advance here (Refracted-driven) to prove the client's
/// loading/IN_GAME path works. If it does, we move this to the real dedicated-driven path (fix A).
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

/// Round 64 / D38: `NotifyGameAttribChange` (comp 4, cmd 80) carrying `GameReady` (+ `CnCGameId`).
///
/// On the **client**, `onGameAttributeUpdated` (sub_1204D70) posts `RtsBlazeJoinGameMessage` and
/// drives the connect chain (MsgSys TCP in CNCO; retail also had UDP 25200).
/// On the **dedicated**, the same notify is required so CNCLive publishes into
/// `RtsServer::handleMessage` (0xA739D0) — mirror via `game_state::enqueue_game_ready_to_dedicated`.
pub fn pushes_game_ready_attrib(gid: i64) -> BlazeResult<Vec<OutgoingPush>> {
    crate::debug_println!(
        "\x1b[38;2;255;215;0m[CNC]\x1b[0m FireFrame: NotifyGameAttribChange(GameReady) (gid={})",
        gid
    );
    let mut attrs = indexmap::IndexMap::new();
    // Source of truth = exe string literals (CnC.server.exe), NOT a shared style:
    //   GameReady / CnCGameId  → game ATTR  (sub_1204D70)
    //   AuthToken              → player ATTR (sub_12A6650) — seeded separately
    //   serverid               → mesh MATR key after CustomAttribute:: strip
    //                            (sub_12016F0 / sub_11FE7F0); also echoed here but
    //                            Join resolves MATR from NotifyGameSetup, not this ATTR.
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

/// After the client completes `finalizeGameCreation` (network mesh ready) the game must leave
/// INITIALIZING and enter PRE_GAME (130) -- otherwise it stays stuck in INITIALIZING and never starts.
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
