//! CAS catalog (`cas.cat`) + payload extraction from `cas_XX.cas`.
//! CNC / IceBloc: `NyanNyanNyanNyan` magic + 32-byte legacy entries (not TnT CatHeader counts).

use flate2::read::ZlibDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

const NYAN_MAGIC: &[u8; 16] = b"NyanNyanNyanNyan";

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub sha1: [u8; 20],
    pub offset: u32,
    pub size: u32,
    pub cas_index: u32,
    pub range_start: u32,
    pub encrypted: bool,
    pub compressed_hint: bool,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub entries: HashMap<[u8; 20], CatalogEntry>,
    pub cas_paths: HashMap<u32, PathBuf>,
    pub format_label: String,
}

impl Catalog {
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
            cas_paths: HashMap::new(),
            format_label: "No cas.cat".into(),
        }
    }

    /// Load `cas.cat` when present. Missing catalog is OK (non-CAS / layout-only titles).
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let cat_path = data_dir.join("cas.cat");
        if !cat_path.exists() {
            // Still discover cas_*.cas if present (some layouts keep blobs without a cat).
            let mut cas_paths = HashMap::new();
            discover_cas_files(data_dir, &mut cas_paths);
            return Ok(Self {
                entries: HashMap::new(),
                cas_paths,
                format_label: "No cas.cat (filesystem / non-CAS only)".into(),
            });
        }
        let raw = std::fs::read(&cat_path).map_err(|e| format!("read cas.cat: {e}"))?;
        let unwrapped = crate::core::frostex::dbobject::unwrap_db_bytes(&raw);
        let (entries, format_label) = parse_catalog_bytes(&unwrapped)?;

        let mut cas_paths = HashMap::new();
        discover_cas_files(data_dir, &mut cas_paths);

        Ok(Self {
            entries,
            cas_paths,
            format_label,
        })
    }

    pub fn get(&self, sha1: &[u8; 20]) -> Option<&CatalogEntry> {
        self.entries.get(sha1)
    }

    pub fn extract(&self, sha1: &[u8; 20], prefer_compressed: bool) -> Result<Vec<u8>, String> {
        let entry = self.get(sha1).ok_or_else(|| {
            if self.entries.is_empty() {
                format!(
                    "SHA1 {} — no cas.cat index (non-CAS package or missing catalog)",
                    hex::encode(sha1)
                )
            } else {
                format!("SHA1 {} not in cas.cat", hex::encode(sha1))
            }
        })?;
        self.extract_entry(entry, prefer_compressed)
    }

    pub fn extract_entry(
        &self,
        entry: &CatalogEntry,
        prefer_compressed: bool,
    ) -> Result<Vec<u8>, String> {
        let cas_path = self.cas_paths.get(&entry.cas_index).ok_or_else(|| {
            format!(
                "cas_{:02} missing for SHA1 {}",
                entry.cas_index,
                hex::encode(entry.sha1)
            )
        })?;
        let mut file = std::fs::File::open(cas_path).map_err(|e| e.to_string())?;
        use std::io::{Seek, SeekFrom};
        file.seek(SeekFrom::Start(entry.offset as u64))
            .map_err(|e| e.to_string())?;
        let mut raw = vec![0u8; entry.size as usize];
        file.read_exact(&mut raw).map_err(|e| e.to_string())?;

        let compressed = prefer_compressed || entry.compressed_hint;
        if compressed {
            if let Ok(de) = try_fb_decompress(&raw) {
                return Ok(de);
            }
        }
        if let Ok(de) = try_fb_decompress(&raw) {
            if de.len() >= raw.len() || looks_like_payload(&de) {
                return Ok(de);
            }
        }
        Ok(raw)
    }

    /// Read only the first `max` bytes of a catalog payload (for huge blobs).
    pub fn extract_prefix(
        &self,
        sha1: &[u8; 20],
        prefer_compressed: bool,
        max: usize,
    ) -> Result<(Vec<u8>, bool), String> {
        let full = self.extract(sha1, prefer_compressed)?;
        if full.len() <= max {
            Ok((full, false))
        } else {
            Ok((full[..max].to_vec(), true))
        }
    }
}

fn discover_cas_files(data_dir: &Path, cas_paths: &mut HashMap<u32, PathBuf>) {
    let Ok(entries) = std::fs::read_dir(data_dir) else {
        return;
    };
    for ent in entries.flatten() {
        let path = ent.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !(name.starts_with("cas_") && name.ends_with(".cas")) {
            continue;
        }
        let num_str = name
            .trim_start_matches("cas_")
            .trim_end_matches(".cas");
        if let Ok(idx) = num_str.parse::<u32>() {
            cas_paths.insert(idx, path);
        }
    }
}

fn looks_like_payload(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    matches!(
        &data[0..4],
        [0xCE, 0xD1, 0xB2, 0x0F]
            | [0x0F, 0xB2, 0xD1, 0xCE]
            | [0xCE, 0xD1, 0xB4, 0x0F]
            | [0x0F, 0xB4, 0xD1, 0xCE]
            | [0x44, 0x44, 0x53, 0x20]
    )
}

fn parse_catalog_bytes(data: &[u8]) -> Result<(HashMap<[u8; 20], CatalogEntry>, String), String> {
    if data.len() >= 16 && &data[0..16] == NYAN_MAGIC {
        // Real TnT CatHeader has sane entry counts; CNC keeps Nyan magic + legacy 32-byte rows.
        if looks_like_tn_header(data) {
            return parse_tn_catalog(data);
        }
        let (map, _) = parse_legacy_catalog(data)?;
        let n = map.len();
        return Ok((map, format!("Nyan+legacy ({n} entries)")));
    }
    parse_legacy_catalog(data)
}

fn looks_like_tn_header(data: &[u8]) -> bool {
    if data.len() < 32 {
        return false;
    }
    let entry_count = u32::from_le_bytes(data[16..20].try_into().unwrap()) as u64;
    let patched = u32::from_le_bytes(data[20..24].try_into().unwrap()) as u64;
    let encrypted = u32::from_le_bytes(data[24..28].try_into().unwrap()) as u64;
    if entry_count == 0 || entry_count > 5_000_000 {
        return false;
    }
    if patched > 5_000_000 || encrypted > 5_000_000 {
        return false;
    }
    let need = 32u64
        .saturating_add(entry_count.saturating_mul(36))
        .saturating_add(patched.saturating_mul(60))
        .saturating_add(encrypted.saturating_mul(36 + 64));
    need <= data.len() as u64 + 4096
}

fn parse_tn_catalog(data: &[u8]) -> Result<(HashMap<[u8; 20], CatalogEntry>, String), String> {
    if data.len() < 32 {
        return Err("cas.cat too short for CatHeader".into());
    }
    let entry_count = u32::from_le_bytes(data[16..20].try_into().unwrap()) as usize;
    let mut map = HashMap::new();
    let mut off = 32usize;
    for _ in 0..entry_count {
        if off + 36 > data.len() {
            break;
        }
        let mut sha1 = [0u8; 20];
        sha1.copy_from_slice(&data[off..off + 20]);
        let file_offset = u32::from_le_bytes(data[off + 20..off + 24].try_into().unwrap());
        let size = u32::from_le_bytes(data[off + 24..off + 28].try_into().unwrap());
        let range_start = u32::from_le_bytes(data[off + 28..off + 32].try_into().unwrap());
        let file_number = data[off + 32] as u32;
        let mask = data[off + 33];
        map.insert(
            sha1,
            CatalogEntry {
                sha1,
                offset: file_offset,
                size,
                cas_index: file_number,
                range_start,
                encrypted: mask & 1 != 0,
                compressed_hint: false,
            },
        );
        off += 36;
    }
    let count = map.len();
    Ok((map, format!("TnT CatHeader ({count} entries)")))
}

fn parse_legacy_catalog(data: &[u8]) -> Result<(HashMap<[u8; 20], CatalogEntry>, String), String> {
    let mut map = HashMap::new();
    let mut off = if data.len() >= 16 { 16usize } else { 0 };
    while off + 32 <= data.len() {
        let mut sha1 = [0u8; 20];
        sha1.copy_from_slice(&data[off..off + 20]);
        let file_offset = u32::from_le_bytes(data[off + 20..off + 24].try_into().unwrap());
        let size = i32::from_le_bytes(data[off + 24..off + 28].try_into().unwrap());
        let cas_index = i32::from_le_bytes(data[off + 28..off + 32].try_into().unwrap());
        if size > 0 && cas_index >= 0 {
            map.insert(
                sha1,
                CatalogEntry {
                    sha1,
                    offset: file_offset,
                    size: size as u32,
                    cas_index: cas_index as u32,
                    range_start: 0,
                    encrypted: false,
                    compressed_hint: false,
                },
            );
        }
        off += 32;
    }
    let count = map.len();
    Ok((map, format!("Legacy cas.cat ({count} entries)")))
}

pub fn try_fb_decompress(raw: &[u8]) -> Result<Vec<u8>, String> {
    if let Ok(out) = zlib_block_stream(raw) {
        if !out.is_empty() {
            return Ok(out);
        }
    }
    if let Ok(out) = fb2013_block_stream(raw) {
        if !out.is_empty() {
            return Ok(out);
        }
    }
    Err("no decompress path matched".into())
}

fn zlib_block_stream(raw: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut i = 0usize;
    if raw.len() < 8 {
        return Err("too short".into());
    }
    while i + 8 <= raw.len() {
        let u_size = i32::from_be_bytes(raw[i..i + 4].try_into().unwrap());
        let c_size = i32::from_be_bytes(raw[i + 4..i + 8].try_into().unwrap());
        i += 8;
        if u_size <= 0 || c_size <= 0 || i + c_size as usize > raw.len() {
            return Err("bad zlib frame".into());
        }
        let chunk = &raw[i..i + c_size as usize];
        i += c_size as usize;
        match inflate_zlib(chunk) {
            Ok(de) => out.extend_from_slice(&de),
            Err(_) => out.extend_from_slice(chunk),
        }
        if i + 8 > raw.len() {
            break;
        }
        if i < raw.len() && i + 8 <= raw.len() {
            let next_u = i32::from_be_bytes(raw[i..i + 4].try_into().unwrap());
            if next_u <= 0 || next_u > 16 * 1024 * 1024 {
                break;
            }
        }
    }
    if out.is_empty() {
        Err("empty".into())
    } else {
        Ok(out)
    }
}

fn fb2013_block_stream(raw: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 8 <= raw.len() {
        let num1 = u32::from_be_bytes(raw[i..i + 4].try_into().unwrap());
        let num2 = u32::from_be_bytes(raw[i + 4..i + 8].try_into().unwrap());
        i += 8;
        let uncompressed = (num1 & 0x00FF_FFFF) as usize;
        let compressed = (num2 & 0x000F_FFFF) as usize;
        let comp_type = ((num2 >> 24) & 0xFF) as u8;
        if compressed == 0 || i + compressed > raw.len() {
            return Err("bad 2013 frame".into());
        }
        let chunk = &raw[i..i + compressed];
        i += compressed;
        match comp_type {
            0x00 => out.extend_from_slice(chunk),
            0x02 => match inflate_zlib(chunk) {
                Ok(de) => out.extend_from_slice(&de),
                Err(_) => out.extend_from_slice(chunk),
            },
            _ => {
                if uncompressed == compressed {
                    out.extend_from_slice(chunk);
                } else {
                    return Err(format!("unsupported compression 0x{comp_type:02X}"));
                }
            }
        }
        if i >= raw.len() {
            break;
        }
    }
    if out.is_empty() {
        Err("empty".into())
    } else {
        Ok(out)
    }
}

fn inflate_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut dec = ZlibDecoder::new(data);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}
