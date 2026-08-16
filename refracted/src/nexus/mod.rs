//! **Nexus** -- Refracted's identity/account backend (personas, accounts, future login/logout).
//!
//! A Refracted account is powered by this Nexus layer. It is **not** EA Nucleus and is **not**
//! a second protocol stack.
//!
//! ## Relationship to Blaze
//! - [`crate::blaze::components`] stays the registry for **Blaze wire** names (including EA
//!   component **1002** `NucleusIdentityComponent` RPC labels). Those names stay on the wire.
//! - This module holds emulator-side **policy and state** used when building Blaze responses
//!   (persona/account fields sourced from [`crate::common::user_profile`] / [`crate::session`]).
//!
//! ## Wire reality
//! Titles that call **NucleusIdentity** over Blaze send **1002** packets on the **same Blaze
//! connection** as everything else. What stays internal here is *our* choice of when we
//! **synthesize** or **map** those responses from Nexus vs. pass-through.
//!
//! ## UI
//! **Settings → Accounts** is the desktop surface for this layer (JSON personas for local
//! development). Headless production uses MySQL identity; game clients authenticate.

pub mod backend;
pub mod identity;
pub mod log;

pub use backend::NexusBackend;
pub use log::{log_blaze_to_nexus, log_nexus_to_blaze};
