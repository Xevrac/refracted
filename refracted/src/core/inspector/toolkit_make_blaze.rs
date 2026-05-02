use crate::blaze::protocol::fire2frame::{get_command_name, Fire2FramePacket, NAMED_BLAZE_COMMANDS};
use crate::blaze::protocol::MessageType;
use crate::blaze::server::toolkit_inject::broadcast_toolkit_blaze_wire;
use crate::blaze::tdf::{TdfEncoder, TdfTreeNode, TdfTreeParser};
use crate::core::inspector::blaze_inspector::tdf_tree_to_plaintext;
use crate::core::inspector::inspector_module::{format_hex_dump, CapturedPacket};
use bytes::{Bytes, BytesMut};
use egui;
use std::cell::Cell;

fn normalize_four_char_tag(raw: &str) -> String {
    let mut s: String = raw
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(4)
        .collect();
    while s.len() < 4 {
        s.push(' ');
    }
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TdfMakeFieldKind {
    #[default]
    String,
    Int,
    BlobHex,
    RawHex,
}

pub struct BlazeTdfMakeRow {
    pub tag: String,
    pub kind: TdfMakeFieldKind,
    pub value: String,
}

impl Default for BlazeTdfMakeRow {
    fn default() -> Self {
        Self {
            tag: String::new(),
            kind: TdfMakeFieldKind::String,
            value: String::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BlazeMakeUIMode {
    #[default]
    Easy,
    Advanced,
}

pub struct BlazeMakeWorkbenchState {
    pub ui_mode: BlazeMakeUIMode,
    pub component_s: String,
    pub command_s: String,
    pub msg_num_s: String,
    pub msg_ty: MessageType,

    pub preset_pick: usize,
    pub use_preset: bool,
    /// When [`Self::use_preset`] is on, `Some(i)` means rows already match preset index `i`; `None` triggers a template fill.
    pub blaze_rows_from_preset: Option<usize>,

    pub wrap_struct: bool,
    pub struct_outer_tag: String,

    pub rows: Vec<BlazeTdfMakeRow>,

    pub tdf_tree_preview: Option<String>,
    pub tdf_preview_err: Option<String>,
    pub payload_hex: Option<String>,
    pub wire_hex: Option<String>,
    pub build_err: Option<String>,
    pub blaze_action_note: Option<String>,
}

impl Default for BlazeMakeWorkbenchState {
    fn default() -> Self {
        Self {
            ui_mode: BlazeMakeUIMode::Easy,
            component_s: "0x0009".into(),
            command_s: "0x0002".into(),
            msg_num_s: "1".into(),
            msg_ty: MessageType::Message,
            preset_pick: 0,
            use_preset: true,
            blaze_rows_from_preset: None,
            wrap_struct: false,
            struct_outer_tag: "ROOT".into(),
            rows: vec![BlazeTdfMakeRow::default()],
            tdf_tree_preview: None,
            tdf_preview_err: None,
            payload_hex: None,
            wire_hex: None,
            build_err: None,
            blaze_action_note: None,
        }
    }
}

fn parse_u16_radix(t: &str) -> Result<u16, String> {
    let t = t.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16).map_err(|_| format!("Bad hex u16 '{}'", t))
    } else {
        t.parse().map_err(|_| format!("Bad u16 '{}'", t))
    }
}

fn parse_u32_radix(t: &str) -> Result<u32, String> {
    let t = t.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|_| format!("Bad hex u32 '{}'", t))
    } else {
        t.parse().map_err(|_| format!("Bad u32 '{}'", t))
    }
}

fn parse_hex_loose(s: &str) -> Result<Vec<u8>, String> {
    let hex_clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex_clean.len() % 2 != 0 {
        return Err("Hex must have an even number of digits".into());
    }
    hex::decode(&hex_clean).map_err(|e| format!("{}", e))
}

fn build_tdf_payload(state: &BlazeMakeWorkbenchState) -> Result<Bytes, String> {
    let mut acc = BytesMut::new();
    for row in &state.rows {
        let tag = normalize_four_char_tag(&row.tag);
        match row.kind {
            TdfMakeFieldKind::String => {
                acc.extend_from_slice(&TdfEncoder::encode_string(&tag, row.value.trim()));
            }
            TdfMakeFieldKind::Int => {
                let v: i32 = row
                    .value
                    .trim()
                    .parse()
                    .map_err(|_| format!("Row {}: invalid int", tag))?;
                acc.extend_from_slice(&TdfEncoder::encode_int(&tag, v));
            }
            TdfMakeFieldKind::BlobHex => {
                let raw = parse_hex_loose(row.value.trim())?;
                acc.extend_from_slice(&TdfEncoder::encode_binary(&tag, &raw));
            }
            TdfMakeFieldKind::RawHex => {
                let raw = parse_hex_loose(row.value.trim())?;
                acc.extend_from_slice(&raw);
            }
        }
    }

    let inner = acc.freeze();
    if state.wrap_struct {
        let outer = normalize_four_char_tag(state.struct_outer_tag.trim());
        if outer.chars().all(|c| c.is_whitespace()) {
            return Err("Struct tag required when wrap is enabled".into());
        }
        Ok(TdfEncoder::encode_struct(&outer, inner.as_ref()))
    } else {
        Ok(inner)
    }
}

fn refresh_preview(state: &mut BlazeMakeWorkbenchState) {
    state.tdf_preview_err = None;
    state.tdf_tree_preview = None;
    match build_tdf_payload(state) {
        Ok(bytes) => match TdfTreeParser::parse_packet(bytes.as_ref()) {
            Ok(tree) => state.tdf_tree_preview = Some(tdf_tree_to_plaintext(&tree)),
            Err(e) => state.tdf_preview_err = Some(format!("{:?}", e)),
        },
        Err(e) => state.tdf_preview_err = Some(e),
    }
}

fn try_make_fire2_packet(state: &BlazeMakeWorkbenchState) -> Result<Fire2FramePacket, String> {
    let c = parse_u16_radix(&state.component_s)?;
    let cmd = parse_u16_radix(&state.command_s)?;
    let num = parse_u32_radix(&state.msg_num_s)?;
    let payload = build_tdf_payload(state)?;
    Ok(Fire2FramePacket::new_send(c, cmd, num, state.msg_ty, payload))
}

fn apply_build_outputs(state: &mut BlazeMakeWorkbenchState, pkt: &Fire2FramePacket) {
    let wire = pkt.to_bytes();
    state.payload_hex = Some(hex::encode(pkt.payload.as_ref()));
    state.wire_hex = Some(hex::encode(wire.as_ref()));
    state.build_err = None;
    refresh_preview(state);
}

fn build_or_err(state: &mut BlazeMakeWorkbenchState) -> Result<(), String> {
    let pkt = try_make_fire2_packet(state)?;
    apply_build_outputs(state, &pkt);
    Ok(())
}

/// Interpret listener `CapturedPacket.msg_type` strings (`REQUEST`, `REPLY`, …).
pub fn message_type_from_capture_label(s: &str) -> MessageType {
    match s.trim().to_uppercase().as_str() {
        "REPLY" => MessageType::Reply,
        "NOTIFICATION" => MessageType::Notification,
        "ERROR_REPLY" => MessageType::ErrorReply,
        "PING" => MessageType::Ping,
        "PING_REPLY" => MessageType::PingReply,
        "REQUEST" | "MESSAGE" => MessageType::Message,
        _ => MessageType::Message,
    }
}

/// Fill Make → Blaze from a listener row; payload becomes one **Raw (hex)** row for editing.
pub fn prefill_from_captured_packet(make: &mut BlazeMakeWorkbenchState, packet: &CapturedPacket) {
    make.msg_ty = message_type_from_capture_label(&packet.msg_type);
    make.msg_num_s = format!("{}", packet.msg_num);
    make.component_s = format!("0x{:04x}", packet.component);
    make.command_s = format!("0x{:04x}", packet.command);

    make.use_preset = false;
    make.blaze_rows_from_preset = None;
    for (i, ent) in NAMED_BLAZE_COMMANDS.iter().enumerate() {
        if ent.component == packet.component && ent.command == packet.command {
            make.use_preset = true;
            make.preset_pick = i;
            break;
        }
    }

    if packet.payload.is_empty() {
        make.rows = vec![BlazeTdfMakeRow::default()];
    } else {
        make.rows = vec![BlazeTdfMakeRow {
            tag: String::new(),
            kind: TdfMakeFieldKind::RawHex,
            value: hex::encode(&packet.payload),
        }];
    }
    make.wrap_struct = false;
    make.build_err = None;
    make.blaze_action_note = None;
    make.payload_hex = None;
    make.wire_hex = None;
    refresh_preview(make);
}

fn build_preset_request_sample(component: u16, command: u16) -> Vec<u8> {
    let mut v = Vec::new();
    match (component, command) {
        (0x0009, 0x02) => {}
        (0x0009, 0x07) => {
            v.extend_from_slice(&TdfEncoder::encode_string("CFID", "BlazeSDK"));
            v.extend_from_slice(&TdfEncoder::encode_string("PLAT", "pc"));
        }
        (0x0009, 0x01) => {
            v.extend_from_slice(&TdfEncoder::encode_string("CFID", "BlazeSDK"));
        }
        (0x0009, 0x08) => {}
        (0x0009, 0x05) => {}
        (0x0001, 0x0a) => {
            v.extend_from_slice(&TdfEncoder::encode_string("MAIL", "example@example.com"));
            v.extend_from_slice(&TdfEncoder::encode_string("PASS", ""));
            v.extend_from_slice(&TdfEncoder::encode_int("PLAT", 4));
        }
        (0x0001, 0x46) => {}
        (0x7802, 0x14) => {
            v.extend_from_slice(&TdfEncoder::encode_string("BPS ", "1000"));
            v.extend_from_slice(&TdfEncoder::encode_int("PORT", 3659));
        }
        (0x7802, 0x0c) => {
            v.extend_from_slice(&TdfEncoder::encode_string("NAME", "Player"));
        }
        (0x7802, 0x08) => {
            v.extend_from_slice(&TdfEncoder::encode_int("HWFG", 0));
        }
        (0x7802, 0x3c) => {}
        (0x0004, 0x03) => {
            v.extend_from_slice(&TdfEncoder::encode_int("JGS ", 0));
        }
        (0x0004, 0x11) => {}
        (0, 0) => {}
        _ => {}
    }
    v
}

fn decimal_in_outer_parens(s: &str) -> Option<i64> {
    let open = s.rfind('(')?;
    let close = s.rfind(')')?;
    if close <= open + 1 {
        return None;
    }
    s[open + 1..close].trim().parse().ok()
}

fn blob_display_to_hex_field(display: &str) -> Option<String> {
    let contiguous: String = display
        .split_whitespace()
        .filter_map(|tok| {
            let t = tok.trim_end_matches('.');
            if t.len() == 2 && t.chars().all(|c| c.is_ascii_hexdigit()) {
                Some(t)
            } else {
                None
            }
        })
        .collect();
    (!contiguous.is_empty()).then_some(contiguous.to_uppercase())
}

fn leaf_to_make_row(node: &TdfTreeNode) -> Option<BlazeTdfMakeRow> {
    if !node.children.is_empty() {
        return None;
    }
    let tag = node.tag.trim().to_string();
    match node.value_type.as_str() {
        "STRING" => {
            let val = if node.value_display == "(empty)" {
                String::new()
            } else {
                node.value_display.clone()
            };
            Some(BlazeTdfMakeRow {
                tag,
                kind: TdfMakeFieldKind::String,
                value: val,
            })
        }
        "INTEGER" => {
            let txt = decimal_in_outer_parens(&node.value_display)
                .map(|d| d.to_string())
                .unwrap_or_else(|| node.value_display.clone());
            let _: i32 = txt.trim().parse().ok()?;
            Some(BlazeTdfMakeRow {
                tag,
                kind: TdfMakeFieldKind::Int,
                value: txt,
            })
        }
        "INT64" => {
            let d = decimal_in_outer_parens(&node.value_display)?;
            let i32v = i32::try_from(d).ok()?;
            Some(BlazeTdfMakeRow {
                tag,
                kind: TdfMakeFieldKind::Int,
                value: i32v.to_string(),
            })
        }
        "BLOB" => {
            let hex = blob_display_to_hex_field(&node.value_display)?;
            Some(BlazeTdfMakeRow {
                tag,
                kind: TdfMakeFieldKind::BlobHex,
                value: hex,
            })
        }
        _ => None,
    }
}

fn flattenable_primitive_type(ty: &str) -> bool {
    matches!(ty, "STRING" | "INTEGER" | "INT64" | "BLOB")
}

fn struct_children_all_primitive(children: &[TdfTreeNode]) -> bool {
    children
        .iter()
        .all(|n| n.children.is_empty() && flattenable_primitive_type(&n.value_type))
}

fn try_tdf_payload_to_make_rows(payload: &[u8]) -> Option<(Vec<BlazeTdfMakeRow>, bool, String)> {
    if payload.is_empty() {
        return None;
    }
    let tree = TdfTreeParser::parse_packet(payload).ok()?;
    if tree.is_empty() {
        return None;
    }

    if tree.len() == 1 && tree[0].value_type == "STRUCT" && !tree[0].children.is_empty() {
        if !struct_children_all_primitive(&tree[0].children) {
            return None;
        }
        let mut rows = Vec::with_capacity(tree[0].children.len());
        for ch in &tree[0].children {
            rows.push(leaf_to_make_row(ch)?);
        }
        let outer = tree[0].tag.trim().to_string();
        return Some((rows, true, outer));
    }

    if tree.iter().any(|n| !n.children.is_empty()) {
        return None;
    }
    let mut rows = Vec::with_capacity(tree.len());
    for n in &tree {
        rows.push(leaf_to_make_row(n)?);
    }
    Some((rows, false, "ROOT".into()))
}

fn apply_preset_template_to_rows(state: &mut BlazeMakeWorkbenchState, pick: usize) {
    let Some(ent) = NAMED_BLAZE_COMMANDS.get(pick) else {
        return;
    };
    let sample = build_preset_request_sample(ent.component, ent.command);
    if let Some((rows, wrap, outer)) = try_tdf_payload_to_make_rows(sample.as_slice()) {
        state.rows = rows;
        state.wrap_struct = wrap;
        state.struct_outer_tag = outer;
    } else if sample.is_empty() {
        state.rows = vec![BlazeTdfMakeRow::default()];
        state.wrap_struct = false;
    } else {
        state.rows = vec![BlazeTdfMakeRow {
            tag: String::new(),
            kind: TdfMakeFieldKind::RawHex,
            value: hex::encode(&sample),
        }];
        state.wrap_struct = false;
    }
    state.blaze_rows_from_preset = Some(pick);
    refresh_preview(state);
}

fn maybe_sync_blaze_preset_rows(state: &mut BlazeMakeWorkbenchState) {
    if !state.use_preset {
        return;
    }
    let pick = state.preset_pick;
    if state.blaze_rows_from_preset == Some(pick) {
        return;
    }
    apply_preset_template_to_rows(state, pick);
}

fn kind_easy_title(kind: TdfMakeFieldKind) -> &'static str {
    match kind {
        TdfMakeFieldKind::String => "Text",
        TdfMakeFieldKind::Int => "Integer",
        TdfMakeFieldKind::BlobHex => "Tagged binary (hex)",
        TdfMakeFieldKind::RawHex => "Raw hex segment",
    }
}

fn render_envelope(ui: &mut egui::Ui, state: &mut BlazeMakeWorkbenchState) -> bool {
    let prev_preset_on = state.use_preset;
    let preset_row = ui.horizontal(|ui| {
        ui.checkbox(&mut state.use_preset, "Preset");
        if state.use_preset {
            if state.preset_pick >= NAMED_BLAZE_COMMANDS.len() {
                state.preset_pick = 0;
            }
            let selected_text =
                if let Some(entry) = NAMED_BLAZE_COMMANDS.get(state.preset_pick) {
                    format!("{} (0x{:04x}:{:04x})", entry.name, entry.component, entry.command)
                } else {
                    "(none)".into()
                };
            egui::ComboBox::from_id_source("blaze_presets")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (i, entry) in NAMED_BLAZE_COMMANDS.iter().enumerate() {
                        if ui
                            .selectable_label(state.preset_pick == i, format!(
                                "{} (0x{:04x}:{:04x})",
                                entry.name, entry.component, entry.command
                            ))
                            .clicked()
                        {
                            state.blaze_rows_from_preset = None;
                            state.preset_pick = i;
                        }
                    }
                });
            if let Some(ent) = NAMED_BLAZE_COMMANDS.get(state.preset_pick) {
                state.component_s = format!("0x{:04x}", ent.component);
                state.command_s = format!("0x{:04x}", ent.command);
            }
        }
        if prev_preset_on && !state.use_preset {
            state.blaze_rows_from_preset = None;
        }
    });

    let fields_row = ui.horizontal(|ui| {
        ui.label("component");
        ui.add(
            egui::TextEdit::singleline(&mut state.component_s)
                .desired_width(90.0)
                .interactive(!state.use_preset),
        );
        ui.label("command");
        ui.add(
            egui::TextEdit::singleline(&mut state.command_s)
                .desired_width(90.0)
                .interactive(!state.use_preset),
        );
        ui.label("msg num");
        ui.add(egui::TextEdit::singleline(&mut state.msg_num_s).desired_width(80.0));
        ui.label("type");
        egui::ComboBox::from_id_source("blaze_msg_ty_make")
            .selected_text(state.msg_ty.to_string())
            .show_ui(ui, |ui| {
                for ty in [
                    MessageType::Message,
                    MessageType::Reply,
                    MessageType::Notification,
                    MessageType::ErrorReply,
                    MessageType::Ping,
                    MessageType::PingReply,
                ] {
                    ui.selectable_value(&mut state.msg_ty, ty, ty.to_string());
                }
            });
    });

    let env_changed = preset_row.response.changed() || fields_row.response.changed();

    if let Ok(c) = parse_u16_radix(&state.component_s) {
        if let Ok(cmd) = parse_u16_radix(&state.command_s) {
            let label = get_command_name(c, cmd)
                .map(|s| format!("{}", s))
                .unwrap_or_else(|| "custom / unknown opcode".into());
            ui.label(egui::RichText::new(label).weak());
        }
    }

    env_changed
}

fn render_tdf_preview_block(ui: &mut egui::Ui, state: &BlazeMakeWorkbenchState, preview_height: f32) {
    if let Some(ref err) = state.tdf_preview_err {
        ui.label(egui::RichText::new(err).color(egui::Color32::YELLOW));
    }
    if let Some(ref tree) = state.tdf_tree_preview {
        ui.label(egui::RichText::new("TDF struct (preview)").heading());
        egui::ScrollArea::vertical()
            .max_height(preview_height)
            .show(ui, |ui| {
                ui.monospace(egui::RichText::new(tree.as_str()).size(11.0));
            });
    } else if state.tdf_preview_err.is_none() {
        ui.label(
            egui::RichText::new("Preview empty — fix field values or wrap options.")
                .weak()
                .italics(),
        );
    }
}

fn render_wire_dump(ui: &mut egui::Ui, state: &BlazeMakeWorkbenchState) {
    if let Some(ref pay) = state.payload_hex {
        if let Ok(bytes) = hex::decode(pay) {
            ui.label(egui::RichText::new("Hex:").heading());
            ui.monospace(
                egui::RichText::new(format_hex_dump(&bytes, 4096)).size(10.0),
            );
        }
    }
    if let Some(ref w) = state.wire_hex {
        if let Ok(bytes) = hex::decode(w) {
            ui.label(egui::RichText::new("Wire hex:").weak());
            ui.monospace(
                egui::RichText::new(format_hex_dump(&bytes, 8192)).size(10.0),
            );
        }
    }
}

fn render_make_actions(ui: &mut egui::Ui, state: &mut BlazeMakeWorkbenchState, ctx: &egui::Context) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .button(egui::RichText::new("Build wire").strong())
            .on_hover_text(
                "Assemble payload + header; refresh hex summaries and TDF preview.",
            )
            .clicked()
        {
            state.blaze_action_note = None;
            state.payload_hex = None;
            state.wire_hex = None;
            if let Err(e) = build_or_err(state) {
                state.build_err = Some(e);
            }
        }
        if ui
            .button(egui::RichText::new("Send to client").strong())
            .on_hover_text(
                "Push plaintext Fire2Frame on the emulator inject bus; each Blaze TCP client encrypts with c_out if needed.",
            )
            .clicked()
        {
            state.blaze_action_note = None;
            match try_make_fire2_packet(state) {
                Ok(pkt) => {
                    apply_build_outputs(state, &pkt);
                    let wire = pkt.to_bytes().to_vec();
                    match broadcast_toolkit_blaze_wire(wire) {
                        Ok(n) => {
                            state.blaze_action_note =
                                Some(format!("Inject sent ({} subscriber(s)).", n));
                        }
                        Err(_) => {
                            state.blaze_action_note = Some(
                                "Inject channel idle — connect a Blaze client on the emulator first."
                                    .into(),
                            );
                        }
                    }
                }
                Err(e) => state.build_err = Some(e),
            }
        }

        match try_make_fire2_packet(state) {
            Ok(pkt) => {
                if ui
                    .small_button("Copy wire (hex)")
                    .on_hover_text("Rebuild wire from current fields into clipboard.")
                    .clicked()
                {
                    apply_build_outputs(state, &pkt);
                    ctx.copy_text(hex::encode(pkt.to_bytes()));
                }
                if ui
                    .small_button("Copy payload (hex)")
                    .on_hover_text("TDF payload only (decoded bytes as hex).")
                    .clicked()
                {
                    apply_build_outputs(state, &pkt);
                    ctx.copy_text(hex::encode(pkt.payload.as_ref()));
                }
            }
            Err(_) => {
                ui.add_enabled(false, egui::Button::new("Copy wire (hex)"));
                ui.add_enabled(false, egui::Button::new("Copy payload (hex)"));
            }
        }

        if ui
            .small_button("Save wire…")
            .on_hover_text("Save full packet bytes (binary).")
            .clicked()
        {
            state.blaze_action_note = None;
            match try_make_fire2_packet(state) {
                Ok(pkt) => {
                    apply_build_outputs(state, &pkt);
                    let raw = pkt.to_bytes();
                    let file = rfd::FileDialog::new()
                        .add_filter("Binary", &["bin"])
                        .add_filter("All files", &["*"])
                        .set_file_name("blaze_fire2frame.bin")
                        .save_file();
                    if let Some(path) = file {
                        if let Err(e) = std::fs::write(path, raw) {
                            state.blaze_action_note = Some(format!("Save failed: {}", e));
                        } else {
                            state.blaze_action_note = Some("Wire saved.".into());
                        }
                    }
                }
                Err(e) => state.build_err = Some(e),
            }
        }
    });

    if let Some(ref note) = state.blaze_action_note {
        ui.label(
            egui::RichText::new(note.as_str())
                .weak()
                .color(egui::Color32::from_rgb(140, 200, 140)),
        );
    }
}

fn render_easy_body(
    ui: &mut egui::Ui,
    state: &mut BlazeMakeWorkbenchState,
    ctx: &egui::Context,
    envelope_dirty: bool,
) {
    ui.label(egui::RichText::new(
        "Easy mode shows a live TDF preview while you edit fields. Switch to Advanced for raw hex rows and full control.",
    )
    .weak()
    .size(11.0));

    ui.label(egui::RichText::new("Structure preview").heading());
    ui.add_space(4.0);

    render_tdf_preview_block(ui, state, 220.0);
    ui.separator();

    let dirty = Cell::new(false);

    ui.horizontal(|ui| {
        if ui.checkbox(&mut state.wrap_struct, "Wrap in one STRUCT").changed() {
            dirty.set(true);
        }
        ui.label("outer tag");
        if ui
            .add(
                egui::TextEdit::singleline(&mut state.struct_outer_tag).desired_width(72.0),
            )
            .changed()
        {
            dirty.set(true);
        }
    });

    ui.horizontal(|ui| {
        ui.menu_button("Add field", |ui| {
            if ui.button("Text (UTF-8 string)").clicked() {
                state.rows.push(BlazeTdfMakeRow {
                    kind: TdfMakeFieldKind::String,
                    ..Default::default()
                });
                dirty.set(true);
                ui.close_menu();
            }
            if ui.button("Integer (i32)").clicked() {
                state.rows.push(BlazeTdfMakeRow {
                    kind: TdfMakeFieldKind::Int,
                    ..Default::default()
                });
                dirty.set(true);
                ui.close_menu();
            }
            if ui.button("Binary with tag (hex)").clicked() {
                state.rows.push(BlazeTdfMakeRow {
                    kind: TdfMakeFieldKind::BlobHex,
                    ..Default::default()
                });
                dirty.set(true);
                ui.close_menu();
            }
            if ui
                .button("Raw hex line (paste)")
                .on_hover_text("Appends literal bytes; no TDF tag wrapper.")
                .clicked()
            {
                state.rows.push(BlazeTdfMakeRow {
                    kind: TdfMakeFieldKind::RawHex,
                    ..Default::default()
                });
                dirty.set(true);
                ui.close_menu();
            }
        });
        if ui.small_button("Remove last").clicked() && state.rows.len() > 1 {
            state.rows.pop();
            dirty.set(true);
        }
    });

    let mut remove_at: Option<usize> = None;
    for (i, row) in state.rows.iter_mut().enumerate() {
        let title = format!("Field {} · {} · «{}»", i + 1, kind_easy_title(row.kind), row.tag.trim());
        ui.group(|ui| {
            ui.label(egui::RichText::new(title).strong());
            ui.horizontal(|ui| {
                ui.label("4-char tag");
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut row.tag)
                            .desired_width(64.0)
                            .hint_text("e.g. STRN"),
                    )
                    .changed()
                {
                    dirty.set(true);
                }
                ui.label("as");
                egui::ComboBox::from_id_source(format!("tdf_easy_kind_{}", i))
                    .selected_text(kind_easy_title(row.kind))
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(&mut row.kind, TdfMakeFieldKind::String, "Text")
                            .clicked()
                        {
                            dirty.set(true);
                        }
                        if ui
                            .selectable_value(&mut row.kind, TdfMakeFieldKind::Int, "Integer")
                            .clicked()
                        {
                            dirty.set(true);
                        }
                        if ui
                            .selectable_value(
                                &mut row.kind,
                                TdfMakeFieldKind::BlobHex,
                                "Tagged binary (hex)",
                            )
                            .clicked()
                        {
                            dirty.set(true);
                        }
                        if ui
                            .selectable_value(
                                &mut row.kind,
                                TdfMakeFieldKind::RawHex,
                                "Raw hex segment",
                            )
                            .clicked()
                        {
                            dirty.set(true);
                        }
                    });
                if ui.small_button("✕").clicked() {
                    remove_at = Some(i);
                }
            });
            let hint = match row.kind {
                TdfMakeFieldKind::String => "value (UTF-8)",
                TdfMakeFieldKind::Int => "integer (decimal, i32)",
                TdfMakeFieldKind::BlobHex => "hex body (spaces optional)",
                TdfMakeFieldKind::RawHex => "paste hex bytes verbatim",
            };
            if ui
                .add(
                    egui::TextEdit::multiline(&mut row.value)
                        .desired_width(ui.available_width())
                        .desired_rows(if row.kind == TdfMakeFieldKind::RawHex { 5 } else { 2 })
                        .hint_text(hint),
                )
                .changed()
            {
                dirty.set(true);
            }
        });
    }
    if let Some(i) = remove_at {
        if state.rows.len() > 1 {
            state.rows.remove(i);
            dirty.set(true);
        }
    }

    if envelope_dirty || dirty.get() || state.tdf_tree_preview.is_none() {
        refresh_preview(state);
    }

    ui.separator();
    ui.label(egui::RichText::new("Outputs").weak());
    render_make_actions(ui, state, ctx);
    ui.separator();

    render_wire_dump(ui, state);
}

fn render_advanced_body(ui: &mut egui::Ui, state: &mut BlazeMakeWorkbenchState, ctx: &egui::Context) {
    ui.label(egui::RichText::new(
        "Advanced mode keeps the compact row editor: raw concatenation order, pasted hex blobs, and manual Refresh.",
    )
    .weak()
    .size(11.0));

    ui.horizontal(|ui| {
        ui.checkbox(&mut state.wrap_struct, "Wrap payload in single STRUCT");
        ui.label("tag");
        ui.add(egui::TextEdit::singleline(&mut state.struct_outer_tag).desired_width(72.0));
    });

    ui.label(egui::RichText::new("TDF fields (top → bottom concatenated):").weak());

    ui.horizontal(|ui| {
        if ui.button("Add row").clicked() {
            state.rows.push(BlazeTdfMakeRow::default());
            refresh_preview(state);
        }
        if ui.button("Refresh TDF preview").clicked() {
            refresh_preview(state);
        }
    });

    let mut remove_at: Option<usize> = None;
    for (i, row) in state.rows.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("tag");
                ui.add(
                    egui::TextEdit::singleline(&mut row.tag)
                        .desired_width(56.0)
                        .hint_text("e.g. STRN"),
                );
                ui.label("kind");
                egui::ComboBox::from_id_source(format!("tdf_kind_{}", i))
                    .selected_text(match row.kind {
                        TdfMakeFieldKind::String => "String",
                        TdfMakeFieldKind::Int => "Int",
                        TdfMakeFieldKind::BlobHex => "Blob (hex)",
                        TdfMakeFieldKind::RawHex => "Raw (hex)",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut row.kind, TdfMakeFieldKind::String, "String");
                        ui.selectable_value(&mut row.kind, TdfMakeFieldKind::Int, "Int");
                        ui.selectable_value(&mut row.kind, TdfMakeFieldKind::BlobHex, "Blob (hex)");
                        ui.selectable_value(&mut row.kind, TdfMakeFieldKind::RawHex, "Raw (hex)");
                    });
                if ui.small_button("✕").clicked() {
                    remove_at = Some(i);
                }
            });
            let hint = match row.kind {
                TdfMakeFieldKind::String => "UTF-8 text",
                TdfMakeFieldKind::Int => "decimal i32",
                TdfMakeFieldKind::BlobHex => "hex bytes (no spaces required)",
                TdfMakeFieldKind::RawHex => "raw segment hex (not TDF-wrapped)",
            };
            ui.add(
                egui::TextEdit::singleline(&mut row.value)
                    .hint_text(hint)
                    .desired_width(ui.available_width()),
            );
        });
    }
    if let Some(i) = remove_at {
        if state.rows.len() > 1 {
            state.rows.remove(i);
        }
        refresh_preview(state);
    }

    render_tdf_preview_block(ui, state, 160.0);

    ui.separator();
    render_make_actions(ui, state, ctx);

    if let Some(ref e) = state.build_err {
        ui.label(egui::RichText::new(e).color(egui::Color32::RED));
    }
    render_wire_dump(ui, state);
}

pub fn render_blaze_make(ui: &mut egui::Ui, state: &mut BlazeMakeWorkbenchState, ctx: &egui::Context) {
    ui.label(
        egui::RichText::new("Blaze packet builder")
            .heading()
            .size(15.0),
    );
    ui.label(egui::RichText::new("Construct Fire2Frame envelopes and TDF payloads.").weak().size(11.0));

    ui.horizontal(|ui| {
        ui.label("Mode");
        ui.selectable_value(&mut state.ui_mode, BlazeMakeUIMode::Easy, "Easy");
        ui.selectable_value(&mut state.ui_mode, BlazeMakeUIMode::Advanced, "Advanced");
    });

    ui.add_space(4.0);
    let envelope_dirty = render_envelope(ui, state);
    maybe_sync_blaze_preset_rows(state);

    ui.separator();

    match state.ui_mode {
        BlazeMakeUIMode::Easy => render_easy_body(ui, state, ctx, envelope_dirty),
        BlazeMakeUIMode::Advanced => render_advanced_body(ui, state, ctx),
    }

    if state.ui_mode == BlazeMakeUIMode::Easy {
        if let Some(ref e) = state.build_err {
            ui.label(egui::RichText::new(e).color(egui::Color32::RED));
        }
    }
}
