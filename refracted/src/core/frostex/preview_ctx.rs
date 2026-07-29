//! Cloneable extraction context for background preview jobs.

use crate::core::frostex::catalog::Catalog;
use crate::core::frostex::ebx::EbxGuidTable;
use crate::core::frostex::index::{AssetRef, ChunkSource, DataIndex};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PreviewCtx {
    pub catalog: Catalog,
    pub chunk_map: HashMap<[u8; 16], ChunkSource>,
    pub ebx_guid_table: EbxGuidTable,
}

impl PreviewCtx {
    pub fn from_index(index: &DataIndex) -> Self {
        Self::from_index_with_table(index, EbxGuidTable::new())
    }

    pub fn from_index_with_table(index: &DataIndex, ebx_guid_table: EbxGuidTable) -> Self {
        Self {
            catalog: index.catalog.clone(),
            chunk_map: index.chunk_map.clone(),
            ebx_guid_table,
        }
    }

    pub fn peek_bytes(&self, asset: &AssetRef, max: usize) -> Result<(Vec<u8>, bool), String> {
        let bytes = self.extract_bytes(asset)?;
        if bytes.len() > max {
            Ok((bytes[..max].to_vec(), true))
        } else {
            Ok((bytes, false))
        }
    }

    pub fn extract_bytes(&self, asset: &AssetRef) -> Result<Vec<u8>, String> {
        if let Some(sha1) = asset.sha1 {
            return self.catalog.extract(&sha1, true);
        }
        if let (Some(path), Some(offset)) = (&asset.path, asset.payload_offset) {
            let size = asset.size_hint.unwrap_or(0);
            return read_file_slice(path, offset, size);
        }
        if let Some(guid) = asset.chunk_guid {
            return self.get_chunk_bytes(&guid);
        }
        if let Some(path) = &asset.path {
            return std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()));
        }
        Err("asset has no payload".into())
    }

    pub fn get_chunk_bytes(&self, guid: &[u8; 16]) -> Result<Vec<u8>, String> {
        let src = self
            .chunk_map
            .get(guid)
            .ok_or_else(|| format!("chunk {} not found", hex::encode(guid)))?;
        match src {
            ChunkSource::Cas { sha1 } => self.catalog.extract(sha1, true),
            ChunkSource::File { path, offset, size } => read_file_slice(path, *offset, *size),
        }
    }
}

fn read_file_slice(path: &PathBuf, offset: u64, size: u64) -> Result<Vec<u8>, String> {
    use std::io::{Read, Seek, SeekFrom};
    if size == 0 {
        return Err(format!("zero-size slice from {}", path.display()));
    }
    if size > 512 * 1024 * 1024 {
        return Err(format!("refusing huge slice ({size} bytes)"));
    }
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("seek {}: {e}", path.display()))?;
    let mut buf = vec![0u8; size as usize];
    file.read_exact(&mut buf)
        .map_err(|e| format!("read {} @ {offset}+{size}: {e}", path.display()))?;
    match crate::core::frostex::catalog::try_fb_decompress(&buf) {
        Ok(de) if !de.is_empty() => Ok(de),
        _ => Ok(buf),
    }
}
