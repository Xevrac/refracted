//! Non-CAS Frostbite2 superbundle payloads (Nicknine / NFS:TR / BF3-style).
//! Magic `0x970D1C13`, big-endian meta, then 16-byte-aligned payloads in the same `.sb`.

use crate::core::frostex::index::{AssetKind, AssetRef, ChunkSource, TreeNode, TreeNodeKind};
use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

const NONCAS_MAGIC: u32 = 0x970D_1C13;

#[derive(Debug, Clone)]
pub struct NonCasEntry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub original_size: u64,
    pub sha1: Option<[u8; 20]>,
    pub res_type: Option<u32>,
    pub chunk_guid: Option<[u8; 16]>,
}

#[derive(Debug, Clone)]
pub struct NonCasBundle {
    pub ebx: Vec<NonCasEntry>,
    pub res: Vec<NonCasEntry>,
    pub chunks: Vec<NonCasEntry>,
}

pub fn looks_like_noncas(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }
    // First u32 is meta size; magic is the next u32.
    let magic_be = u32::from_be_bytes(data[4..8].try_into().unwrap());
    let magic_le = u32::from_le_bytes(data[4..8].try_into().unwrap());
    magic_be == NONCAS_MAGIC || magic_le == NONCAS_MAGIC
}

pub fn parse_noncas_bundle_at(sb_path: &Path, bundle_offset: u64) -> Result<NonCasBundle, String> {
    let mut file = std::fs::File::open(sb_path)
        .map_err(|e| format!("open {}: {e}", sb_path.display()))?;
    file.seek(SeekFrom::Start(bundle_offset))
        .map_err(|e| format!("seek {}: {e}", sb_path.display()))?;
    let mut peek = [0u8; 8];
    file.read_exact(&mut peek)
        .map_err(|e| format!("read header: {e}"))?;
    let magic_be = u32::from_be_bytes(peek[4..8].try_into().unwrap());
    let be = if magic_be == NONCAS_MAGIC {
        true
    } else if u32::from_le_bytes(peek[4..8].try_into().unwrap()) == NONCAS_MAGIC {
        false
    } else {
        return Err(format!(
            "not a noncas bundle (magic 0x{magic_be:08X}) at {bundle_offset}"
        ));
    };

    let file_len = file.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    let to_read = (file_len - bundle_offset) as usize;
    // Need meta + string table; payloads are read later via absolute offsets.
    // Cap buffer but keep enough for large string tables.
    let meta_size_peek = if be {
        u32::from_be_bytes(peek[0..4].try_into().unwrap())
    } else {
        u32::from_le_bytes(peek[0..4].try_into().unwrap())
    } as usize;
    let cap = to_read
        .min(256 * 1024 * 1024)
        .max(meta_size_peek.saturating_add(64).min(to_read));
    file.seek(SeekFrom::Start(bundle_offset))
        .map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; cap];
    let n = file.read(&mut buf).map_err(|e| format!("read bundle: {e}"))?;
    buf.truncate(n);
    parse_noncas_bytes(&buf, bundle_offset, be)
}

fn parse_noncas_bytes(data: &[u8], file_base: u64, be: bool) -> Result<NonCasBundle, String> {
    let mut c = Cursor::new(data);
    let meta_size = read_u32(&mut c, be)? as u64;
    let meta_start = c.position();
    let meta_end = meta_start + meta_size;
    if meta_end as usize > data.len() {
        return Err(format!(
            "noncas meta extends past buffer ({meta_end} > {})",
            data.len()
        ));
    }

    let magic = read_u32(&mut c, be)?;
    if magic != NONCAS_MAGIC {
        return Err(format!("bad noncas magic 0x{magic:08X}"));
    }
    let _num_entry = read_u32(&mut c, be)? as usize;
    let num_ebx = read_u32(&mut c, be)? as usize;
    let num_res = read_u32(&mut c, be)? as usize;
    let num_chunks = read_u32(&mut c, be)? as usize;
    let offset_string = read_u32(&mut c, be)? as u64 + meta_start;
    let _offset_chunk_meta = read_u32(&mut c, be)? as u64 + meta_start;
    let _size_chunk_meta = read_u32(&mut c, be)?;

    let total = num_ebx + num_res + num_chunks;
    let mut sha1_list = Vec::with_capacity(total);
    for _ in 0..total {
        let mut s = [0u8; 20];
        c.read_exact(&mut s).map_err(|e| e.to_string())?;
        sha1_list.push(s);
    }

    let mut ebx_meta = Vec::with_capacity(num_ebx);
    for _ in 0..num_ebx {
        ebx_meta.push((
            read_u32(&mut c, be)? as u64,
            read_u32(&mut c, be)? as u64,
            read_u32(&mut c, be)? as u64,
        ));
    }
    let mut res_meta = Vec::with_capacity(num_res);
    for _ in 0..num_res {
        res_meta.push((
            read_u32(&mut c, be)? as u64,
            read_u32(&mut c, be)? as u64,
            read_u32(&mut c, be)? as u64,
        ));
    }
    let mut res_types = Vec::with_capacity(num_res);
    for _ in 0..num_res {
        res_types.push(read_u32(&mut c, be)?);
    }
    for _ in 0..num_res {
        let mut meta = [0u8; 16];
        c.read_exact(&mut meta).map_err(|e| e.to_string())?;
        let _ = meta;
    }

    let mut chunk_meta = Vec::with_capacity(num_chunks);
    for _ in 0..num_chunks {
        let mut guid = [0u8; 16];
        c.read_exact(&mut guid).map_err(|e| e.to_string())?;
        // Guid stored big-endian in noncas stream when be=true; keep raw bytes for map key.
        let range_start = read_u32(&mut c, be)? as u64;
        let range_end = read_u32(&mut c, be)? as u64;
        let _logical = read_u32(&mut c, be)?;
        let size = range_end.saturating_sub(range_start);
        chunk_meta.push((guid, size));
    }

    // Optional chunkMeta DbObject — skip by seeking to string/payload via meta_end.
    // Names:
    let mut ebx = Vec::with_capacity(num_ebx);
    for (i, (off_str, size, original)) in ebx_meta.into_iter().enumerate() {
        let name = read_cstring_at(data, offset_string + off_str)?;
        ebx.push(NonCasEntry {
            name,
            offset: 0, // filled below
            size,
            original_size: original,
            sha1: sha1_list.get(i).copied(),
            res_type: None,
            chunk_guid: None,
        });
    }
    let mut res = Vec::with_capacity(num_res);
    for (i, (off_str, size, original)) in res_meta.into_iter().enumerate() {
        let name = read_cstring_at(data, offset_string + off_str)?;
        res.push(NonCasEntry {
            name,
            offset: 0,
            size,
            original_size: original,
            sha1: sha1_list.get(num_ebx + i).copied(),
            res_type: res_types.get(i).copied(),
            chunk_guid: None,
        });
    }
    let mut chunks = Vec::with_capacity(num_chunks);
    for (i, (guid, size)) in chunk_meta.into_iter().enumerate() {
        chunks.push(NonCasEntry {
            name: hex::encode(guid),
            offset: 0,
            size,
            original_size: size,
            sha1: sha1_list.get(num_ebx + num_res + i).copied(),
            res_type: None,
            chunk_guid: Some(guid),
        });
    }

    // Payload walk starts at meta_end, 16-byte aligned; offsets are absolute in the SB file.
    let mut pos = meta_end;
    for entry in ebx.iter_mut().chain(res.iter_mut()).chain(chunks.iter_mut()) {
        pos = align16(pos);
        // Convert buffer-relative to file-absolute.
        entry.offset = file_base + pos;
        pos += entry.size;
        if pos as usize > data.len() {
            // Payload extends beyond our buffered prefix — still valid on disk.
            // Keep offset/size; extract will read from file.
        }
    }

    Ok(NonCasBundle { ebx, res, chunks })
}

pub fn noncas_to_tree_nodes(
    parent_id: &str,
    bundle: &NonCasBundle,
    sb_path: &Path,
    assets: &mut BTreeMap<String, AssetRef>,
    chunk_map: &mut HashMap<[u8; 16], ChunkSource>,
) -> Vec<TreeNode> {
    let mut children = Vec::new();
    let groups: [(&str, TreeNodeKind, AssetKind, &Vec<NonCasEntry>); 3] = [
        ("ebx", TreeNodeKind::Ebx, AssetKind::Ebx, &bundle.ebx),
        ("res", TreeNodeKind::Res, AssetKind::Res, &bundle.res),
        ("chunks", TreeNodeKind::Chunk, AssetKind::Chunk, &bundle.chunks),
    ];
    for (field, node_kind, asset_kind, entries) in groups {
        if entries.is_empty() {
            continue;
        }
        let mut group = TreeNode::new(
            format!("{parent_id}/{field}"),
            field.to_ascii_uppercase(),
            TreeNodeKind::Directory,
        );
        for (idx, entry) in entries.iter().enumerate() {
            let asset_id = format!("{parent_id}/{field}/{idx}");
            let mut node = TreeNode::new(asset_id.clone(), entry.name.clone(), node_kind.clone());
            node.asset_id = Some(asset_id.clone());
            if let Some(g) = entry.chunk_guid {
                chunk_map.insert(
                    g,
                    ChunkSource::File {
                        path: sb_path.to_path_buf(),
                        offset: entry.offset,
                        size: entry.size,
                    },
                );
            }
            assets.insert(
                asset_id.clone(),
                AssetRef {
                    id: asset_id,
                    name: entry.name.clone(),
                    kind: asset_kind.clone(),
                    path: Some(sb_path.to_path_buf()),
                    sha1: None, // force SB slice extract for non-CAS (ignore catalog sha1)
                    chunk_guid: entry.chunk_guid,
                    size_hint: Some(entry.size),
                    res_type: entry.res_type,
                    payload_offset: Some(entry.offset),
                },
            );
            group.children.push(node);
        }
        children.push(group);
    }
    children
}

fn read_u32(c: &mut Cursor<&[u8]>, be: bool) -> Result<u32, String> {
    let mut b = [0u8; 4];
    c.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(if be {
        u32::from_be_bytes(b)
    } else {
        u32::from_le_bytes(b)
    })
}

fn read_cstring_at(data: &[u8], off: u64) -> Result<String, String> {
    let start = off as usize;
    if start >= data.len() {
        return Ok(format!("str_{off}"));
    }
    let end = data[start..]
        .iter()
        .position(|b| *b == 0)
        .map(|p| start + p)
        .unwrap_or(data.len());
    Ok(String::from_utf8_lossy(&data[start..end]).into_owned())
}

fn align16(v: u64) -> u64 {
    let rem = v % 16;
    if rem == 0 {
        v
    } else {
        v + 16 - rem
    }
}
