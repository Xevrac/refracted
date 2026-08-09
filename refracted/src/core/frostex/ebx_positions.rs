//! World-space entity placement harvest from Frostbite 2 EBX.
//!
//! Ports the BlueprintTransform / LinearTransform walk from `ebx_viewer6.py`
//! so FrostEx can export per-map CSVs Prism uses as an authored-position benchmark.

use crate::core::frostex::ebx::{EbxGuid, EbxGuidTable};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Bucket for EBX that is not under a `levels/…` path (shared art, UI, etc.).
pub const OTHER_MAP_KEY: &str = "_other";

const WORLD_TRANSFORM_NAMES: &[&str] = &[
    "blueprinttransform",
    "worldtransform",
    "placementtransform",
];

const LABEL_PRIORITY: &[&str] = &[
    "blueprint",
    "scriptobjecttag",
    "name",
    "objectname",
    "unitname",
];

/// One placed instance (or asset root) with optional world transform.
#[derive(Debug, Clone)]
pub struct EbxPlacement {
    pub name: String,
    pub class_name: String,
    pub fnv1a: u32,
    pub pos: Option<[f32; 3]>,
    pub right: Option<[f32; 3]>,
    pub up: Option<[f32; 3]>,
    pub forward: Option<[f32; 3]>,
    /// Dashed GUID matching `EbxGuid::format()` / System.Guid "D".
    pub guid: String,
    pub file_path: String,
}

impl EbxPlacement {
    pub fn has_position(&self) -> bool {
        self.pos.is_some()
    }
}

/// 32-bit FNV-1a of a lowercase UTF-8 string (matches ebx_viewer6 / Prism helper).
pub fn fnv1a_32(s: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for b in s.to_ascii_lowercase().as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(16777619);
    }
    h
}

/// Strip trailing `/<32-hex instance GUID>` from a Blueprint FileRef / Class path.
pub fn clean_blueprint_path(value: &str) -> String {
    if let Some((head, tail)) = value.rsplit_once('/') {
        if tail.len() == 32 && tail.chars().all(|c| c.is_ascii_hexdigit()) {
            return head.to_string();
        }
    }
    value.to_string()
}

/// Parse EBX bytes and harvest placements. `file_path` is recorded on each row.
pub fn extract_ebx_placements(
    data: &[u8],
    file_path: &str,
    table: Option<&EbxGuidTable>,
) -> Result<Vec<EbxPlacement>, String> {
    let dbx = crate::core::frostex::ebx::parse_dbx_for_positions(data)?;
    Ok(collect_placements(&dbx, file_path, table))
}

/// Keep rows that have a real world position (bench CSV default).
pub fn filter_with_positions(rows: &[EbxPlacement]) -> Vec<EbxPlacement> {
    rows.iter()
        .filter(|r| r.has_position())
        .cloned()
        .collect()
}

/// Derive a stable map bucket from an EBX rip path.
///
/// Examples:
/// - `ebx/levels/mp/pve/firstplayable_mphorde_final/foo.txt`
///   → `levels/mp/pve/firstplayable_mphorde_final`
/// - `ebx/levels/sp/alpha_tutorial/alpha_tutorial.txt`
///   → `levels/sp/alpha_tutorial`
/// - `_rts/art/...` → `_other`
pub fn map_key_from_ebx_path(path: &str) -> String {
    let norm = path.replace('\\', "/").to_ascii_lowercase();
    let mut s = norm.as_str();
    if let Some(rest) = s.strip_prefix("./") {
        s = rest;
    }
    if let Some(rest) = s.strip_prefix("ebx/") {
        s = rest;
    }

    let Some(levels_at) = s.find("levels/") else {
        return OTHER_MAP_KEY.to_string();
    };
    let rest = &s[levels_at..];
    let parts: Vec<&str> = rest
        .split('/')
        .filter(|p| !p.is_empty())
        .map(|p| p.strip_suffix(".txt").unwrap_or(p))
        .collect();
    if parts.len() < 2 {
        return OTHER_MAP_KEY.to_string();
    }

    match parts[1] {
        "mp" if parts.len() >= 4 => format!("levels/mp/{}/{}", parts[2], parts[3]),
        "sp" if parts.len() >= 3 => format!("levels/sp/{}", parts[2]),
        "test" if parts.len() >= 5 => {
            // levels/test/design/mo/mo_mp_horde_2p/...
            format!(
                "levels/test/{}/{}/{}",
                parts[2], parts[3], parts[4]
            )
        }
        "test" if parts.len() >= 3 => format!("levels/test/{}", parts[2]),
        name => format!("levels/{name}"),
    }
}

/// Summary of a per-map positions dump.
#[derive(Debug, Clone, Default)]
pub struct MapCsvWriteSummary {
    pub map_count: usize,
    pub row_count: usize,
    /// Relative paths under the dump root, e.g. `entity_positions/levels/mp/pve/….csv`
    pub files: Vec<(String, usize)>,
}

/// Group rows by [`map_key_from_ebx_path`] and write one CSV each under `out_dir/entity_positions/`.
pub fn write_placements_by_map(
    out_dir: &Path,
    rows: &[EbxPlacement],
) -> Result<MapCsvWriteSummary, String> {
    let mut groups: BTreeMap<String, Vec<&EbxPlacement>> = BTreeMap::new();
    for row in rows {
        groups
            .entry(map_key_from_ebx_path(&row.file_path))
            .or_default()
            .push(row);
    }

    let base = out_dir.join("entity_positions");
    let mut summary = MapCsvWriteSummary {
        map_count: groups.len(),
        ..MapCsvWriteSummary::default()
    };

    for (map_key, group) in &groups {
        let owned: Vec<EbxPlacement> = group.iter().map(|r| (*r).clone()).collect();
        let csv_rel = format!("{map_key}.csv");
        let csv_path = path_under(&base, &csv_rel);
        write_placements_csv(&csv_path, &owned)?;
        summary.row_count += owned.len();
        summary
            .files
            .push((format!("entity_positions/{csv_rel}"), owned.len()));
    }

    Ok(summary)
}

fn path_under(base: &Path, rel_fwd_slashes: &str) -> PathBuf {
    let mut out = base.to_path_buf();
    for seg in rel_fwd_slashes.split('/') {
        if !seg.is_empty() {
            out.push(seg);
        }
    }
    out
}

/// Write machine-readable CSV (UTF-8 with BOM for Excel). Numeric XYZ columns for Prism.
pub fn write_placements_csv(path: &Path, rows: &[EbxPlacement]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let mut f = std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    // BOM
    f.write_all(&[0xEF, 0xBB, 0xBF])
        .map_err(|e| format!("bom: {e}"))?;
    writeln!(
        f,
        "map,name,class,fnv1a,x,y,z,right_x,right_y,right_z,up_x,up_y,up_z,forward_x,forward_y,forward_z,guid,path"
    )
    .map_err(|e| format!("header: {e}"))?;
    for r in rows {
        let map_key = map_key_from_ebx_path(&r.file_path);
        let (x, y, z) = match r.pos {
            Some(p) => (Some(p[0]), Some(p[1]), Some(p[2])),
            None => (None, None, None),
        };
        writeln!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_escape(&map_key),
            csv_escape(&r.name),
            csv_escape(&r.class_name),
            r.fnv1a,
            fmt_opt_f32(x),
            fmt_opt_f32(y),
            fmt_opt_f32(z),
            fmt_opt_vec3_comp(r.right, 0),
            fmt_opt_vec3_comp(r.right, 1),
            fmt_opt_vec3_comp(r.right, 2),
            fmt_opt_vec3_comp(r.up, 0),
            fmt_opt_vec3_comp(r.up, 1),
            fmt_opt_vec3_comp(r.up, 2),
            fmt_opt_vec3_comp(r.forward, 0),
            fmt_opt_vec3_comp(r.forward, 1),
            fmt_opt_vec3_comp(r.forward, 2),
            csv_escape(&r.guid),
            csv_escape(&r.file_path),
        )
        .map_err(|e| format!("row: {e}"))?;
    }
    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn fmt_opt_f32(v: Option<f32>) -> String {
    match v {
        Some(n) => format!("{n:.6}"),
        None => String::new(),
    }
}

fn fmt_opt_vec3_comp(v: Option<[f32; 3]>, i: usize) -> String {
    match v {
        Some(p) => format!("{:.6}", p[i]),
        None => String::new(),
    }
}

// ── walk helpers (mirror ebx_viewer6 extract_positions_from_dbx) ─────────────

use crate::core::frostex::ebx::position_tree::{Complex, DbxTree, Field, FieldValue};

fn collect_placements(
    dbx: &DbxTree,
    file_path: &str,
    table: Option<&EbxGuidTable>,
) -> Vec<EbxPlacement> {
    let mut out = Vec::new();
    for (guid, instance) in &dbx.instances {
        match harvest_instance(dbx, guid, instance, file_path, table) {
            Ok(row) => out.push(row),
            Err(_) => {
                out.push(EbxPlacement {
                    name: instance.name.clone(),
                    class_name: instance.name.clone(),
                    fnv1a: 0,
                    pos: None,
                    right: None,
                    up: None,
                    forward: None,
                    guid: guid.format(),
                    file_path: file_path.to_string(),
                });
            }
        }
    }
    out
}

fn harvest_instance(
    dbx: &DbxTree,
    guid: &EbxGuid,
    instance: &Complex,
    file_path: &str,
    table: Option<&EbxGuidTable>,
) -> Result<EbxPlacement, String> {
    let label = find_instance_label(dbx, &instance.fields, table, 0);
    let name = label.unwrap_or_else(|| {
        if !dbx.true_filename.is_empty() {
            dbx.true_filename.clone()
        } else {
            instance.name.clone()
        }
    });

    let transform = find_named_transform(&instance.fields, WORLD_TRANSFORM_NAMES, 0)
        .or_else(|| search_fields_for_transform(&instance.fields, 0));

    let (pos, right, up, forward) = match transform {
        Some(t) => (t.pos, t.right, t.up, t.forward),
        None => (None, None, None, None),
    };

    Ok(EbxPlacement {
        fnv1a: if name.is_empty() { 0 } else { fnv1a_32(&name) },
        name,
        class_name: instance.name.clone(),
        pos,
        right,
        up,
        forward,
        guid: guid.format(),
        file_path: file_path.to_string(),
    })
}

struct TransformParts {
    pos: Option<[f32; 3]>,
    right: Option<[f32; 3]>,
    up: Option<[f32; 3]>,
    forward: Option<[f32; 3]>,
}

fn collect_all_floats(obj: &Complex) -> Vec<f32> {
    let mut floats = Vec::new();
    for fl in &obj.fields {
        match &fl.value {
            FieldValue::F64(n) => floats.push(*n as f32),
            FieldValue::I64(n) => floats.push(*n as f32),
            FieldValue::U64(n) => floats.push(*n as f32),
            FieldValue::Complex(c) | FieldValue::Array(c) => {
                floats.extend(collect_all_floats(c));
            }
            _ => {}
        }
    }
    floats
}

fn find_vec3_field(cmplx: &Complex, name: &str) -> Option<[f32; 3]> {
    for field in &cmplx.fields {
        if field.name.eq_ignore_ascii_case(name) {
            if let FieldValue::Complex(inner) = &field.value {
                let floats = collect_all_floats(inner);
                if floats.len() >= 3 {
                    return Some([floats[0], floats[1], floats[2]]);
                }
            }
        }
    }
    None
}

fn find_transform_in_complex(cmplx: &Complex) -> Option<TransformParts> {
    let trans = find_vec3_field(cmplx, "trans");
    let right = find_vec3_field(cmplx, "right");
    let up = find_vec3_field(cmplx, "up");
    let forward = find_vec3_field(cmplx, "forward");
    if trans.is_some() || right.is_some() || up.is_some() || forward.is_some() {
        return Some(TransformParts {
            pos: trans,
            right,
            up,
            forward,
        });
    }

    let mut vals = [None, None, None];
    for f in &cmplx.fields {
        let n = f.name.to_ascii_lowercase();
        let num = match &f.value {
            FieldValue::F64(v) => Some(*v as f32),
            FieldValue::I64(v) => Some(*v as f32),
            FieldValue::U64(v) => Some(*v as f32),
            _ => None,
        };
        if let Some(v) = num {
            match n.as_str() {
                "x" => vals[0] = Some(v),
                "y" => vals[1] = Some(v),
                "z" => vals[2] = Some(v),
                _ => {}
            }
        }
    }
    if let (Some(x), Some(y), Some(z)) = (vals[0], vals[1], vals[2]) {
        return Some(TransformParts {
            pos: Some([x, y, z]),
            right: None,
            up: None,
            forward: None,
        });
    }

    let floats = collect_all_floats(cmplx);
    if floats.len() >= 12 {
        return Some(TransformParts {
            pos: Some([floats[9], floats[10], floats[11]]),
            right: Some([floats[0], floats[1], floats[2]]),
            up: Some([floats[3], floats[4], floats[5]]),
            forward: Some([floats[6], floats[7], floats[8]]),
        });
    }
    if floats.len() == 3 {
        return Some(TransformParts {
            pos: Some([floats[0], floats[1], floats[2]]),
            right: None,
            up: None,
            forward: None,
        });
    }
    None
}

fn is_zero_pos(p: &[f32; 3]) -> bool {
    p[0] == 0.0 && p[1] == 0.0 && p[2] == 0.0
}

fn search_fields_for_transform(fields: &[Field], depth: usize) -> Option<TransformParts> {
    if depth > 10 {
        return None;
    }
    for field in fields {
        let fname = field.name.to_ascii_lowercase();
        let is_spatial = fname.contains("transform")
            || fname.contains("position")
            || fname.contains("translation")
            || fname.contains("location");
        if is_spatial {
            if let FieldValue::Complex(c) = &field.value {
                if let Some(t) = find_transform_in_complex(c) {
                    if let Some(pos) = t.pos {
                        if !is_zero_pos(&pos) {
                            return Some(t);
                        }
                    }
                }
            }
        }
    }
    for field in fields {
        match &field.value {
            FieldValue::Complex(c) | FieldValue::Array(c) => {
                if let Some(t) = search_fields_for_transform(&c.fields, depth + 1) {
                    return Some(t);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_named_transform(
    fields: &[Field],
    target_names: &[&str],
    depth: usize,
) -> Option<TransformParts> {
    if depth > 12 {
        return None;
    }
    for field in fields {
        let fname = field.name.to_ascii_lowercase();
        if target_names.iter().any(|n| *n == fname) {
            if let FieldValue::Complex(c) = &field.value {
                if let Some(t) = find_transform_in_complex(c) {
                    return Some(t);
                }
            }
        }
    }
    for field in fields {
        match &field.value {
            FieldValue::Complex(c) | FieldValue::Array(c) => {
                if let Some(t) = find_named_transform(&c.fields, target_names, depth + 1) {
                    return Some(t);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_instance_label(
    dbx: &DbxTree,
    fields: &[Field],
    table: Option<&EbxGuidTable>,
    _depth: usize,
) -> Option<String> {
    let mut best = None;
    walk_label(dbx, fields, table, 0, &mut best);
    best.map(|(_, v)| v)
}

fn walk_label(
    dbx: &DbxTree,
    fields: &[Field],
    table: Option<&EbxGuidTable>,
    depth: usize,
    best: &mut Option<(usize, String)>,
) {
    if depth > 12 {
        return;
    }
    for field in fields {
        let fname = field.name.to_ascii_lowercase();
        if let Some(idx) = LABEL_PRIORITY.iter().position(|n| *n == fname) {
            if let Some(val) = field_as_label(dbx, field, table) {
                if best.as_ref().map(|(i, _)| idx < *i).unwrap_or(true) {
                    let cleaned = if fname == "blueprint" {
                        clean_blueprint_path(&val)
                    } else {
                        val
                    };
                    *best = Some((idx, cleaned));
                }
            }
        }
    }
    for field in fields {
        match &field.value {
            FieldValue::Complex(c) | FieldValue::Array(c) => {
                walk_label(dbx, &c.fields, table, depth + 1, best);
            }
            _ => {}
        }
    }
}

fn field_as_label(dbx: &DbxTree, field: &Field, table: Option<&EbxGuidTable>) -> Option<String> {
    match &field.value {
        FieldValue::Text(s) if s != "*nullString*" && s != "*nullRef*" && !s.is_empty() => {
            Some(s.clone())
        }
        FieldValue::ClassRef(v) => resolve_class_ref(dbx, *v, table),
        _ => None,
    }
}

fn resolve_class_ref(dbx: &DbxTree, v: u32, table: Option<&EbxGuidTable>) -> Option<String> {
    if (v >> 31) != 0 {
        let idx = (v & 0x7fff_ffff) as usize;
        let (file_g, inst_g) = dbx.external_guids.get(idx)?;
        if let Some(name) = table.and_then(|t| t.get(&file_g.format())) {
            return Some(format!("{}/{}", name, inst_g.format().replace('-', "")));
        }
        return Some(format!(
            "{}/{}",
            file_g.format(),
            inst_g.format().replace('-', "")
        ));
    }
    if v == 0 {
        return None;
    }
    let idx = (v as usize).saturating_sub(1);
    dbx.internal_guids.get(idx).map(|g| g.format())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_known() {
        // Same algorithm as Python ebx_viewer6 fnv1a_32
        assert_eq!(fnv1a_32(""), 2166136261);
        assert_ne!(fnv1a_32("UnitName_NSResourceCenter"), 0);
    }

    #[test]
    fn cleans_blueprint_guid_suffix() {
        let raw = "_Rts/Art/Neutral/NS_ResourceCenter/NS_ResourceCenter_New/0b9161734ea0519d7aa748e9fe0a5dff";
        assert_eq!(
            clean_blueprint_path(raw),
            "_Rts/Art/Neutral/NS_ResourceCenter/NS_ResourceCenter_New"
        );
        assert_eq!(clean_blueprint_path("no/guid/here"), "no/guid/here");
    }

    #[test]
    fn map_key_from_level_paths() {
        assert_eq!(
            map_key_from_ebx_path(
                "ebx/levels/mp/pve/firstplayable_mphorde_final/firstplayable_mphorde_final.txt"
            ),
            "levels/mp/pve/firstplayable_mphorde_final"
        );
        assert_eq!(
            map_key_from_ebx_path(
                "ebx/levels/mp/pve/firstplayable_mphorde_final/apa_classicgeneral.txt"
            ),
            "levels/mp/pve/firstplayable_mphorde_final"
        );
        assert_eq!(
            map_key_from_ebx_path("ebx/levels/sp/alpha_tutorial/alpha_tutorial.txt"),
            "levels/sp/alpha_tutorial"
        );
        assert_eq!(
            map_key_from_ebx_path("ebx/levels/frontendtest/frontendtest.txt"),
            "levels/frontendtest"
        );
        assert_eq!(
            map_key_from_ebx_path("_rts/art/neutral/ns_resourcecenter/foo.txt"),
            OTHER_MAP_KEY
        );
    }
}
