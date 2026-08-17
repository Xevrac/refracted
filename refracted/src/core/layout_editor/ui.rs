//! egui UI for the Frostbite layout editor (parity with layout_editor.py).

use super::model::*;
use egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::collections::HashMap;

#[derive(Default)]
pub struct LayoutEditorState {
    pub doc: LayoutDoc,
    pub status: String,
    pub selected: Option<NodePath>,
    pub tab: EditorTab,
    /// Editable prop drafts: name → text
    pub prop_drafts: HashMap<String, String>,
    pub canvas_scale: f32,
    pub canvas_origin: Pos2,
    pub drag: Option<CanvasDrag>,
    pub add_class_pending: bool,
    pub tree_rows: Vec<(NodePath, Option<String>)>,
    pub widget_flat: Vec<(NodePath, Area, u32)>,
    pub canvas_screen_rects: Vec<(NodePath, Rect)>,
    /// Max widget depth in the open document.
    pub max_depth: u32,
    /// Focus depth for the canvas filter.
    pub layer_max: u32,
    /// When false (default), show only `depth == layer_max`. When true, show `0..=layer_max`.
    pub layer_inclusive: bool,
    /// In exclusive mode, draw shallower widgets as faint context (not labeled).
    pub fade_parents: bool,
    /// Draw text labels on visible focus-depth widgets (selection always labeled).
    pub show_labels: bool,
}

impl LayoutEditorState {
    fn canvas_filter_defaults(&mut self) {
        self.layer_max = 0;
        self.layer_inclusive = false;
        self.fade_parents = true;
        self.show_labels = false;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTab {
    #[default]
    TreeProps,
    Canvas,
}

#[derive(Clone)]
pub struct CanvasDrag {
    pub mode: DragMode,
    pub start: Pos2,
    pub orig_area: Area,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DragMode {
    Move,
    Resize,
}

pub fn cnc_layout_editor_available() -> bool {
    crate::common::game::get_current_game_id().eq_ignore_ascii_case("cnc")
}

fn refresh_tree(state: &mut LayoutEditorState) {
    state.tree_rows.clear();
    if let Some(root) = state.doc.root.as_ref() {
        walk_tree(root, &mut state.tree_rows);
    }
}

fn refresh_widgets(state: &mut LayoutEditorState) {
    state.widget_flat.clear();
    state.max_depth = 0;
    let Some(root) = state.doc.root.as_ref() else {
        return;
    };
    for (i, child) in root.children.iter().enumerate() {
        if let xmltree::XMLNode::Element(obj) = child {
            if obj.name == "object" {
                walk_widgets(obj, &vec![i], 0.0, 0.0, 0, &mut state.widget_flat);
            }
        }
    }
    for (_, _, d) in &state.widget_flat {
        state.max_depth = state.max_depth.max(*d);
    }
    if state.layer_max > state.max_depth {
        state.layer_max = state.max_depth;
    }
}

fn select_path(state: &mut LayoutEditorState, path: NodePath) {
    state.selected = Some(path.clone());
    state.prop_drafts.clear();
    if let Some(root) = state.doc.root.as_ref() {
        if let Some(el) = get_at(root, &path) {
            for (name, _ty, val) in leaf_props_owned_names(el) {
                state
                    .prop_drafts
                    .insert(name, val.unwrap_or_default());
            }
        }
    }
}

fn open_file(state: &mut LayoutEditorState) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Frostbite Layout", &["layout"])
        .pick_file()
    {
        match LayoutDoc::load_path(&path) {
            Ok(doc) => {
                state.doc = doc;
                state.status = format!("Loaded {}", path.display());
                state.selected = None;
                state.prop_drafts.clear();
                refresh_tree(state);
                refresh_widgets(state);
                state.canvas_filter_defaults();
                state.canvas_scale = 0.5;
                state.canvas_origin = Pos2::new(10.0, 10.0);
                zoom_fit(state, Vec2::new(640.0, 400.0));
            }
            Err(e) => state.status = format!("Open failed: {e}"),
        }
    }
}

fn save_file(state: &mut LayoutEditorState) {
    if state.doc.path.is_none() {
        save_file_as(state);
        return;
    }
    match state.doc.save() {
        Ok(()) => {
            state.status = format!(
                "Saved {}",
                state
                    .doc
                    .path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            );
        }
        Err(e) => state.status = format!("Save failed: {e}"),
    }
}

fn save_file_as(state: &mut LayoutEditorState) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Frostbite Layout", &["layout"])
        .set_file_name("untitled.layout")
        .save_file()
    {
        match state.doc.save_as(&path) {
            Ok(()) => state.status = format!("Saved {}", path.display()),
            Err(e) => state.status = format!("Save failed: {e}"),
        }
    }
}

fn zoom_fit(state: &mut LayoutEditorState, canvas_size: Vec2) {
    refresh_widgets(state);
    let Some((_, area, _)) = state.widget_flat.first() else {
        return;
    };
    let w = (area[2] - area[0]).max(100.0);
    let h = (area[3] - area[1]).max(100.0);
    let cw = canvas_size.x.max(200.0);
    let ch = canvas_size.y.max(200.0);
    state.canvas_scale = ((cw - 20.0) / w).min((ch - 20.0) / h).max(0.05);
    state.canvas_origin = Pos2::new(10.0, 10.0);
}

fn apply_props(state: &mut LayoutEditorState) {
    let Some(path) = state.selected.clone() else {
        return;
    };
    let Some(root) = state.doc.root.as_mut() else {
        return;
    };
    let Some(el) = get_at_mut(root, &path) else {
        return;
    };
    for (name, value) in state.prop_drafts.clone() {
        set_prop_value(el, &name, &value);
    }
    state.doc.dirty = true;
    refresh_tree(state);
    refresh_widgets(state);
    state.status = "Changes applied (unsaved)".into();
}

fn do_add_child(state: &mut LayoutEditorState, cls: &str) {
    let Some(path) = state.selected.clone() else {
        state.status = "Select a widget first".into();
        return;
    };
    let template = match state.doc.class_templates.get(cls).cloned() {
        Some(t) => t,
        None => {
            state.status = format!("No template for {cls}");
            return;
        }
    };
    let cid = state.doc.new_control_id();
    let Some(root) = state.doc.root.as_mut() else {
        return;
    };
    let Some(parent) = get_at_mut(root, &path) else {
        return;
    };
    let parent_area = get_area(parent);
    let w = ((parent_area[2] - parent_area[0]).max(20.0)).min(100.0);
    let h = ((parent_area[3] - parent_area[1]).max(20.0)).min(60.0);
    add_child(parent, &template, [10.0, 10.0, 10.0 + w, 10.0 + h], &cid);
    state.doc.reindex();
    state.doc.dirty = true;
    refresh_tree(state);
    refresh_widgets(state);
    state.status = format!("Added {cls} child (unsaved)");
}

fn do_duplicate(state: &mut LayoutEditorState) {
    let Some(path) = state.selected.clone() else {
        return;
    };
    if path.len() < 2 {
        state.status = "Can't duplicate root object via list".into();
        return;
    }
    let Some(root) = state.doc.root.as_ref() else {
        return;
    };
    let Some(el) = get_at(root, &path) else {
        return;
    };
    let mut clone = el.clone();
    let area = get_area(el);
    let new_area = [area[0] + 12.0, area[1] + 12.0, area[2] + 12.0, area[3] + 12.0];
    let prop_path = &path[..path.len() - 1];
    let cid = state.doc.new_control_id();
    let Some(root) = state.doc.root.as_mut() else {
        return;
    };
    if let Some(cid_prop) = find_prop_mut(&mut clone, "ControlID") {
        cid_prop.attributes.insert("value".into(), cid);
    }
    set_area(&mut clone, new_area);
    let Some(prop) = get_at_mut(root, prop_path) else {
        state.status = "Duplicate: parent list not found".into();
        return;
    };
    if prop.name != "prop" {
        state.status = "Duplicate: not inside a list prop".into();
        return;
    }
    prop.children
        .push(xmltree::XMLNode::Element(clone));
    if let Some(c) = prop.attributes.get_mut("count") {
        if let Ok(n) = c.parse::<i32>() {
            *c = (n + 1).to_string();
        }
    }
    state.doc.reindex();
    state.doc.dirty = true;
    refresh_tree(state);
    refresh_widgets(state);
    state.status = "Duplicated (unsaved)".into();
}

fn do_delete(state: &mut LayoutEditorState) {
    let Some(path) = state.selected.clone() else {
        return;
    };
    if path.is_empty() {
        return;
    }
    let Some(root) = state.doc.root.as_mut() else {
        return;
    };
    if path.len() == 1 {
        let idx = path[0];
        if idx < root.children.len() {
            root.children.remove(idx);
            state.selected = None;
            state.doc.dirty = true;
            refresh_tree(state);
            refresh_widgets(state);
            state.status = "Deleted (unsaved)".into();
        }
        return;
    }
    let prop_path = &path[..path.len() - 1];
    let idx = *path.last().unwrap();
    let Some(prop) = get_at_mut(root, prop_path) else {
        return;
    };
    if prop.name != "prop" || idx >= prop.children.len() {
        return;
    }
    prop.children.remove(idx);
    if let Some(c) = prop.attributes.get_mut("count") {
        if let Ok(n) = c.parse::<i32>() {
            *c = (n - 1).max(0).to_string();
        }
    }
    state.selected = None;
    state.doc.dirty = true;
    refresh_tree(state);
    refresh_widgets(state);
    state.status = "Deleted (unsaved)".into();
}

fn do_move(state: &mut LayoutEditorState, direction: i32) {
    let Some(path) = state.selected.clone() else {
        return;
    };
    if path.is_empty() {
        return;
    }
    let idx = *path.last().unwrap();
    let new_idx = idx as i32 + direction;
    if new_idx < 0 {
        return;
    }
    let Some(root) = state.doc.root.as_mut() else {
        return;
    };
    let prop_path: NodePath = path[..path.len().saturating_sub(1)].to_vec();
    let ok = if prop_path.is_empty() {
        if new_idx as usize >= root.children.len() {
            false
        } else {
            root.children.swap(idx, new_idx as usize);
            true
        }
    } else {
        let Some(prop) = get_at_mut(root, &prop_path) else {
            return;
        };
        move_sibling_in_prop(prop, idx, direction)
    };
    if ok {
        let mut new_path = path;
        let last = new_path.last_mut().unwrap();
        *last = (*last as i32 + direction) as usize;
        state.selected = Some(new_path);
        state.doc.dirty = true;
        refresh_tree(state);
        refresh_widgets(state);
        state.status = "Reordered (unsaved)".into();
    }
}

fn render_toolbar(ui: &mut egui::Ui, state: &mut LayoutEditorState) {
    ui.horizontal(|ui| {
        if ui.button("Open…").clicked() {
            open_file(state);
        }
        if ui.button("Save").clicked() {
            save_file(state);
        }
        if ui.button("Save As…").clicked() {
            save_file_as(state);
        }
        ui.separator();
        if ui.button("Add Child…").clicked() {
            state.add_class_pending = true;
        }
        if ui.button("Duplicate").clicked() {
            do_duplicate(state);
        }
        if ui.button("Delete").clicked() {
            do_delete(state);
        }
        // ASCII labels — egui default font often lacks ↑/↓ (renders as □).
        if ui.button("Move up").on_hover_text("Reorder selected sibling earlier").clicked() {
            do_move(state, -1);
        }
        if ui.button("Move down").on_hover_text("Reorder selected sibling later").clicked() {
            do_move(state, 1);
        }
    });
}

pub fn render_layout_editor(ui: &mut egui::Ui, state: &mut LayoutEditorState) {
    if !cnc_layout_editor_available() {
        ui.label("Layout Editor is only available when Command & Conquer is the selected game.");
        return;
    }

    if state.canvas_scale <= 0.0 {
        state.canvas_scale = 0.5;
    }

    ui.horizontal(|ui| {
        ui.heading("Frostbite Layout Editor");
        if state.doc.dirty {
            ui.colored_label(Color32::YELLOW, "• unsaved");
        }
        if let Some(ref path) = state.doc.path {
            if let Some(name) = path.file_name() {
                ui.label(
                    egui::RichText::new(format!("— {}", name.to_string_lossy()))
                        .weak(),
                );
            }
        }
    });

    render_toolbar(ui, state);

    if state.add_class_pending {
        egui::Window::new("Add Child — pick class")
            .collapsible(false)
            .resizable(true)
            .default_size([280.0, 360.0])
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for cls in state.doc.all_classes.clone() {
                        if ui.selectable_label(false, &cls).clicked() {
                            do_add_child(state, &cls);
                            state.add_class_pending = false;
                        }
                    }
                });
                if ui.button("Cancel").clicked() {
                    state.add_class_pending = false;
                }
            });
    }

    if let Some(ref path) = state.doc.path {
        ui.label(egui::RichText::new(path.display().to_string()).small().weak());
    } else {
        ui.label(egui::RichText::new("Open a .layout file to begin.").weak());
    }
    if !state.status.is_empty() {
        ui.label(&state.status);
    }

    ui.separator();

    // Match Python notebook: Tree/Properties | Visual Canvas
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.tab, EditorTab::TreeProps, "Tree / Properties");
        ui.selectable_value(&mut state.tab, EditorTab::Canvas, "Visual Canvas");
    });

    match state.tab {
        EditorTab::TreeProps => render_tree_props_tab(ui, state),
        EditorTab::Canvas => render_canvas_tab(ui, state),
    }
}

fn render_tree_props_tab(ui: &mut egui::Ui, state: &mut LayoutEditorState) {
    let total = ui.available_width();
    let tree_w = (total * 0.38).clamp(200.0, 420.0);

    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(tree_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label(egui::RichText::new("Tree").strong());
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_source("layout_tree_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        render_tree(ui, state);
                    });
            },
        );

        ui.separator();

        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label(egui::RichText::new("Properties").strong());
                ui.separator();
                render_props(ui, state);
            },
        );
    });
}

fn render_tree(ui: &mut egui::Ui, state: &mut LayoutEditorState) {
    if state.tree_rows.is_empty() {
        if state.doc.root.is_some() {
            refresh_tree(state);
        } else {
            ui.label("(no document)");
            return;
        }
    }
    let rows = state.tree_rows.clone();
    for (path, prop_name) in rows {
        let depth = path.len().saturating_sub(1);
        let indent = (depth as f32) * 12.0;
        let label = if let Some(root) = state.doc.root.as_ref() {
            get_at(root, &path)
                .map(|el| node_label(el, prop_name.as_deref()))
                .unwrap_or_else(|| "?".into())
        } else {
            "?".into()
        };
        let selected = state.selected.as_ref() == Some(&path);
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let resp = ui.selectable_label(selected, label);
            if resp.clicked() {
                select_path(state, path.clone());
            }
            resp.context_menu(|ui| {
                if ui.button("Add Child…").clicked() {
                    select_path(state, path.clone());
                    state.add_class_pending = true;
                    ui.close_menu();
                }
                if ui.button("Duplicate").clicked() {
                    select_path(state, path.clone());
                    do_duplicate(state);
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    select_path(state, path.clone());
                    do_delete(state);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Move up").clicked() {
                    select_path(state, path.clone());
                    do_move(state, -1);
                    ui.close_menu();
                }
                if ui.button("Move down").clicked() {
                    select_path(state, path.clone());
                    do_move(state, 1);
                    ui.close_menu();
                }
            });
        });
    }
}

fn render_props(ui: &mut egui::Ui, state: &mut LayoutEditorState) {
    let Some(path) = state.selected.clone() else {
        ui.label("No selection");
        return;
    };
    let header = {
        let Some(root) = state.doc.root.as_ref() else {
            return;
        };
        let Some(el) = get_at(root, &path) else {
            ui.label("Selection invalid");
            return;
        };
        let cls = el
            .attributes
            .get("cls")
            .cloned()
            .unwrap_or_else(|| el.name.clone());
        let clsid = el.attributes.get("clsid").cloned().unwrap_or_default();
        for (name, _ty, val) in leaf_props_owned_names(el) {
            state
                .prop_drafts
                .entry(name)
                .or_insert_with(|| val.unwrap_or_default());
        }
        if clsid.is_empty() {
            cls
        } else {
            format!("{cls}  ({clsid})")
        }
    };
    ui.label(egui::RichText::new(header).strong());

    let fields: Vec<(String, String)> = {
        let Some(root) = state.doc.root.as_ref() else {
            return;
        };
        let Some(el) = get_at(root, &path) else {
            return;
        };
        leaf_props_owned_names(el)
            .into_iter()
            .map(|(n, t, _)| (n, t))
            .collect()
    };

    egui::ScrollArea::vertical()
        .id_source("layout_props_scroll")
        .show(ui, |ui| {
            egui::Grid::new("layout_props")
                .num_columns(3)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    for (name, ptype) in &fields {
                        ui.label(name);
                        if name == "Area" && ptype == "rectf" {
                            render_area_field(ui, state, name);
                        } else {
                            let draft = state.prop_drafts.entry(name.clone()).or_default();
                            if ptype == "bool" {
                                let mut is_true = draft.eq_ignore_ascii_case("true");
                                if ui.checkbox(&mut is_true, "").changed() {
                                    *draft = if is_true {
                                        "True".into()
                                    } else {
                                        "False".into()
                                    };
                                }
                            } else {
                                ui.add(egui::TextEdit::singleline(draft).desired_width(280.0));
                            }
                        }
                        ui.label(egui::RichText::new(ptype).small().weak());
                        ui.end_row();
                    }
                });
            if ui.button("Apply changes").clicked() {
                apply_props(state);
            }
        });
}

fn render_area_field(ui: &mut egui::Ui, state: &mut LayoutEditorState, name: &str) {
    let draft = state.prop_drafts.entry(name.to_string()).or_default();
    let mut parts: Vec<f32> = draft
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect();
    while parts.len() < 4 {
        parts.push(0.0);
    }
    let mut l = parts[0];
    let mut t = parts[1];
    let mut r = parts[2];
    let mut b = parts[3];
    ui.horizontal(|ui| {
        ui.label("L");
        ui.add(egui::DragValue::new(&mut l).speed(1.0));
        ui.label("T");
        ui.add(egui::DragValue::new(&mut t).speed(1.0));
        ui.label("R");
        ui.add(egui::DragValue::new(&mut r).speed(1.0));
        ui.label("B");
        ui.add(egui::DragValue::new(&mut b).speed(1.0));
    });
    *draft = format!("{l},{t},{r},{b}");
}

fn render_canvas_tab(ui: &mut egui::Ui, state: &mut LayoutEditorState) {
    if state.doc.root.is_none() {
        ui.label("Open a .layout file to begin.");
        return;
    }
    refresh_widgets(state);

    let focus = state.layer_max;
    let inclusive = state.layer_inclusive;
    let primary: Vec<_> = state
        .widget_flat
        .iter()
        .filter(|(_, _, d)| {
            if inclusive {
                *d <= focus
            } else {
                *d == focus
            }
        })
        .cloned()
        .collect();
    let ghosts: Vec<_> = if !inclusive && state.fade_parents {
        state
            .widget_flat
            .iter()
            .filter(|(_, _, d)| *d < focus)
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    ui.horizontal(|ui| {
        if ui.button("Zoom to fit").clicked() {
            let size = ui.available_size();
            zoom_fit(state, size);
        }
        ui.separator();
        ui.label("Depth");
        let cap = state.max_depth.max(1);
        let mut layer = state.layer_max as i32;
        let depth_hint = if state.layer_inclusive {
            format!("0..{layer}")
        } else {
            format!("only {layer}")
        };
        if ui
            .add(egui::Slider::new(&mut layer, 0..=cap as i32).text(depth_hint))
            .changed()
        {
            state.layer_max = layer.max(0) as u32;
        }
        ui.checkbox(&mut state.layer_inclusive, "≤ depth")
            .on_hover_text(
                "Off (default): this depth only.\nOn: every widget from 0 through this depth (noisy).",
            );
        if !state.layer_inclusive {
            ui.checkbox(&mut state.fade_parents, "Fade parents")
                .on_hover_text("Draw shallower depths as faint outlines for context");
        }
        ui.checkbox(&mut state.show_labels, "Labels")
            .on_hover_text("Name labels on focus-depth widgets (selection always labeled)");
        if ui
            .button("All")
            .on_hover_text("Show every nested widget (≤ max depth)")
            .clicked()
        {
            state.layer_inclusive = true;
            state.layer_max = state.max_depth;
        }
        ui.separator();
        let mode = if state.layer_inclusive {
            format!("≤{}", state.layer_max)
        } else {
            format!("={}", state.layer_max)
        };
        ui.label(format!(
            "{} focus / {} total ({mode})  scale={:.2}",
            primary.len(),
            state.widget_flat.len(),
            state.canvas_scale
        ));
    });
    ui.label(
        egui::RichText::new(
            "drag body = move · corner = resize · wheel = zoom · step Depth to peel the HUD",
        )
        .small()
        .weak(),
    );

    render_canvas(ui, state, &primary, &ghosts);
}

fn render_canvas(
    ui: &mut egui::Ui,
    state: &mut LayoutEditorState,
    primary: &[(NodePath, Area, u32)],
    ghosts: &[(NodePath, Area, u32)],
) {
    let (response, painter) = ui.allocate_painter(
        ui.available_size_before_wrap().max(Vec2::new(480.0, 360.0)),
        Sense::click_and_drag(),
    );
    let rect = response.rect;
    painter.rect_filled(rect, 0.0, Color32::from_rgb(0x20, 0x24, 0x2c));

    // Scroll-wheel zoom toward pointer
    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let before = state.canvas_scale;
            let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
            state.canvas_scale = (state.canvas_scale * factor).clamp(0.05, 8.0);
            if let Some(pos) = response.hover_pos() {
                // Keep world point under cursor stable.
                let local = pos - rect.min;
                let ox = state.canvas_origin.x;
                let oy = state.canvas_origin.y;
                let wx = (local.x - ox) / before;
                let wy = (local.y - oy) / before;
                state.canvas_origin.x = local.x - wx * state.canvas_scale;
                state.canvas_origin.y = local.y - wy * state.canvas_scale;
            }
        }
    }

    let s = state.canvas_scale;
    let origin = state.canvas_origin;
    state.canvas_screen_rects.clear();

    let canvas_area = rect.width() * rect.height();
    let selected = state.selected.clone();
    let show_labels = state.show_labels;

    let to_screen = |abs: &Area| -> Rect {
        let mut x0 = rect.min.x + origin.x + abs[0] * s;
        let mut y0 = rect.min.y + origin.y + abs[1] * s;
        let mut x1 = rect.min.x + origin.x + abs[2] * s;
        let mut y1 = rect.min.y + origin.y + abs[3] * s;
        if x1 < x0 {
            std::mem::swap(&mut x0, &mut x1);
        }
        if y1 < y0 {
            std::mem::swap(&mut y0, &mut y1);
        }
        Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, y1))
    };

    // Parent ghosts first (context only — not hit-tested).
    for (_path, abs, _depth) in ghosts {
        let r = to_screen(abs);
        painter.rect_stroke(
            r,
            0.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x6a, 0x78, 0x88, 45)),
        );
    }

    for (path, abs, _depth) in primary {
        let r = to_screen(abs);
        state.canvas_screen_rects.push((path.clone(), r));

        let cls = state
            .doc
            .root
            .as_ref()
            .and_then(|root| get_at(root, path))
            .and_then(|el| el.attributes.get("cls"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let color = class_color(cls);
        let is_sel = selected.as_ref() == Some(path);
        let outline = if is_sel {
            Color32::from_rgb(0xff, 0xcc, 0x00)
        } else {
            color
        };
        let stroke_w = if is_sel { 2.0 } else { 1.0 };

        if is_sel {
            let rect_area = r.width() * r.height();
            if rect_area < 0.35 * canvas_area {
                painter.rect_filled(
                    r,
                    0.0,
                    Color32::from_rgba_unmultiplied(0xff, 0xcc, 0x00, 30),
                );
            }
        }
        painter.rect_stroke(r, 0.0, Stroke::new(stroke_w, outline));

        if is_sel || show_labels {
            if let Some(root) = state.doc.root.as_ref() {
                if let Some(el) = get_at(root, path) {
                    let mut label = cls.to_string();
                    if let Some(c) = find_prop(el, "Comment") {
                        if let Some(v) = c.attributes.get("value") {
                            if !v.is_empty() {
                                label.push_str(&format!(" \"{v}\""));
                            }
                        }
                    }
                    if r.width() > 36.0 && r.height() > 12.0 {
                        painter.text(
                            Pos2::new(r.min.x + 3.0, r.min.y + 3.0),
                            egui::Align2::LEFT_TOP,
                            label,
                            egui::FontId::proportional(11.0),
                            if is_sel {
                                Color32::from_rgb(0xff, 0xee, 0xaa)
                            } else {
                                Color32::from_rgb(0xc8, 0xd0, 0xd8)
                            },
                        );
                    }
                }
            }
        }
        if is_sel {
            let hs = 6.0;
            let handle = Rect::from_center_size(r.max, Vec2::splat(hs * 2.0));
            painter.rect_filled(handle, 0.0, Color32::from_rgb(0xff, 0xcc, 0x00));
            painter.rect_stroke(handle, 0.0, Stroke::new(1.0, Color32::BLACK));
        }
    }

    // Interaction — hit test only among currently drawn (layer-filtered) rects
    if response.clicked() || response.drag_started() {
        if let Some(pos) = response.interact_pointer_pos() {
            let (mode, hit) = hit_test(state, pos);
            if let Some(path) = hit {
                select_path(state, path.clone());
                if let Some(root) = state.doc.root.as_ref() {
                    if let Some(el) = get_at(root, &path) {
                        let orig = get_area(el);
                        if response.drag_started() {
                            state.drag = Some(CanvasDrag {
                                mode: mode.unwrap_or(DragMode::Move),
                                start: pos,
                                orig_area: orig,
                            });
                        }
                    }
                }
            } else if response.clicked() {
                state.selected = None;
                state.prop_drafts.clear();
            }
        }
    }

    if response.dragged() {
        if let (Some(drag), Some(path), Some(pos)) = (
            state.drag.clone(),
            state.selected.clone(),
            response.interact_pointer_pos(),
        ) {
            let dx = (pos.x - drag.start.x) / s;
            let dy = (pos.y - drag.start.y) / s;
            let [l, t, r, b] = drag.orig_area;
            let new_rect = match drag.mode {
                DragMode::Move => [l + dx, t + dy, r + dx, b + dy],
                DragMode::Resize => [l, t, (r + dx).max(l + 4.0), (b + dy).max(t + 4.0)],
            };
            if let Some(root) = state.doc.root.as_mut() {
                if let Some(el) = get_at_mut(root, &path) {
                    set_area(el, new_rect);
                }
            }
        }
    }

    if response.drag_stopped() {
        if state.drag.is_some() {
            state.doc.dirty = true;
            if let Some(path) = state.selected.clone() {
                select_path(state, path);
            }
            refresh_widgets(state);
            state.status = "Moved/resized (unsaved)".into();
        }
        state.drag = None;
    }

    response.context_menu(|ui| {
        if ui.button("Add Child…").clicked() {
            state.add_class_pending = true;
            ui.close_menu();
        }
        if ui.button("Duplicate").clicked() {
            do_duplicate(state);
            ui.close_menu();
        }
        if ui.button("Delete").clicked() {
            do_delete(state);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Zoom to fit").clicked() {
            zoom_fit(state, rect.size());
            ui.close_menu();
        }
    });
}

fn hit_test(state: &LayoutEditorState, pos: Pos2) -> (Option<DragMode>, Option<NodePath>) {
    if let Some(sel) = &state.selected {
        for (path, r) in &state.canvas_screen_rects {
            if path == sel {
                let hs = 6.0;
                let handle = Rect::from_center_size(r.max, Vec2::splat(hs * 2.0));
                if handle.contains(pos) {
                    return (Some(DragMode::Resize), Some(path.clone()));
                }
            }
        }
    }
    let mut best: Option<(NodePath, f32)> = None;
    for (path, r) in &state.canvas_screen_rects {
        if r.contains(pos) {
            let a = r.width() * r.height();
            if best.as_ref().map(|(_, ba)| a < *ba).unwrap_or(true) {
                best = Some((path.clone(), a));
            }
        }
    }
    (
        Some(DragMode::Move),
        best.map(|(p, _)| p),
    )
}
