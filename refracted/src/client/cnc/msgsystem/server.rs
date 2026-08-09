//!
//! Production architecture (single server-side owner):
//! - **Prism** `prism.cnc.network.dll` on the dedicated owns ServerHost join + post-StartGame
//!   gameplay frames (co-located with the native sim).
//!   orchestration toward `:18388`.
//!
//! Do not reintroduce an embedded ServerHost here for production joins -- that splits the
//! session away from the dedicated sim and stalls at "Waiting for remaining players".

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};

use socket2::{Domain, Socket, Type};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

use super::log::log_rts_system;
use super::proxy::{log_upstream_missing, relay_pair, try_connect_upstream};
use super::LOG_TAG;

static RTS_LISTENER_STARTED: AtomicBool = AtomicBool::new(false);

pub fn spawn(port: u16) {
    if RTS_LISTENER_STARTED.swap(true, Ordering::SeqCst) {
        warn!("[{LOG_TAG}] spawn skipped -- listener task already started");
        return;
    }
    let bind = SocketAddr::from(([0, 0, 0, 0], port));
    tokio::spawn(async move {
        if let Err(e) = accept_loop(bind).await {
            RTS_LISTENER_STARTED.store(false, Ordering::SeqCst);
            error!("[{LOG_TAG}] server on {bind} exited: {e}");
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

pub async fn accept_loop(bind: SocketAddr) -> std::io::Result<()> {
    let listener = bind_listener(bind)?;
    info!("[{LOG_TAG}] Message hub listening on {bind} -- bridge?dedicated :18387 (Prism ServerHost)");
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("[{LOG_TAG}] accept error: {e}");
                continue;
            }
        };
        tokio::spawn(async move {
            if let Err(e) = handle_conn(stream, peer).await {
                warn!("[{LOG_TAG}] {peer} conn ended: {e}");
            }
        });
    }
}

async fn handle_conn(client: TcpStream, peer: SocketAddr) -> std::io::Result<()> {
    log_rts_system(peer, "client connected -- probing dedicated ServerHost");
    if let Some(server) = try_connect_upstream().await {
        return relay_pair(client, server, peer).await;
    }

    log_upstream_missing(peer);
    warn!(
        "[{LOG_TAG}] {peer} closing -- dedicated Prism ServerHost required on 127.0.0.1:18387 \
         (no embedded join host; production MsgSys is owned by prism.cnc.network.dll)"
    );
    Ok(())
}
