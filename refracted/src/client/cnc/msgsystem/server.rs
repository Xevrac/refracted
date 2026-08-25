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
    log_upstream_missing, peek_client_hello_persona, relay_pair, try_connect_upstream_to,
};

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
        // One live match: splice immediately. ClientHello arrives after ConnectSuccess.
        log_rts_system(
            peer,
            &format!("client connected -- unambiguous ServerHost {up} (no ClientHello peek)"),
        );
        let gid = crate::client::cnc::dedicated_pool::gid_for_msgsys_upstream(up);
        (Some(up), Vec::new(), gid)
    } else {
        log_rts_system(peer, "client connected -- identifying match session");
        let (prefix, persona) = peek_client_hello_persona(&mut client).await?;
        if let Some(pid) = persona {
            log_rts_system(peer, &format!("ClientHello persona={pid}"));
        }
        let up = crate::client::cnc::dedicated_pool::resolve_client_msgsys_upstream(peer, persona);
        match up {
            Some(addr) => log_rts_system(peer, &format!("session route → {addr}")),
            None => log_rts_system(
                peer,
                "no session route (need joinGame for this persona / dedicated bind)",
            ),
        }
        let gid = persona
            .and_then(|pid| {
                let gids = super::super::game_state::gids_for_human_persona(pid as i64);
                if gids.len() == 1 {
                    Some(gids[0])
                } else {
                    None
                }
            })
            .or_else(|| up.and_then(crate::client::cnc::dedicated_pool::gid_for_msgsys_upstream));
        (up, prefix, gid)
    };

    let Some(upstream) = upstream else {
        log_upstream_missing(peer, None);
        warn!(
            "{RTS_TAG} {peer} closing -- no per-session MsgSys route \
             (refusing to splice onto another dedicated's sim)"
        );
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
    warn!(
        "{RTS_TAG} {peer} closing -- dedicated Prism ServerHost required \
         (no embedded join host; production MsgSys is owned by prism.cnc.network.dll)"
    );
    Ok(())
}
