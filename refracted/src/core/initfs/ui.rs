//! egui Initfs editor / extractor / syntax-aware viewer.

use super::codec::{
    format_aes_key, load_initfs, parse_aes_key_hex, save_initfs, LoadedInitfs, DEFAULT_AES_KEY,
};
use super::payload::{
    add_payload, apply_payload_bytes, collect_payloads, normalize_newlines, platform_from_path,
    remove_payload, rename_payload, render_hex, render_hex_text, safe_export_path, ListViewMode,
    Payload, SortMode, ViewMode,
};
use super::syntax::highlight;
use egui::{Color32, FontId, RichText, Vec2};
use std::collections::BTreeSet;
use std::path::PathBuf;

const HEX_VIEW_LIMIT: usize = 512 * 1024;

#[derive(Default)]
pub struct InitfsState {
    pub loaded: Option<LoadedInitfs>,
    payloads: Vec<Payload>,
    selected: Option<usize>,
    search: String,
    sort: SortMode,
    list_view: ListViewMode,
    view_mode: ViewMode,
    editor: String,
    status: String,
    error: Option<String>,
    find_open: bool,
    find_query: String,
    find_replace: String,
    find_match_case: bool,
    aes_prompt: Option<PathBuf>,
    aes_input: String,
    add_open: bool,
    add_name: String,
    add_content: String,
    rename_open: bool,
    rename_to: String,
    recent: Vec<PathBuf>,
    active_exts: BTreeSet<String>,
    show_all_exts: bool,
    log: Vec<String>,
    pending_key: Option<[u8; 16]>,
}

impl InitfsState {
    fn log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
        if self.log.len() > 200 {
            let extra = self.log.len() - 200;
            self.log.drain(..extra);
        }
    }

    fn stored_keys(&self) -> Vec<[u8; 16]> {
        load_stored_keys()
    }

    fn flush_editor(&mut self) {
        let Some(idx) = self.selected else {
            return;
        };
        let Some(p) = self.payloads.get_mut(idx) else {
            return;
        };
        if p.read_only || self.view_mode != ViewMode::Text || !p.is_text {
            return;
        }
        if self.editor == p.text {
            return;
        }
        p.text = self.editor.clone();
        p.bytes = normalize_newlines(&p.text, &p.orig_bytes);
        if let Some(loaded) = self.loaded.as_mut() {
            if let Err(e) = apply_payload_bytes(&mut loaded.root, idx, p.bytes.clone()) {
                self.error = Some(e);
            }
        }
    }

    fn select(&mut self, idx: usize) {
        if self.selected == Some(idx) {
            return;
        }
        self.flush_editor();
        self.selected = Some(idx);
        self.reload_editor();
    }

    fn reload_editor(&mut self) {
        let Some(idx) = self.selected else {
            self.editor.clear();
            return;
        };
        let Some(p) = self.payloads.get(idx) else {
            self.editor.clear();
            return;
        };
        self.editor = match self.view_mode {
            ViewMode::Text => p.text.clone(),
            ViewMode::Hex => {
                let slice = if p.bytes.len() > HEX_VIEW_LIMIT {
                    &p.bytes[..HEX_VIEW_LIMIT]
                } else {
                    &p.bytes
                };
                let mut s = render_hex(slice);
                if p.bytes.len() > HEX_VIEW_LIMIT {
                    s.push_str(&format!(
                        "\n… truncated ({} bytes, showing first {HEX_VIEW_LIMIT})\n",
                        p.bytes.len()
                    ));
                }
                s
            }
            ViewMode::HexText => {
                let slice = if p.bytes.len() > HEX_VIEW_LIMIT {
                    &p.bytes[..HEX_VIEW_LIMIT]
                } else {
                    &p.bytes
                };
                let mut s = render_hex_text(slice);
                if p.bytes.len() > HEX_VIEW_LIMIT {
                    s.push_str(&format!(
                        "\n… truncated ({} bytes, showing first {HEX_VIEW_LIMIT})\n",
                        p.bytes.len()
                    ));
                }
                s
            }
        };
    }

    fn rebuild_payloads(&mut self) {
        let sel_name = self
            .selected
            .and_then(|i| self.payloads.get(i).map(|p| p.name.clone()));
        if let Some(loaded) = &self.loaded {
            self.payloads = collect_payloads(&loaded.root);
        } else {
            self.payloads.clear();
        }
        self.selected = sel_name.and_then(|n| {
            self.payloads
                .iter()
                .position(|p| p.name.eq_ignore_ascii_case(&n))
        });
        if self.selected.is_none() && !self.payloads.is_empty() {
            self.selected = Some(0);
        }
        self.reload_editor();
    }

    fn try_open(&mut self, path: PathBuf, prompted: Option<[u8; 16]>) {
        self.error = None;
        let keys = self.stored_keys();
        match load_initfs(&path, &keys, prompted) {
            Ok(loaded) => {
                if loaded.had_encrypted {
                    if let Some(k) = loaded.aes_key {
                        store_key(k);
                    }
                }
                self.log(format!(
                    "Loaded {} ({}, {} encrypted)",
                    path.display(),
                    loaded.kind.as_str(),
                    if loaded.had_encrypted { "AES" } else { "not" }
                ));
                push_recent(&mut self.recent, path.clone());
                self.loaded = Some(loaded);
                self.aes_prompt = None;
                self.pending_key = None;
                self.rebuild_payloads();
                self.status = format!(
                    "{} · {} payload(s)",
                    self.loaded
                        .as_ref()
                        .map(|l| l.kind.as_str())
                        .unwrap_or("?"),
                    self.payloads.len()
                );
                self.show_all_exts = true;
                self.active_exts.clear();
            }
            Err(e) if e.contains("AES-wrapped") && prompted.is_none() => {
                self.aes_prompt = Some(path);
                self.aes_input.clear();
                self.error = Some("File is AES-encrypted — enter a 16-byte key (hex).".into());
            }
            Err(e) => {
                self.error = Some(e);
                self.loaded = None;
                self.payloads.clear();
            }
        }
    }

    fn open_picker(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Initfs File")
            .pick_file()
        {
            self.try_open(path, None);
        }
    }

    fn save(&mut self, save_as: bool) {
        self.flush_editor();
        let Some(loaded) = self.loaded.as_mut() else {
            self.error = Some("No initfs loaded.".into());
            return;
        };
        let dest = if save_as {
            match rfd::FileDialog::new()
                .set_file_name(
                    loaded
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("initfs"),
                )
                .save_file()
            {
                Some(p) => p,
                None => return,
            }
        } else {
            loaded.path.clone()
        };
        match save_initfs(loaded, &dest) {
            Ok(()) => {
                loaded.path = dest.clone();
                for p in &mut self.payloads {
                    p.orig_bytes = p.bytes.clone();
                    p.orig_text = p.text.clone();
                }
                self.status = format!("Saved {}", dest.display());
                self.log(self.status.clone());
                self.error = None;
            }
            Err(e) => self.error = Some(e),
        }
    }

    fn export_selected(&mut self) {
        self.flush_editor();
        let Some(idx) = self.selected else {
            self.error = Some("Select a payload first.".into());
            return;
        };
        let Some(p) = self.payloads.get(idx) else {
            return;
        };
        let name = p
            .name
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(&p.name)
            .to_string();
        if let Some(path) = rfd::FileDialog::new().set_file_name(&name).save_file() {
            if let Err(e) = std::fs::write(&path, &p.bytes) {
                self.error = Some(format!("export: {e}"));
            } else {
                self.status = format!("Exported {}", path.display());
                self.log(self.status.clone());
            }
        }
    }

    fn export_all(&mut self) {
        self.flush_editor();
        if self.payloads.is_empty() {
            self.error = Some("No payloads to export.".into());
            return;
        }
        let Some(dir) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let mut ok = 0usize;
        let mut err = 0usize;
        for p in &self.payloads {
            let dest = safe_export_path(&dir, &p.name);
            if let Some(parent) = dest.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&dest, &p.bytes) {
                Ok(()) => ok += 1,
                Err(_) => err += 1,
            }
        }
        self.status = format!("Exported {ok} payload(s) to {}", dir.display());
        if err > 0 {
            self.status.push_str(&format!(" ({err} failed)"));
        }
        self.log(self.status.clone());
    }

    fn import_payload(&mut self) {
        if self.loaded.is_none() {
            self.error = Some("Load an initfs first.".into());
            return;
        }
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.error = Some(format!("import: {e}"));
                return;
            }
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("imported.bin")
            .replace('\\', "/");
        if let Some(idx) = self.selected {
            if let Some(loaded) = self.loaded.as_mut() {
                if let Err(e) = apply_payload_bytes(&mut loaded.root, idx, bytes.clone()) {
                    self.error = Some(e);
                    return;
                }
            }
            if let Some(p) = self.payloads.get_mut(idx) {
                p.bytes = bytes.clone();
                let (text, is_text, read_only) = super::payload::payload_to_text(&p.name, &bytes);
                p.text = text;
                p.is_text = is_text;
                p.read_only = read_only;
            }
            self.reload_editor();
            self.status = format!("Imported into {}", name);
        } else if let Some(loaded) = self.loaded.as_mut() {
            if let Err(e) = add_payload(&mut loaded.root, &name, &bytes) {
                self.error = Some(e);
                return;
            }
            self.rebuild_payloads();
            self.status = format!("Added {name}");
        }
        self.log(self.status.clone());
    }

    fn filtered_indices(&self) -> Vec<usize> {
        let q = self.search.to_ascii_lowercase();
        let mut idx: Vec<usize> = self
            .payloads
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if !q.is_empty() && !p.name.to_ascii_lowercase().contains(&q) {
                    return false;
                }
                if !self.show_all_exts && !self.active_exts.is_empty() {
                    if !self.active_exts.contains(&p.ext()) {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();
        match self.sort {
            SortMode::Default => {}
            SortMode::Az => idx.sort_by(|&a, &b| self.payloads[a].name.cmp(&self.payloads[b].name)),
            SortMode::Za => idx.sort_by(|&a, &b| self.payloads[b].name.cmp(&self.payloads[a].name)),
            SortMode::BigSmall => {
                idx.sort_by(|&a, &b| self.payloads[b].bytes.len().cmp(&self.payloads[a].bytes.len()))
            }
            SortMode::SmallBig => {
                idx.sort_by(|&a, &b| self.payloads[a].bytes.len().cmp(&self.payloads[b].bytes.len()))
            }
        }
        idx
    }

    fn find_next(&mut self) {
        if self.find_query.is_empty() {
            return;
        }
        let q = if self.find_match_case {
            self.find_query.clone()
        } else {
            self.find_query.to_ascii_lowercase()
        };
        let hay = if self.find_match_case {
            self.editor.clone()
        } else {
            self.editor.to_ascii_lowercase()
        };
        if let Some(pos) = hay.find(&q) {
            self.status = format!("Found at byte offset {pos}");
        } else {
            self.status = "Not found in current payload".into();
        }
    }

    fn replace_all_current(&mut self) {
        if self.find_query.is_empty() {
            return;
        }
        if self.view_mode != ViewMode::Text {
            self.error = Some("Replace is only available in Text view.".into());
            return;
        }
        let count = if self.find_match_case {
            self.editor.matches(&self.find_query).count()
        } else {
            self.editor
                .to_ascii_lowercase()
                .matches(&self.find_query.to_ascii_lowercase())
                .count()
        };
        if self.find_match_case {
            self.editor = self.editor.replace(&self.find_query, &self.find_replace);
        } else {
            // case-insensitive replace: walk matches
            let q = self.find_query.to_ascii_lowercase();
            let mut out = String::new();
            let lower = self.editor.to_ascii_lowercase();
            let mut last = 0usize;
            let mut search = 0usize;
            while let Some(rel) = lower[search..].find(&q) {
                let abs = search + rel;
                out.push_str(&self.editor[last..abs]);
                out.push_str(&self.find_replace);
                last = abs + self.find_query.len();
                search = last;
            }
            out.push_str(&self.editor[last..]);
            self.editor = out;
        }
        self.status = format!("Replaced {count} occurrence(s)");
        self.flush_editor();
    }

    fn close_file(&mut self) {
        self.flush_editor();
        self.loaded = None;
        self.payloads.clear();
        self.selected = None;
        self.editor.clear();
        self.status = "Closed".into();
        self.error = None;
    }
}

fn keys_dir() -> PathBuf {
    if let Ok(base) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(base).join("Refracted").join("initfs-keys");
    }
    PathBuf::from("data").join("initfs-keys")
}

fn load_stored_keys() -> Vec<[u8; 16]> {
    let dir = keys_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut keys = Vec::new();
    for ent in rd.flatten() {
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("key") {
            continue;
        }
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(k) = parse_aes_key_hex(&s) {
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
        }
    }
    keys
}

fn store_key(key: [u8; 16]) {
    let dir = keys_dir();
    let _ = std::fs::create_dir_all(&dir);
    for existing in load_stored_keys() {
        if existing == key {
            return;
        }
    }
    let name = format!(
        "{}.key",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let _ = std::fs::write(dir.join(name), format_aes_key(&key));
}

fn push_recent(recent: &mut Vec<PathBuf>, path: PathBuf) {
    recent.retain(|p| p != &path);
    recent.insert(0, path);
    recent.truncate(10);
}

pub fn render_initfs(ui: &mut egui::Ui, state: &mut InitfsState) {
    render_aes_prompt(ui.ctx(), state);
    render_add_dialog(ui.ctx(), state);
    render_rename_dialog(ui.ctx(), state);

    ui.horizontal(|ui| {
        if ui.button("Open").on_hover_text("Load an initfs_* file").clicked() {
            state.open_picker();
        }
        let loaded = state.loaded.is_some();
        if ui.add_enabled(loaded, egui::Button::new("Save")).clicked() {
            state.save(false);
        }
        if ui.add_enabled(loaded, egui::Button::new("Save As")).clicked() {
            state.save(true);
        }
        if ui
            .add_enabled(loaded, egui::Button::new("Export"))
            .on_hover_text("Export selected payload")
            .clicked()
        {
            state.export_selected();
        }
        if ui
            .add_enabled(loaded, egui::Button::new("Export All"))
            .on_hover_text("Extract every payload into a folder")
            .clicked()
        {
            state.export_all();
        }
        if ui
            .add_enabled(loaded, egui::Button::new("Import"))
            .on_hover_text("Replace selected payload (or add if none selected)")
            .clicked()
        {
            state.import_payload();
        }
        if ui.add_enabled(loaded, egui::Button::new("Find")).clicked() {
            state.find_open = !state.find_open;
        }
        if ui.add_enabled(loaded, egui::Button::new("Close")).clicked() {
            state.close_file();
        }
        ui.separator();
        if let Some(loaded) = &state.loaded {
            ui.add(
                egui::Label::new(loaded.path.display().to_string())
                    .truncate(true)
                    .sense(egui::Sense::hover()),
            )
            .on_hover_text(loaded.path.display().to_string());
        } else {
            ui.label("No initfs loaded");
        }
    });

    if state.find_open {
        ui.horizontal(|ui| {
            ui.label("Find");
            ui.add(
                egui::TextEdit::singleline(&mut state.find_query)
                    .desired_width(160.0)
                    .hint_text("text"),
            );
            ui.label("Replace");
            ui.add(egui::TextEdit::singleline(&mut state.find_replace).desired_width(120.0));
            ui.checkbox(&mut state.find_match_case, "Case");
            if ui.button("Next").clicked() {
                state.find_next();
            }
            if ui.button("Replace all").clicked() {
                state.replace_all_current();
            }
        });
    }

    if let Some(err) = &state.error {
        ui.colored_label(Color32::from_rgb(220, 80, 80), err);
    }
    if !state.status.is_empty() {
        ui.label(RichText::new(&state.status).weak());
    }

    ui.separator();

    let row_w = ui.available_width().max(1.0);
    let mut fill_h = ui.available_height();
    if !fill_h.is_finite() || fill_h < 8.0 {
        fill_h = (ui.ctx().screen_rect().height() * 0.55).clamp(200.0, 2000.0);
    }
    let body_h = (fill_h - 28.0).max(120.0);
    let gap = ui.spacing().item_spacing.x.max(6.0);
    let usable = (row_w - gap).max(200.0);
    let left_w = (usable * 0.34).clamp(220.0, 480.0);
    let right_w = (usable - left_w).max(180.0);

    ui.allocate_ui_with_layout(
        Vec2::new(row_w, body_h),
        egui::Layout::left_to_right(egui::Align::Min),
        |ui| {
            ui.set_min_height(body_h);
            ui.set_max_height(body_h);
            ui.spacing_mut().item_spacing.x = gap;

            ui.allocate_ui_with_layout(
                Vec2::new(left_w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(left_w);
                    ui.set_max_width(left_w);
                    ui.set_min_height(body_h);
                    ui.set_max_height(body_h);
                    render_payload_list(ui, state);
                },
            );

            ui.separator();

            ui.allocate_ui_with_layout(
                Vec2::new(right_w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(right_w);
                    ui.set_max_width(right_w);
                    ui.set_min_height(body_h);
                    ui.set_max_height(body_h);
                    render_editor(ui, state);
                },
            );
        },
    );

    ui.separator();
    render_footer(ui, state);
}

fn render_payload_list(ui: &mut egui::Ui, state: &mut InitfsState) {
    ui.horizontal(|ui| {
        ui.heading("Payloads");
        if ui
            .small_button(state.list_view.label())
            .on_hover_text("Cycle names / tree / folder")
            .clicked()
        {
            state.list_view = state.list_view.cycle();
        }
        egui::ComboBox::from_id_source("initfs_sort")
            .selected_text(match state.sort {
                SortMode::Default => "Default",
                SortMode::Az => "A–Z",
                SortMode::Za => "Z–A",
                SortMode::BigSmall => "Large",
                SortMode::SmallBig => "Small",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.sort, SortMode::Default, "Default");
                ui.selectable_value(&mut state.sort, SortMode::Az, "A–Z");
                ui.selectable_value(&mut state.sort, SortMode::Za, "Z–A");
                ui.selectable_value(&mut state.sort, SortMode::BigSmall, "Large first");
                ui.selectable_value(&mut state.sort, SortMode::SmallBig, "Small first");
            });
    });
    ui.add(
        egui::TextEdit::singleline(&mut state.search)
            .hint_text("Filter payloads…")
            .desired_width(f32::INFINITY),
    );

    let exts: BTreeSet<String> = state.payloads.iter().map(|p| p.ext()).collect();
    if !exts.is_empty() {
        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(state.show_all_exts, "all")
                .clicked()
            {
                state.show_all_exts = true;
                state.active_exts.clear();
            }
            for ext in &exts {
                let on = !state.show_all_exts && state.active_exts.contains(ext);
                if ui.selectable_label(on, ext).clicked() {
                    state.show_all_exts = false;
                    if !state.active_exts.remove(ext) {
                        state.active_exts.insert(ext.clone());
                    }
                    if state.active_exts.is_empty() {
                        state.show_all_exts = true;
                    }
                }
            }
        });
    }

    ui.horizontal(|ui| {
        let enabled = state.loaded.is_some();
        if ui.add_enabled(enabled, egui::Button::new("Add")).clicked() {
            state.add_open = true;
            state.add_name.clear();
            state.add_content.clear();
        }
        if ui
            .add_enabled(state.selected.is_some(), egui::Button::new("Rename"))
            .clicked()
        {
            if let Some(i) = state.selected {
                state.rename_to = state.payloads.get(i).map(|p| p.name.clone()).unwrap_or_default();
                state.rename_open = true;
            }
        }
        if ui
            .add_enabled(state.selected.is_some(), egui::Button::new("Revert"))
            .clicked()
        {
            if let Some(i) = state.selected {
                if let Some(p) = state.payloads.get_mut(i) {
                    p.bytes = p.orig_bytes.clone();
                    p.text = p.orig_text.clone();
                    if let Some(loaded) = state.loaded.as_mut() {
                        let _ = apply_payload_bytes(&mut loaded.root, i, p.bytes.clone());
                    }
                }
                state.reload_editor();
            }
        }
        if ui
            .add_enabled(state.selected.is_some(), egui::Button::new("Remove"))
            .clicked()
        {
            if let Some(i) = state.selected {
                if let Some(loaded) = state.loaded.as_mut() {
                    if let Err(e) = remove_payload(&mut loaded.root, i) {
                        state.error = Some(e);
                    } else {
                        state.rebuild_payloads();
                    }
                }
            }
        }
    });

    ui.separator();
    let indices = state.filtered_indices();
    let tree_h = ui.available_height().max(64.0);
    egui::ScrollArea::both()
        .id_source("initfs_payload_list")
        .auto_shrink([false, false])
        .max_height(tree_h)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            if state.loaded.is_none() {
                ui.label("Open an initfs_* file (PVZ / BF3 / DA / MEA / AES).");
                if !state.recent.is_empty() {
                    ui.separator();
                    ui.label(RichText::new("Recent").weak());
                    let recent = state.recent.clone();
                    for p in recent {
                        if ui.link(p.display().to_string()).clicked() {
                            state.try_open(p, None);
                            break;
                        }
                    }
                }
                return;
            }
            match state.list_view {
                ListViewMode::Names => {
                    for i in indices {
                        render_payload_row(ui, state, i, false);
                    }
                }
                ListViewMode::Tree | ListViewMode::Folder => {
                    render_tree(ui, state, &indices);
                }
            }
        });
}

fn render_payload_row(ui: &mut egui::Ui, state: &mut InitfsState, i: usize, leaf_only: bool) {
    let Some(p) = state.payloads.get(i) else {
        return;
    };
    let selected = state.selected == Some(i);
    let dirty = p.dirty();
    let shown = if leaf_only {
        p.name
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(&p.name)
            .to_string()
    } else {
        p.name.clone()
    };
    let label = if dirty {
        format!("*{shown}")
    } else {
        shown
    };
    let mut rt = RichText::new(label).monospace();
    if dirty {
        rt = rt.italics().strong();
    }
    let response = ui.selectable_label(selected, rt);
    if response.clicked() {
        state.select(i);
    }
    response.context_menu(|ui| {
        if ui.button("Copy name").clicked() {
            ui.output_mut(|o| {
                o.copied_text = state
                    .payloads
                    .get(i)
                    .map(|p| p.name.clone())
                    .unwrap_or_default()
            });
            ui.close_menu();
        }
        if ui.button("Export").clicked() {
            state.selected = Some(i);
            state.export_selected();
            ui.close_menu();
        }
        if ui.button("Revert").clicked() {
            if let Some(p) = state.payloads.get_mut(i) {
                p.bytes = p.orig_bytes.clone();
                p.text = p.orig_text.clone();
                if let Some(loaded) = state.loaded.as_mut() {
                    let _ = apply_payload_bytes(&mut loaded.root, i, p.bytes.clone());
                }
            }
            if state.selected == Some(i) {
                state.reload_editor();
            }
            ui.close_menu();
        }
        if ui.button("Remove").clicked() {
            if let Some(loaded) = state.loaded.as_mut() {
                if let Err(e) = remove_payload(&mut loaded.root, i) {
                    state.error = Some(e);
                } else {
                    state.rebuild_payloads();
                }
            }
            ui.close_menu();
        }
    });
}

fn render_tree(ui: &mut egui::Ui, state: &mut InitfsState, indices: &[usize]) {
    render_tree_level(ui, state, indices, 0);
}

fn path_segments(name: &str) -> Vec<&str> {
    name.split(['/', '\\']).filter(|s| !s.is_empty()).collect()
}

fn render_tree_level(ui: &mut egui::Ui, state: &mut InitfsState, indices: &[usize], depth: usize) {
    let mut folders: Vec<(String, Vec<usize>)> = Vec::new();
    let mut files: Vec<usize> = Vec::new();
    for &i in indices {
        let segs = path_segments(&state.payloads[i].name);
        if segs.len() > depth + 1 {
            let head = segs[depth].to_string();
            if let Some(slot) = folders.iter_mut().find(|(h, _)| *h == head) {
                slot.1.push(i);
            } else {
                folders.push((head, vec![i]));
            }
        } else {
            files.push(i);
        }
    }
    folders.sort_by(|a, b| a.0.cmp(&b.0));
    for (folder, kids) in folders {
        egui::CollapsingHeader::new(RichText::new(&folder).strong())
            .id_source(("initfs_folder", depth, folder.clone()))
            .default_open(depth < 2)
            .show(ui, |ui| {
                render_tree_level(ui, state, &kids, depth + 1);
            });
    }
    for i in files {
        render_payload_row(ui, state, i, true);
    }
}

fn render_editor(ui: &mut egui::Ui, state: &mut InitfsState) {
    ui.horizontal(|ui| {
        let name = state
            .selected
            .and_then(|i| state.payloads.get(i).map(|p| p.name.as_str()))
            .unwrap_or("—");
        ui.heading("Contents");
        ui.label(RichText::new(name).weak().monospace());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(state.view_mode.label()).clicked() {
                state.flush_editor();
                state.view_mode = state.view_mode.cycle();
                state.reload_editor();
            }
            if let Some(i) = state.selected {
                if let Some(p) = state.payloads.get(i) {
                    if p.read_only {
                        ui.colored_label(Color32::from_rgb(180, 140, 60), "read-only");
                    } else if state.view_mode != ViewMode::Text {
                        ui.colored_label(Color32::from_rgb(140, 140, 140), "view");
                    }
                }
            }
        });
    });
    ui.separator();

    let name = state
        .selected
        .and_then(|i| state.payloads.get(i).map(|p| p.name.clone()))
        .unwrap_or_default();
    let read_only = state.view_mode != ViewMode::Text
        || state
            .selected
            .and_then(|i| state.payloads.get(i).map(|p| p.read_only || !p.is_text))
            .unwrap_or(true);

    let dark = ui.visuals().dark_mode;
    let font = FontId::monospace(13.0);
    let mode = state.view_mode;
    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
        let mut job = highlight(text, &name, mode, dark, font.clone());
        job.wrap.max_width = wrap_width;
        ui.fonts(|f| f.layout_job(job))
    };

    let editor_h = ui.available_height().max(64.0);
    egui::ScrollArea::both()
        .id_source("initfs_editor")
        .auto_shrink([false, false])
        .max_height(editor_h)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut state.editor)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(32)
                    .layouter(&mut layouter)
                    .interactive(!read_only && state.loaded.is_some()),
            );
        });
    if !read_only {
        state.flush_editor();
    }
}

fn render_footer(ui: &mut egui::Ui, state: &InitfsState) {
    ui.horizontal(|ui| {
        if let Some(loaded) = &state.loaded {
            ui.label(format!("Loaded: {}", loaded.path.file_name().and_then(|n| n.to_str()).unwrap_or("?")));
            ui.separator();
            ui.label(format!("Type: {}", loaded.kind.as_str()));
            ui.separator();
            ui.label(format!(
                "Platform: {}",
                platform_from_path(&loaded.path.to_string_lossy())
            ));
            if loaded.had_encrypted {
                ui.separator();
                ui.label("AES");
            }
        } else {
            ui.label("Loaded: —");
        }
        ui.separator();
        if let Some(i) = state.selected {
            if let Some(p) = state.payloads.get(i) {
                ui.label(format!("Editing: {}", p.name));
                ui.separator();
                let changed = if p.dirty() {
                    p.bytes.len() as i64 - p.orig_bytes.len() as i64
                } else {
                    0
                };
                ui.label(format!("Changed: {changed:+} bytes · {} bytes", p.bytes.len()));
            }
        } else {
            ui.label("Editing: —");
        }
        ui.separator();
        ui.label(format!("{} payload(s)", state.payloads.len()));
    });
}

fn render_aes_prompt(ctx: &egui::Context, state: &mut InitfsState) {
    let Some(path) = state.aes_prompt.clone() else {
        return;
    };
    egui::Window::new("AES key")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Encrypted initfs:\n{}",
                path.display()
            ));
            ui.label("32 hex characters (16 bytes). Default InitfsTools key is tried first.");
            ui.add(
                egui::TextEdit::singleline(&mut state.aes_input)
                    .hint_text("0102030405060708090A0B0C0D0E0F10")
                    .desired_width(280.0),
            );
            ui.horizontal(|ui| {
                if ui.button("Use default key").clicked() {
                    state.aes_input = format_aes_key(&DEFAULT_AES_KEY);
                }
                if ui.button("Open").clicked() {
                    match parse_aes_key_hex(&state.aes_input) {
                        Ok(k) => {
                            state.pending_key = Some(k);
                            state.aes_prompt = None;
                            state.try_open(path.clone(), Some(k));
                        }
                        Err(e) => state.error = Some(e),
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.aes_prompt = None;
                    state.error = None;
                }
            });
        });
}

fn render_add_dialog(ctx: &egui::Context, state: &mut InitfsState) {
    if !state.add_open {
        return;
    }
    egui::Window::new("Add payload")
        .collapsible(false)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.add(
                    egui::TextEdit::singleline(&mut state.add_name)
                        .hint_text("Scripts/Foo.lua")
                        .desired_width(280.0),
                );
            });
            ui.label("Content");
            ui.add(
                egui::TextEdit::multiline(&mut state.add_content)
                    .desired_width(f32::INFINITY)
                    .desired_rows(12)
                    .font(egui::TextStyle::Monospace),
            );
            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    let name = state.add_name.trim().replace('\\', "/");
                    if name.is_empty() {
                        state.error = Some("Name required.".into());
                    } else if let Some(loaded) = state.loaded.as_mut() {
                        match add_payload(&mut loaded.root, &name, state.add_content.as_bytes()) {
                            Ok(()) => {
                                state.add_open = false;
                                state.rebuild_payloads();
                                if let Some(i) = state.payloads.iter().position(|p| p.name == name) {
                                    state.select(i);
                                }
                                state.status = format!("Added {name}");
                            }
                            Err(e) => state.error = Some(e),
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.add_open = false;
                }
            });
        });
}

fn render_rename_dialog(ctx: &egui::Context, state: &mut InitfsState) {
    if !state.rename_open {
        return;
    }
    egui::Window::new("Rename payload")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.add(egui::TextEdit::singleline(&mut state.rename_to).desired_width(280.0));
            ui.horizontal(|ui| {
                if ui.button("Rename").clicked() {
                    let new_name = state.rename_to.trim().replace('\\', "/");
                    if let Some(i) = state.selected {
                        if let Some(loaded) = state.loaded.as_mut() {
                            match rename_payload(&mut loaded.root, i, &new_name) {
                                Ok(()) => {
                                    state.rename_open = false;
                                    state.rebuild_payloads();
                                    state.status = format!("Renamed to {new_name}");
                                }
                                Err(e) => state.error = Some(e),
                            }
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.rename_open = false;
                }
            });
        });
}

