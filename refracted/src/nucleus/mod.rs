//! **Nucleus** — game-agnostic identity/account facade inside Refracted.
//!
//! ## Relationship to Blaze
//! - [`crate::blaze::components`] stays the registry for **Blaze wire** names (including component **1002**
//!   `NucleusIdentityComponent` RPC labels).
//! - This module is **not** a second protocol stack. It holds emulator-side **policy and state** used when
//!   building Blaze responses (e.g. persona/account fields sourced from [`crate::common::user_profile`] /
//!   [`crate::session`]).
//!
//! ## Wire reality (correcting a common simplification)
//! Titles that call **NucleusIdentity** over Blaze send **1002** packets on the **same Blaze connection**
//! as everything else — the game client *can* see that traffic. What stays “internal” in *this* app is
//! *our* choice of when we **synthesize** or **map** those responses from the Nucleus layer vs. pass-through.
//!
//! ## UI
//! **Settings → Accounts** is the primary surface for this layer (profiles, session field source-of-truth
//! before/during titles). It can grow (entitlements, opt-in, etc.) without moving Blaze tables.

pub mod backend;
pub mod log;

pub use backend::NucleusBackend;
pub use log::{log_blaze_to_nucleus, log_nucleus_to_blaze};
