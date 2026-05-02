//! Refracted library: protocol servers, HTTP/gRPC handling, and per-title client behavior.
//!
//! **Layout (intentional):**
//! - [`core`] — Server lifecycle, console/logging, the **toolkit** (inspectors, research proxies).
//! - [`blaze`], [`http`], [`web`], [`lsx`], [`qos`], [`rtm`] — Service layers (listeners and handlers).
//! - [`grpc`], [`jwt`], [`session`], [`crypto`] — Shared building blocks for auth and wire formats.
//! - [`client`] — Title-specific logic (`labs`, `cnc`, …), selected by [`common::game`] from `games.json`.
//!
//! The binary in `main.rs` hosts the desktop UI; the library is structured so servers and handlers stay testable and separated from egui.

// Core modules
pub mod core;

// Blaze protocol modules
pub mod blaze;

// HTTP modules
pub mod http;

// Web modules
pub mod web;

// LSX modules
pub mod lsx;

// QoS modules
pub mod qos;

// RTM modules
pub mod rtm;

// Session management
pub mod session;

// JWT handling
pub mod jwt;

// gRPC handling
pub mod grpc;

// Crypto
pub mod crypto;

// Common utilities
pub mod common;

// Per-game (client) configuration and title-specific logic
pub mod client;

// Re-export commonly used types
pub use common::error::{BlazeError, BlazeResult};
pub use common::startup_progress::get_current_startup_message;
pub use core::console::*;
