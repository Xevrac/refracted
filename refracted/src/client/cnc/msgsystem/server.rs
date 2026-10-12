//! Shared hub `:18386` multiplexes by ClientHello persona to the assigned ServerHost.
//! Each dedicated also gets a pinned hub (`MsgSysPort - 1`) that only splices to that host.

use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::OnceLock;

use parking_lot::Mutex;
use socket2::{Domain, Socket, Type};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

use super::log::{log_rts_system, RTS_TAG};
use super::proxy::{
    log_upstream_missing, peek_protocol_version_identity, relay_pair, try_connect_upstream_to,
};

/// How long the hub waits for the client's `ProtocolVersion` (sent right after connect).
const PROTOCOL_VERSION_WAIT: std::time::Duration = std::time::Duration::from_secs(3);

fn listening_hubs() -> &'static Mutex<HashSet<u16>> {
    static HUBS: OnceLock<Mutex<HashSet<u16>>> = OnceLock::new();
    HUBS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn claim_hub_port(port: u16) -> bool {
    listening_hubs().lock().insert(port)
}

pub fn spawn(port: u16) {
    spawn_hub(port, None);
}

/// Pinned hub: every accept on `hub_port` splices only to this ServerHost.
pub fn spawn_pinned(hub_port: u16, serverhost_port: u16) {
    if hub_port == 0 || serverhost_port == 0 || hub_port == serverhost_port {
        return;
    }
    spawn_hub(hub_port, Some(serverhost_port));
}

fn spawn_hub(port: u16, pinned_serverhost: Option<u16>) {
    if !claim_hub_port(port) {
        return;
    }
    let bind = SocketAddr::from(([0, 0, 0, 0], port));
    tokio::spawn(async move {
        if let Err(e) = accept_loop(bind, pinned_serverhost).await {
            listening_hubs().lock().remove(&port);
            error!("{RTS_TAG} server on {bind} exited: {e}");
        }
    });
}

fn bind_listener(bind: SocketAddr) -> std::io::Result<TcpListener> {
    let domain = match bind.ip() {
        std::net::IpAddr::V4(_) => Domain::IPV4,
        std::net::IpAddr::V6(_) => Domain::IPV6,
    };
    let socket = Socket::new(domain, Type::STREAM, None)?;

    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawSocket;
        const SO_EXCLUSIVEADDRUSE: i32 = !(winapi::um::winsock2::SO_REUSEADDR);
        let enable: i32 = 1;
        let rc = unsafe {
            winapi::um::winsock2::setsockopt(
                socket.as_raw_socket() as usize,
                winapi::um::winsock2::SOL_SOCKET,
                SO_EXCLUSIVEADDRUSE,
                &enable as *const i32 as *const i8,
                std::mem::size_of::<i32>() as i32,
            )
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    socket.bind(&bind.into())?;
    socket.listen(128)?;
    socket.set_nonblocking(true)?;
    TcpListener::from_std(socket.into())
}

pub async fn accept_loop(
    bind: SocketAddr,
    pinned_serverhost: Option<u16>,
) -> std::io::Result<()> {
    let listener = bind_listener(bind)?;
    if let Some(serverhost) = pinned_serverhost {
        info!(
            "{RTS_TAG} Message hub listening on {bind} -- pinned to dedicated ServerHost :{serverhost}"
        );
    } else {
        info!(
            "{RTS_TAG} Message hub listening on {bind} -- multiplex by client session (ClientHello → assigned dedicated)"
        );
    }
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("{RTS_TAG} accept error: {e}");
                continue;
            }
        };
        let _ = stream.set_nodelay(true);
        tokio::spawn(async move {
            if let Err(e) = handle_conn(stream, peer, pinned_serverhost).await {
                warn!("{RTS_TAG} {peer} conn ended: {e}");
            }
        });
    }
}

async fn handle_conn(
    mut client: TcpStream,
    peer: SocketAddr,
    pinned_serverhost: Option<u16>,
) -> std::io::Result<()> {
    let mut client_persona: Option<u64> = None;
    let (upstream, prefix, match_gid) = if let Some(serverhost) = pinned_serverhost {
        log_rts_system(
            peer,
            &format!("client connected -- pinned hub → ServerHost :{serverhost}"),
        );
        let up = crate::client::cnc::dedicated_pool::msgsys_upstream_for_serverhost_port(
            serverhost,
        )
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], serverhost)));
        let gid = crate::client::cnc::dedicated_pool::gid_for_msgsys_upstream(up);
        (Some(up), Vec::new(), gid)
    } else if let Some(up) =
        crate::client::cnc::dedicated_pool::resolve_unambiguous_msgsys_upstream()
    {
        // One live ServerHost: nothing to choose, splice now.
        log_rts_system(
            peer,
            &format!("client connected -- sole ServerHost {up} (no identification needed)"),
        );
        let gid = crate::client::cnc::dedicated_pool::gid_for_msgsys_upstream(up);
        (Some(up), Vec::new(), gid)
    } else {
        // (H486) Several servers: the client's ProtocolVersion auth token (its AuthToken
        // attribute, the persona id) names the player; the source address is the fallback
        // for a client that sent none (H483).
        log_rts_system(peer, "client connected -- identifying match session");
        let id = peek_protocol_version_identity(&mut client, PROTOCOL_VERSION_WAIT).await?;
        client_persona = id.persona;
        if let Some(hex) = id.protocol_version_hex.as_deref() {
            // (H491) Info, not debug: these lines are how a refused join is diagnosed.
            info!(
                "{RTS_TAG} {peer} ProtocolVersion payload={hex} auth persona={}",
                id.persona.map(|p| p.to_string()).unwrap_or_else(|| "-".into())
            );
        }
        let up = crate::client::cnc::dedicated_pool::resolve_client_msgsys_upstream(peer, id.persona);
        if id.protocol_version_hex.is_none() {
            info!(
                "{RTS_TAG} {peer} first client frame was not ProtocolVersion ({} bytes read)",
                id.prefix.len()
            );
        }
        match (up, id.persona) {
            (Some(addr), Some(pid)) => info!("{RTS_TAG} {peer} session route persona={pid} → {addr}"),
            (Some(addr), None) => info!("{RTS_TAG} {peer} session route by client address → {addr}"),
            (None, Some(pid)) => info!(
                "{RTS_TAG} {peer} no session route for persona={pid} (not seated in an assigned match)"
            ),
            (None, None) => info!(
                "{RTS_TAG} {peer} no session route (no auth token; client address ambiguous or unknown)"
            ),
        }
        let gid = id
            .persona
            .and_then(|pid| {
                let gids = super::super::game_state::gids_for_human_persona(pid as i64);
                if gids.len() == 1 {
                    Some(gids[0])
                } else {
                    None
                }
            })
            .or_else(|| up.and_then(crate::client::cnc::dedicated_pool::gid_for_msgsys_upstream));
        (up, id.prefix, gid)
    };

    let Some(upstream) = upstream else {
        log_upstream_missing(peer, None);
        warn!(
            "{RTS_TAG} {peer} closing -- no per-session MsgSys route \
             (refusing to splice onto another dedicated's sim)"
        );
        // H500: a seated client is on the loading screen; send it back to the menu (lost
        // connection). A client with no match at all has nothing to return from.
        if let (Some(pid), Some(gid)) = (client_persona, match_gid) {
            super::super::game_state::signal_client_lost_game_server(gid, pid as i64);
        }
        return Ok(());
    };

    if let Some(server) = try_connect_upstream_to(upstream).await {
        if !prefix.is_empty() {
            let mut server = server;
            server.write_all(&prefix).await?;
            server.flush().await?;
            return relay_pair(client, server, peer, match_gid).await;
        }
        return relay_pair(client, server, peer, match_gid).await;
    }

    log_upstream_missing(peer, Some(upstream));
    warn!("{RTS_TAG} {peer} closing -- could not connect to its match ServerHost {upstream}");
    // H500: same as the server closing mid-match: back to the menu with the lost-connection modal.
    match (client_persona, match_gid) {
        (Some(pid), gid) => {
            super::super::game_state::signal_client_lost_game_server(gid.unwrap_or(0), pid as i64)
        }
        (None, Some(gid)) => super::super::game_state::signal_lost_game_server(gid),
        (None, None) => {}
    }
    Ok(())
}
