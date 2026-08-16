//! Directional console lines for **Nexus identity work** (login/logout, profile→session sync, data pulls).
//! Intentionally **not** used for every Blaze packet -- use normal Blaze/inspector logging there.

// Dark blue family, two shades to read direction at a glance.
const TAG_N2B: &str = "\x1b[38;2;40;80;160m[Nexus → Blaze]\x1b[0m";
const TAG_B2N: &str = "\x1b[38;2;25;55;190m[Blaze → Nexus]\x1b[0m";

/// Identity or account data **pushed** from the Nexus layer into session/Blaze handler inputs.
pub fn log_nexus_to_blaze(msg: impl AsRef<str>) {
    crate::console_println!("{} {}", TAG_N2B, msg.as_ref());
}

/// Data or events **ingested** from Blaze handling back into the Nexus model (e.g. after a 1002
/// NucleusIdentity response is interpreted -- call only from intentional bridge points).
pub fn log_blaze_to_nexus(msg: impl AsRef<str>) {
    crate::console_println!("{} {}", TAG_B2N, msg.as_ref());
}
