//! Live Fire2Frame injection from Toolkit → connected Blaze TCP clients (experimentation).

use std::sync::OnceLock;
use tokio::sync::broadcast;

static TOOLKIT_BLAZE_INJECT: OnceLock<broadcast::Sender<Vec<u8>>> = OnceLock::new();

fn inject_sender() -> &'static broadcast::Sender<Vec<u8>> {
    TOOLKIT_BLAZE_INJECT.get_or_init(|| {
        let (tx, _rx) = broadcast::channel::<Vec<u8>>(64);
        tx
    })
}

pub fn subscribe_toolkit_blaze_wire() -> broadcast::Receiver<Vec<u8>> {
    inject_sender().subscribe()
}

/// Enqueue plaintext Fire2Frame wire; each connection encrypts (`c_out`) before writing if needed.
pub fn broadcast_toolkit_blaze_wire(wire: Vec<u8>) -> Result<usize, broadcast::error::SendError<Vec<u8>>> {
    inject_sender().send(wire)
}
