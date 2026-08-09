//!
//! Production split:
//! - Prism `prism.cnc.network.dll` on dedicated = ServerHost (join + gameplay MsgSys)
//!
//! Join control plane (Victory retail, owned by Prism):

pub mod host; // wire-shape helpers only -- not a production ServerHost
pub mod log;
pub mod messages;
pub mod negotiation;
pub mod proxy;
pub mod server;
pub mod simucloud;
pub mod wire;

pub const LOG_TAG: &str = "RTS";

pub const MESSAGING_TCP_PORT: u16 = 18386;

pub use proxy::DEDICATED_SERVERHOST_PORT;
