// gRPC Inspector - UI for viewing captured gRPC requests/responses

use crate::core::inspector::inspector_module::*;
use crate::grpc::{grpc_body_decode_capture, peel_grpc_data_frames_detailed};
use egui::Color32;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GrpcListDirectionFilter {
    #[default]
    All,
    ClientToServer,
    ServerToClient,
}

impl GrpcListDirectionFilter {
    fn matches(self, d: GrpcDirection) -> bool {
        match self {
            GrpcListDirectionFilter::All => true,
            GrpcListDirectionFilter::ClientToServer => d == GrpcDirection::ClientToServer,
            GrpcListDirectionFilter::ServerToClient => d == GrpcDirection::ServerToClient,
        }
    }

    fn label(self) -> &'static str {
        match self {
            GrpcListDirectionFilter::All => "All directions",
            GrpcListDirectionFilter::ClientToServer => "Client→Server",
            GrpcListDirectionFilter::ServerToClient => "Server→Client",
        }
    }
}

fn grpc_row_matches(g: &CapturedGrpc, filter_trim: &str, dir: GrpcListDirectionFilter) -> bool {
    if !dir.matches(g.direction) {
        return false;
    }
    let ft = filter_trim.trim();
    if ft.is_empty() {
        return true;
    }
    let f = ft.to_lowercase();
    let hay = format!(
        "{} {} {} {} {} {} seq={}",
        g.direction.to_string(),
        g.method,
        g.path,
        g.host,
        g.body_size,
        g.grpc_status.as_deref().unwrap_or(""),
        g.capture_seq
    )
    .to_lowercase();
    if hay.contains(&f) {
        return true;
    }
    if f.chars().all(|c| c.is_ascii_hexdigit()) && f.len() >= 4 && f.len() % 2 == 0 {
        if let Ok(pat) = hex::decode(f.replace(' ', "")) {
            if !pat.is_empty() && pat.len() <= g.body.len() {
                return g.body.windows(pat.len()).any(|w| w == pat.as_slice());
            }
        }
    }
    false
}

/// State for gRPC inspector UI
pub struct GrpcInspectorState {
    pub selected_index: Option<usize>,
    pub show_plaintext: bool,
    pub list_filter: String,
    pub direction_filter: GrpcListDirectionFilter,
    pub pinned_seq: HashSet<u64>,
    /// Listener → Toolkit Make (gRPC), same pattern as Blaze `open_make_from_index`.
    pub open_make_from_index: Option<usize>,
}

impl GrpcInspectorState {
    pub fn new() -> Self {
        Self {
            selected_index: None,
            show_plaintext: false,
            list_filter: String::new(),
            direction_filter: GrpcListDirectionFilter::default(),
            pinned_seq: HashSet::new(),
            open_make_from_index: None,
        }
    }
}

fn chunks_for_listen(grpc: &CapturedGrpc) -> Vec<Vec<u8>> {
    if !grpc.protobuf_chunks.is_empty() {
        grpc.protobuf_chunks.clone()
    } else {
        grpc_body_decode_capture(&grpc.body).protobuf_chunks
    }
}

/// Render gRPC inspector UI
pub fn render_grpc_inspector(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    state: &mut GrpcInspectorState,
    buffer: GrpcBuffer,
) {
    let (grpc_list, total_n) = {
        let buf = buffer.lock();
        let total_n = buf.len();
        let mut rows: Vec<(usize, CapturedGrpc)> = buf
            .iter()
            .enumerate()
            .map(|(i, g)| (i, g.clone()))
            .filter(|(_, g)| grpc_row_matches(g, &state.list_filter, state.direction_filter))
            .collect();
        rows.sort_by(|(ia, ga), (ib, gb)| {
            let ap = state.pinned_seq.contains(&ga.capture_seq);
            let bp = state.pinned_seq.contains(&gb.capture_seq);
            ap.cmp(&bp).then_with(|| ib.cmp(ia))
        });
        (rows, total_n)
    };
    let count = grpc_list.len();

    // Top toolbar
    ui.horizontal(|ui| {
        ui.label(format!("gRPC (showing {} / {} in buffer)", count, total_n));
        ui.checkbox(&mut state.show_plaintext, "Plaintext");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Copy to clipboard button
            if ui.button("📋").clicked() {
                let buffer = buffer.lock();
                let mut output = String::new();
                output.push_str("=== gRPC Inspection Data ===\n\n");

                for (idx, grpc) in buffer.iter().enumerate().rev() {
                    output.push_str(&format!("gRPC #{}:\n", idx));
                    output.push_str(&format!("  Direction: {}\n", grpc.direction.to_string()));
                    output.push_str(&format!("  Method: {}\n", grpc.method));
                    output.push_str(&format!("  Path: {}\n", grpc.path));
                    output.push_str(&format!("  Host: {}\n", grpc.host));
                    output.push_str(&format!("  Body Size: {} bytes\n", grpc.body_size));
                    output.push_str(&format!("  Capture seq: {}\n", grpc.capture_seq));
                    output.push_str(&format!("  Compressed: {}\n", grpc.is_compressed));
                    if let Some(ref status) = grpc.grpc_status {
                        output.push_str(&format!("  gRPC Status: {}\n", status));
                    }
                    output.push_str("\n  Headers:\n");
                    for (key, value) in &grpc.headers {
                        output.push_str(&format!("    {}: {}\n", key, value));
                    }
                    output.push_str("\n  Body:\n");
                    if let Some(ref protobuf) = grpc.protobuf_data {
                        let body_text = bytes_to_plaintext(protobuf);
                        output.push_str(&format!("    {}\n", body_text));
                    } else {
                        let body_text = bytes_to_plaintext(&grpc.body);
                        output.push_str(&format!("    {}\n", body_text));
                    }
                    output.push_str("\n");
                }

                ctx.copy_text(output);
            }

            // Save As button
            if ui.button("Save As...").clicked() {
                let buffer = buffer.lock();
                let mut output = String::new();
                output.push_str("=== gRPC Inspection Data ===\n\n");

                for (idx, grpc) in buffer.iter().enumerate().rev() {
                    output.push_str(&format!("gRPC #{}:\n", idx));
                    output.push_str(&format!("  Direction: {}\n", grpc.direction.to_string()));
                    output.push_str(&format!("  Method: {}\n", grpc.method));
                    output.push_str(&format!("  Path: {}\n", grpc.path));
                    output.push_str(&format!("  Host: {}\n", grpc.host));
                    output.push_str(&format!("  Body Size: {} bytes\n", grpc.body_size));
                    output.push_str(&format!("  Capture seq: {}\n", grpc.capture_seq));
                    output.push_str(&format!("  Compressed: {}\n", grpc.is_compressed));
                    if let Some(ref status) = grpc.grpc_status {
                        output.push_str(&format!("  gRPC Status: {}\n", status));
                    }
                    output.push_str("\n  Headers:\n");
                    for (key, value) in &grpc.headers {
                        output.push_str(&format!("    {}: {}\n", key, value));
                    }
                    output.push_str("\n  Body:\n");
                    if let Some(ref protobuf) = grpc.protobuf_data {
                        let body_text = bytes_to_plaintext(protobuf);
                        output.push_str(&format!("    {}\n", body_text));
                    } else {
                        let body_text = bytes_to_plaintext(&grpc.body);
                        output.push_str(&format!("    {}\n", body_text));
                    }
                    output.push_str("\n");
                }
                drop(buffer);

                let file = rfd::FileDialog::new()
                    .add_filter("Text files", &["txt"])
                    .add_filter("All files", &["*"])
                    .set_file_name("grpc.txt")
                    .save_file();

                if let Some(path) = file {
                    if let Err(e) = std::fs::write(&path, output) {
                        eprintln!("Failed to save gRPC data: {}", e);
                    }
                }
            }

            if ui.button("Clear").clicked() {
                let mut buf = buffer.lock();
                buf.clear();
                state.selected_index = None;
                state.open_make_from_index = None;
                state.pinned_seq.clear();
            }
        });
    });

    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.add(
            egui::TextEdit::singleline(&mut state.list_filter)
                .desired_width(220.0)
                .hint_text("path, host, method, seq, hex…"),
        );
        egui::ComboBox::from_id_source("grpc_dir_filter")
            .selected_text(state.direction_filter.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.direction_filter,
                    GrpcListDirectionFilter::All,
                    GrpcListDirectionFilter::All.label(),
                );
                ui.selectable_value(
                    &mut state.direction_filter,
                    GrpcListDirectionFilter::ClientToServer,
                    GrpcListDirectionFilter::ClientToServer.label(),
                );
                ui.selectable_value(
                    &mut state.direction_filter,
                    GrpcListDirectionFilter::ServerToClient,
                    GrpcListDirectionFilter::ServerToClient.label(),
                );
            });
    });

    ui.separator();

    // Two-panel layout: List | Details
    ui.columns(2, |columns| {
        // Left panel: gRPC list
        columns[0].vertical(|ui| {
            ui.heading("gRPC List");
            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("grpc_list_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (idx, grpc) in &grpc_list {
                        let is_selected = state.selected_index == Some(*idx);
                        let direction_color = match grpc.direction {
                            GrpcDirection::ClientToServer => Color32::from_rgb(100, 150, 255),
                            GrpcDirection::ServerToClient => Color32::from_rgb(255, 150, 100),
                        };

                        ui.horizontal(|ui| {
                            let pinned = state.pinned_seq.contains(&grpc.capture_seq);
                            if ui
                                .selectable_label(pinned, "📌")
                                .on_hover_text("Pin / unpin")
                                .clicked()
                            {
                                if pinned {
                                    state.pinned_seq.remove(&grpc.capture_seq);
                                } else {
                                    state.pinned_seq.insert(grpc.capture_seq);
                                }
                            }

                            let response = ui.selectable_label(
                                is_selected,
                                format!(
                                    "[{}] {} {} | {} bytes | seq={}",
                                    grpc.direction.to_string(),
                                    grpc.method,
                                    grpc.path,
                                    grpc.body_size,
                                    grpc.capture_seq
                                ),
                            );

                            if response.clicked() {
                                state.selected_index = Some(*idx);
                            }

                            if is_selected {
                                ui.painter().rect_filled(
                                    response.rect,
                                    0.0,
                                    direction_color.linear_multiply(0.2),
                                );
                            }
                        });
                    }
                });
        });

        // Right panel: Details
        columns[1].vertical(|ui| {
            ui.heading("Details");
            ui.separator();

            if let Some(idx) = state.selected_index {
                if ui
                    .button("Open in Make (gRPC)")
                    .on_hover_text("Jump to Toolkit → Make → gRPC with this body as hex.")
                    .clicked()
                {
                    state.open_make_from_index = Some(idx);
                }
                ui.separator();
                if let Some(grpc) = buffer.lock().get(idx) {
                    render_grpc_details(ui, grpc, state.show_plaintext);
                }
            } else {
                ui.label("Select a gRPC request/response to view details");
            }
        });
    });
}

/// Convert bytes to plaintext, falling back to hex if invalid UTF-8
fn bytes_to_plaintext(data: &[u8]) -> String {
    // Try to decode as UTF-8
    if let Ok(text) = std::str::from_utf8(data) {
        // Check if it's mostly printable ASCII or valid UTF-8
        if text.chars().all(|c| c.is_ascii() || !c.is_control() || c == '\n' || c == '\r' || c == '\t') {
            return text.to_string();
        }
    }
    // Fall back to hex dump
    format_hex_dump(data, 4096)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProtobufWireDumpOpts {
    /// Length-delimited fields are shown as strings/escaped bytes only (never as nested `{}` messages).
    pub prefer_raw_strings: bool,
}

/// Decode protobuf wire format to human-readable text
fn decode_protobuf_raw(data: &[u8], indent: usize, opts: ProtobufWireDumpOpts) -> String {
    decode_protobuf_raw_with_pos(data, indent, 0, None, opts).0
}

/// Protobuf wire walk (field numbers, wire types, length-delimited nests) for toolkit decoding.
pub fn protobuf_wire_format_dump_with_opts(data: &[u8], opts: ProtobufWireDumpOpts) -> String {
    decode_protobuf_raw(data, 0, opts)
}

pub fn protobuf_wire_format_dump(data: &[u8]) -> String {
    protobuf_wire_format_dump_with_opts(data, ProtobufWireDumpOpts::default())
}

fn trim_trailing_zero_bytes(mut data: &[u8]) -> &[u8] {
    while data.last().copied() == Some(0u8) {
        data = &data[..data.len().saturating_sub(1)];
    }
    data
}

fn protobuf_field_tag_looks_reasonable(tag: u64, tag_byte_len: usize) -> bool {
    if tag_byte_len == 0 || tag_byte_len > 5 {
        return false;
    }
    let field_num = (tag >> 3) as u32;
    let wire_type = (tag & 7) as u32;
    field_num > 0 && field_num < 536_870_912 && wire_type <= 5
}

fn protobuf_nested_output_usable(nested: &str) -> bool {
    nested.lines().any(|line| {
        let t = line.trim_start();
        if t.starts_with('<') || t.contains("<decode error") || t.contains("<invalid field tag") {
            return false;
        }
        let Some(fc) = t.chars().next() else {
            return false;
        };
        fc.is_ascii_digit()
            && t.contains(':')
            && !t.contains(": <unknown wire type ")
    })
}

/// Submessages never start with an end-group tag; ASCII like `b't'` (0x74) decodes to field 14 / wire 4.
fn length_delimited_nested_prefix_plausible(payload: &[u8]) -> bool {
    use crate::grpc::decode_varint;
    let Ok((tag, tl)) = decode_varint(payload, 0) else {
        return false;
    };
    if !protobuf_field_tag_looks_reasonable(tag, tl) {
        return false;
    }
    (tag & 7) != 4
}

fn utf8_ratio_non_control(s: &str) -> f32 {
    let n = s.chars().count().max(1);
    let good = s
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
        .count();
    good as f32 / n as f32
}

/// Protobuf framing uses many C0 controls (they are valid UTF-8 bytes). Skip “human UTF-8” shortcuts when
/// this fraction is too high — prefer nested protobuf instead of `\u{{0012}}` escapes.
fn utf8_c0_control_frac_except_ws(s: &str) -> f32 {
    let n = s.chars().count().max(1);
    let bad = s
        .chars()
        .filter(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        .count();
    bad as f32 / n as f32
}

#[inline]
fn has_ascii_controls_except_tab_lf_cr(payload: &[u8]) -> bool {
    payload
        .iter()
        .any(|&b| b < 32 && ![9u8, 10, 13].contains(&b))
}

fn format_ld_mixed_ascii_hex(payload: &[u8]) -> String {
    const INLINE_MAX: usize = 1024;
    let take = payload.len().min(INLINE_MAX);
    let mut s = String::with_capacity(take.saturating_mul(2));
    for &b in &payload[..take] {
        match b {
            b'\\' => s.push_str("\\\\"),
            b'"' => s.push_str("\\\""),
            b'\t' => s.push_str("\\t"),
            b'\n' => s.push_str("\\n"),
            b'\r' => s.push_str("\\r"),
            32..=126 => s.push(b as char),
            _ => s.push_str(&format!("\\x{:02x}", b)),
        }
    }
    if payload.len() > take {
        s.push_str(&format!(" … (+{} bytes)", payload.len() - take));
    }
    s
}

/// Escapes for protobuf length-delimited UTF-8 quoted in the dissect view: use `\xNN` for ASCII controls
/// (excluding tab/LF/CR already spelled out), `\u{….}` for unusual Unicode controls only.
fn grpc_wire_push_utf8_quoted_escapes(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() && (c as u32) < 128 && !matches!(c, '\n' | '\r' | '\t') => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c if c.is_control() => {
                out.push_str(&format!("\\u{{{:04x}}}", c as u32));
            }
            c => out.push(c),
        }
    }
}

/// True for real copy (titles, inbox text). False for STX-prefixed keys, embedded protobuf-ish UTF-8, NUL-terminated records.
fn length_delimited_utf8_human_quotable(payload: &[u8], prefer_raw_strings: bool) -> bool {
    let Ok(s) = std::str::from_utf8(payload) else {
        return false;
    };
    if s.chars().count() < 2 {
        return false;
    }
    if payload.contains(&0) {
        return false;
    }
    if has_ascii_controls_except_tab_lf_cr(payload) {
        return false;
    }
    let ctrl_char = utf8_c0_control_frac_except_ws(s);
    if ctrl_char
        > if prefer_raw_strings {
            0.09
        } else {
            0.048
        }
    {
        return false;
    }
    let ratio = utf8_ratio_non_control(s);
    let thresh = if prefer_raw_strings { 0.52 } else { 0.68 };
    ratio >= thresh && s.chars().any(|c| c.is_alphabetic() || c.is_numeric())
}

fn length_delimited_use_mixed_hex_line(payload: &[u8]) -> bool {
    if std::str::from_utf8(payload).is_err() {
        return true;
    }
    if payload.iter().any(|&b| b == 0) {
        return true;
    }
    if payload.first() == Some(&0x02) {
        return true;
    }
    has_ascii_controls_except_tab_lf_cr(payload)
}

/// Product titles and similar: valid UTF-8 with letters — do not treat as nested protobuf just because
/// the first byte forms a varint (e.g. `t` → fake "end group").
fn length_delimited_prefers_utf8_display(payload: &[u8], prefer_raw_strings: bool) -> bool {
    length_delimited_utf8_human_quotable(payload, prefer_raw_strings)
}

/// Between strict [`length_delimited_utf8_human_quotable`] and mixed-hex blobs (e.g. catalog STX rows).
fn length_delimited_utf8_relaxed_plain_quote(payload: &[u8], prefer_raw_strings: bool) -> bool {
    let Ok(s) = std::str::from_utf8(payload) else {
        return false;
    };
    if s.chars().count() < 2 {
        return false;
    }
    if payload.contains(&0) {
        return false;
    }
    if has_ascii_controls_except_tab_lf_cr(payload) {
        return false;
    }
    let ctrl_char = utf8_c0_control_frac_except_ws(s);
    if ctrl_char
        > if prefer_raw_strings {
            0.095
        } else {
            0.058
        }
    {
        return false;
    }
    let ratio = utf8_ratio_non_control(s);
    let thresh = if prefer_raw_strings { 0.52 } else { 0.72 };
    ratio > thresh && s.chars().any(|c| c.is_alphabetic() || c.is_numeric())
}

fn wire_varint_field_payload_fits(data: &[u8], tag_pos: usize) -> bool {
    use crate::grpc::decode_varint;
    let Ok((tag, tl)) = decode_varint(data, tag_pos) else {
        return false;
    };
    if !protobuf_field_tag_looks_reasonable(tag, tl) {
        return false;
    }
    let wt = (tag & 7) as u32;
    let p = tag_pos + tl;
    match wt {
        0 => decode_varint(data, p).is_ok(),
        1 => p + 8 <= data.len(),
        2 => {
            let Ok((inner_len, lc)) = decode_varint(data, p) else {
                return false;
            };
            let inner = inner_len as usize;
            p + lc + inner <= data.len()
        }
        5 => p + 4 <= data.len(),
        _ => false,
    }
}

/// Decode protobuf with position tracking and optional end group matching
/// Returns (output_string, final_position)
fn decode_protobuf_raw_with_pos(
    data: &[u8],
    indent: usize,
    start_pos: usize,
    end_group: Option<(u32, u32)>, // (field_num, wire_type) to stop at
    opts: ProtobufWireDumpOpts,
) -> (String, usize) {
    use crate::grpc::decode_varint;
    
    let mut output = String::new();
    let indent_str = "  ".repeat(indent);
    let mut pos = start_pos;
    
    while pos < data.len() {
        // Check if we've reached the end group marker
        if let Some((end_field_num, end_wire_type)) = end_group {
            // Peek at the next tag to see if it's the end group
            if let Ok((peek_tag, _)) = decode_varint(data, pos) {
                let peek_field_num = (peek_tag >> 3) as u32;
                let peek_wire_type = (peek_tag & 0x7) as u32;
                if peek_wire_type == end_wire_type && peek_field_num == end_field_num {
                    // Found the end group, stop parsing
                    break;
                }
            }
        }
        
        // Decode field tag (field number + wire type)
        let (field_tag, tag_len) = match decode_varint(data, pos) {
            Ok((tag, len)) => {
                // Validate: field tag should be reasonable
                // Field numbers are typically 1-536870911 (max 29-bit number)
                // Tag length should be reasonable (1-5 bytes for varint)
                if len > 5 || tag > 0x1FFFFFFF {
                    // Likely invalid - show raw bytes and try to continue
                    let bytes_to_show = (data.len() - pos).min(8);
                    let raw_bytes: Vec<String> = data[pos..pos + bytes_to_show]
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect();
                    output.push_str(&format!(
                        "{}<invalid field tag at offset {}: tag=0x{:x}, len={}, bytes=[{}]>\n",
                        indent_str,
                        pos,
                        tag,
                        len,
                        raw_bytes.join(" ")
                    ));
                    // Skip one byte and try to continue
                    pos += 1;
                    continue;
                }
                (tag, len)
            },
            Err(_) => {
                // Can't decode varint - show raw bytes and try to continue
                let bytes_to_show = (data.len() - pos).min(8);
                let raw_bytes: Vec<String> = data[pos..pos + bytes_to_show]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                output.push_str(&format!(
                    "{}<decode error at offset {}: bytes=[{}]>\n",
                    indent_str,
                    pos,
                    raw_bytes.join(" ")
                ));
                // Skip one byte and try to continue
                pos += 1;
                continue;
            }
        };
        
        pos += tag_len;
        if pos >= data.len() {
            break;
        }
        
        let field_num = (field_tag >> 3) as u32;
        let wire_type = (field_tag & 0x7) as u32;

        // Additional validation: field number should be > 0
        if field_num == 0 {
            pos -= tag_len;
            output.push_str(&format!(
                "{}<invalid field number 0 at offset {}, tag=0x{:x}>\n",
                indent_str,
                pos,
                field_tag
            ));
            pos += 1;
            continue;
        }

        match wire_type {
            0 => {
                // Varint (int32, int64, uint32, uint64, sint32, sint64, bool, enum)
                let (value, varint_len) = match decode_varint(data, pos) {
                    Ok((v, len)) => (v, len),
                    Err(_) => break,
                };
                pos += varint_len;
                output.push_str(&format!("{}{}: {}\n", indent_str, field_num, value));
            }
            1 => {
                // Fixed64 (fixed64, sfixed64, double)
                if pos + 8 > data.len() {
                    break;
                }
                let bytes = [
                    data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
                    data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7],
                ];
                let value = u64::from_le_bytes(bytes);
                let double_value = f64::from_bits(value);
                output.push_str(&format!("{}{}: {}\n", indent_str, field_num, double_value));
                pos += 8;
            }
            2 => {
                // Length-delimited (string, bytes, embedded messages, packed repeated fields)
                let (length, len_consumed) = match decode_varint(data, pos) {
                    Ok((len, consumed)) => (len, consumed),
                    Err(_) => break,
                };
                pos += len_consumed;

                const LENGTH_DELIMITED_OFFBY1_MAX: usize = 512;
                let need = length as usize;
                let rem_avail = data.len().saturating_sub(pos);

                let field_data = if pos + need <= data.len() {
                    let slice = &data[pos..pos + need];
                    pos += need;
                    slice
                } else if need == rem_avail.saturating_add(1)
                    && rem_avail > 0
                    && need <= LENGTH_DELIMITED_OFFBY1_MAX
                {
                    output.push_str(&format!(
                        "{}// field {}: declared len {} exceeds capture by 1 (using {} bytes)\n",
                        indent_str, field_num, length, rem_avail
                    ));
                    let slice = &data[pos..];
                    pos = data.len();
                    slice
                } else {
                    output.push_str(&format!(
                        "{}{}: <length-delimited truncated: declared len {} > remaining {} bytes — tail as hex>\n",
                        indent_str, field_num, length, rem_avail
                    ));
                    if rem_avail > 0 {
                        output.push_str(&format!("{}{}: \"", indent_str, field_num));
                        for &byte in data[pos..].iter().take(96) {
                            output.push_str(&format!("\\x{:02x}", byte));
                        }
                        if rem_avail > 96 {
                            output.push_str("...");
                        }
                        output.push_str("\"\n");
                    }
                    pos = data.len();
                    break;
                };

                let payload = trim_trailing_zero_bytes(field_data);
                if payload.is_empty() {
                    output.push_str(&format!("{}{}: <empty length-delimited>\n", indent_str, field_num));
                    continue;
                }

                if length_delimited_prefers_utf8_display(payload, opts.prefer_raw_strings) {
                    let string_value = std::str::from_utf8(payload).expect("utf-8 checked above");
                    let mut escaped = String::new();
                    grpc_wire_push_utf8_quoted_escapes(&mut escaped, string_value);
                    output.push_str(&format!("{}{}: \"{}\"\n", indent_str, field_num, escaped));
                    continue;
                }

                let nested_ok = (!opts.prefer_raw_strings).then(|| {
                    if !length_delimited_nested_prefix_plausible(payload) {
                        return None;
                    }
                    decode_varint(payload, 0)
                        .ok()
                        .filter(|(tag, tag_len)| protobuf_field_tag_looks_reasonable(*tag, *tag_len))
                        .and_then(|_| {
                            let (nested, nested_pos) =
                                decode_protobuf_raw_with_pos(payload, indent + 1, 0, None, opts);
                            let tail_empty_or_zeros = payload.get(nested_pos..).unwrap_or(&[]).iter().all(|&b| b == 0);
                            let consumed = nested_pos.min(payload.len());
                            let consumption_ok = consumed * 10 >= payload.len() * 8;
                            if nested_pos <= payload.len()
                                && nested_pos > 0
                                && consumption_ok
                                && protobuf_nested_output_usable(&nested)
                                && (nested_pos == payload.len() || tail_empty_or_zeros)
                            {
                                Some(nested)
                            } else {
                                None
                            }
                        })
                })
                .flatten();

                if let Some(nested) = nested_ok.as_ref() {
                    output.push_str(&format!("{}{} {{\n", indent_str, field_num));
                    output.push_str(&nested);
                    output.push_str(&format!("{}}}\n", indent_str));
                    continue;
                }

                let prefer_raw = opts.prefer_raw_strings;
                if length_delimited_utf8_relaxed_plain_quote(payload, prefer_raw) {
                    let string_value =
                        std::str::from_utf8(payload).expect("relaxed quotable implies utf8");
                    let escaped = string_value
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t");
                    output.push_str(&format!("{}{}: \"{}\"\n", indent_str, field_num, escaped));
                    continue;
                }

                let display_data = payload;
                if std::str::from_utf8(display_data).is_ok() {
                    if length_delimited_use_mixed_hex_line(display_data) {
                        output.push_str(&format!(
                            "{}// {} bytes (mixed binary — \\x escapes, printable ASCII unchanged)\n",
                            indent_str,
                            display_data.len()
                        ));
                        output.push_str(&format!(
                            "{}{}: \"{}\"\n",
                            indent_str,
                            field_num,
                            format_ld_mixed_ascii_hex(display_data)
                        ));
                    } else {
                        let string_value = std::str::from_utf8(display_data).unwrap();
                        let mut escaped = String::new();
                        grpc_wire_push_utf8_quoted_escapes(&mut escaped, string_value);
                        output.push_str(&format!(
                            "{}// {} bytes UTF-8 (unusual BMP controls — escaped)\n",
                            indent_str,
                            display_data.len()
                        ));
                        output.push_str(&format!("{}{}: \"{}\"\n", indent_str, field_num, escaped));
                    }
                } else {
                    output.push_str(&format!(
                        "{}// {} bytes (non-UTF-8 hex)\n",
                        indent_str,
                        display_data.len()
                    ));
                    output.push_str(&format!("{}{}: \"", indent_str, field_num));
                    for &byte in display_data.iter().take(200) {
                        output.push_str(&format!("\\x{:02x}", byte));
                    }
                    if display_data.len() > 200 {
                        output.push_str("...");
                    }
                    output.push_str("\"\n");
                }
            }
            3 => {
                // Start group (deprecated, but still used in some legacy protobuf)
                // Parse the group recursively until we find the matching end group
                output.push_str(&format!("{}{} {{\n", indent_str, field_num));
                
                // Recursively parse the group content, stopping at the matching end group
                let (group_output, new_pos) = decode_protobuf_raw_with_pos(
                    data,
                    indent + 1,
                    pos,
                    Some((field_num, 4)), // Stop at end group (wire type 4) with same field number
                    opts,
                );
                output.push_str(&group_output);
                
                // Skip the end group tag
                if new_pos < data.len() {
                    let (end_tag, end_tag_len) = match decode_varint(data, new_pos) {
                        Ok((tag, len)) => (tag, len),
                        Err(_) => {
                            pos = new_pos;
                            output.push_str(&format!("{}}}\n", indent_str));
                            continue;
                        }
                    };
                    let end_field_num = (end_tag >> 3) as u32;
                    let end_wire_type = (end_tag & 0x7) as u32;
                    if end_wire_type == 4 && end_field_num == field_num {
                        pos = new_pos + end_tag_len; // Skip the end group tag
                    } else {
                        pos = new_pos; // Didn't find expected end group, use current position
                    }
                } else {
                    pos = new_pos;
                }
                
                output.push_str(&format!("{}}}\n", indent_str));
            }
            4 => {
                // End group (deprecated, but still used in some legacy protobuf)
                // End group is just a tag, no data follows
                // Note: This should normally be consumed as part of a start group (wire type 3)
                // But if we encounter it standalone, we'll just note it
                output.push_str(&format!(
                    "{}{}: <end group> [tag: 0x{:x}]\n",
                    indent_str,
                    field_num,
                    field_tag
                ));
                // No data to skip, just the tag which we already consumed
            }
            5 => {
                // Fixed32 (fixed32, sfixed32, float)
                if pos + 4 > data.len() {
                    break;
                }
                let bytes = [data[pos], data[pos + 1], data[pos + 2], data[pos + 3]];
                let value = u32::from_le_bytes(bytes);
                let float_value = f32::from_bits(value);
                output.push_str(&format!("{}{}: {}\n", indent_str, field_num, float_value));
                pos += 4;
            }
            6 | 7 => {
                pos -= tag_len;
                let scan_start = pos;
                let scan_end = data.len().min(pos.saturating_add(4096));
                let mut resync = None;
                let mut j = pos + 1;
                while j + 1 < scan_end {
                    if wire_varint_field_payload_fits(data, j) {
                        resync = Some(j);
                        break;
                    }
                    j += 1;
                }
                if let Some(at) = resync {
                    output.push_str(&format!(
                        "{}<reserved wire type {} at offset {}; resync scan → plausible tag at {}>\n",
                        indent_str, wire_type, scan_start, at
                    ));
                    pos = at;
                } else {
                    output.push_str(&format!(
                        "{}<reserved wire type {} at offset {}; no resync — skip 1 byte>\n",
                        indent_str, wire_type, scan_start
                    ));
                    pos = scan_start + 1;
                }
            }
            _ => {
                pos -= tag_len;
                if pos < data.len() {
                    output.push_str(&format!(
                        "{}<skip 1 byte at {} (unexpected wire type bits {})>\n",
                        indent_str, pos, wire_type
                    ));
                    pos += 1;
                } else {
                    break;
                }
            }
        }
    }
    
    (output, pos)
}

fn render_grpc_details(ui: &mut egui::Ui, grpc: &CapturedGrpc, show_plaintext: bool) {
    egui::ScrollArea::vertical()
        .id_source("grpc_details_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Direction:").heading());
            let direction_color = match grpc.direction {
                GrpcDirection::ClientToServer => Color32::from_rgb(100, 150, 255),
                GrpcDirection::ServerToClient => Color32::from_rgb(255, 150, 100),
            };
            ui.label(egui::RichText::new(grpc.direction.to_string()).color(direction_color));
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Method:").heading());
            ui.label(&grpc.method);
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Path:").heading());
            ui.label(&grpc.path);
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Host:").heading());
            ui.label(&grpc.host);
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Compressed (frame bits):").heading());
            ui.label(if grpc.is_compressed { "Yes (gzip in gRPC DATA)" } else { "No" });
            ui.add_space(5.0);

            if let Some((_k, enc)) = grpc
                .headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("grpc-encoding"))
            {
                ui.label(egui::RichText::new("grpc-encoding header:").heading());
                ui.label(egui::RichText::new(enc).weak());
                ui.add_space(5.0);
            }

            if let Some(ref status) = grpc.grpc_status {
                ui.label(egui::RichText::new("gRPC Status:").heading());
                ui.label(status);
                ui.add_space(5.0);
            }

            ui.label(egui::RichText::new("Body Size:").heading());
            ui.label(format!("{} bytes", grpc.body_size));
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Headers:").heading());
            ui.add_space(5.0);
            for (key, value) in &grpc.headers {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{}:", key))
                        .family(egui::FontFamily::Monospace)
                        .size(10.0));
                    ui.label(egui::RichText::new(value)
                        .family(egui::FontFamily::Monospace)
                        .size(10.0));
                });
            }
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(5.0);

            if !show_plaintext {
                ui.label(egui::RichText::new("Hex (HTTP/2 DATA payload):").heading());
                ui.add_space(5.0);
                egui::ScrollArea::vertical()
                    .id_source("body_scroll_hex")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format_hex_dump(&grpc.body, 4096))
                                .family(egui::FontFamily::Monospace)
                                .size(10.0),
                        );
                    });
            } else {
                let chunks = chunks_for_listen(grpc);
                let (_, slack, _) = peel_grpc_data_frames_detailed(&grpc.body);

                if !chunks.is_empty() {
                    ui.label(egui::RichText::new("Protobuf (schema-free wire walk):").heading());
                    ui.weak(format!(
                        "{} gRPC message partition(s) peeled from DATA (length prefix per message).",
                        chunks.len(),
                    ));
                    ui.add_space(4.0);
                    for (i, protobuf) in chunks.iter().enumerate() {
                        egui::CollapsingHeader::new(format!(
                            "Message {} — {} bytes",
                            i + 1,
                            protobuf.len()
                        ))
                        .default_open(i == 0)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_source(format!("grpc_listen_msg_{}", i))
                                .max_height(260.0)
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(decode_protobuf_raw(protobuf, 0, ProtobufWireDumpOpts::default()))
                                            .family(egui::FontFamily::Monospace)
                                            .size(10.0),
                                    );
                                });
                        });
                    }
                    if !slack.is_empty() {
                        egui::CollapsingHeader::new(format!("Slack tail — {} bytes", slack.len()))
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.weak(
                                    "After last full gRPC frame; may be padding or HPACK/trailer noise.",
                                );
                                egui::ScrollArea::vertical()
                                    .id_source("grpc_listen_slack")
                                    .max_height(120.0)
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(format_hex_dump(slack, 512))
                                                .family(egui::FontFamily::Monospace)
                                                .size(10.0),
                                        );
                                        ui.separator();
                                        ui.label(egui::RichText::new("Protobuf heuristic:").weak());
                                        ui.label(
                                            egui::RichText::new(decode_protobuf_raw(slack, 0, ProtobufWireDumpOpts::default()))
                                                .family(egui::FontFamily::Monospace)
                                                .size(10.0),
                                        );
                                    });
                            });
                    }
                } else if let Some(ref protobuf) = grpc.protobuf_data {
                    ui.label(egui::RichText::new("Protobuf (single chunk):").heading());
                    ui.add_space(5.0);
                    egui::ScrollArea::vertical()
                        .id_source("protobuf_scroll")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(decode_protobuf_raw(protobuf, 0, ProtobufWireDumpOpts::default()))
                                    .family(egui::FontFamily::Monospace)
                                    .size(10.0),
                            );
                        });
                } else {
                    ui.label(egui::RichText::new("Body (no gRPC frames peeled — heuristic):").heading());
                    ui.add_space(5.0);
                    egui::ScrollArea::vertical()
                        .id_source("body_scroll_plain")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let decoded = decode_protobuf_raw(&grpc.body, 0, ProtobufWireDumpOpts::default());
                            let display_text = if decoded.lines().count() >= 2 {
                                decoded
                            } else {
                                bytes_to_plaintext(&grpc.body)
                            };
                            ui.label(
                                egui::RichText::new(display_text)
                                    .family(egui::FontFamily::Monospace)
                                    .size(10.0),
                            );
                        });
                }
            }
        });
}

