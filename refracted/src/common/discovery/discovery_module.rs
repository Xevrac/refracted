/// Discovery mechanism to track new component/command combinations from clients
/// Uses Fire2Frame protocol component/command IDs
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::sync::OnceLock;

static DISCOVERED_COMMANDS: OnceLock<Mutex<HashSet<(u16, u16)>>> = OnceLock::new();
static FIRST_SEEN_SEQ: OnceLock<Mutex<HashMap<(u16, u16), u64>>> = OnceLock::new();

fn get_discovered_set() -> &'static Mutex<HashSet<(u16, u16)>> {
    DISCOVERED_COMMANDS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn get_first_seen_map() -> &'static Mutex<HashMap<(u16, u16), u64>> {
    FIRST_SEEN_SEQ.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Check if a component/command combination has been seen before
/// Returns true if this is a new discovery
pub fn check_and_record(component: u16, command: u16) -> bool {
    let discovered = get_discovered_set();
    let mut set = discovered.lock().unwrap();
    set.insert((component, command))
}

/// When [`check_and_record`] returns true, store the toolkit buffer seq for the client request
/// that triggered the first unhandled handler (for jumping in Listen).
pub fn record_first_seen_seq_if_new(
    is_new_discovery: bool,
    component: u16,
    command: u16,
    incoming_capture_seq: Option<u64>,
) {
    if !is_new_discovery {
        return;
    }
    let Some(seq) = incoming_capture_seq else {
        return;
    };
    let mut m = get_first_seen_map().lock().unwrap();
    m.entry((component, command)).or_insert(seq);
}

/// Get all discovered component/command combinations
pub fn get_discovered_commands() -> Vec<(u16, u16)> {
    let discovered = get_discovered_set();
    let set = discovered.lock().unwrap();
    set.iter().copied().collect()
}

pub fn first_seen_capture_seq(component: u16, command: u16) -> Option<u64> {
    let m = get_first_seen_map().lock().unwrap();
    m.get(&(component, command)).copied()
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryExportRow {
    pub component: u16,
    pub command: u16,
    pub component_name: String,
    pub command_label: String,
    pub first_seen_capture_seq: Option<u64>,
}

pub fn discovery_export_rows() -> Vec<DiscoveryExportRow> {
    let mut pairs = get_discovered_commands();
    pairs.sort_by_key(|(c, k)| (*c, *k));
    let first = get_first_seen_map().lock().unwrap();
    pairs
        .into_iter()
        .map(|(component, command)| DiscoveryExportRow {
            component,
            command,
            component_name: crate::blaze::components::get_component_name(component).to_string(),
            command_label: crate::blaze::components::get_command_name(component, command)
                .unwrap_or_else(|| format!("UnknownCommand({})", command)),
            first_seen_capture_seq: first.get(&(component, command)).copied(),
        })
        .collect()
}

pub fn discovery_json() -> String {
    let rows = discovery_export_rows();
    match serde_json::to_string_pretty(&rows) {
        Ok(s) => s,
        Err(e) => format!("{{\"error\":\"{}\"}}", e),
    }
}

pub fn discovery_csv() -> String {
    let mut out = String::from("component,command,component_name,command_label,first_seen_capture_seq\n");
    for r in discovery_export_rows() {
        let seq_s = r
            .first_seen_capture_seq
            .map(|s| s.to_string())
            .unwrap_or_default();
        let cmd_esc = r.command_label.replace('"', "\"\"");
        out.push_str(&format!(
            "{},{},\"{}\",\"{}\",{}\n",
            r.component, r.command, r.component_name, cmd_esc, seq_s
        ));
    }
    out
}

pub fn clear_discovery_tracking() {
    if let Ok(mut s) = get_discovered_set().lock() {
        s.clear();
    }
    if let Ok(mut m) = get_first_seen_map().lock() {
        m.clear();
    }
}

/// Check if a command has been discovered before
pub fn is_discovered(component: u16, command: u16) -> bool {
    let discovered = get_discovered_set();
    let set = discovered.lock().unwrap();
    set.contains(&(component, command))
}

