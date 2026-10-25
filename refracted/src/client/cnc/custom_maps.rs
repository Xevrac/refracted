//! Custom maps: the new levels in a lobby server's mods. Each mod's `maps.json` (written by the
//! editor's Build Mod, format `refracted-maps/1`) reaches the backend inside the server's report of
//! the mods it offers (`lobby_mods`); this module only checks the entries. The backend never reads
//! a game folder for them.

use serde_json::{json, Map, Value};

const MAX_LABEL: usize = 64;
const MAX_PLAYERS: i64 = 8;
const MAX_MAPS: usize = 32;
/// The editor's top-down snapshot, base64 JPEG (Prism keeps it to 256 KiB).
const MAX_MINIMAP_DATA: usize = 360 * 1024;
const MINIMAP_DATA_PREFIX: &str = "data:image/jpeg;base64,";

/// The checked maps of one mod's `maps.json` text, sorted by label; nothing if it doesn't parse.
pub fn clean_maps(maps_json: &str, mod_name: &str, version: u32) -> Vec<Value> {
    let Ok(doc) = serde_json::from_str::<Value>(maps_json) else {
        return Vec::new();
    };
    let mut out: Vec<Value> = doc
        .get("maps")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|e| clean_entry(e, mod_name, version)).take(MAX_MAPS).collect())
        .unwrap_or_default();
    out.sort_by(|a, b| {
        let la = a.get("label").and_then(Value::as_str).unwrap_or_default().to_ascii_lowercase();
        let lb = b.get("label").and_then(Value::as_str).unwrap_or_default().to_ascii_lowercase();
        la.cmp(&lb)
    });
    out
}

fn finite(v: Option<&Value>) -> Option<f64> {
    v.and_then(Value::as_f64).filter(|f| f.is_finite())
}

/// `{ minX, maxX, minZ, maxZ }` with positive width and depth.
fn clean_bounds(b: Option<&Value>) -> Option<Value> {
    let b = b?;
    let (a, c, d, e) = (finite(b.get("minX"))?, finite(b.get("maxX"))?, finite(b.get("minZ"))?, finite(b.get("maxZ"))?);
    (c > a && e > d).then(|| json!({ "minX": a, "maxX": c, "minZ": d, "maxZ": e }))
}

/// A JPEG data URI and nothing else, so the shell can use it as an image source as is.
fn clean_minimap_data(v: Option<&Value>) -> Option<String> {
    let s = v?.as_str()?;
    let body = s.strip_prefix(MINIMAP_DATA_PREFIX)?;
    let ok = s.len() <= MAX_MINIMAP_DATA
        && !body.is_empty()
        && body.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'/' | b'='));
    ok.then(|| s.to_string())
}

/// Only what the shell needs, checked: a level path under `Levels/`, a short label, a plain minimap file name.
fn clean_entry(entry: &Value, mod_name: &str, version: u32) -> Option<Value> {
    let path = entry.get("path")?.as_str()?.trim();
    let lower = path.to_ascii_lowercase();
    if !lower.starts_with("levels/") || path.contains("..") || path.contains('\\') || path.len() > 200 {
        return None;
    }
    let leaf = path.rsplit('/').next().unwrap_or(path).to_string();
    let mut label: String = entry
        .get("label")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&leaf)
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    if label.chars().count() > MAX_LABEL {
        label = label.chars().take(MAX_LABEL).collect();
    }
    let minimap = entry
        .get("minimap")
        .and_then(Value::as_str)
        .filter(|m| !m.is_empty() && m.chars().all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c)) && !m.contains(".."))
        .map(str::to_string)
        .unwrap_or_else(|| format!("{leaf}.png"));

    let mut starts = Vec::new();
    for s in entry.get("starts").and_then(Value::as_array).cloned().unwrap_or_default() {
        let Some(id) = s.get("id").and_then(Value::as_i64).filter(|i| (1..=20).contains(i)) else {
            continue;
        };
        let mut start = Map::new();
        start.insert("id".into(), json!(id));
        if let (Some(x), Some(z)) = (finite(s.get("x")), finite(s.get("z"))) {
            start.insert("x".into(), json!(x));
            start.insert("z".into(), json!(z));
        }
        starts.push(Value::Object(start));
    }
    let players = entry
        .get("players")
        .and_then(Value::as_i64)
        .filter(|n| (1..=MAX_PLAYERS).contains(n))
        .unwrap_or(starts.len() as i64)
        .clamp(1, MAX_PLAYERS);

    let mut map = Map::new();
    map.insert("path".into(), json!(path));
    map.insert("label".into(), json!(label));
    map.insert("minimap".into(), json!(minimap));
    map.insert("players".into(), json!(players));
    map.insert("starts".into(), Value::Array(starts));
    map.insert("mod".into(), json!(mod_name));
    map.insert("version".into(), json!(version));
    if let Some(source) = entry.get("source").and_then(Value::as_str).filter(|s| s.to_ascii_lowercase().starts_with("levels/")) {
        map.insert("source".into(), json!(source));
    }
    if let Some(bounds) = clean_bounds(entry.get("bounds")) {
        map.insert("bounds".into(), bounds);
    }
    if let (Some(data), Some(bounds)) = (clean_minimap_data(entry.get("minimapData")), clean_bounds(entry.get("minimapBounds"))) {
        map.insert("minimapData".into(), json!(data));
        map.insert("minimapBounds".into(), bounds);
    }
    Some(Value::Object(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HELLO: &str = r#"{"format":"refracted-maps/1","maps":[{"path":"Levels/MP/PVP/helloworld/helloworld","label":"Hello World","source":"Levels/MP/PVP/DM_Smalltown_1v1_CR/DM_Smalltown_1v1_CR","minimap":"helloworld.png","players":2,"starts":[{"id":1,"x":-179.9,"z":315.2},{"id":2,"x":179.9,"z":-309.8}],"bounds":{"minX":-290,"maxX":306,"minZ":-420,"maxZ":426}}]}"#;

    #[test]
    fn keeps_a_good_entry() {
        let maps = clean_maps(HELLO, "mod-a", 3);
        assert_eq!(maps.len(), 1);
        let m = &maps[0];
        assert_eq!(m["path"], "Levels/MP/PVP/helloworld/helloworld");
        assert_eq!(m["label"], "Hello World");
        assert_eq!(m["mod"], "mod-a");
        assert_eq!(m["version"], 3);
        assert_eq!(m["players"], 2);
        assert_eq!(m["starts"].as_array().unwrap().len(), 2);
        assert_eq!(m["bounds"]["maxZ"], 426.0);
        assert_eq!(m["source"], "Levels/MP/PVP/DM_Smalltown_1v1_CR/DM_Smalltown_1v1_CR");
    }

    #[test]
    fn keeps_a_snapshot_only_as_a_jpeg_data_uri_with_bounds() {
        let good = r#"{"maps":[{"path":"Levels/MP/PVP/a/a","minimapData":"data:image/jpeg;base64,/9j/4AAQ","minimapBounds":{"minX":-10,"maxX":10,"minZ":-10,"maxZ":10}}]}"#;
        let maps = clean_maps(good, "mod-a", 1);
        assert_eq!(maps[0]["minimapData"], "data:image/jpeg;base64,/9j/4AAQ");
        assert_eq!(maps[0]["minimapBounds"]["maxX"], 10.0);

        for bad in [
            r#"{"maps":[{"path":"Levels/MP/PVP/a/a","minimapData":"data:text/html;base64,PHNjcmlwdD4=","minimapBounds":{"minX":0,"maxX":1,"minZ":0,"maxZ":1}}]}"#,
            r#"{"maps":[{"path":"Levels/MP/PVP/a/a","minimapData":"data:image/jpeg;base64,/9j/<x>","minimapBounds":{"minX":0,"maxX":1,"minZ":0,"maxZ":1}}]}"#,
            r#"{"maps":[{"path":"Levels/MP/PVP/a/a","minimapData":"data:image/jpeg;base64,/9j/4AAQ"}]}"#,
        ] {
            let maps = clean_maps(bad, "mod-a", 1);
            assert!(maps[0].get("minimapData").is_none(), "{bad}");
        }
    }

    #[test]
    fn drops_entries_that_are_not_levels_or_carry_unsafe_names() {
        let maps = clean_maps(
            r#"{"maps":[
                {"path":"../../Windows/x","label":"bad"},
                {"path":"Levels/MP/PVP/ok/ok","minimap":"../../secret.png","players":99},
                {"path":"Levels\\MP\\x","label":"backslash"}
            ]}"#,
            "mod-a",
            1,
        );
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0]["label"], "ok");
        assert_eq!(maps[0]["minimap"], "ok.png");
        assert_eq!(maps[0]["players"], 1);
    }

    #[test]
    fn broken_text_gives_nothing() {
        assert!(clean_maps("not json", "mod-a", 1).is_empty());
        assert!(clean_maps("{}", "mod-a", 1).is_empty());
    }
}
