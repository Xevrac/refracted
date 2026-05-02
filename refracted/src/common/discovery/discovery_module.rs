/// Discovery mechanism to track new component/command combinations from clients
/// Uses Fire2Frame protocol component/command IDs
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;

static DISCOVERED_COMMANDS: OnceLock<Mutex<HashSet<(u16, u16)>>> = OnceLock::new();

fn get_discovered_set() -> &'static Mutex<HashSet<(u16, u16)>> {
    DISCOVERED_COMMANDS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Check if a component/command combination has been seen before
/// Returns true if this is a new discovery
pub fn check_and_record(component: u16, command: u16) -> bool {
    let discovered = get_discovered_set();
    let mut set = discovered.lock().unwrap();
    set.insert((component, command))
}

/// Get all discovered component/command combinations
pub fn get_discovered_commands() -> Vec<(u16, u16)> {
    let discovered = get_discovered_set();
    let set = discovered.lock().unwrap();
    set.iter().copied().collect()
}

/// Check if a command has been discovered before
pub fn is_discovered(component: u16, command: u16) -> bool {
    let discovered = get_discovered_set();
    let set = discovered.lock().unwrap();
    set.contains(&(component, command))
}

