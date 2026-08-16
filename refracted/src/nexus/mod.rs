//! **Nexus** — Refracted identity (personas, accounts, login).
//!
//! Not EA Nucleus and not a second protocol stack. Blaze still uses wire name
//! `NucleusIdentityComponent` (1002). This module holds emulator policy and state.
//!
//! Desktop: Settings → Accounts (JSON). Headless production: MySQL; clients authenticate.

pub mod backend;
pub mod identity;
pub mod log;

pub use backend::NexusBackend;
pub use log::{log_blaze_to_nexus, log_nexus_to_blaze};
