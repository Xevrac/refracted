//! Rip/export helpers: recover usable file extensions (and PNG for pictures).

use crate::core::frostex::archetype::Archetype;
use crate::core::frostex::dbobject::parse_db_bytes;
use crate::core::frostex::ebx::{
    dump_ebx_text_with_table, is_ebx, register_ebx_guid, EbxGuidTable,
};
use crate::core::frostex::ebx_positions::{
    extract_ebx_placements, filter_with_positions, write_placements_by_map, EbxPlacement,
};
use crate::core::frostex::index::{AssetKind, AssetRef, DataIndex, OpenProgress, TreeNodeKind};
use crate::core::frostex::meshset::{decode_meshset, looks_like_meshset};
use crate::core::frostex::preview::{looks_textual, safe_filename};
use crate::core::frostex::preview_ctx::PreviewCtx;
use crate::core::frostex::texture::{
    decode_frostbite_dx_texture, decode_standard_image, decode_texture,
    parse_dx_texture_streaming_guid,
};
use parking_lot::Mutex;
use std::collections::{BTreeMap, HashSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Soft ceiling for a single asset during full dump / rip.
pub const DUMP_SIZE_LIMIT: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct DumpOptions {
    pub toc: bool,
    pub sb: bool,
    pub ebx: bool,
    pub res: bool,
    pub chunk: bool,
    pub file: bool,
    /// After EBX dump, harvest BlueprintTransform placements → entity_positions/<map>.csv
    pub ebx_positions: bool,
}

impl Default for DumpOptions {
    fn default() -> Self {
        Self {
            toc: true,
            sb: true,
            ebx: true,
            res: true,
            chunk: true,
            file: true,
            ebx_positions: true,
        }
    }
}

impl DumpOptions {
    pub fn includes(&self, kind: AssetKind) -> bool {
        match kind {
            AssetKind::Toc => self.toc,
            AssetKind::Sb => self.sb,
            AssetKind::Ebx => self.ebx,
            AssetKind::Res => self.res,
            AssetKind::Chunk => self.chunk,
            AssetKind::File => self.file,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DumpEntry {
    pub asset_name: String,
    pub rel_path: String,
    pub kind: String,
    pub ok: bool,
    pub detail: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct DumpReport {
    pub out_dir: PathBuf,
    pub data_dir: PathBuf,
    pub total: usize,
    pub wrote: usize,
    pub failed: usize,
    pub skipped: usize,
    pub bytes_written: u64,
    pub by_kind: BTreeMap<String, usize>,
    pub entries: Vec<DumpEntry>,
    pub elapsed_secs: f32,
    pub report_path: Option<PathBuf>,
    pub error: Option<String>,
}

impl DumpReport {
    pub fn summary_line(&self) -> String {
        if let Some(err) = &self.error {
            return format!("Full dump failed: {err}");
        }
        format!(
            "Full dump: wrote {}/{} → {} ({} failed, {} skipped, {:.1}s)",
            self.wrote,
            self.total,
            self.out_dir.display(),
            self.failed,
            self.skipped,
            self.elapsed_secs
        )
    }

    pub fn write_report_file(&mut self) -> Result<PathBuf, String> {
        let path = self.out_dir.join("DUMP_REPORT.txt");
        let mut body = String::new();
        body.push_str("FrostEx full dump report\n");
        body.push_str("========================\n");
        body.push_str(&format!("Data:   {}\n", self.data_dir.display()));
        body.push_str(&format!("Output: {}\n", self.out_dir.display()));
        body.push_str(&format!(
            "Result: wrote {}  failed {}  skipped {}  total {}\n",
            self.wrote, self.failed, self.skipped, self.total
        ));
        body.push_str(&format!(
            "Bytes:  {}\n",
            format_bytes(self.bytes_written)
        ));
        body.push_str(&format!("Time:   {:.1}s\n", self.elapsed_secs));
        if !self.by_kind.is_empty() {
            body.push_str("\nWrote by kind:\n");
            for (k, n) in &self.by_kind {
                body.push_str(&format!("  {k}: {n}\n"));
            }
        }
        body.push_str("\n--- Files ---\n");
        for e in &self.entries {
            let status = if e.ok { "OK" } else { "FAIL" };
            body.push_str(&format!(
                "[{status}] {}  {}  {}  {}\n",
                e.kind,
                e.rel_path,
                if e.ok {
                    format_bytes(e.bytes)
                } else {
                    String::new()
                },
                e.detail
            ));
        }
        std::fs::write(&path, body).map_err(|e| format!("write report: {e}"))?;
        self.report_path = Some(path.clone());
        Ok(path)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DumpProgress {
    pub done: usize,
    pub total: usize,
    pub phase: String,
    pub wrote: usize,
    pub failed: usize,
    pub skipped: usize,
    pub finished: Option<DumpReport>,
}

/// Re-open Data, expand everything, dump under `{parent}/frostex/`, write report.
pub fn run_full_dump(
    data_dir: PathBuf,
    parent_dir: PathBuf,
    options: DumpOptions,
    progress: Arc<Mutex<DumpProgress>>,
) {
    let started = Instant::now();
    let out_dir = if parent_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.eq_ignore_ascii_case("frostex"))
        .unwrap_or(false)
    {
        parent_dir
    } else {
        parent_dir.join("frostex")
    };

    {
        let mut p = progress.lock();
        p.phase = "Creating frostex folder…".into();
        p.done = 0;
        p.total = 1;
    }

    if let Err(err) = std::fs::create_dir_all(&out_dir) {
        let report = DumpReport {
            out_dir: out_dir.clone(),
            data_dir: data_dir.clone(),
            error: Some(format!("mkdir {}: {err}", out_dir.display())),
            elapsed_secs: started.elapsed().as_secs_f32(),
            ..DumpReport::default()
        };
        progress.lock().finished = Some(report);
        return;
    }

    {
        let mut p = progress.lock();
        p.phase = "Opening data index…".into();
    }
    let open_progress = Arc::new(Mutex::new(OpenProgress {
        done: 0,
        total: 1,
        phase: "Opening…".into(),
    }));

    let mut index = match DataIndex::open(data_dir.clone(), open_progress) {
        Ok(idx) => idx,
        Err(err) => {
            progress.lock().finished = Some(DumpReport {
                out_dir,
                data_dir,
                error: Some(err),
                elapsed_secs: started.elapsed().as_secs_f32(),
                ..DumpReport::default()
            });
            return;
        }
    };

    {
        let mut p = progress.lock();
        p.phase = "Expanding packages…".into();
        p.done = 0;
        p.total = 1;
    }
    let mut expand_stack = vec!["root".to_string()];
    let mut expanded_seen = HashSet::new();
    let mut expanded_done = 0usize;
    while let Some(node_id) = expand_stack.pop() {
        if !expanded_seen.insert(node_id.clone()) {
            continue;
        }

        let Some((kind, loaded, label)) = index.find_node(&node_id).map(|n| {
            (
                n.kind.clone(),
                n.loaded,
                if n.label.is_empty() {
                    node_id.clone()
                } else {
                    n.label.clone()
                },
            )
        }) else {
            continue;
        };

        {
            let mut p = progress.lock();
            p.phase = format!("Expanding {label}…");
            p.done = expanded_done;
            p.total = expanded_done + expand_stack.len() + 1;
        }

        if !loaded && matches!(kind, TreeNodeKind::Toc | TreeNodeKind::Sb | TreeNodeKind::Bundle) {
            if let Err(err) = index.ensure_expanded(&node_id) {
                progress.lock().finished = Some(DumpReport {
                    out_dir,
                    data_dir,
                    error: Some(format!("expand {label}: {err}")),
                    elapsed_secs: started.elapsed().as_secs_f32(),
                    ..DumpReport::default()
                });
                return;
            }
        }

        if let Some(node) = index.find_node(&node_id) {
            for child in node.children.iter().rev() {
                let should_walk = matches!(
                    child.kind,
                    TreeNodeKind::Directory
                        | TreeNodeKind::Toc
                        | TreeNodeKind::Sb
                        | TreeNodeKind::Bundle
                ) || !child.loaded;
                if should_walk && !expanded_seen.contains(&child.id) {
                    expand_stack.push(child.id.clone());
                }
            }
        }

        expanded_done += 1;
        let mut p = progress.lock();
        p.done = expanded_done;
        p.total = expanded_done + expand_stack.len();
    }

    {
        let mut p = progress.lock();
        p.phase = "Collecting assets…".into();
    }

    let assets: Vec<_> = index
        .collect_rippable("root")
        .into_iter()
        .filter(|asset| options.includes(asset.kind.clone()))
        .collect();
    let total = assets.len();
    {
        let mut p = progress.lock();
        p.total = total.max(1);
        p.done = 0;
        p.phase = format!("Dumping {total} assets…");
    }

    let ctx = PreviewCtx::from_index(&index);
    drop(index);

    // Build EBX GUID→Name table so Class links resolve like IceBloc.
    let mut ebx_guid_table = EbxGuidTable::new();
    if options.ebx {
        let mut p = progress.lock();
        p.phase = "Indexing EBX GUIDs…".into();
        drop(p);
        for asset in assets.iter().filter(|a| a.kind == AssetKind::Ebx) {
            if asset.size_hint.unwrap_or(0) > DUMP_SIZE_LIMIT {
                continue;
            }
            if let Ok(bytes) = ctx.extract_bytes(asset) {
                register_ebx_guid(&bytes, &mut ebx_guid_table);
            }
        }
    }

    let mut report = DumpReport {
        out_dir: out_dir.clone(),
        data_dir: data_dir.clone(),
        total,
        ..DumpReport::default()
    };

    let mut placement_rows: Vec<EbxPlacement> = Vec::new();
    let harvest_positions = options.ebx_positions && options.ebx;

    for asset in assets {
        {
            let mut p = progress.lock();
            p.phase = asset.name.clone();
        }

        let kind = format!("{:?}", asset.kind);
        let rel = rip_relative_path(&asset);
        let dest = out_dir.join(&rel);

        let entry = if asset.size_hint.unwrap_or(0) > DUMP_SIZE_LIMIT {
            report.skipped += 1;
            DumpEntry {
                asset_name: asset.name.clone(),
                rel_path: rel,
                kind,
                ok: false,
                detail: format!("skipped: larger than {}", format_bytes(DUMP_SIZE_LIMIT)),
                bytes: 0,
            }
        } else {
            match prepare_rip_with_table(&ctx, &asset, &dest, Some(&ebx_guid_table)).and_then(
                |(bytes, final_path)| {
                    if harvest_positions && asset.kind == AssetKind::Ebx {
                        if let Ok(raw) = ctx.extract_bytes(&asset) {
                            let rel_ebx = final_path
                                .strip_prefix(&out_dir)
                                .map(|p| p.to_string_lossy().replace('\\', "/"))
                                .unwrap_or_else(|_| asset.name.clone());
                            if let Ok(rows) =
                                extract_ebx_placements(&raw, &rel_ebx, Some(&ebx_guid_table))
                            {
                                placement_rows.extend(rows);
                            }
                        }
                    }
                    if let Some(parent) = final_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
                    }
                    let n = bytes.len() as u64;
                    std::fs::write(&final_path, &bytes)
                        .map_err(|e| format!("write {}: {e}", final_path.display()))?;
                    let rel_out = final_path
                        .strip_prefix(&out_dir)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or_else(|_| final_path.display().to_string());
                    Ok((rel_out, n))
                },
            ) {
                Ok((rel_out, n)) => {
                    report.wrote += 1;
                    report.bytes_written += n;
                    *report.by_kind.entry(kind.clone()).or_default() += 1;
                    DumpEntry {
                        asset_name: asset.name.clone(),
                        rel_path: rel_out,
                        kind,
                        ok: true,
                        detail: "ok".into(),
                        bytes: n,
                    }
                }
                Err(err) => {
                    report.failed += 1;
                    DumpEntry {
                        asset_name: asset.name.clone(),
                        rel_path: rel,
                        kind,
                        ok: false,
                        detail: err,
                        bytes: 0,
                    }
                }
            }
        };

        report.entries.push(entry);
        {
            let mut p = progress.lock();
            p.done += 1;
            p.wrote = report.wrote;
            p.failed = report.failed;
            p.skipped = report.skipped;
        }
    }

    if harvest_positions {
        let with_pos = filter_with_positions(&placement_rows);
        {
            let mut p = progress.lock();
            p.phase = format!(
                "Writing entity_positions/ ({} with pos / {} instances)…",
                with_pos.len(),
                placement_rows.len()
            );
        }
        match write_placements_by_map(&out_dir, &with_pos) {
            Ok(summary) => {
                report.wrote += summary.files.len();
                *report.by_kind.entry("EbxPositions".into()).or_default() += summary.files.len();
                for (rel, n) in &summary.files {
                    let bytes = out_dir
                        .join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
                        .metadata()
                        .map(|m| m.len())
                        .unwrap_or(0);
                    report.entries.push(DumpEntry {
                        asset_name: rel.clone(),
                        rel_path: rel.clone(),
                        kind: "EbxPositions".into(),
                        ok: true,
                        detail: format!("{n} placements"),
                        bytes,
                    });
                }
                report.entries.push(DumpEntry {
                    asset_name: "entity_positions/".into(),
                    rel_path: "entity_positions/".into(),
                    kind: "EbxPositions".into(),
                    ok: true,
                    detail: format!(
                        "{} maps, {} placements with world pos (of {} instances)",
                        summary.map_count,
                        summary.row_count,
                        placement_rows.len()
                    ),
                    bytes: 0,
                });
            }
            Err(err) => {
                report.failed += 1;
                report.entries.push(DumpEntry {
                    asset_name: "entity_positions/".into(),
                    rel_path: "entity_positions/".into(),
                    kind: "EbxPositions".into(),
                    ok: false,
                    detail: err,
                    bytes: 0,
                });
            }
        }
    } else if options.ebx_positions && !options.ebx {
        report.entries.push(DumpEntry {
            asset_name: "entity_positions/".into(),
            rel_path: "entity_positions/".into(),
            kind: "EbxPositions".into(),
            ok: false,
            detail: "skipped: enable EBX — positions CSV is harvested from EBX, not TOC".into(),
            bytes: 0,
        });
        report.skipped += 1;
    }

    report.elapsed_secs = started.elapsed().as_secs_f32();
    if let Err(err) = report.write_report_file() {
        report.error = Some(err);
    }

    progress.lock().finished = Some(report);
}

fn format_bytes(n: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let f = n as f64;
    if f >= GB {
        format!("{:.2} GB", f / GB)
    } else if f >= MB {
        format!("{:.2} MB", f / MB)
    } else if f >= KB {
        format!("{:.1} KB", f / KB)
    } else {
        format!("{n} B")
    }
}

/// Build a relative output path under ebx/res/chunks/… with a sensible extension.
pub fn rip_relative_path(asset: &AssetRef) -> String {
    let folder = match asset.kind {
        AssetKind::Ebx => "ebx",
        AssetKind::Res => "res",
        AssetKind::Chunk => "chunks",
        AssetKind::Toc => "toc",
        AssetKind::Sb => "sb",
        AssetKind::File => "files",
    };

    if matches!(asset.kind, AssetKind::Toc | AssetKind::Sb | AssetKind::File) {
        if let Some(path) = &asset.path {
            if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                return format!("{folder}/{}", safe_filename(fname));
            }
        }
    }

    let stem = path_stem(asset);
    let ext = extension_hint(asset);
    format!("{folder}/{stem}.{ext}")
}

/// Extract bytes and optionally transcode pictures to PNG; return (bytes, final path).
pub fn prepare_rip(
    ctx: &PreviewCtx,
    asset: &AssetRef,
    dest: &Path,
) -> Result<(Vec<u8>, PathBuf), String> {
    let table = if ctx.ebx_guid_table.is_empty() {
        None
    } else {
        Some(&ctx.ebx_guid_table)
    };
    prepare_rip_with_table(ctx, asset, dest, table)
}

pub fn prepare_rip_with_table(
    ctx: &PreviewCtx,
    asset: &AssetRef,
    dest: &Path,
    ebx_table: Option<&EbxGuidTable>,
) -> Result<(Vec<u8>, PathBuf), String> {
    let bytes = ctx.extract_bytes(asset)?;

    // Full IceBloc-style EBX instance dump (not string-table stubs).
    if is_ebx(&bytes) {
        let text = dump_ebx_text_with_table(&bytes, ebx_table);
        return Ok((text.into_bytes(), with_extension(dest, "txt")));
    }
    if matches!(asset.kind, AssetKind::Toc | AssetKind::Sb) {
        if let Ok(obj) = parse_db_bytes(&bytes) {
            let text = obj.to_pretty(0);
            return Ok((text.into_bytes(), with_extension(dest, "txt")));
        }
    }

    // Prefer a real PNG when we can decode a picture.
    if let Some((png, label)) = try_rip_as_png(ctx, asset, &bytes) {
        let path = with_extension(dest, "png");
        let _ = label;
        return Ok((png, path));
    }

    // MeshSet RES → OBJ (+ sidecar SMD), IceBloc-compatible.
    if looks_like_meshset(&bytes) {
        let mesh = decode_meshset(&bytes, &|guid| ctx.get_chunk_bytes(guid))?;
        let smd_path = with_extension(dest, "smd");
        if let Some(parent) = smd_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
        std::fs::write(&smd_path, mesh.to_smd())
            .map_err(|e| format!("write {}: {e}", smd_path.display()))?;
        return Ok((mesh.to_obj().into_bytes(), with_extension(dest, "obj")));
    }

    let ext = sniff_extension(asset, &bytes);
    Ok((bytes, with_extension(dest, ext)))
}

fn try_rip_as_png(
    ctx: &PreviewCtx,
    asset: &AssetRef,
    bytes: &[u8],
) -> Option<(Vec<u8>, String)> {
    let arch = Archetype::from_asset(asset);
    let want = arch == Archetype::Picture
        || matches!(asset.kind, AssetKind::Res | AssetKind::Chunk)
        || looks_image_name(&asset.name);

    if !want {
        return None;
    }

    // Already a standard image container — re-encode to PNG for consistency,
    // or keep original if already PNG.
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        return Some((bytes.to_vec(), "PNG".into()));
    }
    if let Ok(tex) = decode_standard_image(bytes) {
        return encode_png(&tex.rgba, tex.width, tex.height).ok().map(|b| (b, tex.format_label));
    }
    if let Some(guid) = parse_dx_texture_streaming_guid(bytes) {
        if let Ok(chunk) = ctx.get_chunk_bytes(&guid).or_else(|_| {
            asset
                .chunk_guid
                .map(|g| ctx.get_chunk_bytes(&g))
                .unwrap_or_else(|| Err("no chunk".into()))
        }) {
            if let Ok(tex) = decode_frostbite_dx_texture(bytes, &chunk) {
                return encode_png(&tex.rgba, tex.width, tex.height)
                    .ok()
                    .map(|b| (b, tex.format_label));
            }
        }
    }
    if let Ok(tex) = decode_texture(bytes) {
        return encode_png(&tex.rgba, tex.width, tex.height)
            .ok()
            .map(|b| (b, tex.format_label));
    }
    None
}

fn encode_png(rgba: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let img = image::RgbaImage::from_raw(width as u32, height as u32, rgba.to_vec())
        .ok_or_else(|| "rgba buffer size mismatch".to_string())?;
    let mut cursor = Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("png encode: {e}"))?;
    Ok(cursor.into_inner())
}

fn sniff_extension(asset: &AssetRef, bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 {
        if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
            return "png";
        }
        if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xD8, 0xFF] {
            return "jpg";
        }
        if bytes.starts_with(b"DDS ") {
            return "dds";
        }
        if bytes.starts_with(b"GIF8") {
            return "gif";
        }
        if bytes.starts_with(b"BM") {
            return "bmp";
        }
        if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
            return "webp";
        }
        if bytes.starts_with(b"OggS") {
            return "ogg";
        }
        if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WAVE" {
            return "wav";
        }
        if is_ebx(bytes) {
            return "ebx";
        }
    }

    // Plain text / configs / markup.
    if let Some(ext) = sniff_text_extension(asset, bytes) {
        return ext;
    }

    extension_hint(asset)
}

fn sniff_text_extension(asset: &AssetRef, bytes: &[u8]) -> Option<&'static str> {
    let name = asset.name.to_ascii_lowercase();
    // Name already implies a text-ish type.
    if name.ends_with(".txt")
        || name.ends_with(".cfg")
        || name.ends_with(".ini")
        || name.ends_with(".xml")
        || name.ends_with(".json")
        || name.ends_with(".csv")
        || name.ends_with(".log")
        || name.ends_with(".lua")
        || name.ends_with(".js")
        || name.ends_with(".html")
        || name.ends_with(".htm")
    {
        if let Some(ext) = Path::new(&name).extension().and_then(|e| e.to_str()) {
            return Some(match ext {
                "txt" => "txt",
                "cfg" => "cfg",
                "ini" => "ini",
                "xml" => "xml",
                "json" => "json",
                "csv" => "csv",
                "log" => "log",
                "lua" => "lua",
                "js" => "js",
                "html" | "htm" => "html",
                _ => "txt",
            });
        }
    }

    if !looks_textual(bytes) {
        return None;
    }

    let sample = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
    let trimmed = sample.trim_start();
    if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
        return Some("xml");
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Some("json");
    }
    if name.contains("config") || name.contains("settings") || name.ends_with("cfg") {
        return Some("cfg");
    }
    if name.contains("ini") {
        return Some("ini");
    }
    Some("txt")
}

fn extension_hint(asset: &AssetRef) -> &'static str {
    match asset.kind {
        AssetKind::Ebx => "ebx",
        AssetKind::Toc => "toc",
        AssetKind::Sb => "sb",
        AssetKind::Chunk => "chunk",
        AssetKind::File => {
            let n = asset.name.to_ascii_lowercase();
            if let Some(ext) = Path::new(&n).extension().and_then(|e| e.to_str()) {
                return match ext {
                    "toc" => "toc",
                    "sb" => "sb",
                    "cas" => "cas",
                    "cat" => "cat",
                    "dds" => "dds",
                    "png" => "png",
                    "txt" => "txt",
                    "cfg" => "cfg",
                    "ini" => "ini",
                    "xml" => "xml",
                    "json" => "json",
                    _ => "bin",
                };
            }
            "bin"
        }
        AssetKind::Res => res_type_extension(asset.res_type, &asset.name),
    }
}

fn res_type_extension(res_type: Option<u32>, name: &str) -> &'static str {
    if let Some(t) = res_type {
        match t {
            // textures
            0x5C4954A6 | 0xBCC7FB86 | 0x6BDE20BA | 0x957C32B1 | 0xC417BBD3 | 0x31E779A2
            | 0x41D57E10 | 0x2FF88D9E | 0x93BAA23F | 0x921476CA | 0xACD91FE8 => {
                return "dds";
            }
            // meshes
            0x49B156D4 | 0xBA02FEE0 | 0xC22CF759 | 0x3264E585 | 0x30B4A553 => {
                return "mesh";
            }
            // audio
            0xB2C465F6 | 0xC78B9D9D => return "wave",
            _ => {}
        }
    }
    let n = name.to_ascii_lowercase();
    if looks_image_name(&n) {
        "dds"
    } else if n.contains("mesh") || n.contains("model") {
        "mesh"
    } else if n.contains("sound") || n.contains("wave") || n.contains("audio") {
        "wave"
    } else {
        "res"
    }
}

fn path_stem(asset: &AssetRef) -> String {
    let name = asset.name.replace('\\', "/");
    let name = name.trim_start_matches('/');
    let safe: String = name
        .split('/')
        .map(|part| {
            let s = safe_filename(part);
            if s.is_empty() {
                "unnamed".into()
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    if safe.is_empty() {
        safe_filename(&asset.id)
    } else {
        // Strip a trailing extension so we can re-apply the recovered one.
        let p = Path::new(&safe);
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            if let Some(parent) = p.parent() {
                if parent.as_os_str().is_empty() {
                    stem.to_string()
                } else {
                    parent.join(stem).to_string_lossy().replace('\\', "/")
                }
            } else {
                stem.to_string()
            }
        } else {
            safe
        }
    }
}

fn with_extension(path: &Path, ext: &str) -> PathBuf {
    let ext = ext.trim_start_matches('.');
    let mut out = path.to_path_buf();
    // If the chosen path already ends with this extension, keep it.
    if out
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
    {
        return out;
    }
    // Replace bogus/missing extension.
    out.set_extension(ext);
    out
}

fn looks_image_name(n: &str) -> bool {
    let n = n.to_ascii_lowercase();
    n.contains("texture")
        || n.contains("diffuse")
        || n.contains("normal")
        || n.contains("specular")
        || n.contains("_di")
        || n.contains("_nm")
        || n.contains("_sp")
        || n.contains("lut")
}
