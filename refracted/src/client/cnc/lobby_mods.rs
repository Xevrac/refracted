//! Mods in the lobby (refracted.toolset docs/specs/content-lobby-mods.md).
//!
//! A dedicated server reports the mods it offers (every signed, verified package in its Update
//! folder) with its engine-peer report. A lobby's host turns the server's live mods on and off;
//! restart-class mods are mounted at the server's start, so they are always on. The backend only
//! keeps these lists: it never stores, serves or relays mod files.

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use parking_lot::Mutex;
use serde_json::{json, Value};

use super::{custom_maps, dedicated_pool, game_state};
use crate::http::handlers::handlers_module::HttpResponse;

const MAX_MODS: usize = 64;
const MAX_LEVELS: usize = 32;

fn offered_by_instance() -> &'static Mutex<HashMap<String, Vec<Value>>> {
    static M: OnceLock<Mutex<HashMap<String, Vec<Value>>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

fn enabled_by_gid() -> &'static Mutex<HashMap<i64, BTreeSet<String>>> {
    static M: OnceLock<Mutex<HashMap<i64, BTreeSet<String>>>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(HashMap::new()))
}

fn is_lower_hex(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn is_package_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 32 && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

fn is_author(s: &str) -> bool {
    !s.is_empty() && s.chars().count() <= 48 && s.trim() == s && !s.chars().any(|c| c.is_control())
}

fn is_key(s: &str) -> bool {
    s.len() == 44
        && s.ends_with('=')
        && s[..43].bytes().all(|c| c.is_ascii_alphanumeric() || c == b'+' || c == b'/')
}

fn is_fingerprint(s: &str) -> bool {
    let parts: Vec<&str> = s.split(' ').collect();
    parts.len() == 4 && parts.iter().all(|p| is_lower_hex(p, 4))
}

fn is_level(s: &str) -> bool {
    s.len() <= 200
        && s.to_ascii_lowercase().starts_with("levels/")
        && !s.contains("..")
        && !s.contains("//")
        && s.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'/'))
}

/// One reported mod, re-built from checked fields only; `None` drops it.
fn clean_mod(v: &Value) -> Option<Value> {
    let id = v.get("id")?.as_str()?;
    let name = v.get("name")?.as_str()?;
    let version = v.get("version")?.as_u64()?;
    let author = v.get("author")?.as_str()?;
    let key = v.get("key")?.as_str()?;
    let fingerprint = v.get("fingerprint")?.as_str()?;
    let class = v.get("class")?.as_str()?;
    let bytes = v.get("bytes")?.as_u64()?;
    let patches = v.get("patches").and_then(Value::as_u64).unwrap_or(0);
    if !is_lower_hex(id, 64)
        || !is_package_name(name)
        || version == 0
        || version > u32::MAX as u64
        || !is_author(author)
        || !is_key(key)
        || !is_fingerprint(fingerprint)
        || !(class == "live" || class == "restart")
        || patches > 1_000_000
    {
        return None;
    }
    let levels: Vec<Value> = v
        .get("levels")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .filter(|s| is_level(s))
                .take(MAX_LEVELS)
                .map(|s| Value::String(s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    // The mod's maps.json, sent as a string so a bad file can't break the rest of the report.
    let maps = v
        .get("mapsJson")
        .and_then(Value::as_str)
        .filter(|t| t.len() <= 1 << 20)
        .map(|t| custom_maps::clean_maps(t, name, version as u32))
        .unwrap_or_default();
    Some(json!({
        "id": id, "name": name, "version": version, "author": author, "key": key,
        "fingerprint": fingerprint, "class": class, "bytes": bytes, "patches": patches, "levels": levels,
        "maps": maps,
    }))
}

/// The checked mods of a report body's `mods` array (duplicates by id dropped).
pub fn sanitize_offered_mods(v: &Value) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    for m in v.as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
        if out.len() >= MAX_MODS {
            break;
        }
        if let Some(clean) = clean_mod(m) {
            if !out.iter().any(|o| o["id"] == clean["id"]) {
                out.push(clean);
            }
        }
    }
    out
}

/// Called with each engine-peer report that carries a JSON body.
pub fn note_offered(instance: &str, mods: Vec<Value>) {
    let instance = instance.trim();
    if instance.is_empty() {
        return;
    }
    offered_by_instance().lock().insert(instance.to_string(), mods);
}

pub fn offered_for_session(session: u64) -> Vec<Value> {
    dedicated_pool::instance_for_session(session)
        .and_then(|i| offered_by_instance().lock().get(&i).cloned())
        .unwrap_or_default()
}

fn offered_for_gid(gid: i64) -> Vec<Value> {
    game_state::dedicated_session_for_gid(gid).map(offered_for_session).unwrap_or_default()
}

/// Server browser summary: name, version and class of each offered mod, and whether it adds maps.
pub fn browser_summary(session: u64) -> Value {
    let non_empty = |v: &Value| v.as_array().is_some_and(|a| !a.is_empty());
    Value::Array(
        offered_for_session(session)
            .iter()
            .map(|m| {
                json!({
                    "name": m["name"], "version": m["version"], "class": m["class"],
                    "maps": non_empty(&m["levels"]) || non_empty(&m["maps"]),
                })
            })
            .collect(),
    )
}

/// The lobby's mods with `on`: restart-class always, live when the host turned it on.
fn lobby_state(gid: i64) -> Value {
    let enabled = enabled_by_gid().lock().get(&gid).cloned().unwrap_or_default();
    let mods: Vec<Value> = offered_for_gid(gid)
        .into_iter()
        .map(|mut m| {
            let id = m["id"].as_str().unwrap_or_default().to_string();
            let on = m["class"] == "restart" || enabled.contains(&id);
            m["on"] = Value::Bool(on);
            m
        })
        .collect();
    json!({ "ok": true, "gid": gid, "host": game_state::host_persona_for_gid(gid), "mods": mods })
}

/// Ids of the lobby's mods that are on (for the shell's map list and the players' checks).
pub fn enabled_ids(gid: i64) -> Vec<String> {
    lobby_state(gid)["mods"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|m| m["on"] == true)
                .filter_map(|m| m["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn query_i64(query: Option<&str>, key: &str) -> i64 {
    query
        .unwrap_or("")
        .split('&')
        .find_map(|p| p.split_once('=').filter(|(k, _)| *k == key).and_then(|(_, v)| v.trim().parse().ok()))
        .unwrap_or(0)
}

fn reply(status: u16, v: Value) -> HttpResponse {
    HttpResponse::new(status, "application/json", v.to_string().into_bytes())
}

/// `GET /cnc/lobby-mods?gid=` — the lobby's server's mods and which are on.
pub fn handle_get(query: Option<&str>) -> HttpResponse {
    let gid = query_i64(query, "gid");
    if gid <= 0 {
        return reply(400, json!({ "ok": false, "error": "gid required" }));
    }
    reply(200, lobby_state(gid))
}

/// `POST /cnc/lobby-mods?gid=&pid=` with `{ "enabled": [id, ...] }` — host only. Every id must be a
/// live mod the lobby's server offers; restart-class mods can't be turned off here.
pub fn handle_post(query: Option<&str>, body: &[u8]) -> HttpResponse {
    let gid = query_i64(query, "gid");
    let pid = query_i64(query, "pid");
    if gid <= 0 {
        return reply(400, json!({ "ok": false, "error": "gid required" }));
    }
    let host = game_state::host_persona_for_gid(gid);
    if pid <= 0 || host <= 0 || pid != host {
        return reply(403, json!({ "ok": false, "error": "host only", "host": host }));
    }
    let wanted: Vec<String> = serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|v| v.get("enabled").and_then(Value::as_array).cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let offered = offered_for_gid(gid);
    let mut set = BTreeSet::new();
    for id in wanted {
        match offered.iter().find(|m| m["id"] == id.as_str()) {
            Some(m) if m["class"] == "live" => {
                set.insert(id);
            }
            Some(_) => {}
            None => {
                return reply(409, json!({ "ok": false, "error": "not offered by this server", "id": id }));
            }
        }
    }
    {
        let mut m = enabled_by_gid().lock();
        m.retain(|g, _| game_state::dedicated_session_for_gid(*g).is_some());
        m.insert(gid, set);
    }
    reply(200, lobby_state(gid))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> Value {
        json!({
            "id": "e".repeat(64), "name": "mod-icxmy7", "version": 3, "author": "Xevrac",
            "key": format!("{}=", "A".repeat(43)), "fingerprint": "8687 d4d4 731f 35fd",
            "class": "restart", "bytes": 92_000_000u64, "patches": 28,
            "levels": ["Levels/MP/PVP/helloworld/helloworld", "../etc", "Levels/a b"],
        })
    }

    #[test]
    fn keeps_checked_fields_only() {
        let mut m = good();
        m["evil"] = json!("<script>");
        let out = sanitize_offered_mods(&json!([m]));
        assert_eq!(out.len(), 1);
        assert!(out[0].get("evil").is_none());
        assert_eq!(out[0]["levels"], json!(["Levels/MP/PVP/helloworld/helloworld"]));
    }

    #[test]
    fn drops_bad_entries_and_duplicates() {
        let mut bad_id = good();
        bad_id["id"] = json!("E".repeat(64));
        let mut bad_class = good();
        bad_class["class"] = json!("exe");
        let mut bad_author = good();
        bad_author["author"] = json!("a\u{0007}b");
        let mut bad_name = good();
        bad_name["name"] = json!("../mods");
        let out = sanitize_offered_mods(&json!([good(), good(), bad_id, bad_class, bad_author, bad_name, 5, "x"]));
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn carries_checked_maps() {
        let mut m = good();
        m["mapsJson"] = json!(r#"{"maps":[{"path":"Levels/MP/PVP/helloworld/helloworld","label":"Hello World","players":2},{"path":"../x"}]}"#);
        let out = sanitize_offered_mods(&json!([m]));
        assert_eq!(out[0]["maps"].as_array().unwrap().len(), 1);
        assert_eq!(out[0]["maps"][0]["mod"], "mod-icxmy7");
        let mut broken = good();
        broken["mapsJson"] = json!("{not json");
        assert_eq!(sanitize_offered_mods(&json!([broken]))[0]["maps"], json!([]));
    }

    #[test]
    fn caps_the_list() {
        let many: Vec<Value> = (0..100)
            .map(|i| {
                let mut m = good();
                m["id"] = json!(format!("{:064x}", i));
                m
            })
            .collect();
        assert_eq!(sanitize_offered_mods(&Value::Array(many)).len(), MAX_MODS);
    }
}
