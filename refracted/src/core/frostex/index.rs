//! Lazy Frostbite data index and CAS extraction bridge.

use crate::core::frostex::catalog::Catalog;
use crate::core::frostex::dbobject::{load_db_file, parse_db_bytes, DbObject, DbValue};
use parking_lot::Mutex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Default)]
pub struct OpenProgress {
    pub done: usize,
    pub total: usize,
    pub phase: String,
}

#[derive(Debug, Clone)]
pub struct OpenJob {
    progress: Arc<Mutex<OpenProgress>>,
    result: Arc<Mutex<Option<Result<DataIndex, String>>>>,
}

impl OpenJob {
    pub fn progress(&self) -> OpenProgress {
        self.progress.lock().clone()
    }

    pub fn take_result(&self) -> Option<Result<DataIndex, String>> {
        self.result.lock().take()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeNodeKind {
    Directory,
    Toc,
    Sb,
    Bundle,
    Ebx,
    Res,
    Chunk,
    File,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub path: Option<PathBuf>,
    pub kind: TreeNodeKind,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub loaded: bool,
    pub asset_id: Option<String>,
}

impl TreeNode {
    pub(crate) fn new(id: String, label: String, kind: TreeNodeKind) -> Self {
        Self {
            id,
            label,
            path: None,
            kind,
            children: Vec::new(),
            expanded: false,
            loaded: true,
            asset_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetKind {
    Toc,
    Sb,
    Ebx,
    Res,
    Chunk,
    File,
}

#[derive(Debug, Clone)]
pub enum ChunkSource {
    Cas {
        sha1: [u8; 20],
    },
    /// Non-CAS chunk payload inside a companion `.sb` (or similar) file.
    File {
        path: PathBuf,
        offset: u64,
        size: u64,
    },
}

#[derive(Debug, Clone)]
pub struct AssetRef {
    pub id: String,
    pub name: String,
    pub kind: AssetKind,
    pub path: Option<PathBuf>,
    pub sha1: Option<[u8; 20]>,
    pub chunk_guid: Option<[u8; 16]>,
    pub size_hint: Option<u64>,
    pub res_type: Option<u32>,
    /// When set with `path`, read `path[offset .. offset+size_hint]` (non-CAS chunks).
    pub payload_offset: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DataIndex {
    pub data_dir: PathBuf,
    pub catalog: Catalog,
    pub root: TreeNode,
    pub assets: BTreeMap<String, AssetRef>,
    pub chunk_map: HashMap<[u8; 16], ChunkSource>,
}

impl DataIndex {
    pub fn start_open(data_dir: PathBuf) -> OpenJob {
        let progress = Arc::new(Mutex::new(OpenProgress {
            done: 0,
            total: 1,
            phase: "Starting...".into(),
        }));
        let result = Arc::new(Mutex::new(None));
        let progress_thread = progress.clone();
        let result_thread = result.clone();
        thread::spawn(move || {
            let opened = Self::open(data_dir, progress_thread);
            *result_thread.lock() = Some(opened);
        });
        OpenJob { progress, result }
    }

    pub fn open(data_dir: PathBuf, progress: Arc<Mutex<OpenProgress>>) -> Result<Self, String> {
        {
            let mut p = progress.lock();
            p.phase = "Loading catalog...".into();
            p.total = 1;
            p.done = 0;
        }
        // cas.cat is optional — many titles are non-CAS / superbundle-only.
        let catalog = match Catalog::load(&data_dir) {
            Ok(cat) => cat,
            Err(err) => {
                let mut cat = Catalog::empty();
                cat.format_label = format!("cas.cat unreadable ({err})");
                cat
            }
        };
        {
            let mut p = progress.lock();
            p.phase = format!("Scanning files… ({})", catalog.format_label);
            p.done = 1;
        }

        let mut assets = BTreeMap::new();
        let mut root = build_file_tree(&data_dir, &mut assets)?;
        if root.children.is_empty()
            && !data_dir.join("layout.toc").is_file()
            && std::fs::read_dir(&data_dir).map(|d| d.count()).unwrap_or(0) == 0
        {
            return Err(format!("empty or unreadable data folder: {}", data_dir.display()));
        }

        let toc_paths = collect_toc_paths(&root);
        {
            let mut p = progress.lock();
            p.phase = "Indexing TOC chunks...".into();
            p.done = 0;
            p.total = toc_paths.len().max(1);
        }

        let mut chunk_map = HashMap::new();
        for (i, path) in toc_paths.iter().enumerate() {
            if let Ok(obj) = load_db_file(path) {
                let sb = companion_sb_path(path);
                collect_chunk_sources(&obj, sb.as_deref(), &mut chunk_map);
            }
            let mut p = progress.lock();
            p.done = i + 1;
        }

        sort_tree(&mut root);
        {
            let mut p = progress.lock();
            p.phase = format!("Done — {}", catalog.format_label);
            p.done = p.total;
        }

        Ok(Self {
            data_dir,
            catalog,
            root,
            assets,
            chunk_map,
        })
    }

    pub fn ensure_expanded(&mut self, node_id: &str) -> Result<(), String> {
        let (path, kind, already_loaded) = match self.find_node(node_id) {
            Some(node) => (node.path.clone(), node.kind.clone(), node.loaded),
            None => return Err(format!("node not found: {node_id}")),
        };
        if already_loaded {
            if let Some(node) = self.find_node_mut(node_id) {
                node.expanded = true;
            }
            return Ok(());
        }

        let path = path.ok_or_else(|| "node has no path to expand".to_string())?;
        // Linked layout packages are Toc/Sb; also accept Bundle that resolved a path.
        let mut children = match kind {
            TreeNodeKind::Toc | TreeNodeKind::Sb | TreeNodeKind::Bundle => {
                self.parse_manifest_children(node_id, &path)?
            }
            _ => {
                return Err(format!(
                    "cannot expand {:?} node (expected package)",
                    kind
                ))
            }
        };
        sort_nodes(&mut children);

        if let Some(node) = self.find_node_mut(node_id) {
            node.children = children;
            node.loaded = true;
            node.expanded = true;
            // Promote linked bundle stubs to Toc once we know they parse as packages.
            if matches!(node.kind, TreeNodeKind::Bundle) {
                node.kind = TreeNodeKind::Toc;
            }
        }
        Ok(())
    }

    pub fn get_asset(&self, asset_id: &str) -> Option<&AssetRef> {
        self.assets.get(asset_id)
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
        Err("asset has no extractable payload".into())
    }

    pub fn peek_bytes(&self, asset: &AssetRef, max: usize) -> Result<(Vec<u8>, bool), String> {
        if let Some(sha1) = asset.sha1 {
            return self.catalog.extract_prefix(&sha1, true, max);
        }
        let bytes = self.extract_bytes(asset)?;
        if bytes.len() > max {
            Ok((bytes[..max].to_vec(), true))
        } else {
            Ok((bytes, false))
        }
    }

    pub fn get_chunk_bytes(&self, guid: &[u8; 16]) -> Result<Vec<u8>, String> {
        let src = self
            .chunk_map
            .get(guid)
            .ok_or_else(|| format!("chunk {} not found in TOC/SB index", hex::encode(guid)))?;
        match src {
            ChunkSource::Cas { sha1 } => self.catalog.extract(sha1, true),
            ChunkSource::File { path, offset, size } => read_file_slice(path, *offset, *size),
        }
    }

    pub fn find_node(&self, id: &str) -> Option<&TreeNode> {
        find_node(&self.root, id)
    }

    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut TreeNode> {
        find_node_mut(&mut self.root, id)
    }

    /// Expand this node and all expandable descendants (TOC/SB/Bundle/dirs).
    pub fn expand_recursive(&mut self, node_id: &str) -> Result<(), String> {
        let _ = self.ensure_expanded(node_id);
        let child_ids: Vec<String> = self
            .find_node(node_id)
            .map(|n| n.children.iter().map(|c| c.id.clone()).collect())
            .unwrap_or_default();
        for id in child_ids {
            let should = self
                .find_node(&id)
                .map(|n| {
                    matches!(
                        n.kind,
                        TreeNodeKind::Directory
                            | TreeNodeKind::Toc
                            | TreeNodeKind::Sb
                            | TreeNodeKind::Bundle
                    ) || !n.loaded
                })
                .unwrap_or(false);
            if should {
                self.expand_recursive(&id)?;
            }
        }
        Ok(())
    }

    /// Collect extractable leaf assets under `node_id` (caller should expand first).
    pub fn collect_rippable(&self, node_id: &str) -> Vec<AssetRef> {
        let mut out = Vec::new();
        if let Some(node) = self.find_node(node_id) {
            collect_rippable_walk(node, self, &mut out);
        }
        out
    }

    fn parse_manifest_children(&mut self, parent_id: &str, path: &Path) -> Result<Vec<TreeNode>, String> {
        let raw = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let obj = parse_db_bytes(&raw)?;
        let sb_path = companion_sb_path(path);
        collect_chunk_sources(&obj, sb_path.as_deref(), &mut self.chunk_map);

        let mut children = Vec::new();
        let mut bundles = Vec::new();
        collect_named_objects(&obj, "bundles", &mut bundles);
        collect_named_objects(&obj, "bundles2", &mut bundles);

        // layout.toc lists install/superbundle *names* (often bare strings).
        let mut names = Vec::new();
        collect_named_strings(&obj, "superBundles", &mut names);
        collect_named_strings(&obj, "superbundles", &mut names);
        collect_named_strings(&obj, "installChunks", &mut names);

        let mut seen_packages = HashSet::new();
        if bundles.is_empty() && names.is_empty() {
            let mut manifest = TreeNode::new(
                format!("{parent_id}/manifest"),
                "contents".into(),
                TreeNodeKind::Bundle,
            );
            manifest.children = asset_nodes_from_object(
                parent_id,
                &obj,
                &mut self.assets,
                sb_path.as_deref(),
                &mut self.chunk_map,
            );
            if manifest.children.is_empty() {
                let aid = format!("{parent_id}/raw");
                manifest.asset_id = Some(aid.clone());
                self.assets.insert(
                    aid.clone(),
                    AssetRef {
                        id: aid,
                        name: path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("manifest")
                            .to_string(),
                        kind: AssetKind::Toc,
                        path: Some(path.to_path_buf()),
                        sha1: None,
                        chunk_guid: None,
                        size_hint: None,
                        res_type: None,
                        payload_offset: None,
                    },
                );
                manifest.loaded = true;
            }
            children.push(manifest);
        } else {
            for (idx, bundle) in bundles.into_iter().enumerate() {
                let name = object_string(bundle, &["name", "id", "path"])
                    .or_else(|| bundle.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("bundle_{idx}"));
                let key = normalize_package_key(&name);
                if !seen_packages.insert(key) {
                    continue;
                }
                children.push(self.make_bundle_node(parent_id, idx, bundle, path)?);
            }
            for (idx, name) in names.into_iter().enumerate() {
                let key = normalize_package_key(&name);
                if !seen_packages.insert(key) {
                    continue;
                }
                children.push(self.make_named_package_node(parent_id, idx + 10_000, &name));
            }
        }

        let direct = asset_nodes_from_object(
            parent_id,
            &obj,
            &mut self.assets,
            sb_path.as_deref(),
            &mut self.chunk_map,
        );
        children.extend(direct);

        Ok(children)
    }

    fn make_bundle_node(
        &mut self,
        parent_id: &str,
        idx: usize,
        bundle: &DbObject,
        package_path: &Path,
    ) -> Result<TreeNode, String> {
        let name = object_string(bundle, &["name", "id", "path"])
            .or_else(|| bundle.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("bundle_{idx}"));
            // Prefer a short label for deep bundle ids.
            let label = name.rsplit('/').next().unwrap_or(name.as_str()).to_string();
            let mut node = TreeNode::new(
                format!("{parent_id}/bundle/{idx}"),
                label,
                TreeNodeKind::Bundle,
            );
        node.children = asset_nodes_from_object(
            &node.id,
            bundle,
            &mut self.assets,
            companion_sb_path(package_path).as_deref(),
            &mut self.chunk_map,
        );

        // TOC bundles usually only store id/offset/size → payload lives in companion .sb.
        if node.children.is_empty() {
            let offset = object_i64(bundle, &["offset"]);
            let size = object_i64(bundle, &["size"]);
            if let (Some(offset), Some(size)) = (offset, size) {
                if offset >= 0 && size > 0 {
                    if let Some(sb_path) = companion_sb_path(package_path) {
                        // Non-CAS bundles use a binary format (0x970D1C13), not DbObject.
                        if let Ok(noncas) = crate::core::frostex::noncas::parse_noncas_bundle_at(
                            &sb_path,
                            offset as u64,
                        ) {
                            node.children = crate::core::frostex::noncas::noncas_to_tree_nodes(
                                &node.id,
                                &noncas,
                                &sb_path,
                                &mut self.assets,
                                &mut self.chunk_map,
                            );
                            node.loaded = true;
                            return Ok(node);
                        }
                        match load_sb_bundle_object(&sb_path, offset as u64, size as u64) {
                            Ok(payload) => {
                                collect_chunk_sources(
                                    &payload,
                                    Some(&sb_path),
                                    &mut self.chunk_map,
                                );
                                // Prefer SB-file offsets for entries that have them (non-CAS
                                // payloads living beside DbObject meta, or cas:false hybrids).
                                node.children = asset_nodes_from_object(
                                    &node.id,
                                    &payload,
                                    &mut self.assets,
                                    Some(&sb_path),
                                    &mut self.chunk_map,
                                );
                                // If package has no cas.cat, re-bind sha1-backed leaves that
                                // also carry offset/size onto the SB file.
                                rebind_casless_offsets(
                                    &node.id,
                                    &payload,
                                    &sb_path,
                                    &mut self.assets,
                                    &mut self.chunk_map,
                                    self.catalog.entries.is_empty(),
                                );
                                node.loaded = true;
                                if node.children.is_empty() {
                                    let aid = format!("{}/sb_raw", node.id);
                                    node.asset_id = Some(aid.clone());
                                    self.assets.insert(
                                        aid.clone(),
                                        AssetRef {
                                            id: aid,
                                            name: format!("{name} (bundle meta)"),
                                            kind: AssetKind::File,
                                            path: Some(sb_path),
                                            sha1: None,
                                            chunk_guid: None,
                                            size_hint: Some(size as u64),
                                            res_type: None,
                                            payload_offset: Some(offset as u64),
                                        },
                                    );
                                }
                                return Ok(node);
                            }
                            Err(err) => {
                                let aid = format!("{}/sb_err", node.id);
                                node.asset_id = Some(aid.clone());
                                node.loaded = true;
                                self.assets.insert(
                                    aid.clone(),
                                    AssetRef {
                                        id: aid,
                                        name: format!("{name} (sb read failed: {err})"),
                                        kind: AssetKind::File,
                                        path: Some(sb_path),
                                        sha1: None,
                                        chunk_guid: None,
                                        size_hint: Some(size as u64),
                                        res_type: None,
                                        payload_offset: None,
                                    },
                                );
                                return Ok(node);
                            }
                        }
                    }
                }
            }

            // layout.toc style: name-only references to other packages.
            if let Some(toc_path) = resolve_package_toc(&self.data_dir, &name) {
                let is_sb = toc_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("sb"))
                    .unwrap_or(false);
                let asset_id = format!("file:{}", toc_path.display());
                node.path = Some(toc_path.clone());
                node.kind = if is_sb {
                    TreeNodeKind::Sb
                } else {
                    TreeNodeKind::Toc
                };
                node.loaded = false;
                node.asset_id = Some(asset_id.clone());
                let asset_kind = if is_sb {
                    AssetKind::Sb
                } else {
                    AssetKind::Toc
                };
                self.assets.entry(asset_id.clone()).or_insert_with(|| AssetRef {
                    id: asset_id,
                    name: name.clone(),
                    kind: asset_kind,
                    path: Some(toc_path),
                    sha1: None,
                    chunk_guid: None,
                    size_hint: None,
                    res_type: None,
                    payload_offset: None,
                });
            } else {
                let aid = format!("{}/info", node.id);
                node.asset_id = Some(aid.clone());
                node.loaded = true;
                self.assets.insert(
                    aid.clone(),
                    AssetRef {
                        id: aid,
                        name: format!("{name} (unresolved package)"),
                        kind: AssetKind::File,
                        path: None,
                        sha1: first_sha1(bundle),
                        chunk_guid: None,
                        size_hint: object_i64(bundle, &["size"]).map(|v| v.max(0) as u64),
                        res_type: None,
                        payload_offset: None,
                    },
                );
            }
        } else {
            node.loaded = true;
        }
        Ok(node)
    }

    fn make_named_package_node(&mut self, parent_id: &str, idx: usize, name: &str) -> TreeNode {
        let mut node = TreeNode::new(
            format!("{parent_id}/pkg/{idx}"),
            name.to_string(),
            TreeNodeKind::Bundle,
        );
        if let Some(toc_path) = resolve_package_toc(&self.data_dir, name) {
            let is_sb = toc_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("sb"))
                .unwrap_or(false);
            let asset_id = format!("file:{}", toc_path.display());
            node.path = Some(toc_path.clone());
            node.kind = if is_sb {
                TreeNodeKind::Sb
            } else {
                TreeNodeKind::Toc
            };
            node.loaded = false;
            node.asset_id = Some(asset_id.clone());
            let asset_kind = if is_sb {
                AssetKind::Sb
            } else {
                AssetKind::Toc
            };
            self.assets.entry(asset_id.clone()).or_insert_with(|| AssetRef {
                id: asset_id,
                name: name.to_string(),
                kind: asset_kind,
                path: Some(toc_path),
                sha1: None,
                chunk_guid: None,
                size_hint: None,
                res_type: None,
            payload_offset: None,
            });
        } else {
            node.loaded = true;
            let aid = format!("{}/missing", node.id);
            node.asset_id = Some(aid.clone());
            self.assets.insert(
                aid.clone(),
                AssetRef {
                    id: aid,
                    name: format!("{name} (missing .toc)"),
                    kind: AssetKind::File,
                    path: None,
                    sha1: None,
                    chunk_guid: None,
                    size_hint: None,
                    res_type: None,
                payload_offset: None,
                },
            );
        }
        node
    }
}

fn companion_sb_path(package_path: &Path) -> Option<PathBuf> {
    let ext = package_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("sb") {
        return Some(package_path.to_path_buf());
    }
    let sb = package_path.with_extension("sb");
    if sb.is_file() {
        Some(sb)
    } else {
        None
    }
}

fn load_sb_bundle_object(sb_path: &Path, offset: u64, size: u64) -> Result<DbObject, String> {
    use std::io::{Read, Seek, SeekFrom};
    if size == 0 || size > 256 * 1024 * 1024 {
        return Err(format!("implausible bundle size {size}"));
    }
    let mut file =
        std::fs::File::open(sb_path).map_err(|e| format!("open {}: {e}", sb_path.display()))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("seek {}: {e}", sb_path.display()))?;
    let mut buf = vec![0u8; size as usize];
    file.read_exact(&mut buf)
        .map_err(|e| format!("read bundle @ {offset} from {}: {e}", sb_path.display()))?;
    if crate::core::frostex::noncas::looks_like_noncas(&buf) {
        return Err("noncas binary bundle (handled separately)".into());
    }
    match parse_db_bytes(&buf) {
        Ok(obj) => Ok(obj),
        Err(err) => match crate::core::frostex::catalog::try_fb_decompress(&buf) {
            Ok(de) => parse_db_bytes(&de),
            Err(_) => Err(err),
        },
    }
}

/// When cas.cat is missing, prefer SB file offsets over SHA1 for entries that have both.
fn rebind_casless_offsets(
    parent_id: &str,
    obj: &DbObject,
    sb_path: &Path,
    assets: &mut BTreeMap<String, AssetRef>,
    chunk_map: &mut HashMap<[u8; 16], ChunkSource>,
    catalog_empty: bool,
) {
    if !catalog_empty {
        return;
    }
    for field in ["ebx", "res", "chunks"] {
        let mut values = Vec::new();
        collect_named_objects(obj, field, &mut values);
        for (idx, item) in values.iter().enumerate() {
            let offset = object_i64(item, &["offset"]).map(|v| v.max(0) as u64);
            let size = object_i64(item, &["size"]).map(|v| v.max(0) as u64);
            let (Some(off), Some(sz)) = (offset, size) else {
                continue;
            };
            if sz == 0 {
                continue;
            }
            let asset_id = format!("{parent_id}/{field}/{idx}");
            if let Some(asset) = assets.get_mut(&asset_id) {
                asset.path = Some(sb_path.to_path_buf());
                asset.payload_offset = Some(off);
                asset.size_hint = Some(sz);
                asset.sha1 = None;
            }
            if let Some(g) = object_guid(item) {
                chunk_map.insert(
                    g,
                    ChunkSource::File {
                        path: sb_path.to_path_buf(),
                        offset: off,
                        size: sz,
                    },
                );
            }
        }
    }
}

fn normalize_package_key(name: &str) -> String {
    name.trim()
        .trim_start_matches('/')
        .replace('\\', "/")
        .trim_end_matches(".toc")
        .trim_end_matches(".sb")
        .trim_end_matches(".TOC")
        .trim_end_matches(".SB")
        .to_ascii_lowercase()
}

fn resolve_package_toc(data_dir: &Path, name: &str) -> Option<PathBuf> {
    let name = name.trim().trim_start_matches('/').replace('\\', "/");
    if name.is_empty() {
        return None;
    }
    let stem = name
        .trim_end_matches(".toc")
        .trim_end_matches(".sb")
        .trim_end_matches(".TOC")
        .trim_end_matches(".SB");
    let base = stem.rsplit('/').next().unwrap_or(stem);

    let mut candidates: Vec<PathBuf> = vec![
        data_dir.join(format!("{stem}.toc")),
        data_dir.join(stem).with_extension("toc"),
        data_dir.join(format!("{stem}.sb")),
        data_dir.join("Win32").join(format!("{base}.toc")),
        data_dir.join("win32").join(format!("{base}.toc")),
        data_dir.join("Win32").join(format!("{stem}.toc")),
        data_dir.join("win32").join(format!("{stem}.toc")),
    ];
    for prefix in ["Win32/", "win32/", "PS3/", "Xenon/", "Gen4a/", "Gen4b/"] {
        if let Some(rest) = stem.strip_prefix(prefix) {
            candidates.push(data_dir.join(format!("{rest}.toc")));
            candidates.push(data_dir.join(prefix.trim_end_matches('/')).join(format!("{rest}.toc")));
            candidates.push(data_dir.join(format!("{stem}.toc")));
        }
    }

    if let Some(found) = candidates.into_iter().find(|p| p.is_file()) {
        return Some(found);
    }

    // Last resort: case-insensitive match against known TOC paths under data_dir.
    let want = normalize_package_key(stem);
    find_toc_by_key(data_dir, &want)
}

fn find_toc_by_key(data_dir: &Path, want_key: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, data_dir: &Path, want_key: &str) -> Option<PathBuf> {
        let entries = std::fs::read_dir(dir).ok()?;
        for ent in entries.flatten() {
            let path = ent.path();
            if path.is_dir() {
                if let Some(found) = walk(&path, data_dir, want_key) {
                    return Some(found);
                }
                continue;
            }
            let name = path.file_name()?.to_string_lossy();
            let lower = name.to_ascii_lowercase();
            if !(lower.ends_with(".toc") || lower.ends_with(".sb")) {
                continue;
            }
            let rel = path.strip_prefix(data_dir).unwrap_or(&path);
            let key = normalize_package_key(&rel.to_string_lossy());
            if key == want_key || key.ends_with(want_key) || want_key.ends_with(&key) {
                return Some(path);
            }
        }
        None
    }
    walk(data_dir, data_dir, want_key)
}

fn collect_named_strings(obj: &DbObject, field_name: &str, out: &mut Vec<String>) {
    if obj.name.eq_ignore_ascii_case(field_name) {
        match &obj.value {
            DbValue::Array(items) | DbValue::Object(items) => {
                for item in items {
                    if let Some(s) = item.as_str() {
                        out.push(s.to_string());
                    } else if let Some(s) = object_string(item, &["name", "id", "path"]) {
                        out.push(s);
                    }
                }
            }
            DbValue::String(s) => out.push(s.clone()),
            _ => {}
        }
    }
    if let Some(fields) = obj.as_object_fields() {
        for f in fields {
            collect_named_strings(f, field_name, out);
        }
    }
}

fn build_file_tree(data_dir: &Path, assets: &mut BTreeMap<String, AssetRef>) -> Result<TreeNode, String> {
    let label = data_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("data")
        .to_string();
    let mut root = TreeNode::new("root".into(), label, TreeNodeKind::Directory);
    root.path = Some(data_dir.to_path_buf());
    root.children = build_dir_children(data_dir, "root", assets)?;
    Ok(root)
}

fn build_dir_children(
    dir: &Path,
    parent_id: &str,
    assets: &mut BTreeMap<String, AssetRef>,
) -> Result<Vec<TreeNode>, String> {
    let mut out = Vec::new();
    for ent in std::fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))? {
        let ent = match ent {
            Ok(v) => v,
            Err(_) => continue,
        };
        let path = ent.path();
        let name = ent.file_name().to_string_lossy().to_string();
        let id = format!("{parent_id}/{}", sanitize_id(&name));
        if path.is_dir() {
            let mut node = TreeNode::new(id.clone(), name, TreeNodeKind::Directory);
            node.path = Some(path.clone());
            node.children = build_dir_children(&path, &id, assets).unwrap_or_default();
            out.push(node);
        } else {
            let lower = name.to_ascii_lowercase();
            let (kind, asset_kind, lazy) = if lower.ends_with(".toc") {
                (TreeNodeKind::Toc, AssetKind::Toc, true)
            } else if lower.ends_with(".sb") {
                (TreeNodeKind::Sb, AssetKind::Sb, true)
            } else {
                (TreeNodeKind::File, AssetKind::File, false)
            };
            let asset_id = format!("file:{}", path.display());
            let size_hint = std::fs::metadata(&path).ok().map(|m| m.len());
            let mut node = TreeNode::new(id, name.clone(), kind);
            node.path = Some(path.clone());
            node.loaded = !lazy;
            node.asset_id = Some(asset_id.clone());
            assets.insert(
                asset_id.clone(),
                AssetRef {
                    id: asset_id,
                    name,
                    kind: asset_kind,
                    path: Some(path),
                    sha1: None,
                    chunk_guid: None,
                    size_hint,
                    res_type: None,
                payload_offset: None,
                },
            );
            out.push(node);
        }
    }
    Ok(out)
}

fn asset_nodes_from_object(
    parent_id: &str,
    obj: &DbObject,
    assets: &mut BTreeMap<String, AssetRef>,
    sb_path: Option<&Path>,
    chunk_map: &mut HashMap<[u8; 16], ChunkSource>,
) -> Vec<TreeNode> {
    let mut children = Vec::new();
    let groups = [
        ("ebx", TreeNodeKind::Ebx, AssetKind::Ebx),
        ("res", TreeNodeKind::Res, AssetKind::Res),
        ("chunks", TreeNodeKind::Chunk, AssetKind::Chunk),
    ];

    for (field, node_kind, asset_kind) in groups {
        let mut values = Vec::new();
        collect_named_objects(obj, field, &mut values);
        if values.is_empty() {
            continue;
        }

        let mut group = TreeNode::new(
            format!("{parent_id}/{field}"),
            field.to_ascii_uppercase(),
            TreeNodeKind::Directory,
        );
        for (idx, item) in values.iter().enumerate() {
            let name = object_string(item, &["name", "id"])
                .or_else(|| object_guid(item).map(hex::encode))
                .unwrap_or_else(|| format!("{field}_{idx}"));
            let sha1 = first_sha1(item);
            let guid = object_guid(item);
            let res_type = object_i64(item, &["resType", "type"]).map(|v| v as u32);
            let size_hint = object_i64(item, &["size", "originalSize", "logicalSize"])
                .map(|v| v.max(0) as u64);
            let offset = object_i64(item, &["offset"]).map(|v| v.max(0) as u64);

            // Non-CAS chunks: payload is a slice of the companion .sb.
            let (path, payload_offset) = if sha1.is_none() {
                if let (Some(sb), Some(off), Some(sz)) = (sb_path, offset, size_hint) {
                    if let Some(g) = guid {
                        chunk_map.insert(
                            g,
                            ChunkSource::File {
                                path: sb.to_path_buf(),
                                offset: off,
                                size: sz,
                            },
                        );
                    }
                    (Some(sb.to_path_buf()), Some(off))
                } else {
                    (None, None)
                }
            } else {
                if let (Some(g), Some(s)) = (guid, sha1) {
                    chunk_map.insert(g, ChunkSource::Cas { sha1: s });
                }
                (None, None)
            };

            let asset_id = format!("{parent_id}/{field}/{idx}");
            let mut node = TreeNode::new(asset_id.clone(), name.clone(), node_kind.clone());
            node.asset_id = Some(asset_id.clone());
            assets.insert(
                asset_id.clone(),
                AssetRef {
                    id: asset_id,
                    name,
                    kind: asset_kind.clone(),
                    path,
                    sha1,
                    chunk_guid: guid,
                    size_hint,
                    res_type,
                    payload_offset,
                },
            );
            group.children.push(node);
        }
        sort_nodes(&mut group.children);
        children.push(group);
    }

    children
}

fn collect_chunk_sources(
    obj: &DbObject,
    sb_path: Option<&Path>,
    out: &mut HashMap<[u8; 16], ChunkSource>,
) {
    let guid = object_guid(obj);
    let sha1 = first_sha1(obj);
    let offset = object_i64(obj, &["offset"]).map(|v| v.max(0) as u64);
    let size = object_i64(obj, &["size"]).map(|v| v.max(0) as u64);
    if let Some(g) = guid {
        if let Some(s) = sha1 {
            out.insert(g, ChunkSource::Cas { sha1: s });
        } else if let (Some(path), Some(off), Some(sz)) = (sb_path, offset, size) {
            if sz > 0 {
                out.insert(
                    g,
                    ChunkSource::File {
                        path: path.to_path_buf(),
                        offset: off,
                        size: sz,
                    },
                );
            }
        }
    }
    if let Some(fields) = obj.as_object_fields() {
        for f in fields {
            collect_chunk_sources(f, sb_path, out);
        }
    }
}

fn read_file_slice(path: &Path, offset: u64, size: u64) -> Result<Vec<u8>, String> {
    use std::io::{Read, Seek, SeekFrom};
    if size == 0 {
        return Err(format!("zero-size slice from {}", path.display()));
    }
    if size > 512 * 1024 * 1024 {
        return Err(format!("refusing huge slice ({size} bytes) from {}", path.display()));
    }
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("seek {}: {e}", path.display()))?;
    let mut buf = vec![0u8; size as usize];
    file.read_exact(&mut buf)
        .map_err(|e| format!("read {} @ {offset}+{size}: {e}", path.display()))?;
    // Non-CAS chunks may still be zlib-framed.
    match crate::core::frostex::catalog::try_fb_decompress(&buf) {
        Ok(de) if !de.is_empty() => Ok(de),
        _ => Ok(buf),
    }
}

fn collect_toc_paths(root: &TreeNode) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn walk(node: &TreeNode, out: &mut Vec<PathBuf>) {
        if matches!(node.kind, TreeNodeKind::Toc | TreeNodeKind::Sb) {
            if let Some(path) = &node.path {
                out.push(path.clone());
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(root, &mut out);
    out
}

fn collect_named_objects<'a>(obj: &'a DbObject, field_name: &str, out: &mut Vec<&'a DbObject>) {
    if obj.name.eq_ignore_ascii_case(field_name) {
        match &obj.value {
            DbValue::Array(items) | DbValue::Object(items) => {
                for item in items {
                    out.push(item);
                }
            }
            _ => out.push(obj),
        }
    }
    if let Some(fields) = obj.as_object_fields() {
        for f in fields {
            collect_named_objects(f, field_name, out);
        }
    }
}

fn object_string(obj: &DbObject, names: &[&str]) -> Option<String> {
    for name in names {
        if let Some(v) = obj.field(name).and_then(|f| f.as_str()) {
            return Some(v.to_string());
        }
    }
    None
}

fn object_i64(obj: &DbObject, names: &[&str]) -> Option<i64> {
    for name in names {
        if let Some(v) = obj.field(name).and_then(|f| f.as_i64()) {
            return Some(v);
        }
    }
    None
}

fn object_guid(obj: &DbObject) -> Option<[u8; 16]> {
    for name in ["id", "guid", "chunkId", "chunkGuid"] {
        if let Some(v) = obj.field(name).and_then(|f| f.as_guid()) {
            return Some(v);
        }
    }
    match &obj.value {
        DbValue::Guid(g) => Some(*g),
        DbValue::Object(fields) | DbValue::Array(fields) => {
            for f in fields {
                if let Some(v) = object_guid(f) {
                    return Some(v);
                }
            }
            None
        }
        _ => None,
    }
}

fn first_sha1(obj: &DbObject) -> Option<[u8; 20]> {
    match &obj.value {
        DbValue::Sha1(s) => Some(*s),
        DbValue::Object(fields) | DbValue::Array(fields) => {
            for preferred in ["sha1", "sha", "cas", "payload"] {
                if let Some(s) = obj.field(preferred).and_then(|f| f.as_sha1()) {
                    return Some(s);
                }
            }
            for f in fields {
                if let Some(s) = first_sha1(f) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}

fn find_node<'a>(node: &'a TreeNode, id: &str) -> Option<&'a TreeNode> {
    if node.id == id {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node(child, id) {
            return Some(found);
        }
    }
    None
}

fn find_node_mut<'a>(node: &'a mut TreeNode, id: &str) -> Option<&'a mut TreeNode> {
    if node.id == id {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, id) {
            return Some(found);
        }
    }
    None
}

fn sort_tree(node: &mut TreeNode) {
    sort_nodes(&mut node.children);
    for child in &mut node.children {
        sort_tree(child);
    }
}

fn sort_nodes(nodes: &mut [TreeNode]) {
    nodes.sort_by(|a, b| {
        let ak = !matches!(a.kind, TreeNodeKind::Directory);
        let bk = !matches!(b.kind, TreeNodeKind::Directory);
        ak.cmp(&bk)
            .then_with(|| a.label.to_ascii_lowercase().cmp(&b.label.to_ascii_lowercase()))
    });
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

fn is_rippable_asset(asset: &AssetRef) -> bool {
    match asset.kind {
        AssetKind::Ebx | AssetKind::Res | AssetKind::Chunk => {
            asset.sha1.is_some()
                || asset.chunk_guid.is_some()
                || (asset.path.is_some() && asset.payload_offset.is_some())
        }
        // Whole files on disk (configs, cats, loose assets) plus sliced payloads.
        AssetKind::File => asset.sha1.is_some() || asset.path.is_some(),
        // TOC/SB themselves can be copied as whole files when they have a path.
        AssetKind::Toc | AssetKind::Sb => asset.path.is_some() && asset.payload_offset.is_none(),
    }
}

fn collect_rippable_walk(node: &TreeNode, index: &DataIndex, out: &mut Vec<AssetRef>) {
    if let Some(aid) = &node.asset_id {
        if let Some(asset) = index.get_asset(aid) {
            if is_rippable_asset(asset) {
                out.push(asset.clone());
            }
        }
    }
    for child in &node.children {
        collect_rippable_walk(child, index, out);
    }
}

#[cfg(test)]
mod sb_bundle_tests {
    use super::*;
    use parking_lot::Mutex;
    use std::sync::Arc;

    #[test]
    fn globals_toc_expands_sb_assets() {
        let data = PathBuf::from(
            r"D:\_DATA\Projects\RE\Command and Conquer\Bin\Command & Conquer\Data",
        );
        // CAS extract tests need a catalog; non-CAS path still opens without it.
        if !data.join("cas.cat").is_file() && !data.join("layout.toc").is_file() {
            return;
        }
        let progress = Arc::new(Mutex::new(OpenProgress::default()));
        let mut index = DataIndex::open(data, progress).expect("open");

        fn find_main_globals(node: &TreeNode) -> Option<String> {
            if matches!(node.kind, TreeNodeKind::Toc) {
                if let Some(path) = &node.path {
                    let s = path.to_string_lossy().replace('\\', "/");
                    if s.ends_with("/Win32/Globals.toc") && !s.contains("/_DEBUG_/") {
                        return Some(node.id.clone());
                    }
                }
            }
            for c in &node.children {
                if let Some(id) = find_main_globals(c) {
                    return Some(id);
                }
            }
            None
        }

        let toc_id = find_main_globals(&index.root).expect("Win32/Globals.toc in tree");
        index.ensure_expanded(&toc_id).expect("expand toc");
        let toc = index.find_node(&toc_id).expect("toc node");
        assert!(
            toc.children.len() >= 4,
            "expected bundles, got {}",
            toc.children.len()
        );
        let first = &toc.children[0];
        assert!(
            !first.children.is_empty(),
            "first bundle should have EBX/RES/CHUNKS groups"
        );
        let mut leaf_with_sha = 0usize;
        fn count_sha(node: &TreeNode, assets: &BTreeMap<String, AssetRef>, out: &mut usize) {
            if let Some(aid) = &node.asset_id {
                if assets.get(aid).and_then(|a| a.sha1).is_some() {
                    *out += 1;
                }
            }
            for c in &node.children {
                count_sha(c, assets, out);
            }
        }
        count_sha(first, &index.assets, &mut leaf_with_sha);
        assert!(
            leaf_with_sha > 0,
            "expected CAS-backed assets under first bundle"
        );

        // Spot-check CAS extract for one EBX leaf.
        let mut sample = None;
        fn first_sha_asset<'a>(
            node: &TreeNode,
            assets: &'a BTreeMap<String, AssetRef>,
            out: &mut Option<&'a AssetRef>,
        ) {
            if out.is_some() {
                return;
            }
            if let Some(aid) = &node.asset_id {
                if let Some(a) = assets.get(aid) {
                    if a.sha1.is_some() && matches!(a.kind, AssetKind::Ebx) {
                        *out = Some(a);
                        return;
                    }
                }
            }
            for c in &node.children {
                first_sha_asset(c, assets, out);
            }
        }
        first_sha_asset(first, &index.assets, &mut sample);
        let sample = sample.expect("ebx sample");
        let bytes = index.extract_bytes(sample).expect("extract ebx");
        assert!(!bytes.is_empty(), "extracted ebx empty");
    }

    #[test]
    fn en_toc_noncas_chunk_extracts() {
        let data = PathBuf::from(
            r"D:\_DATA\Projects\RE\Command and Conquer\Bin\Command & Conquer\Data",
        );
        // CAS extract tests need a catalog; non-CAS path still opens without it.
        if !data.join("cas.cat").is_file() && !data.join("layout.toc").is_file() {
            return;
        }
        let progress = Arc::new(Mutex::new(OpenProgress::default()));
        let mut index = DataIndex::open(data, progress).expect("open");

        fn find_en(node: &TreeNode) -> Option<String> {
            if matches!(node.kind, TreeNodeKind::Toc) {
                if let Some(path) = &node.path {
                    let s = path.to_string_lossy().replace('\\', "/");
                    if s.ends_with("/Win32/Loc/en.toc") {
                        return Some(node.id.clone());
                    }
                }
            }
            for c in &node.children {
                if let Some(id) = find_en(c) {
                    return Some(id);
                }
            }
            None
        }

        let toc_id = find_en(&index.root).expect("en.toc");
        index.ensure_expanded(&toc_id).expect("expand");
        let toc = index.find_node(&toc_id).expect("node");
        let chunks = toc
            .children
            .iter()
            .find(|c| c.label.eq_ignore_ascii_case("CHUNKS"))
            .expect("CHUNKS group");
        let leaf = chunks.children.first().expect("chunk leaf");
        let asset = index
            .get_asset(leaf.asset_id.as_deref().unwrap())
            .expect("asset");
        assert!(asset.payload_offset.is_some() || asset.sha1.is_some());
        let bytes = index.extract_bytes(asset).expect("extract chunk");
        assert!(bytes.len() > 16, "chunk too small: {}", bytes.len());
    }
}

