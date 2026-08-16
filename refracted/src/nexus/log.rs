//! Nexus identity log lines (not used for every Blaze packet).

// Dark blue family.
const TAG_N2B: &str = "\x1b[38;2;40;80;160m[Nexus → Blaze]\x1b[0m";
const TAG_B2N: &str = "\x1b[38;2;25;55;190m[Blaze → Nexus]\x1b[0m";

/// Identity pushed from Nexus into session/Blaze inputs.
pub fn log_nexus_to_blaze(msg: impl AsRef<str>) {
    crate::console_println!("{} {}", TAG_N2B, msg.as_ref());
}

/// Events ingested from Blaze into the Nexus model.
pub fn log_blaze_to_nexus(msg: impl AsRef<str>) {
    crate::console_println!("{} {}", TAG_B2N, msg.as_ref());
}
