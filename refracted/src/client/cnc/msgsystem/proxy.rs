//! Transparent TCP bridge between game `ClientHost` and the assigned dedicated
//! `ServerHost` (`-Prism.MsgSysPort`, default 18387). Close if that dedicated is not listening.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Instant};
use tracing::warn;

use super::log::{
    log_client_to_rts, log_rts_system, log_rts_to_client, log_rts_to_server, log_server_to_rts,
    RTS_TAG,
};
use super::messages::{
    decode_client_hello, decode_load_map_id, decode_protocol_version_auth_token,
    ALLOW_INPUT_CHANGE_TYPE_ID, CLIENT_HELLO_TYPE_ID, LOAD_MAP_TYPE_ID,
    PLACED_BUILD_COMPLETED_TYPE_ID, PROTOCOL_VERSION_TYPE_ID, START_GAME_TYPE_ID,
};
use super::wire::SimpleFrame;

pub const DEDICATED_SERVERHOST_PORT: u16 = 18387;

// H498: a 200 ms per-attempt cap cancelled every handshake to a server farther than 200 ms
// round trip (overseas over the VPN), so it could never connect. Each attempt now gets
// `MAX_SERVER_RTT` (1200 ms, the supported maximum); a lost SYN or "refused" (ServerHost not
// listening yet) is retried with a fresh attempt until the budget ends.
const UPSTREAM_RETRY_INTERVAL: Duration = Duration::from_millis(100);
/// Wait for ServerHost (load-complete / cmd 220).
const UPSTREAM_WAIT_BUDGET: Duration = Duration::from_secs(30);
const RELAY_CHUNK: usize = 4096;

pub async fn try_connect_upstream_to(upstream: SocketAddr) -> Option<TcpStream> {
    let deadline = Instant::now() + UPSTREAM_WAIT_BUDGET;
    let mut attempts: u32 = 0;
    let mut last_err = String::new();

    while Instant::now() < deadline {
        attempts += 1;
        let remaining = deadline.saturating_duration_since(Instant::now());
        match timeout(remaining.min(super::MAX_SERVER_RTT), TcpStream::connect(upstream)).await {
            Ok(Ok(stream)) => {
                // MsgSys frames are small and latency-bound; Nagle + delayed ACK stalls each hop.
                let _ = stream.set_nodelay(true);
                log_rts_system(
                    upstream,
                    &format!("upstream server connected after {attempts} attempt(s)"),
                );
                return Some(stream);
            }
            Ok(Err(e)) => {
                last_err = e.to_string();
            }
            Err(_) => {
                last_err = "connect timed out".to_string();
            }
        }
        sleep(UPSTREAM_RETRY_INTERVAL).await;
    }

    log_rts_system(
        upstream,
        &format!("no dedicated server on {upstream} after {attempts} attempt(s) ({last_err})"),
    );
    None
}

/// What the client's opening bytes say about who it is.
pub struct ClientIdentity {
    /// Bytes read so far; forwarded to the ServerHost before splicing.
    pub prefix: Vec<u8>,
    /// Persona from `ProtocolVersion`'s auth token (H486), if it carried one.
    pub persona: Option<u64>,
    /// Hex of the `ProtocolVersion` payload, for the hub log.
    pub protocol_version_hex: Option<String>,
}

/// (H486) Read the client's first frame. The client speaks first on the client channel: its
/// `ProtocolVersion` carries the auth token the server's `authenticationFunc` checks, before
/// the server answers with ConnectSuccess. Refracted's `AuthToken` attribute makes that token
/// the persona id, so this one frame names the player even when several share an address.
pub async fn peek_protocol_version_identity(
    client: &mut TcpStream,
    wait: Duration,
) -> std::io::Result<ClientIdentity> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; RELAY_CHUNK];
    let deadline = Instant::now() + wait;
    loop {
        if let Ok(Some((frame, _))) = SimpleFrame::try_read(&buf) {
            let (persona, hex) = if frame.type_id == PROTOCOL_VERSION_TYPE_ID {
                (
                    decode_protocol_version_auth_token(&frame.payload)
                        .map(u64::from_le_bytes)
                        .filter(|&p| p != 0),
                    Some(hex::encode(&frame.payload)),
                )
            } else {
                (persona_from_hello_buf(&buf), None)
            };
            return Ok(ClientIdentity {
                prefix: buf,
                persona,
                protocol_version_hex: hex,
            });
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(ClientIdentity {
                prefix: buf,
                persona: None,
                protocol_version_hex: None,
            });
        }
        match timeout(remaining, client.read(&mut chunk)).await {
            Ok(Ok(0)) => {
                return Ok(ClientIdentity {
                    prefix: buf,
                    persona: None,
                    protocol_version_hex: None,
                })
            }
            Ok(Ok(n)) => buf.extend_from_slice(&chunk[..n]),
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Ok(ClientIdentity {
                    prefix: buf,
                    persona: None,
                    protocol_version_hex: None,
                })
            }
        }
    }
}

fn persona_from_hello_buf(buf: &[u8]) -> Option<u64> {
    let mut rest = buf;
    while let Ok(Some((frame, consumed))) = SimpleFrame::try_read(rest) {
        if frame.type_id == CLIENT_HELLO_TYPE_ID {
            let pid = decode_client_hello(&frame.payload).filter(|&p| p != 0);
            return pid;
        }
        rest = &rest[consumed..];
    }
    None
}

enum RelayClosedBy {
    Client,
    Server,
}

pub async fn relay_pair(
    client: TcpStream,
    server: TcpStream,
    client_peer: SocketAddr,
    match_gid: Option<i64>,
) -> std::io::Result<()> {
    let upstream_peer = server.peer_addr().unwrap_or(client_peer);
    let (mut client_read, mut client_write) = client.into_split();
    let (mut server_read, mut server_write) = server.into_split();

    log_rts_system(
        client_peer,
        &format!("Client?Server (upstream {upstream_peer})"),
    );

    // (H488) The player on this connection, from its ClientHello (u64 PlayerId).
    let hello_persona = std::sync::atomic::AtomicU64::new(0);
    let client_to_server = async {
        relay_direction(
            &mut client_read,
            &mut server_write,
            client_peer,
            RelayLog::ClientToServer,
            None,
            Some(&hello_persona),
        )
        .await
    };

    let server_to_client = async {
        let result = relay_direction(
            &mut server_read,
            &mut client_write,
            upstream_peer,
            RelayLog::ServerToClient,
            match_gid,
            None,
        )
        .await;
        // Upstream died — close the client side.
        let _ = client_write.shutdown().await;
        result
    };

    // A reset by the client (it quit or went back to the front end) is still the client leaving.
    let closed_by = tokio::select! {
        r = client_to_server => {
            if let Err(e) = r {
                log_rts_system(client_peer, &format!("client side ended: {e}"));
            }
            RelayClosedBy::Client
        }
        r = server_to_client => {
            r?;
            RelayClosedBy::Server
        }
    };

    super::log::flush_relay_log_compactor();

    if matches!(closed_by, RelayClosedBy::Client) {
        // (H488) The client closed its MsgSys connection to the match: it left (Quit, or the
        // Defeat/Victory screen back to the front end). Nothing else reports a surrender's exit:
        // the client goes back to its own lobby without a leave-game, so the player stayed
        // seated and the row stayed InGame after the dedicated had unloaded.
        let pid = hello_persona.load(std::sync::atomic::Ordering::Relaxed) as i64;
        if let Some(gid) = match_gid.filter(|g| *g > 0) {
            if pid > 0 {
                super::super::game_state::note_client_left_match(gid, pid);
            }
        }
    }

    if matches!(closed_by, RelayClosedBy::Server) {
        super::super::game_state::signal_lost_game_server(match_gid.unwrap_or(0));
        log_rts_system(
            client_peer,
            &format!(
                "upstream ServerHost closed — match lost signal gid={}",
                match_gid.unwrap_or(0)
            ),
        );
    }

    Ok(())
}

enum RelayLog {
    ClientToServer,
    ServerToClient,
}

async fn relay_direction<R, W>(
    read: &mut R,
    write: &mut W,
    peer: SocketAddr,
    direction: RelayLog,
    start_gid: Option<i64>,
    hello_persona: Option<&std::sync::atomic::AtomicU64>,
) -> std::io::Result<()>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let mut chunk = [0u8; RELAY_CHUNK];
    let mut parse_buf: Vec<u8> = Vec::new();

    loop {
        let n = read.read(&mut chunk).await?;
        if n == 0 {
            return Ok(());
        }

        write.write_all(&chunk[..n]).await?;
        write.flush().await?;

        parse_buf.extend_from_slice(&chunk[..n]);
        while let Ok(Some((frame, consumed))) = SimpleFrame::try_read(&parse_buf) {
            if frame.type_id == CLIENT_HELLO_TYPE_ID {
                if let (Some(out), Some(pid)) = (hello_persona, decode_client_hello(&frame.payload)) {
                    out.store(pid, std::sync::atomic::Ordering::Relaxed);
                }
            }
            // (H415) Dedicated→client StartGame is the match start; lobby phase leaves PreGame here
            if frame.type_id == START_GAME_TYPE_ID {
                if let Some(gid) = start_gid.filter(|g| *g > 0) {
                    super::super::game_state::note_match_started(gid);
                    // Lobby GameReady color= hit the frontend RtsClient; re-push once the
                    // match instance exists so Blaze→A9FED0 can fill map[persona].
                    crate::client::cnc::enqueue_native_house_color_attr_for_gid(gid);
                }
            }
            let extra = if frame.type_id == LOAD_MAP_TYPE_ID {
                decode_load_map_id(&frame.payload)
                    .map(|m| format!("LoadMap \"{m}\""))
                    .unwrap_or_else(|| "relay".to_string())
            } else if frame.type_id == ALLOW_INPUT_CHANGE_TYPE_ID {
                // Payload is a serialized bool -- expect a non-zero Enable byte for dismiss.
                format!("payload={}", hex::encode(&frame.payload))
            } else if frame.type_id == PLACED_BUILD_COMPLETED_TYPE_ID {
                // Completion must be proven at the wire boundary before changing native state.
                format!("payload={}", hex::encode(&frame.payload))
            } else {
                "relay".to_string()
            };
            match direction {
                RelayLog::ClientToServer => {
                    log_client_to_rts(peer, frame.type_id, frame.payload.len(), &extra);
                    log_rts_to_server(peer, frame.type_id, frame.payload.len(), &extra);
                }
                RelayLog::ServerToClient => {
                    log_server_to_rts(peer, frame.type_id, frame.payload.len(), &extra);
                    log_rts_to_client(peer, frame.type_id, frame.payload.len(), &extra);
                }
            }
            parse_buf.drain(..consumed);
        }
    }
}

pub fn log_upstream_missing(peer: SocketAddr, upstream: Option<SocketAddr>) {
    match upstream {
        Some(up) => warn!("{RTS_TAG} {peer} dedicated ServerHost not on {up}"),
        None => warn!("{RTS_TAG} {peer} no MsgSys route for this client session"),
    }
}
