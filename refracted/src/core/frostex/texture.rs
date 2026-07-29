//! Texture and light mesh preview helpers.

use egui::{Color32, ColorImage};

#[derive(Debug, Clone)]
pub struct DxTextureHeader {
    pub width: usize,
    pub height: usize,
    pub mip_count: usize,
    pub format: TextureFormat,
    pub data_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Dxt1,
    Dxt3,
    Dxt5,
    Bgra8,
    Bgr8,
    L8,
    Rgb565,
    Unknown(u32),
}

#[derive(Debug, Clone)]
pub struct DecodedTexture {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
    pub format_label: String,
}

pub fn parse_dx_texture_header(data: &[u8]) -> Option<DxTextureHeader> {
    if data.len() >= 128 && &data[0..4] == b"DDS " {
        let height = le_u32(data, 12)? as usize;
        let width = le_u32(data, 16)? as usize;
        let mip_count = le_u32(data, 28).unwrap_or(1).max(1) as usize;
        let fourcc = le_u32(data, 84).unwrap_or(0);
        let rgb_bits = le_u32(data, 88).unwrap_or(0);
        let r_mask = le_u32(data, 92).unwrap_or(0);
        let g_mask = le_u32(data, 96).unwrap_or(0);
        let b_mask = le_u32(data, 100).unwrap_or(0);
        let a_mask = le_u32(data, 104).unwrap_or(0);
        let format = match fourcc {
            0x3154_5844 => TextureFormat::Dxt1,
            0x3354_5844 => TextureFormat::Dxt3,
            0x3554_5844 => TextureFormat::Dxt5,
            0 => match (rgb_bits, r_mask, g_mask, b_mask, a_mask) {
                (32, 0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0xFF00_0000) => TextureFormat::Bgra8,
                (24, 0x00FF_0000, 0x0000_FF00, 0x0000_00FF, _) => TextureFormat::Bgr8,
                (8, 0x0000_00FF, _, _, _) => TextureFormat::L8,
                (16, 0x0000_F800, 0x0000_07E0, 0x0000_001F, _) => TextureFormat::Rgb565,
                _ => TextureFormat::Unknown(fourcc),
            },
            other => TextureFormat::Unknown(other),
        };
        return Some(DxTextureHeader {
            width,
            height,
            mip_count,
            format,
            data_offset: 128,
        });
    }

    // Some Frostbite texture payloads carry a compact little-endian width/height block before DDS data.
    if data.len() >= 16 {
        let width = le_u16(data, 0)? as usize;
        let height = le_u16(data, 2)? as usize;
        let fmt = le_u32(data, 4)?;
        if (1..=16384).contains(&width) && (1..=16384).contains(&height) {
            let format = match fmt {
                0x3154_5844 | 71 => TextureFormat::Dxt1,
                0x3354_5844 | 74 => TextureFormat::Dxt3,
                0x3554_5844 | 77 => TextureFormat::Dxt5,
                28 | 87 => TextureFormat::Bgra8,
                61 => TextureFormat::L8,
                85 => TextureFormat::Bgr8,
                86 => TextureFormat::Rgb565,
                other => TextureFormat::Unknown(other),
            };
            return Some(DxTextureHeader {
                width,
                height,
                mip_count: le_u16(data, 8).unwrap_or(1).max(1) as usize,
                format,
                data_offset: 16,
            });
        }
    }

    None
}

pub fn decode_texture(data: &[u8]) -> Result<DecodedTexture, String> {
    // Standard image containers first (PNG/JPEG/GIF/WEBP/BMP/TGA…).
    if let Ok(tex) = decode_standard_image(data) {
        return Ok(tex);
    }
    let header = parse_dx_texture_header(data).ok_or_else(|| "texture header not recognized".to_string())?;
    if header.width == 0 || header.height == 0 {
        return Err("texture has zero width or height".into());
    }
    let payload = data
        .get(header.data_offset..)
        .ok_or_else(|| "texture payload missing".to_string())?;
    let rgba = decode_payload(header.format, payload, header.width, header.height)?;
    Ok(DecodedTexture {
        width: header.width,
        height: header.height,
        rgba,
        format_label: format!("{:?}, {} mip(s)", header.format, header.mip_count),
    })
}

pub fn decode_standard_image(data: &[u8]) -> Result<DecodedTexture, String> {
    let img = image::load_from_memory(data).map_err(|e| format!("image decode: {e}"))?;
    let rgba = img.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;
    let fmt = if data.starts_with(&[0x89, b'P', b'N', b'G']) {
        "PNG"
    } else if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
        "JPEG"
    } else if data.starts_with(b"GIF") {
        "GIF"
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        "WEBP"
    } else if data.starts_with(b"BM") {
        "BMP"
    } else {
        "image"
    };
    Ok(DecodedTexture {
        width,
        height,
        rgba: rgba.into_raw(),
        format_label: format!("{fmt} {width}x{height}"),
    })
}

/// IceBloc FB2 DxTexture: version/type/format/flags + dims, guid @ +28.
pub fn parse_dx_texture_streaming_guid(res: &[u8]) -> Option<[u8; 16]> {
    if res.len() < 44 {
        return None;
    }
    let mut guid = [0u8; 16];
    // Both FB2 and FB2013 layouts place the streaming chunk GUID around offset 28.
    guid.copy_from_slice(&res[28..44]);
    // Reject all-zero / clearly invalid.
    if guid.iter().all(|b| *b == 0) {
        return None;
    }
    Some(guid)
}

pub fn decode_frostbite_dx_texture(
    res_header: &[u8],
    chunk_payload: &[u8],
) -> Result<DecodedTexture, String> {
    if res_header.len() < 28 {
        return Err("DxTexture RES too short".into());
    }

    // FB2: version u32, type i32, format u32, flags u32, w/h/d/slices u16...
    let version = le_u32(res_header, 0).unwrap_or(0);
    let (fmt_code, width, height, mips) = if version > 0 && version < 64 {
        let fmt = le_u32(res_header, 8).unwrap_or(0);
        let w = le_u16(res_header, 16).unwrap_or(0) as usize;
        let h = le_u16(res_header, 18).unwrap_or(0) as usize;
        let m = res_header.get(26).copied().unwrap_or(1).max(1) as usize;
        (fmt, w, h, m)
    } else {
        // FB2013-ish: mip offsets, type, format u16...
        let fmt = le_u16(res_header, 12).unwrap_or(0) as u32;
        let w = le_u16(res_header, 18).unwrap_or(0) as usize;
        let h = le_u16(res_header, 20).unwrap_or(0) as usize;
        let m = res_header.get(26).copied().unwrap_or(1).max(1) as usize;
        (fmt, w, h, m)
    };

    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err(format!("implausible DxTexture size {width}x{height}"));
    }

    let format = match fmt_code {
        0x0 | 0x12 => TextureFormat::Dxt1,
        0x1 => TextureFormat::Dxt3,
        0x2 | 0x3 | 0x13 => TextureFormat::Dxt5,
        0x6 => TextureFormat::Bgr8,
        0x9 | 0x1C => TextureFormat::Bgra8,
        0xA => TextureFormat::L8,
        0x5 => TextureFormat::Rgb565,
        other => TextureFormat::Unknown(other),
    };

    // Chunk may still be zlib-framed CAS payload.
    let payload = match crate::core::frostex::catalog::try_fb_decompress(chunk_payload) {
        Ok(d) if !d.is_empty() => d,
        _ => chunk_payload.to_vec(),
    };

    // Sometimes the chunk itself is a DDS.
    if payload.len() >= 128 && &payload[0..4] == b"DDS " {
        return decode_texture(&payload);
    }

    let rgba = decode_payload(format, &payload, width, height)?;
    Ok(DecodedTexture {
        width,
        height,
        rgba,
        format_label: format!("{:?}, {} mip(s)", format, mips),
    })
}

fn decode_payload(
    format: TextureFormat,
    payload: &[u8],
    width: usize,
    height: usize,
) -> Result<Vec<u8>, String> {
    match format {
        TextureFormat::Dxt1 => decode_dxt1(payload, width, height),
        TextureFormat::Dxt3 => decode_dxt3(payload, width, height),
        TextureFormat::Dxt5 => decode_dxt5(payload, width, height),
        TextureFormat::Bgra8 => decode_bgra(payload, width, height),
        TextureFormat::Bgr8 => decode_bgr(payload, width, height),
        TextureFormat::L8 => decode_l8(payload, width, height),
        TextureFormat::Rgb565 => decode_rgb565(payload, width, height),
        TextureFormat::Unknown(v) => Err(format!("unsupported texture format 0x{v:08X}")),
    }
}

pub fn sniff_mesh_positions(data: &[u8]) -> Vec<[f32; 3]> {
    // Prefer dense stride-12 runs (xyz float triplets) with a sane bounding box.
    let mut best: Vec<[f32; 3]> = Vec::new();
    for start in 0..64.min(data.len()) {
        let pts = collect_stride12(data, start);
        if pts.len() > best.len() {
            best = pts;
        }
    }
    // Also try after common MeshSet header sizes.
    for start in [64usize, 128, 256, 512, 1024] {
        if start >= data.len() {
            break;
        }
        let pts = collect_stride12(data, start);
        if pts.len() > best.len() {
            best = pts;
        }
    }
    filter_outliers(best)
}

fn collect_stride12(data: &[u8], start: usize) -> Vec<[f32; 3]> {
    let mut pts = Vec::new();
    let mut i = start;
    let mut run = Vec::new();
    while i + 12 <= data.len() {
        let x = f32::from_le_bytes(data[i..i + 4].try_into().unwrap());
        let y = f32::from_le_bytes(data[i + 4..i + 8].try_into().unwrap());
        let z = f32::from_le_bytes(data[i + 8..i + 12].try_into().unwrap());
        let ok = x.is_finite()
            && y.is_finite()
            && z.is_finite()
            && x.abs() < 50_000.0
            && y.abs() < 50_000.0
            && z.abs() < 50_000.0
            && !(x == 0.0 && y == 0.0 && z == 0.0);
        if ok {
            run.push([x, y, z]);
            if run.len() > 25_000 {
                break;
            }
        } else if run.len() >= 64 {
            if run.len() > pts.len() {
                pts = std::mem::take(&mut run);
            } else {
                run.clear();
            }
        } else {
            run.clear();
        }
        i += 12;
    }
    if run.len() > pts.len() {
        pts = run;
    }
    pts
}

fn filter_outliers(mut pts: Vec<[f32; 3]>) -> Vec<[f32; 3]> {
    if pts.len() < 32 {
        return pts;
    }
    let mut xs: Vec<f32> = pts.iter().map(|p| p[0]).collect();
    let mut ys: Vec<f32> = pts.iter().map(|p| p[1]).collect();
    let mut zs: Vec<f32> = pts.iter().map(|p| p[2]).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    zs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pct = |v: &[f32], p: f32| v[((v.len() as f32 - 1.0) * p) as usize];
    let min_x = pct(&xs, 0.02);
    let max_x = pct(&xs, 0.98);
    let min_y = pct(&ys, 0.02);
    let max_y = pct(&ys, 0.98);
    let min_z = pct(&zs, 0.02);
    let max_z = pct(&zs, 0.98);
    pts.retain(|p| {
        p[0] >= min_x
            && p[0] <= max_x
            && p[1] >= min_y
            && p[1] <= max_y
            && p[2] >= min_z
            && p[2] <= max_z
    });
    pts
}

pub fn point_cloud_preview(points: &[[f32; 3]], size: usize) -> Option<ColorImage> {
    if points.len() < 32 || size == 0 {
        return None;
    }

    // Prefer the projection with the largest planar span (XZ or XY).
    let (ax, ay) = {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for p in points {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        let sx = max[0] - min[0];
        let sy = max[1] - min[1];
        let sz = max[2] - min[2];
        if sx * sz >= sx * sy {
            (0usize, 2usize)
        } else {
            (0usize, 1usize)
        }
    };

    let mut min_a = f32::INFINITY;
    let mut max_a = f32::NEG_INFINITY;
    let mut min_b = f32::INFINITY;
    let mut max_b = f32::NEG_INFINITY;
    for p in points {
        min_a = min_a.min(p[ax]);
        max_a = max_a.max(p[ax]);
        min_b = min_b.min(p[ay]);
        max_b = max_b.max(p[ay]);
    }
    let span_a = (max_a - min_a).max(0.001);
    let span_b = (max_b - min_b).max(0.001);
    let mut img = ColorImage::new([size, size], Color32::from_rgb(12, 14, 18));

    for p in points {
        let x = (((p[ax] - min_a) / span_a) * (size as f32 - 1.0)) as usize;
        let y = (((p[ay] - min_b) / span_b) * (size as f32 - 1.0)) as usize;
        let idx = (size - 1 - y.min(size - 1)) * size + x.min(size - 1);
        let c = &mut img.pixels[idx];
        // Accumulate density for a brighter silhouette.
        let r = c.r().saturating_add(28).min(220);
        let g = c.g().saturating_add(36).min(240);
        let b = c.b().saturating_add(40).min(255);
        *c = Color32::from_rgb(r, g, b);
    }

    Some(img)
}

fn decode_bgra(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let need = width.saturating_mul(height).saturating_mul(4);
    if data.len() < need {
        return Err("BGRA texture payload is truncated".into());
    }
    let mut out = vec![0u8; need];
    for (src, dst) in data[..need].chunks_exact(4).zip(out.chunks_exact_mut(4)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
        dst[3] = src[3];
    }
    Ok(out)
}

fn decode_bgr(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let pixels = width.saturating_mul(height);
    if data.len() < pixels.saturating_mul(3) {
        return Err("BGR texture payload is truncated".into());
    }
    let mut out = vec![0u8; pixels * 4];
    for (src, dst) in data[..pixels * 3].chunks_exact(3).zip(out.chunks_exact_mut(4)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
        dst[3] = 255;
    }
    Ok(out)
}

fn decode_l8(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let pixels = width.saturating_mul(height);
    if data.len() < pixels {
        return Err("L8 texture payload is truncated".into());
    }
    let mut out = vec![0u8; pixels * 4];
    for (v, dst) in data[..pixels].iter().zip(out.chunks_exact_mut(4)) {
        dst[0] = *v;
        dst[1] = *v;
        dst[2] = *v;
        dst[3] = 255;
    }
    Ok(out)
}

fn decode_rgb565(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let pixels = width.saturating_mul(height);
    if data.len() < pixels.saturating_mul(2) {
        return Err("RGB565 texture payload is truncated".into());
    }
    let mut out = vec![0u8; pixels * 4];
    for (src, dst) in data[..pixels * 2].chunks_exact(2).zip(out.chunks_exact_mut(4)) {
        let v = u16::from_le_bytes(src.try_into().unwrap());
        dst[0] = expand5(((v >> 11) & 0x1F) as u8);
        dst[1] = expand6(((v >> 5) & 0x3F) as u8);
        dst[2] = expand5((v & 0x1F) as u8);
        dst[3] = 255;
    }
    Ok(out)
}

fn decode_dxt1(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    decode_dxt(data, width, height, 8, false, false)
}

fn decode_dxt3(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    decode_dxt(data, width, height, 16, true, false)
}

fn decode_dxt5(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    decode_dxt(data, width, height, 16, false, true)
}

fn decode_dxt(
    data: &[u8],
    width: usize,
    height: usize,
    block_size: usize,
    explicit_alpha: bool,
    interp_alpha: bool,
) -> Result<Vec<u8>, String> {
    let bw = (width + 3) / 4;
    let bh = (height + 3) / 4;
    if data.len() < bw.saturating_mul(bh).saturating_mul(block_size) {
        return Err("DXT texture payload is truncated".into());
    }

    let mut out = vec![0u8; width * height * 4];
    let mut off = 0usize;
    for by in 0..bh {
        for bx in 0..bw {
            let mut alpha = [255u8; 16];
            if explicit_alpha {
                for i in 0..16 {
                    let byte = data[off + i / 2];
                    let nibble = if i & 1 == 0 { byte & 0x0F } else { byte >> 4 };
                    alpha[i] = (nibble << 4) | nibble;
                }
                off += 8;
            } else if interp_alpha {
                alpha = decode_dxt5_alpha(&data[off..off + 8]);
                off += 8;
            }

            let c0 = u16::from_le_bytes(data[off..off + 2].try_into().unwrap());
            let c1 = u16::from_le_bytes(data[off + 2..off + 4].try_into().unwrap());
            let bits = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap());
            off += 8;

            let colors = color_table(c0, c1);
            for py in 0..4 {
                for px in 0..4 {
                    let x = bx * 4 + px;
                    let y = by * 4 + py;
                    if x >= width || y >= height {
                        continue;
                    }
                    let i = py * 4 + px;
                    let ci = ((bits >> (i * 2)) & 0x03) as usize;
                    let dst = (y * width + x) * 4;
                    out[dst..dst + 3].copy_from_slice(&colors[ci][0..3]);
                    out[dst + 3] = if !explicit_alpha && !interp_alpha && c0 <= c1 && ci == 3 {
                        0
                    } else {
                        alpha[i]
                    };
                }
            }
        }
    }
    Ok(out)
}

fn decode_dxt5_alpha(data: &[u8]) -> [u8; 16] {
    let a0 = data[0];
    let a1 = data[1];
    let mut table = [0u8; 8];
    table[0] = a0;
    table[1] = a1;
    if a0 > a1 {
        for i in 1..6 {
            table[i + 1] = (((6 - i) as u16 * a0 as u16 + i as u16 * a1 as u16) / 7) as u8;
        }
    } else {
        for i in 1..4 {
            table[i + 1] = (((4 - i) as u16 * a0 as u16 + i as u16 * a1 as u16) / 5) as u8;
        }
        table[6] = 0;
        table[7] = 255;
    }

    let mut bits = 0u64;
    for i in 0..6 {
        bits |= (data[2 + i] as u64) << (8 * i);
    }
    let mut alpha = [255u8; 16];
    for i in 0..16 {
        alpha[i] = table[((bits >> (i * 3)) & 0x07) as usize];
    }
    alpha
}

fn color_table(c0: u16, c1: u16) -> [[u8; 4]; 4] {
    let a = rgb565(c0);
    let b = rgb565(c1);
    let mut out = [[0u8; 4]; 4];
    out[0] = [a[0], a[1], a[2], 255];
    out[1] = [b[0], b[1], b[2], 255];
    if c0 > c1 {
        out[2] = [
            ((2 * a[0] as u16 + b[0] as u16) / 3) as u8,
            ((2 * a[1] as u16 + b[1] as u16) / 3) as u8,
            ((2 * a[2] as u16 + b[2] as u16) / 3) as u8,
            255,
        ];
        out[3] = [
            ((a[0] as u16 + 2 * b[0] as u16) / 3) as u8,
            ((a[1] as u16 + 2 * b[1] as u16) / 3) as u8,
            ((a[2] as u16 + 2 * b[2] as u16) / 3) as u8,
            255,
        ];
    } else {
        out[2] = [
            ((a[0] as u16 + b[0] as u16) / 2) as u8,
            ((a[1] as u16 + b[1] as u16) / 2) as u8,
            ((a[2] as u16 + b[2] as u16) / 2) as u8,
            255,
        ];
        out[3] = [0, 0, 0, 0];
    }
    out
}

fn rgb565(v: u16) -> [u8; 3] {
    [
        expand5(((v >> 11) & 0x1F) as u8),
        expand6(((v >> 5) & 0x3F) as u8),
        expand5((v & 0x1F) as u8),
    ]
}

fn expand5(v: u8) -> u8 {
    (v << 3) | (v >> 2)
}

fn expand6(v: u8) -> u8 {
    (v << 2) | (v >> 4)
}

fn le_u16(data: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(off..off + 2)?.try_into().ok()?))
}

fn le_u32(data: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(off..off + 4)?.try_into().ok()?))
}
