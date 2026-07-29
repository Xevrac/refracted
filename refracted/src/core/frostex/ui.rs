//! egui front-end for FrostEx.

use crate::core::frostex::archetype::Archetype;
use crate::core::frostex::ebx::{register_ebx_guid, EbxGuidTable};
use crate::core::frostex::icons::{FrostIcon, IconAtlas};
use crate::core::frostex::index::{AssetKind, AssetRef, DataIndex, OpenJob, TreeNode, TreeNodeKind};
use crate::core::frostex::preview::{format_size, needs_heavy_preview, PreviewState};
use crate::core::frostex::preview_ctx::PreviewCtx;
use crate::core::frostex::rip::{
    prepare_rip, rip_relative_path, run_full_dump, DumpOptions, DumpProgress, DumpReport,
    DUMP_SIZE_LIMIT,
};
use egui::{ColorImage, TextureHandle, Vec2};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

const RIP_LIMIT: u64 = DUMP_SIZE_LIMIT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewTab {
    Info,
    Text,
    Hex,
    Visual,
}

#[derive(Debug, Clone)]
struct PreviewJob {
    asset_id: String,
    result: Arc<Mutex<Option<PreviewState>>>,
}

#[derive(Debug, Clone, Default)]
struct RipProgress {
    done: usize,
    total: usize,
    phase: String,
    errors: usize,
    finished: Option<String>,
}

struct RipJob {
    progress: Arc<Mutex<RipProgress>>,
}

struct DumpJob {
    progress: Arc<Mutex<DumpProgress>>,
}

struct PendingDump {
    data_dir: PathBuf,
    options: DumpOptions,
}

pub struct FrostExState {
    data_dir: Option<PathBuf>,
    index: Option<DataIndex>,
    open_job: Option<OpenJob>,
    open_error: Option<String>,
    selected_asset_id: Option<String>,
    selected_node_id: Option<String>,
    preview: PreviewState,
    preview_job: Option<PreviewJob>,
    preview_tab: PreviewTab,
    texture: Option<TextureHandle>,
    texture_asset_id: Option<String>,
    rip_status: Option<String>,
    rip_job: Option<RipJob>,
    dump_job: Option<DumpJob>,
    pending_dump: Option<PendingDump>,
    dump_report: Option<DumpReport>,
    show_dump_report: bool,
    icons: IconAtlas,
    /// Shared GUID→Name for IceBloc-shaped EBX Class links (preview + rip).
    ebx_guid_table: Arc<Mutex<EbxGuidTable>>,
    ebx_guid_built_for: usize,
}

impl Default for FrostExState {
    fn default() -> Self {
        Self {
            data_dir: None,
            index: None,
            open_job: None,
            open_error: None,
            selected_asset_id: None,
            selected_node_id: None,
            preview: PreviewState::default(),
            preview_job: None,
            preview_tab: PreviewTab::Info,
            texture: None,
            texture_asset_id: None,
            rip_status: None,
            rip_job: None,
            dump_job: None,
            pending_dump: None,
            dump_report: None,
            show_dump_report: false,
            icons: IconAtlas::default(),
            ebx_guid_table: Arc::new(Mutex::new(EbxGuidTable::new())),
            ebx_guid_built_for: 0,
        }
    }
}

pub fn render_frostex(ui: &mut egui::Ui, state: &mut FrostExState) {
    state.icons.ensure(ui.ctx());
    state.poll_open();
    state.poll_preview();
    state.poll_rip();
    state.poll_dump();

    // Top chrome — keep compact so the split body gets the rest of the height.
    ui.horizontal(|ui| {
        if ui.button("Open data folder").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.start_open(path);
            }
        }
        let dump_busy = state.dump_job.is_some() || state.rip_job.is_some();
        let can_dump = state.index.is_some() && state.data_dir.is_some() && !dump_busy;
        if ui
            .add_enabled(can_dump, egui::Button::new("Full Dump"))
            .on_hover_text("Choose what to export, then pick a folder for frostex/")
            .clicked()
        {
            state.open_full_dump_options();
        }
        if state.dump_report.is_some() && ui.button("Dump Report").clicked() {
            state.show_dump_report = true;
        }
        ui.separator();
        if let Some(dir) = &state.data_dir {
            ui.add(
                egui::Label::new(dir.display().to_string())
                    .truncate(true)
                    .sense(egui::Sense::hover()),
            )
            .on_hover_text(dir.display().to_string());
        } else {
            ui.label("No data folder open");
        }
        if state.open_job.is_some() {
            ui.spinner();
            if let Some(job) = &state.open_job {
                let p = job.progress();
                ui.add(
                    egui::Label::new(format!("{} {}/{}", p.phase, p.done, p.total)).truncate(true),
                );
            }
        }
    });

    if let Some(err) = &state.open_error {
        ui.colored_label(egui::Color32::RED, err);
    }
    if let Some(job) = &state.dump_job {
        let p = job.progress.lock().clone();
        ui.horizontal(|ui| {
            ui.spinner();
            ui.add(
                egui::Label::new(format!(
                    "Full dump: {} {}/{} (wrote {} · fail {} · skip {})",
                    p.phase, p.done, p.total, p.wrote, p.failed, p.skipped
                ))
                .truncate(true),
            );
        });
        ui.ctx().request_repaint();
    } else if let Some(job) = &state.rip_job {
        let p = job.progress.lock().clone();
        ui.horizontal(|ui| {
            ui.spinner();
            ui.add(
                egui::Label::new(format!(
                    "Rip: {} {}/{} ({} err)",
                    p.phase, p.done, p.total, p.errors
                ))
                .truncate(true),
            );
        });
        ui.ctx().request_repaint();
    } else if let Some(status) = &state.rip_status {
        ui.add(egui::Label::new(status.as_str()).truncate(true));
    }

    render_dump_report_window(ui.ctx(), state);
    render_dump_options_window(ui.ctx(), state);

    ui.separator();

    // Constrained split: left tree | right preview. Avoids egui columns overflow.
    let row_w = ui.available_width().max(1.0);
    let mut fill_h = ui.available_height();
    if !fill_h.is_finite() || fill_h < 8.0 {
        fill_h = (ui.ctx().screen_rect().height() * 0.55).clamp(200.0, 2000.0);
    }
    let body_h = fill_h.max(120.0);
    let gap = ui.spacing().item_spacing.x.max(6.0);
    let usable = (row_w - gap).max(200.0);
    let left_w = (usable * 0.40).clamp(220.0, 520.0);
    let right_w = (usable - left_w).max(180.0);

    ui.allocate_ui_with_layout(
        Vec2::new(row_w, body_h),
        egui::Layout::left_to_right(egui::Align::Min),
        |ui| {
            ui.set_min_height(body_h);
            ui.set_max_height(body_h);
            ui.spacing_mut().item_spacing.x = gap;

            // —— Data tree ——
            ui.allocate_ui_with_layout(
                Vec2::new(left_w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(left_w);
                    ui.set_max_width(left_w);
                    ui.set_min_height(body_h);
                    ui.set_max_height(body_h);

                    ui.heading("Data");
                    ui.separator();
                    let tree_h = ui.available_height().max(64.0);
                    egui::ScrollArea::both()
                        .id_source("frostex_tree")
                        .auto_shrink([false, false])
                        .max_height(tree_h)
                        .show(ui, |ui| {
                            ui.set_min_width((left_w - 16.0).max(80.0));
                            if let Some(index) = &state.index {
                                let root = index.root.clone();
                                state.render_node(ui, &root, 0);
                            } else {
                                ui.label("Open a Frostbite Data folder (cas.cat optional).");
                            }
                        });
                },
            );

            ui.separator();

            // —— Preview ——
            ui.allocate_ui_with_layout(
                Vec2::new(right_w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(right_w);
                    ui.set_max_width(right_w);
                    ui.set_min_height(body_h);
                    ui.set_max_height(body_h);

                    ui.heading("Preview");
                    state.render_preview_tabs(ui);
                    ui.separator();
                    let preview_h = ui.available_height().max(64.0);
                    egui::ScrollArea::both()
                        .id_source("frostex_preview_body")
                        .auto_shrink([false, false])
                        .max_height(preview_h)
                        .show(ui, |ui| {
                            ui.set_min_width((right_w - 16.0).max(80.0));
                            state.render_preview(ui);
                        });
                },
            );
        },
    );
}

fn render_dump_report_window(ctx: &egui::Context, state: &mut FrostExState) {
    if !state.show_dump_report {
        return;
    }
    let Some(report) = state.dump_report.clone() else {
        state.show_dump_report = false;
        return;
    };

    let mut open = state.show_dump_report;
    let mut close_clicked = false;
    egui::Window::new("FrostEx Dump Report")
        .open(&mut open)
        .default_size([560.0, 420.0])
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            if let Some(err) = &report.error {
                ui.colored_label(egui::Color32::RED, err);
            }
            ui.label(report.summary_line());
            ui.label(format!("Output: {}", report.out_dir.display()));
            if let Some(path) = &report.report_path {
                ui.label(format!("Report file: {}", path.display()));
            }
            ui.label(format!(
                "Wrote {} · Failed {} · Skipped {} · {}",
                report.wrote,
                report.failed,
                report.skipped,
                format_size(report.bytes_written)
            ));
            if !report.by_kind.is_empty() {
                ui.separator();
                ui.label("Wrote by kind:");
                ui.horizontal_wrapped(|ui| {
                    for (k, n) in &report.by_kind {
                        ui.label(format!("{k}: {n}"));
                    }
                });
            }

            ui.separator();
            let fails: Vec<_> = report.entries.iter().filter(|e| !e.ok).collect();
            ui.heading(format!("Failures ({})", fails.len()));
            egui::ScrollArea::vertical()
                .id_source("frostex_dump_fails")
                .max_height(180.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if fails.is_empty() {
                        ui.label("None — all attempted assets dumped.");
                    } else {
                        for e in fails {
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::LIGHT_RED, "FAIL");
                                ui.add(
                                    egui::Label::new(format!("{} — {}", e.rel_path, e.detail))
                                        .truncate(true),
                                );
                            });
                        }
                    }
                });

            ui.separator();
            ui.collapsing(format!("All files ({})", report.entries.len()), |ui| {
                egui::ScrollArea::vertical()
                    .id_source("frostex_dump_all")
                    .max_height(220.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for e in &report.entries {
                            let tag = if e.ok { "OK" } else { "FAIL" };
                            let color = if e.ok {
                                egui::Color32::LIGHT_GREEN
                            } else {
                                egui::Color32::LIGHT_RED
                            };
                            ui.horizontal(|ui| {
                                ui.colored_label(color, tag);
                                ui.add(
                                    egui::Label::new(format!(
                                        "[{}] {}{}",
                                        e.kind,
                                        e.rel_path,
                                        if e.ok {
                                            format!(" ({})", format_size(e.bytes))
                                        } else {
                                            format!(" — {}", e.detail)
                                        }
                                    ))
                                    .truncate(true),
                                );
                            });
                        }
                    });
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Open output folder").clicked() {
                    reveal_in_file_manager(&report.out_dir);
                }
                if let Some(path) = &report.report_path {
                    if ui.button("Open report file").clicked() {
                        reveal_in_file_manager(path);
                    }
                }
                if ui.button("Close").clicked() {
                    close_clicked = true;
                }
            });
        });

    state.show_dump_report = open && !close_clicked;
}

fn render_dump_options_window(ctx: &egui::Context, state: &mut FrostExState) {
    let Some(pending) = state.pending_dump.as_mut() else {
        return;
    };

    let mut open = true;
    let mut start = false;
    let mut cancel = false;
    egui::Window::new("FrostEx Export Options")
        .open(&mut open)
        .default_width(320.0)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label("Choose which asset families to export:");
            ui.separator();
            ui.checkbox(&mut pending.options.toc, "TOC");
            ui.checkbox(&mut pending.options.sb, "SB");
            ui.checkbox(&mut pending.options.ebx, "EBX");
            ui.checkbox(&mut pending.options.res, "RES");
            ui.checkbox(&mut pending.options.chunk, "Chunks");
            ui.checkbox(&mut pending.options.file, "Loose files");
            ui.separator();

            let any = pending.options.toc
                || pending.options.sb
                || pending.options.ebx
                || pending.options.res
                || pending.options.chunk
                || pending.options.file;
            if !any {
                ui.colored_label(egui::Color32::YELLOW, "Pick at least one asset type.");
            }

            ui.horizontal(|ui| {
                if ui.add_enabled(any, egui::Button::new("Choose Folder")).clicked() {
                    start = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
        });

    if !open || cancel {
        state.pending_dump = None;
        return;
    }
    if start {
        state.start_full_dump_from_pending();
    }
}

fn reveal_in_file_manager(path: &Path) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

impl FrostExState {
    fn start_open(&mut self, data_dir: PathBuf) {
        self.data_dir = Some(data_dir.clone());
        self.index = None;
        self.open_error = None;
        self.selected_asset_id = None;
        self.selected_node_id = None;
        self.preview = PreviewState::default();
        self.texture = None;
        self.texture_asset_id = None;
        self.rip_job = None;
        self.rip_status = None;
        self.dump_job = None;
        self.dump_report = None;
        self.show_dump_report = false;
        *self.ebx_guid_table.lock() = EbxGuidTable::new();
        self.ebx_guid_built_for = 0;
        self.open_job = Some(DataIndex::start_open(data_dir));
    }

    fn poll_open(&mut self) {
        let result = self.open_job.as_ref().and_then(|job| job.take_result());
        if let Some(result) = result {
            self.open_job = None;
            match result {
                Ok(index) => {
                    self.open_error = None;
                    self.rip_status = Some(format!("Catalog: {}", index.catalog.format_label));
                    self.index = Some(index);
                    self.refresh_ebx_guid_table();
                }
                Err(err) => {
                    self.open_error = Some(err);
                    self.index = None;
                }
            }
        }
    }

    fn poll_preview(&mut self) {
        let Some(job) = &self.preview_job else {
            return;
        };
        let Some(preview) = job.result.lock().take() else {
            return;
        };
        if Some(job.asset_id.as_str()) == self.selected_asset_id.as_deref() {
            if preview.image.is_some() {
                self.preview_tab = PreviewTab::Visual;
            } else if !preview.text.is_empty() && !preview.text.starts_with("Binary payload") {
                self.preview_tab = PreviewTab::Text;
            } else if !preview.hex.is_empty() {
                self.preview_tab = PreviewTab::Hex;
            } else {
                self.preview_tab = PreviewTab::Info;
            }
            self.preview = preview;
            self.texture = None;
            self.texture_asset_id = None;
        }
        self.preview_job = None;
    }

    fn poll_rip(&mut self) {
        let Some(job) = &self.rip_job else {
            return;
        };
        let finished = job.progress.lock().finished.clone();
        if let Some(msg) = finished {
            self.rip_status = Some(msg);
            self.rip_job = None;
        }
    }

    fn poll_dump(&mut self) {
        let Some(job) = &self.dump_job else {
            return;
        };
        let finished = job.progress.lock().finished.clone();
        if let Some(report) = finished {
            self.rip_status = Some(report.summary_line());
            self.dump_report = Some(report);
            self.show_dump_report = true;
            self.dump_job = None;
        }
    }

    fn open_full_dump_options(&mut self) {
        if self.dump_job.is_some() || self.rip_job.is_some() {
            return;
        }
        let Some(data_dir) = self.data_dir.clone() else {
            self.rip_status = Some("Open a data folder first".into());
            return;
        };
        if self.index.is_none() {
            self.rip_status = Some("Wait for the data index to finish opening".into());
            return;
        }
        self.pending_dump = Some(PendingDump {
            data_dir,
            options: DumpOptions::default(),
        });
    }

    fn start_full_dump_from_pending(&mut self) {
        if self.dump_job.is_some() || self.rip_job.is_some() {
            return;
        }
        let Some(pending) = self.pending_dump.take() else {
            return;
        };
        let Some(parent) = rfd::FileDialog::new()
            .set_title("Choose folder for frostex dump")
            .pick_folder()
        else {
            self.pending_dump = Some(pending);
            return;
        };

        let progress = Arc::new(Mutex::new(DumpProgress {
            done: 0,
            total: 1,
            phase: "Starting full dump…".into(),
            wrote: 0,
            failed: 0,
            skipped: 0,
            finished: None,
        }));
        let progress_thread = progress.clone();
        let data_dir = pending.data_dir;
        let options = pending.options;
        thread::spawn(move || {
            run_full_dump(data_dir, parent, options, progress_thread);
        });
        self.dump_job = Some(DumpJob { progress });
        self.rip_status = None;
        self.show_dump_report = false;
    }

    fn render_node(&mut self, ui: &mut egui::Ui, node: &TreeNode, depth: usize) {
        ui.horizontal(|ui| {
            ui.add_space((depth * 14) as f32);
            let expandable = !node.loaded || !node.children.is_empty();
            let icon_size = IconAtlas::size();

            if expandable {
                let chevron = if node.expanded {
                    FrostIcon::ChevronDown
                } else {
                    FrostIcon::ChevronRight
                };
                let clicked = if let Some(tex) = self.icons.get(chevron) {
                    ui.add(egui::ImageButton::new(tex).frame(false)).clicked()
                } else {
                    ui.small_button(if node.expanded { "-" } else { "+" })
                        .clicked()
                };
                if clicked {
                    self.toggle_node(&node.id, !node.expanded);
                }
            } else {
                ui.add_space(icon_size.x + 4.0);
            }

            let kind_icon = self.icon_for_node(node);
            if let Some(tex) = self.icons.get(kind_icon) {
                ui.add(egui::Image::new(tex).fit_to_exact_size(icon_size));
            }

            let selected = self.selected_node_id.as_deref() == Some(node.id.as_str())
                || (node.asset_id.is_some()
                    && node.asset_id.as_deref() == self.selected_asset_id.as_deref());
            let label_w = ui.available_width().max(48.0);
            let row_h = ui.spacing().interact_size.y;
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(label_w, row_h), egui::Sense::click());
            if selected {
                ui.painter().rect_filled(
                    rect.expand(1.0),
                    2.0,
                    ui.visuals().selection.bg_fill,
                );
            } else if response.hovered() {
                ui.painter().rect_filled(
                    rect.expand(1.0),
                    2.0,
                    ui.visuals().widgets.hovered.bg_fill,
                );
            }
            let text = if selected {
                egui::RichText::new(&node.label).strong()
            } else {
                egui::RichText::new(&node.label)
            };
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.set_min_size(rect.size());
                ui.add(egui::Label::new(text).truncate(true));
            });
            if response.clicked() {
                self.selected_node_id = Some(node.id.clone());
                if expandable && !node.expanded {
                    self.toggle_node(&node.id, true);
                }
                if let Some(asset_id) = &node.asset_id {
                    self.select_asset(asset_id.clone());
                } else {
                    self.selected_asset_id = None;
                    self.preview = PreviewState {
                        title: node.label.clone(),
                        info: format!(
                            "Node: {}\nKind: {:?}\nChildren: {}",
                            node.id,
                            node.kind,
                            node.children.len()
                        ),
                        text: "Select Rip to export this folder/package recursively.".into(),
                        ..PreviewState::default()
                    };
                    self.preview_tab = PreviewTab::Info;
                }
            }
        });

        if node.expanded {
            for child in &node.children {
                self.render_node(ui, child, depth + 1);
            }
        }
    }

    fn icon_for_node(&self, node: &TreeNode) -> FrostIcon {
        if let Some(asset_id) = &node.asset_id {
            if let Some(index) = &self.index {
                if let Some(asset) = index.get_asset(asset_id) {
                    let arch = Archetype::from_asset(asset);
                    if !matches!(
                        node.kind,
                        TreeNodeKind::Directory
                            | TreeNodeKind::Toc
                            | TreeNodeKind::Sb
                            | TreeNodeKind::Bundle
                    ) {
                        return FrostIcon::for_archetype(arch);
                    }
                }
            }
        }
        FrostIcon::for_tree_node(&node.kind, node.expanded, &node.label)
    }

    fn toggle_node(&mut self, node_id: &str, expanded: bool) {
        if let Some(index) = &mut self.index {
            if expanded {
                if let Err(err) = index.ensure_expanded(node_id) {
                    self.open_error = Some(err);
                } else {
                    self.refresh_ebx_guid_table();
                }
            } else if let Some(node) = index.find_node_mut(node_id) {
                node.expanded = false;
            }
        }
    }

    fn preview_ctx(&self) -> Option<PreviewCtx> {
        let index = self.index.as_ref()?;
        let table = self.ebx_guid_table.lock().clone();
        Some(PreviewCtx::from_index_with_table(index, table))
    }

    /// Rebuild GUID→Name when the indexed EBX set grows (background).
    fn refresh_ebx_guid_table(&mut self) {
        let Some(index) = &self.index else {
            return;
        };
        let count = index.assets.len();
        if count == self.ebx_guid_built_for {
            return;
        }
        self.ebx_guid_built_for = count;
        let ebx_assets: Vec<AssetRef> = index
            .assets
            .values()
            .filter(|a| a.kind == AssetKind::Ebx)
            .cloned()
            .collect();
        if ebx_assets.is_empty() {
            return;
        }
        let ctx = PreviewCtx::from_index(index);
        let table_slot = self.ebx_guid_table.clone();
        thread::spawn(move || {
            let mut table = EbxGuidTable::new();
            for asset in ebx_assets {
                if asset.size_hint.unwrap_or(0) > 8 * 1024 * 1024 {
                    continue;
                }
                if let Ok(bytes) = ctx.extract_bytes(&asset) {
                    register_ebx_guid(&bytes, &mut table);
                }
            }
            *table_slot.lock() = table;
        });
    }

    fn select_asset(&mut self, asset_id: String) {
        self.selected_asset_id = Some(asset_id.clone());
        self.texture = None;
        self.texture_asset_id = None;

        let Some(index) = &self.index else {
            return;
        };
        let Some(asset) = index.get_asset(&asset_id).cloned() else {
            return;
        };

        let extractable =
            asset.sha1.is_some() || asset.chunk_guid.is_some() || asset.path.is_some();
        if !extractable {
            self.preview = PreviewState::build_info_only(
                &asset,
                "No extractable payload (unresolved package or stub entry).",
            );
            self.preview_job = None;
            self.preview_tab = PreviewTab::Info;
            return;
        }

        if !needs_heavy_preview(&asset) {
            self.preview = PreviewState::build_info_only(
                &asset,
                "Expand package contents, or select a leaf asset.",
            );
            self.preview_job = None;
            self.preview_tab = PreviewTab::Info;
            return;
        }

        self.preview = PreviewState {
            asset_id: Some(asset_id.clone()),
            title: asset.name.clone(),
            info: "Loading preview...".into(),
            loading: true,
            ..PreviewState::default()
        };
        self.start_preview_job(asset);
    }

    fn start_preview_job(&mut self, asset: AssetRef) {
        let Some(ctx) = self.preview_ctx() else {
            return;
        };
        let asset_id = asset.id.clone();
        let result = Arc::new(Mutex::new(None));
        let result_thread = result.clone();
        thread::spawn(move || {
            let preview = PreviewState::build_preview(&ctx, &asset);
            *result_thread.lock() = Some(preview);
        });
        self.preview_job = Some(PreviewJob { asset_id, result });
    }

    fn render_preview_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.preview_tab, PreviewTab::Info, "Info");
            ui.selectable_value(&mut self.preview_tab, PreviewTab::Text, "Text");
            ui.selectable_value(&mut self.preview_tab, PreviewTab::Hex, "Hex");
            ui.selectable_value(&mut self.preview_tab, PreviewTab::Visual, "Visual");
            if self.preview.loading || self.preview_job.is_some() {
                ui.spinner();
                ui.label("Loading…");
            }
            let rip_busy = self.rip_job.is_some();
            if ui
                .add_enabled(!rip_busy, egui::Button::new("Rip"))
                .on_hover_text("Export selected asset, or recursively export a TOC/SB/folder")
                .clicked()
            {
                self.rip_selected();
            }
        });
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) {
        // Outer panel already scrolls — keep content plain so the bar doesn't clip.
        if let Some(err) = &self.preview.error {
            ui.colored_label(egui::Color32::RED, err);
        }
        match self.preview_tab {
            PreviewTab::Info => {
                ui.label(&self.preview.title);
                ui.separator();
                ui.monospace(&self.preview.info);
                if self.preview.truncated {
                    ui.label("Preview is truncated.");
                }
            }
            PreviewTab::Text => {
                if self.preview.text.is_empty() {
                    ui.label("No text preview for this asset.");
                } else {
                    ui.monospace(&self.preview.text);
                }
            }
            PreviewTab::Hex => {
                if self.preview.hex.is_empty() {
                    ui.label("No hex dump available.");
                } else {
                    ui.monospace(&self.preview.hex);
                }
            }
            PreviewTab::Visual => {
                if self.preview.image.is_some() {
                    self.render_visual(ui);
                } else {
                    ui.label("No picture/model visual for this asset.");
                    ui.label(
                        "Pictures decode DxTexture + streaming chunk / PNG/JPEG. Models decode MeshSet → visual + OBJ/SMD on Rip.",
                    );
                }
            }
        }
    }

    fn render_visual(&mut self, ui: &mut egui::Ui) {
        let Some(image) = self.preview.image.clone() else {
            ui.label("No visual preview available.");
            return;
        };
        let asset_id = self.preview.asset_id.clone().unwrap_or_default();
        if self.texture.is_none() || self.texture_asset_id.as_deref() != Some(asset_id.as_str()) {
            self.texture = Some(load_texture(ui.ctx(), &asset_id, image));
            self.texture_asset_id = Some(asset_id);
        }
        if let Some(tex) = &self.texture {
            ui.add(egui::Label::new(&self.preview.image_label).truncate(true));
            let avail = ui.available_size();
            let size = fit_size(
                tex.size_vec2(),
                Vec2::new(avail.x.max(64.0), (avail.y - 24.0).max(64.0)),
            );
            ui.add(egui::Image::new(tex).fit_to_exact_size(size));
        }
    }

    fn rip_selected(&mut self) {
        if self.rip_job.is_some() {
            return;
        }
        let Some(node_id) = self.selected_node_id.clone().or_else(|| {
            // Fall back: find node that owns the selected asset.
            let aid = self.selected_asset_id.clone()?;
            let index = self.index.as_ref()?;
            find_node_id_for_asset(&index.root, &aid)
        }) else {
            self.rip_status = Some("Nothing selected to rip".into());
            return;
        };

        self.rip_status = Some("Preparing rip (expanding packages)…".into());
        let assets = {
            let Some(index) = &mut self.index else {
                return;
            };
            if let Err(err) = index.expand_recursive(&node_id) {
                self.rip_status = Some(format!("Rip expand failed: {err}"));
                return;
            }
            index.collect_rippable(&node_id)
        };
        if assets.is_empty() {
            self.rip_status = Some("Nothing extractable under this selection".into());
            return;
        }

        self.refresh_ebx_guid_table();
        let Some(ctx) = self.preview_ctx() else {
            return;
        };

        let single_leaf = assets.len() == 1
            && matches!(
                assets[0].kind,
                AssetKind::Ebx | AssetKind::Res | AssetKind::Chunk | AssetKind::File
            );

        if single_leaf {
            let asset = &assets[0];
            if asset.size_hint.unwrap_or(0) > RIP_LIMIT {
                self.rip_status = Some(format!(
                    "Refused to rip huge payload over {}",
                    format_size(RIP_LIMIT)
                ));
                return;
            }
            let default_name = rip_relative_path(asset)
                .rsplit('/')
                .next()
                .unwrap_or("asset.bin")
                .to_string();
            let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_name)
                .save_file()
            else {
                self.rip_status = None;
                return;
            };
            let asset = asset.clone();
            let progress = Arc::new(Mutex::new(RipProgress {
                done: 0,
                total: 1,
                phase: "Writing…".into(),
                errors: 0,
                finished: None,
            }));
            let progress_thread = progress.clone();
            thread::spawn(move || {
                let msg = match prepare_rip(&ctx, &asset, &path).and_then(|(bytes, dest)| {
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
                    }
                    std::fs::write(&dest, bytes)
                        .map_err(|e| format!("write {}: {e}", dest.display()))?;
                    Ok(dest)
                }) {
                    Ok(dest) => format!("Wrote {}", dest.display()),
                    Err(err) => {
                        progress_thread.lock().errors = 1;
                        format!("Rip failed: {err}")
                    }
                };
                let mut p = progress_thread.lock();
                p.done = 1;
                p.finished = Some(msg);
            });
            self.rip_job = Some(RipJob { progress });
            self.rip_status = None;
            return;
        }

        let Some(out_dir) = rfd::FileDialog::new().pick_folder() else {
            self.rip_status = None;
            return;
        };

        let total = assets.len();
        let progress = Arc::new(Mutex::new(RipProgress {
            done: 0,
            total,
            phase: "Extracting…".into(),
            errors: 0,
            finished: None,
        }));
        let progress_thread = progress.clone();
        thread::spawn(move || {
            let mut wrote = 0usize;
            for asset in assets {
                {
                    let mut p = progress_thread.lock();
                    p.phase = asset.name.clone();
                }
                if asset.size_hint.unwrap_or(0) > RIP_LIMIT {
                    progress_thread.lock().errors += 1;
                } else {
                    let rel = rip_relative_path(&asset);
                    let dest = out_dir.join(&rel);
                    match prepare_rip(&ctx, &asset, &dest).and_then(|(bytes, final_path)| {
                        if let Some(parent) = final_path.parent() {
                            std::fs::create_dir_all(parent)
                                .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
                        }
                        std::fs::write(&final_path, bytes)
                            .map_err(|e| format!("write {}: {e}", final_path.display()))?;
                        Ok(())
                    }) {
                        Ok(()) => wrote += 1,
                        Err(_) => progress_thread.lock().errors += 1,
                    }
                }
                progress_thread.lock().done += 1;
            }
            let errors = progress_thread.lock().errors;
            progress_thread.lock().finished = Some(format!(
                "Rip done: wrote {wrote}/{total} to {} ({} errors)",
                out_dir.display(),
                errors
            ));
        });
        self.rip_job = Some(RipJob { progress });
        self.rip_status = None;
    }
}

fn find_node_id_for_asset(node: &TreeNode, asset_id: &str) -> Option<String> {
    if node.asset_id.as_deref() == Some(asset_id) {
        return Some(node.id.clone());
    }
    for child in &node.children {
        if let Some(id) = find_node_id_for_asset(child, asset_id) {
            return Some(id);
        }
    }
    None
}

fn load_texture(ctx: &egui::Context, name: &str, image: ColorImage) -> TextureHandle {
    ctx.load_texture(
        format!("frostex:{name}"),
        image,
        egui::TextureOptions::LINEAR,
    )
}

fn fit_size(original: Vec2, max: Vec2) -> Vec2 {
    if original.x <= 0.0 || original.y <= 0.0 {
        return Vec2::splat(128.0);
    }
    let scale = (max.x / original.x).min(max.y / original.y).min(1.0);
    Vec2::new(original.x * scale, original.y * scale)
}
