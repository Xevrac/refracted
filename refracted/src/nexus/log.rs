//! Nexus identity log lines (not used for every Blaze packet).

const TAG: &str = "\x1b[38;2;70;120;210m[Nexus]\x1b[0m";

/// Identity pushed from Nexus into session/Blaze inputs.
pub fn log_nexus_to_blaze(msg: impl AsRef<str>) {
    crate::console_println!("{} {}", TAG, msg.as_ref());
}

/// Events ingested from Blaze into the Nexus model.
pub fn log_blaze_to_nexus(msg: impl AsRef<str>) {
    crate::console_println!("{} {}", TAG, msg.as_ref());
}
