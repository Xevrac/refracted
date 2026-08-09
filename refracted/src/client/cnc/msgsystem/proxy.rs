//! Transparent TCP bridge between game `ClientHost` and dedicated Prism `ServerHost` on
//! [`DEDICATED_SERVERHOST_PORT`]. When dedicated is not listening, the hub closes the
//! client connection (Prism owns production ServerHost -- no Refracted join substitute).

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Instant};
use tracing::warn;

use super::log::{
    log_client_to_rts, log_rts_system, log_rts_to_client, log_rts_to_server, log_server_to_rts,
};
use super::messages::{
    decode_load_map_id, ALLOW_INPUT_CHANGE_TYPE_ID, LOAD_MAP_TYPE_ID,
};
use super::wire::SimpleFrame;
use super::LOG_TAG;

pub const DEDICATED_SERVERHOST_PORT: u16 = 18387;

const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_millis(200);
const UPSTREAM_RETRY_INTERVAL: Duration = Duration::from_millis(100);
/// Wait for Prism managed ServerHost (load-complete / cmd 220). Keep under client connect
/// budget but long enough for CLR ExecuteInDefaultAppDomain on a cold dedicated.
const UPSTREAM_WAIT_BUDGET: Duration = Duration::from_secs(8);
const RELAY_CHUNK: usize = 4096;

pub async fn try_connect_upstream() -> Option<TcpStream> {
    let upstream = SocketAddr::from(([127, 0, 0, 1], DEDICATED_SERVERHOST_PORT));
    let deadline = Instant::now() + UPSTREAM_WAIT_BUDGET;
    let mut attempts: u32 = 0;
    let mut last_err = String::new();

    while Instant::now() < deadline {
        attempts += 1;
        match timeout(UPSTREAM_CONNECT_TIMEOUT, TcpStream::connect(upstream)).await {
            Ok(Ok(stream)) => {
                log_rts_system(
                    upstream,
                    &format!(
                        "upstream server connected after {attempts} attempt(s)"
                    ),
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
        &format!(
            "no dedicated server on {upstream} after {attempts} attempt(s) ({last_err})"
        ),
    );
    None
}

pub async fn relay_pair(
    client: TcpStream,
    server: TcpStream,
    client_peer: SocketAddr,
) -> std::io::Result<()> {
    let upstream_peer = server
        .peer_addr()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], DEDICATED_SERVERHOST_PORT)));
    let (mut client_read, mut client_write) = client.into_split();
    let (mut server_read, mut server_write) = server.into_split();

    log_rts_system(
        client_peer,
        &format!("Client?Server (upstream {upstream_peer})"),
    );

    let client_to_server = async {
        relay_direction(
            &mut client_read,
            &mut server_write,
            client_peer,
            RelayLog::ClientToServer,
        )
        .await
    };

    let server_to_client = async {
        relay_direction(
            &mut server_read,
            &mut client_write,
            upstream_peer,
            RelayLog::ServerToClient,
        )
        .await
    };

    let result = tokio::select! {
        r = client_to_server => r,
        r = server_to_client => r,
    };

    super::log::flush_relay_log_compactor();
    result
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
            let extra = if frame.type_id == LOAD_MAP_TYPE_ID {
                decode_load_map_id(&frame.payload)
                    .map(|m| format!("LoadMap \"{m}\""))
                    .unwrap_or_else(|| "relay".to_string())
            } else if frame.type_id == ALLOW_INPUT_CHANGE_TYPE_ID {
                // Payload is a serialized bool -- expect a non-zero Enable byte for dismiss.
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

pub fn log_upstream_missing(peer: SocketAddr) {
    warn!(
        "[{LOG_TAG}] {peer} dedicated server not on 127.0.0.1:{DEDICATED_SERVERHOST_PORT}"
    );
}
