//! Toolkit → emulator Blaze inject: in-process broadcast; each TCP session applies protocol framing.

use std::sync::OnceLock;
use tokio::sync::broadcast;

static BLAZE_INJECT: OnceLock<broadcast::Sender<Vec<u8>>> = OnceLock::new();

fn sender() -> &'static broadcast::Sender<Vec<u8>> {
    BLAZE_INJECT.get_or_init(|| {
        let (tx, _rx) = broadcast::channel::<Vec<u8>>(64);
        tx
    })
}

pub fn subscribe() -> broadcast::Receiver<Vec<u8>> {
    sender().subscribe()
}

pub fn broadcast(wire: Vec<u8>) -> Result<usize, broadcast::error::SendError<Vec<u8>>> {
    sender().send(wire)
}
