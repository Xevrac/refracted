//! GameReporting component (0x1C). The CNC Blaze client is the only writer.
//!
//! `submitGameReport` carries a variable report whose player map (`PLYR`) holds
//! per-player stat maps (`STAT`). A stat named `result` / `victory` / `win` / `loss`
//! with value 1 records that persona. Queries return an empty history.

use crate::blaze::tdf::tdf_tree::{TdfTreeNode, TdfTreeParser, TdfValue};
use crate::blaze::tdf::TdfEncoder;
use crate::common::error::BlazeResult;
use bytes::Bytes;

pub fn handle(command: u16, payload: &[u8]) -> BlazeResult<Bytes> {
    match command {
        1 | 2 | 100 | 101 => apply_report(command, payload),
        _ => {}
    }
    Ok(Bytes::new())
}

fn apply_report(command: u16, payload: &[u8]) {
    let nodes = match TdfTreeParser::parse_packet(payload) {
        Ok(nodes) => nodes,
        Err(err) => {
            crate::debug_println!(
                "\x1b[38;2;100;200;255m[CNC]\x1b[0m game report cmd={} parse failed: {}",
                command,
                err
            );
            return;
        }
    };
    let mut outcomes = Vec::new();
    collect_outcomes(&nodes, "", &mut outcomes);
    outcomes.sort_by_key(|(persona, _)| *persona);
    outcomes.dedup_by_key(|(persona, _)| *persona);
    if outcomes.is_empty() {
        if let Some(victory) = session_result(payload) {
            record_session(command, victory);
            return;
        }
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[CNC]\x1b[0m game report cmd={} bytes={} no player result {:02x?}",
            command,
            payload.len(),
            &payload[..payload.len().min(24)]
        );
        return;
    }
    for (persona, victory) in outcomes {
        match crate::nexus::identity::record_persona_match(persona, victory) {
            Ok(stats) => crate::debug_println!(
                "\x1b[38;2;100;200;255m[CNC]\x1b[0m game report cmd={} persona={} result={} wins={} losses={}",
                command,
                persona,
                if victory { "win" } else { "loss" },
                stats.wins,
                stats.losses
            ),
            Err(err) => crate::debug_println!(
                "\x1b[38;2;100;200;255m[CNC]\x1b[0m game report cmd={} persona={} record failed: {}",
                command,
                persona,
                err
            ),
        }
    }
}

fn session_result(payload: &[u8]) -> Option<bool> {
    match TdfEncoder::find_long_field(payload, "RSLT")? {
        1 => Some(true),
        0 => Some(false),
        _ => None,
    }
}

fn session_persona() -> Option<i64> {
    let sid = crate::session::current_blaze_session_id()?;
    let info = crate::session::blaze_sessions::get_session(sid)?;
    let persona = info.persona_id?;
    (persona > 0).then_some(persona as i64)
}

fn record_session(command: u16, victory: bool) {
    let Some(persona) = session_persona() else {
        crate::debug_println!(
            "\x1b[38;2;100;200;255m[CNC]\x1b[0m game report cmd={} result={} no session persona",
            command,
            if victory { "win" } else { "loss" }
        );
        return;
    };
    match crate::nexus::identity::record_persona_match(persona, victory) {
        Ok(stats) => crate::debug_println!(
            "\x1b[38;2;100;200;255m[CNC]\x1b[0m game report cmd={} persona={} result={} wins={} losses={}",
            command,
            persona,
            if victory { "win" } else { "loss" },
            stats.wins,
            stats.losses
        ),
        Err(err) => crate::debug_println!(
            "\x1b[38;2;100;200;255m[CNC]\x1b[0m game report cmd={} persona={} record failed: {}",
            command,
            persona,
            err
        ),
    }
}

fn collect_outcomes(nodes: &[TdfTreeNode], parent_tag: &str, out: &mut Vec<(i64, bool)>) {
    for node in nodes {
        if node.value_type == "MAP_ENTRY" && parent_tag.trim().eq_ignore_ascii_case("PLYR") {
            if let Some(persona) = parse_persona(&node.tag) {
                if let Some(victory) = player_result(&node.children) {
                    out.push((persona, victory));
                }
            }
        }
        collect_outcomes(&node.children, &node.tag, out);
    }
}

fn player_result(nodes: &[TdfTreeNode]) -> Option<bool> {
    let mut found = None;
    walk_stats(nodes, &mut found);
    found
}

fn walk_stats(nodes: &[TdfTreeNode], found: &mut Option<bool>) {
    for node in nodes {
        if node.value_type == "MAP_ENTRY" {
            if let Some(victory) = stat_result(&node.tag, stat_value(node)) {
                *found = Some(victory);
                return;
            }
        }
        walk_stats(&node.children, found);
        if found.is_some() {
            return;
        }
    }
}

fn stat_result(name: &str, value: Option<f64>) -> Option<bool> {
    let value = value?;
    let one = (value - 1.0).abs() < 0.01;
    let zero = value.abs() < 0.01;
    match name.trim().to_ascii_lowercase().as_str() {
        "result" | "victory" if one => Some(true),
        "result" | "victory" if zero => Some(false),
        "win" | "wins" if one => Some(true),
        "loss" | "losses" if one => Some(false),
        _ => None,
    }
}

fn stat_value(entry: &TdfTreeNode) -> Option<f64> {
    if let Some(value) = number_of(entry) {
        return Some(value);
    }
    entry.children.iter().find_map(number_of)
}

fn number_of(node: &TdfTreeNode) -> Option<f64> {
    if let Some(raw) = &node.raw_value {
        match raw {
            TdfValue::Integer(n) => return Some(*n as f64),
            TdfValue::Float(n) => return Some(*n as f64),
            _ => {}
        }
    }
    let text = node.value_display.trim();
    if text.is_empty() || text == node.tag {
        return None;
    }
    text.parse::<f64>().ok()
}

fn parse_persona(tag: &str) -> Option<i64> {
    let id = tag.trim().parse::<i64>().ok()?;
    (id > 0).then_some(id)
}
