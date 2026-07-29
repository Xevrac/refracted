//! Background-safe preview builders for FrostEx assets.

use crate::core::frostex::dbobject::parse_db_bytes;
use crate::core::frostex::ebx::{dump_ebx_text_with_table, is_ebx, summarize_ebx, EbxGuidTable};
use crate::core::frostex::index::{AssetKind, AssetRef};
use crate::core::frostex::meshset::{decode_meshset, looks_like_meshset};
use crate::core::frostex::preview_ctx::PreviewCtx;
use egui::ColorImage;

const PEEK_LIMIT: usize = 512 * 1024;
const HEAVY_LIMIT: u64 = 64 * 1024 * 1024;
const HEX_LIMIT: usize = 16 * 1024;

#[derive(Debug, Clone, Default)]
pub struct PreviewState {
    pub asset_id: Option<String>,
    pub title: String,
    pub info: String,
    /// Readable / structured text (DbObject dump, EBX names, UTF-8, …).
    pub text: String,
    /// Raw hex dump — always separate from Text.
    pub hex: String,
    pub image: Option<ColorImage>,
    pub image_label: String,
    pub truncated: bool,
    pub error: Option<String>,
    pub loading: bool,
}

impl PreviewState {
    pub fn build_info_only(asset: &AssetRef, message: impl Into<String>) -> Self {
        Self {
            asset_id: Some(asset.id.clone()),
            title: asset.name.clone(),
            info: describe_asset(asset),
            text: message.into(),
            hex: String::new(),
            image: None,
            image_label: String::new(),
            truncated: false,
            error: None,
            loading: false,
        }
    }

    pub fn build_preview(ctx: &PreviewCtx, asset: &AssetRef) -> Self {
        match build_preview_inner(ctx, asset) {
            Ok(mut state) => {
                state.asset_id = Some(asset.id.clone());
                state.title = asset.name.clone();
                state
            }
            Err(err) => Self {
                asset_id: Some(asset.id.clone()),
                title: asset.name.clone(),
                info: describe_asset(asset),
                text: String::new(),
                hex: String::new(),
                image: None,
                image_label: String::new(),
                truncated: false,
                error: Some(err),
                loading: false,
            },
        }
    }
}

pub fn needs_heavy_preview(asset: &AssetRef) -> bool {
    matches!(
        asset.kind,
        AssetKind::Ebx | AssetKind::Res | AssetKind::Chunk | AssetKind::File | AssetKind::Toc | AssetKind::Sb
    )
}

pub fn format_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = size as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{size} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn safe_filename(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

pub fn hex_dump(data: &[u8]) -> String {
    let mut out = String::new();
    for (row, chunk) in data.chunks(16).enumerate() {
        out.push_str(&format!("{:08X}  ", row * 16));
        for i in 0..16 {
            if let Some(b) = chunk.get(i) {
                out.push_str(&format!("{b:02X} "));
            } else {
                out.push_str("   ");
            }
            if i == 7 {
                out.push(' ');
            }
        }
        out.push(' ');
        for b in chunk {
            let c = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            out.push(c);
        }
        out.push('\n');
    }
    out
}

fn ebx_dump(ctx: &PreviewCtx, bytes: &[u8]) -> String {
    let table: Option<&EbxGuidTable> = if ctx.ebx_guid_table.is_empty() {
        None
    } else {
        Some(&ctx.ebx_guid_table)
    };
    dump_ebx_text_with_table(bytes, table)
}

fn build_preview_inner(ctx: &PreviewCtx, asset: &AssetRef) -> Result<PreviewState, String> {
    use crate::core::frostex::archetype::Archetype;
    use crate::core::frostex::texture::{decode_texture, point_cloud_preview, sniff_mesh_positions};

    let arch = Archetype::from_asset(asset);
    let info = format!("{}\nArchetype: {}", describe_asset(asset), arch.as_str());

    if matches!(asset.kind, AssetKind::Toc | AssetKind::Sb) {
        let bytes = ctx.extract_bytes(asset)?;
        let hex = hex_dump(&bytes[..bytes.len().min(HEX_LIMIT)]);
        let text = match parse_db_bytes(&bytes) {
            Ok(obj) => obj.to_pretty(0),
            Err(err) => {
                format!("DbObject parse failed: {err}\n\n(See Hex tab for raw bytes.)")
            }
        };
        return Ok(PreviewState {
            asset_id: Some(asset.id.clone()),
            title: asset.name.clone(),
            info,
            text,
            hex,
            image: None,
            image_label: String::new(),
            truncated: false,
            error: None,
            loading: false,
        });
    }

    if let Some(size) = asset.size_hint {
        if size > HEAVY_LIMIT && asset.sha1.is_some() && arch != Archetype::Picture {
            let (bytes, truncated) = ctx.peek_bytes(asset, PEEK_LIMIT)?;
            return Ok(PreviewState {
                asset_id: Some(asset.id.clone()),
                title: asset.name.clone(),
                info: format!("{info}\nLarge CAS payload: peek-only"),
                text: format!(
                    "Binary payload ({}). Open Hex for a raw dump.",
                    format_size(size)
                ),
                hex: hex_dump(&bytes[..bytes.len().min(HEX_LIMIT)]),
                image: None,
                image_label: String::new(),
                truncated,
                error: None,
                loading: false,
            });
        }
    }

    let (bytes, truncated) = if asset.sha1.is_some() && asset.size_hint.unwrap_or(0) > HEAVY_LIMIT {
        ctx.peek_bytes(asset, PEEK_LIMIT)?
    } else {
        (ctx.extract_bytes(asset)?, false)
    };
    let hex = hex_dump(&bytes[..bytes.len().min(HEX_LIMIT)]);

    // Standard images / DDS / DxTexture for Picture, Res, Chunk, or File.
    if matches!(
        arch,
        Archetype::Picture | Archetype::Data | Archetype::Model
    ) || matches!(
        asset.kind,
        AssetKind::Res | AssetKind::Chunk | AssetKind::File
    ) {
        if let Ok(tex) = try_picture_preview(ctx, asset, &bytes) {
            let image = ColorImage::from_rgba_unmultiplied([tex.width, tex.height], &tex.rgba);
            return Ok(PreviewState {
                asset_id: Some(asset.id.clone()),
                title: asset.name.clone(),
                info: format!(
                    "{info}\nTexture: {}x{} {}",
                    tex.width, tex.height, tex.format_label
                ),
                text: format!(
                    "Picture {}x{} ({})",
                    tex.width, tex.height, tex.format_label
                ),
                hex: hex.clone(),
                image: Some(image),
                image_label: format!("{}x{} {}", tex.width, tex.height, tex.format_label),
                truncated,
                error: None,
                loading: false,
            });
        }
        if let Ok(tex) = decode_texture(&bytes) {
            let image = ColorImage::from_rgba_unmultiplied([tex.width, tex.height], &tex.rgba);
            return Ok(PreviewState {
                asset_id: Some(asset.id.clone()),
                title: asset.name.clone(),
                info: format!(
                    "{info}\nImage: {}x{} {}",
                    tex.width, tex.height, tex.format_label
                ),
                text: format!(
                    "Image {}x{} ({})",
                    tex.width, tex.height, tex.format_label
                ),
                hex: hex.clone(),
                image: Some(image),
                image_label: format!("{}x{} {}", tex.width, tex.height, tex.format_label),
                truncated,
                error: None,
                loading: false,
            });
        }
        if arch == Archetype::Picture {
            return Ok(PreviewState {
                asset_id: Some(asset.id.clone()),
                title: asset.name.clone(),
                info,
                text: "Picture archetype, but decode failed. See Hex for raw bytes.".into(),
                hex,
                image: None,
                image_label: String::new(),
                truncated,
                error: Some(
                    "Could not decode picture (missing streaming chunk or unknown format)".into(),
                ),
                loading: false,
            });
        }
    }

    // Models: MeshSet RES / mesh-named payloads. EBX mesh entries are descriptors only.
    if matches!(asset.kind, AssetKind::Ebx)
        && (asset.name.to_ascii_lowercase().contains("mesh")
            || asset.name.to_ascii_lowercase().contains("model"))
    {
        return Ok(PreviewState {
            asset_id: Some(asset.id.clone()),
            title: asset.name.clone(),
            info: format!("{info}\n{}", summarize_ebx(&bytes)),
            text: format!(
                "{}\n\nThis EBX is a mesh/object descriptor, not vertex geometry.\nOpen the matching MeshSet under RES (same path, often ending in _mesh) for a Visual preview.",
                ebx_dump(ctx, &bytes)
            ),
            hex,
            image: None,
            image_label: String::new(),
            truncated,
            error: None,
            loading: false,
        });
    }

    if arch == Archetype::Model || looks_like_meshset(&bytes) {
        match decode_meshset(&bytes, &|guid| ctx.get_chunk_bytes(guid)) {
            Ok(mesh) => {
                let points = mesh.positions();
                let image = point_cloud_preview(&points, 512);
                let summary = format!(
                    "MeshSet '{}'\nSubsets: {}\nVertices: {}\nFaces: {}\nRip exports OBJ + SMD (IceBloc layout).",
                    mesh.name,
                    mesh.surfaces.len(),
                    mesh.total_vertices(),
                    mesh.total_faces()
                );
                return Ok(PreviewState {
                    asset_id: Some(asset.id.clone()),
                    title: asset.name.clone(),
                    info: format!("{info}\n{}", summary.lines().next().unwrap_or("MeshSet")),
                    text: summary,
                    hex: hex.clone(),
                    image,
                    image_label: if points.is_empty() {
                        String::new()
                    } else {
                        format!("MeshSet ({} verts)", points.len())
                    },
                    truncated,
                    error: None,
                    loading: false,
                });
            }
            Err(err) => {
                let points = sniff_mesh_positions(&bytes);
                if points.len() >= 32 {
                    if let Some(image) = point_cloud_preview(&points, 512) {
                        return Ok(PreviewState {
                            asset_id: Some(asset.id.clone()),
                            title: asset.name.clone(),
                            info: format!(
                                "{info}\nMeshSet decode failed; sniff fallback: {} verts",
                                points.len()
                            ),
                            text: format!(
                                "MeshSet decode failed: {err}\n\nFallback point-cloud from {} sniffed vertices.",
                                points.len()
                            ),
                            hex,
                            image: Some(image),
                            image_label: format!("Point cloud ({} verts)", points.len()),
                            truncated,
                            error: None,
                            loading: false,
                        });
                    }
                }
                return Ok(PreviewState {
                    asset_id: Some(asset.id.clone()),
                    title: asset.name.clone(),
                    info,
                    text: format!("MeshSet decode failed: {err}\nSee Hex for raw bytes."),
                    hex,
                    image: None,
                    image_label: String::new(),
                    truncated,
                    error: Some(err),
                    loading: false,
                });
            }
        }
    }

    if matches!(asset.kind, AssetKind::Res | AssetKind::Chunk) && !looks_like_meshset(&bytes) {
        let points = sniff_mesh_positions(&bytes);
        if points.len() >= 32 {
            if let Some(image) = point_cloud_preview(&points, 512) {
                return Ok(PreviewState {
                    asset_id: Some(asset.id.clone()),
                    title: asset.name.clone(),
                    info: format!(
                        "{info}\nGeometry sniff: {} vertices (ortho projection)",
                        points.len()
                    ),
                    text: format!(
                        "Point-cloud preview from {} sniffed vertices (not a MeshSet header).",
                        points.len()
                    ),
                    hex,
                    image: Some(image),
                    image_label: format!("Point cloud ({} verts)", points.len()),
                    truncated,
                    error: None,
                    loading: false,
                });
            }
        }
    }

    let text = if is_ebx(&bytes) {
        ebx_dump(ctx, &bytes)
    } else if looks_textual(&bytes) {
        String::from_utf8_lossy(&bytes).chars().take(64_000).collect()
    } else {
        format!(
            "Binary payload ({} bytes). Use the Hex tab for a raw dump.",
            bytes.len()
        )
    };

    Ok(PreviewState {
        asset_id: Some(asset.id.clone()),
        title: asset.name.clone(),
        info: if truncated {
            format!("{info}\nPreview is truncated")
        } else if is_ebx(&bytes) {
            format!("{info}\n{}", summarize_ebx(&bytes))
        } else {
            info
        },
        text,
        hex,
        image: None,
        image_label: String::new(),
        truncated,
        error: None,
        loading: false,
    })
}

fn try_picture_preview(
    ctx: &PreviewCtx,
    asset: &AssetRef,
    res_bytes: &[u8],
) -> Result<crate::core::frostex::texture::DecodedTexture, String> {
    use crate::core::frostex::texture::{decode_frostbite_dx_texture, parse_dx_texture_streaming_guid};

    if let Some(guid) = parse_dx_texture_streaming_guid(res_bytes) {
        let chunk = ctx.get_chunk_bytes(&guid).or_else(|_| {
            // Fallback: sometimes chunk guid is already on the asset.
            if let Some(g) = asset.chunk_guid {
                ctx.get_chunk_bytes(&g)
            } else {
                Err("streaming chunk not indexed".into())
            }
        })?;
        return decode_frostbite_dx_texture(res_bytes, &chunk);
    }
    Err("no streaming guid in RES header".into())
}

fn describe_asset(asset: &AssetRef) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Kind: {:?}", asset.kind));
    if let Some(path) = &asset.path {
        lines.push(format!("Path: {}", path.display()));
    }
    if let Some(sha1) = asset.sha1 {
        lines.push(format!("SHA1: {}", hex::encode(sha1)));
    }
    if let Some(guid) = asset.chunk_guid {
        lines.push(format!("Chunk GUID: {}", hex::encode(guid)));
    }
    if let Some(rt) = asset.res_type {
        lines.push(format!("ResType: 0x{rt:08X}"));
    }
    if let Some(size) = asset.size_hint {
        lines.push(format!("Size: {}", format_size(size)));
    }
    lines.join("\n")
}

pub fn looks_textual(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    let sample = &data[..data.len().min(4096)];
    let printable = sample
        .iter()
        .filter(|b| b.is_ascii_graphic() || b.is_ascii_whitespace())
        .count();
    printable * 100 / sample.len() > 85
}
