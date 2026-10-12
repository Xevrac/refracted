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

/// H498: highest Refracted <-> dedicated round trip supported (overseas servers over the VPN).
/// Every connect attempt and the browser ping probe wait this long before giving up an attempt.
pub const MAX_SERVER_RTT: std::time::Duration = std::time::Duration::from_millis(1200);

pub use proxy::DEDICATED_SERVERHOST_PORT;
