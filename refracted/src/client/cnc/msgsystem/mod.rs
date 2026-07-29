//! CNC MessageSystem TCP hub and SimuCloud orchestration.
//!
//! Production split:
//! - Prism `prism.cnc.network.dll` on dedicated = ServerHost (join + gameplay MsgSys)
//! - Refracted = MITM hub `:18386`→`:18387` + SimuCloud orchestrator → `:18388`
//!
//! Join control plane (Victory retail, owned by Prism):
//! `ClientHello` → `ServerHello` / `ServerReadyToStart` / `LoadMap` →
//! `ClientFinishedLoad` → `StartGame` → `AllowInputChange`.
//! `LoadMap` (wire typeId 100) lands in `RtsGameClient_onMessage` case 61
//! (RTS settings / camera / load flags) — not Frostbite `Client::fromNetworkLoadLevel`.

pub mod host; // dump-parity helpers only -- not a production ServerHost
pub mod log;
pub mod messages;
pub mod negotiation;
pub mod proxy;
pub mod server;
pub mod simucloud;
pub mod wire;

pub const LOG_TAG: &str = "RTS";

/// Client → Refracted listen port (Prism patches retail 54321 → here).
pub const MESSAGING_TCP_PORT: u16 = 18386;

pub use proxy::DEDICATED_SERVERHOST_PORT;
