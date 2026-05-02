use crate::core::inspector::grpc_inspector::{
    protobuf_wire_format_dump_with_opts, ProtobufWireDumpOpts,
};
use crate::core::inspector::inspector_module::format_hex_dump;
use crate::grpc::{
    decompress_gzip_bytes, looks_like_gzip_prefix,
    peel_grpc_data_frames_detailed_with_first_len_override, MAX_GRPC_MESSAGE_PAYLOAD,
};
use egui;
use std::sync::mpsc::{self, TryRecvError};

pub struct GrpcMakeWorkbenchState {
    pub hex_input: String,
    pub strip_grpc_framing: bool,
    pub auto_peel_gzip: bool,
    /// Prefer showing length-delimited payloads as escaped strings instead of nested message trees.
    pub prefer_raw_strings: bool,
    /// If non-empty decimal or `0x…`: use as **first-frame** declared gRPC DATA payload byte length instead of parsing bytes 1–4.
    pub manual_first_grpc_payload_len: String,
    /// True when gRPC-prefix peel found nothing at offset 0 (Recovery can search / fix truncated length).
    pub needs_calculate_length: bool,
    /// Background length-recovery (`brute_recover_grpc_dissect`) — UI overlay while running.
    pub length_recovery_busy: bool,
    length_recovery_rx: Option<mpsc::Receiver<Result<(String, Vec<String>), String>>>,
    pub status_hint: Option<String>,
    pub dissect_text: Option<String>,
    pub dissect_err: Option<String>,
}

impl Default for GrpcMakeWorkbenchState {
    fn default() -> Self {
        Self {
            hex_input: String::new(),
            strip_grpc_framing: true,
            auto_peel_gzip: true,
            prefer_raw_strings: false,
            manual_first_grpc_payload_len: String::new(),
            needs_calculate_length: false,
            length_recovery_busy: false,
            length_recovery_rx: None,
            status_hint: None,
            dissect_text: None,
            dissect_err: None,
        }
    }
}

fn parse_hex_loose(input: &str) -> Result<Vec<u8>, String> {
    let hex_clean: String = input.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex_clean.is_empty() {
        return Err("No hex digits".into());
    }
    if hex_clean.len() % 2 != 0 {
        return Err("Hex must have an even number of digits".into());
    }
    hex::decode(&hex_clean).map_err(|e| format!("{}", e))
}

#[inline]
fn has_non_empty_hex_input(input: &str) -> bool {
    parse_hex_loose(input).map(|b| !b.is_empty()).unwrap_or(false)
}

#[inline]
fn grpc_declared_len_at(slice: &[u8]) -> Option<(u8, u32)> {
    if slice.len() < 5 {
        return None;
    }
    let flag = slice[0];
    if flag != 0x00 && flag != 0x01 {
        return None;
    }
    Some((
        flag,
        u32::from_be_bytes([slice[1], slice[2], slice[3], slice[4]]),
    ))
}

fn parse_optional_u32_manual(s: &str) -> Result<Option<u32>, String> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        let h = hex.trim_start_matches('_');
        if h.is_empty() {
            return Err("Empty hex after 0x".into());
        }
        u32::from_str_radix(h, 16).map(Some).map_err(|_| format!("Invalid hex `{}`", t))
    } else {
        t.parse::<u32>()
            .map(Some)
            .map_err(|_| format!("Invalid unsigned integer `{}` (use decimal or 0x…)", t))
    }
}

#[inline]
fn manual_first_payload_len(state: &GrpcMakeWorkbenchState) -> Result<Option<u32>, String> {
    parse_optional_u32_manual(&state.manual_first_grpc_payload_len)
}

fn scroll_max(ui: &egui::Ui) -> f32 {
    const FALLBACK: f32 = 440.0;
    let h = ui.available_height();
    if !h.is_finite() || !(80.0..=8000.0).contains(&h) {
        FALLBACK
    } else {
        h.max(200.0)
    }
}

fn peel_gzip_layers(mut data: Vec<u8>, max_layers: usize) -> (Vec<u8>, Vec<String>) {
    let mut notes = Vec::new();
    for i in 0..max_layers {
        if looks_like_gzip_prefix(&data) {
            match decompress_gzip_bytes(&data) {
                Ok(next) => {
                    notes.push(format!(
                        "gzip layer {}: {} → {} bytes",
                        i + 1,
                        data.len(),
                        next.len()
                    ));
                    data = next;
                }
                Err(e) => {
                    notes.push(format!("gzip magic but decode failed: {}", e));
                    break;
                }
            }
        } else {
            break;
        }
    }
    (data, notes)
}

fn compose_dissect_sections(
    preamble: Option<&str>,
    frames: &[Vec<u8>],
    slack: &[u8],
    opts: ProtobufWireDumpOpts,
) -> String {
    let mut out = String::new();
    if let Some(p) = preamble {
        out.push_str(p);
        out.push('\n');
    }
    for (i, f) in frames.iter().enumerate() {
        out.push_str(&format!(
            "=== protobuf message {} ({} bytes · schema-free wire walk) ===\n",
            i + 1,
            f.len()
        ));
        out.push_str(&protobuf_wire_format_dump_with_opts(f, opts));
        out.push('\n');
    }
    if !slack.is_empty() {
        out.push_str(&format!(
            "\n=== slack after last full gRPC frame ({} bytes) ===\n",
            slack.len()
        ));
        let take = slack.len().min(2048);
        out.push_str(&format_hex_dump(&slack[..take], take));
        if slack.len() > take {
            out.push_str("\n… (slack hex truncated)\n");
        }
        out.push_str("\n-- heuristic protobuf parse on slack --\n");
        out.push_str(&protobuf_wire_format_dump_with_opts(slack, opts));
    }
    out
}

fn score_wire_dissect(dump: &str) -> i32 {
    let mut s = 0i32;
    for line in dump.lines() {
        let line = line.trim_start();
        if line.is_empty() {
            continue;
        }
        if let Some(colon) = line.find(':') {
            let head = line[..colon].trim();
            if head.parse::<u32>().is_ok() {
                s += 10;
            }
        }
        let low = line.to_ascii_lowercase();
        if low.contains("<invalid") {
            s -= 15;
        }
        if low.contains("unknown wire type") {
            s -= 12;
        }
        if low.contains("decode error") {
            s -= 8;
        }
        if low.contains("[remaining bytes") {
            s -= 10;
        }
        if low.contains(": \"") || line.contains("\": \"") {
            s += 2;
        }
    }
    let n = dump.lines().count() as i32;
    s + n.min(40)
}

fn sample_truncation_lengths(inner_len: usize) -> Vec<usize> {
    if inner_len == 0 {
        return Vec::new();
    }
    let mut v = Vec::new();
    v.push(inner_len);
    let mut i = inner_len;
    let step = ((inner_len + 127) / 128).max(1);
    let mut strides = 0;
    while i > 8 && strides < 384 {
        i = i.saturating_sub(step);
        v.push(i);
        strides += 1;
    }
    for trim in [1usize, 2, 4, 8] {
        let j = inner_len.saturating_sub(trim);
        if j > 0 {
            v.push(j);
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn try_truncated_grpc_inner(
    slice_offset: usize,
    compression_flag: u8,
    inner: &[u8],
    opts: ProtobufWireDumpOpts,
) -> Option<(i32, String, Vec<String>)> {
    if inner.is_empty() {
        return None;
    }
    let mut hints = vec![format!(
        "Truncated/overlong gRPC length at offset {}; using {} captured bytes inside frame.",
        slice_offset,
        inner.len(),
    )];
    match compression_flag {
        0x00 => {
            let mut best: Option<(i32, usize, String)> = None;
            for len in sample_truncation_lengths(inner.len()) {
                let chunk = inner.get(..len)?;
                let dump = protobuf_wire_format_dump_with_opts(chunk, opts);
                let sc = score_wire_dissect(&dump);
                if best
                    .as_ref()
                    .map(|(bs, _, _)| sc > *bs)
                    .unwrap_or(true)
                {
                    best = Some((sc, len, dump));
                }
            }
            let (sc, len, _) = best?;
            hints.push(format!(
                "Best uncompressed fit: first {} byte(s) of inner (score {}).",
                len, sc
            ));
            let msg = vec![inner.get(..len)?.to_vec()];
            let rest = inner.get(len..).unwrap_or(&[]);
            let text = compose_dissect_sections(
                Some("=== recovery: corrected implicit message length (uncompressed) ==="),
                &msg,
                rest,
                opts,
            );
            Some((sc, text, hints))
        }
        0x01 => {
            let dec = decompress_gzip_bytes(inner).ok()?;
            hints.push("Recovered gzip-compressed inner with buffer-sized payload.".into());
            let dump = protobuf_wire_format_dump_with_opts(&dec, opts);
            let sc = score_wire_dissect(&dump) + 40;
            let text = compose_dissect_sections(
                Some("=== recovery: truncated frame treated as single gzip blob ==="),
                &[dec],
                &[],
                opts,
            );
            Some((sc, text, hints))
        }
        _ => None,
    }
}

struct RecoveryPick {
    score: i32,
    text: String,
    hints: Vec<String>,
}

fn consider_recovery(best: &mut Option<RecoveryPick>, score: i32, text: String, hints: Vec<String>) {
    let pick = RecoveryPick { score, text, hints };
    if best
        .as_ref()
        .map(|b| pick.score > b.score)
        .unwrap_or(true)
    {
        *best = Some(pick);
    }
}

const RECOVERY_SCAN_BYTES: usize = 16384;

fn brute_recover_grpc_dissect(
    raw: &[u8],
    auto_peel_gzip: bool,
    opts: ProtobufWireDumpOpts,
    manual_first_len_override: Option<u32>,
) -> Result<(String, Vec<String>), String> {
    let max_off = raw.len().min(RECOVERY_SCAN_BYTES).saturating_sub(6);
    let mut best: Option<RecoveryPick> = None;

    for off in 0..=max_off {
        let slice = &raw[off..];
        if slice.len() < 5 {
            break;
        }

        let (frames, slack, any_gz) =
            peel_grpc_data_frames_detailed_with_first_len_override(slice, manual_first_len_override);
        if !frames.is_empty() {
            let mut hints = vec![format!(
                "Found gRPC length-prefix at byte offset {} (0x{:x}).",
                off, off
            )];
            if let Some((flag, wire_decl)) = grpc_declared_len_at(slice) {
                let used = manual_first_len_override.unwrap_or(wire_decl);
                hints.push(format!(
                    "First frame @ this offset: compression_flag=0x{:02x}, wire_declared_payload_length={} (0x{:08x}), peeled_with_length={}{}.",
                    flag,
                    wire_decl,
                    wire_decl,
                    used,
                    if manual_first_len_override.is_some() {
                        " (manual override)"
                    } else {
                        ""
                    }
                ));
                hints.push(format!(
                    "On-wire frame header size=5 B; first_message_span={} B (5 + payload).",
                    5usize.saturating_add(used as usize)
                ));
            }
            if any_gz {
                hints.push("Some frame(s) were gzip-compressed.".into());
            }
            let base = 5000i32 + frames.len() as i32 * 200;
            let q: i32 = frames
                .iter()
                .map(|f| score_wire_dissect(&protobuf_wire_format_dump_with_opts(f, opts)))
                .sum();
            let text = compose_dissect_sections(
                Some("=== recovery: aligned gRPC frame(s) ==="),
                &frames,
                slack,
                opts,
            );
            consider_recovery(&mut best, base + q, text, hints);
            continue;
        }

        let flag = slice[0];
        if flag == 0x00 || flag == 0x01 {
            let declared =
                u32::from_be_bytes([slice[1], slice[2], slice[3], slice[4]]) as usize;
            if declared <= MAX_GRPC_MESSAGE_PAYLOAD && slice.len() < 5 + declared {
                if let Some((sc, text, mh)) =
                    try_truncated_grpc_inner(off, flag, &slice[5..], opts)
                {
                    let hints = mh;
                    consider_recovery(&mut best, sc + (max_off.saturating_sub(off) as i32 / 4096), text, hints);
                }
            }
        }

        if off % 8 != 0 {
            continue;
        }
        let mut slice_plain = slice.to_vec();
        if auto_peel_gzip {
            let (p, _) = peel_gzip_layers(slice_plain.clone(), 4);
            slice_plain = p;
        }
        let naked = protobuf_wire_format_dump_with_opts(slice_plain.as_slice(), opts);
        let sq = score_wire_dissect(&naked);
        if sq > -50 {
            let hints = vec![format!(
                "Naked protobuf heuristic at offset {} (0x{:x}), score {}.",
                off, off, sq
            )];
            let text = compose_dissect_sections(
                Some("=== recovery: protobuf parse from arbitrary offset ==="),
                &[slice_plain],
                &[],
                opts,
            );
            consider_recovery(&mut best, sq, text, hints);
        }
    }

    let pick = best.ok_or_else(|| -> String {
        "Recovery found no plausible gRPC frame or protobuf skew. Try a smaller slice containing only DATA payload."
            .to_string()
    })?;

    Ok((pick.text, pick.hints))
}

fn resolve_dissect_payload(
    raw: &[u8],
    strip_grpc_framing: bool,
    auto_peel_gzip: bool,
    opts: ProtobufWireDumpOpts,
    manual_first_len_override: Option<u32>,
) -> Result<(String, Vec<String>, bool), String> {
    let mut notes = Vec::<String>::new();
    let mut needs_recovery = false;

    if strip_grpc_framing {
        let (frames_at_start, _, any_compressed) =
            peel_grpc_data_frames_detailed_with_first_len_override(raw, manual_first_len_override);
        if any_compressed {
            notes.push("HTTP/2 gRPC DATA used gzip-compressed payloads (decoded).".into());
        }
        if let Some((flag, wire_decl)) = grpc_declared_len_at(raw) {
            let peeled_len = manual_first_len_override.unwrap_or(wire_decl);
            notes.push(format!(
                "First gRPC DATA @ buffer start: compression_flag=0x{:02x}, wire_declared_payload_length={} (0x{:08x}); peel used length={}{}{}.",
                flag,
                wire_decl,
                wire_decl,
                peeled_len,
                if manual_first_len_override.is_some() {
                    " (manual)"
                } else {
                    ""
                },
                format!(
                    " Span on wire≈{} B.",
                    5usize.saturating_add(peeled_len as usize)
                )
            ));
        }
        if !frames_at_start.is_empty() {
            notes.push(format!(
                "Peeled {} length-prefixed gRPC protobuf message(s) from buffer start.",
                frames_at_start.len()
            ));
            let text = compose_dissect_sections(None, &frames_at_start, &[], opts);
            return Ok((text, notes, false));
        }
        needs_recovery = raw.len() >= 5;
        notes.push(
            "No complete gRPC length-prefix at buffer start — trying naked protobuf/gzip.".into(),
        );
    }

    let mut payload = raw.to_vec();
    if auto_peel_gzip {
        let (p, peel_notes) = peel_gzip_layers(payload, 12);
        notes.extend(peel_notes);
        payload = p;
    }

    if payload.is_empty() {
        return Err("Nothing left to dissect.".into());
    }

    Ok((
        protobuf_wire_format_dump_with_opts(&payload, opts),
        notes,
        needs_recovery,
    ))
}

fn run_dissect(state: &mut GrpcMakeWorkbenchState) {
    state.length_recovery_busy = false;
    state.length_recovery_rx = None;
    state.dissect_err = None;
    state.dissect_text = None;
    state.status_hint = None;
    state.needs_calculate_length = false;

    let manual_ov = match manual_first_payload_len(state) {
        Ok(o) => o,
        Err(e) => {
            state.dissect_err = Some(e);
            return;
        }
    };
    let pb_opts = ProtobufWireDumpOpts {
        prefer_raw_strings: state.prefer_raw_strings,
    };
    match parse_hex_loose(&state.hex_input) {
        Ok(raw) => match resolve_dissect_payload(
            &raw,
            state.strip_grpc_framing,
            state.auto_peel_gzip,
            pb_opts,
            manual_ov,
        ) {
            Ok((text, notev, needs_recovery)) => {
                state.needs_calculate_length = state.strip_grpc_framing && needs_recovery;
                if !notev.is_empty() {
                    state.status_hint = Some(notev.join(" "));
                }
                state.dissect_text = Some(text);
            }
            Err(e) => state.dissect_err = Some(e),
        },
        Err(e) => state.dissect_err = Some(e),
    }
}

fn apply_length_recovery_success(state: &mut GrpcMakeWorkbenchState, text: String, hints: Vec<String>) {
    state.dissect_text = Some(text);
    let detail = hints.join(" ");
    state.status_hint = Some(if detail.len() > 1200 {
        format!("{} …", &detail[..1180])
    } else {
        detail
    });
    state.dissect_err = None;
    state.needs_calculate_length = false;
}

fn spawn_length_recovery_async(state: &mut GrpcMakeWorkbenchState) {
    if state.length_recovery_busy {
        return;
    }
    let manual_ov = match manual_first_payload_len(state) {
        Ok(o) => o,
        Err(e) => {
            state.dissect_err = Some(e);
            return;
        }
    };
    let hex = state.hex_input.clone();
    let gz = state.auto_peel_gzip;
    let pb_opts = ProtobufWireDumpOpts {
        prefer_raw_strings: state.prefer_raw_strings,
    };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let got =
            parse_hex_loose(&hex).and_then(|raw| brute_recover_grpc_dissect(&raw, gz, pb_opts, manual_ov));
        let _ = tx.send(got);
    });
    state.length_recovery_rx = Some(rx);
    state.length_recovery_busy = true;
}

fn poll_length_recovery(state: &mut GrpcMakeWorkbenchState, ctx: &egui::Context) {
    if !state.length_recovery_busy {
        return;
    }
    let Some(rx) = state.length_recovery_rx.take() else {
        state.length_recovery_busy = false;
        return;
    };
    match rx.try_recv() {
        Ok(Ok((text, hints))) => {
            apply_length_recovery_success(state, text, hints);
            state.length_recovery_busy = false;
            state.length_recovery_rx = None;
        }
        Ok(Err(e)) => {
            state.dissect_err = Some(e);
            state.length_recovery_busy = false;
            state.length_recovery_rx = None;
        }
        Err(TryRecvError::Empty) => {
            state.length_recovery_rx = Some(rx);
            ctx.request_repaint();
        }
        Err(TryRecvError::Disconnected) => {
            state.length_recovery_busy = false;
            state.length_recovery_rx = None;
            state.dissect_err = Some("Length recovery did not finish.".into());
        }
    }
}

/// ldrs-inspired "bouncy" loader (approximation of `<l-bouncy>`); three dots bouncing in place.
fn paint_ldrs_bouncy_overlay(ui: &egui::Ui, panel: egui::Rect, ctx: &egui::Context) {
    let panel = panel.intersect(ui.clip_rect());
    if !(panel.width() > 1.0 && panel.height() > 1.0) {
        return;
    }

    ui.interact(
        panel,
        egui::Id::new("grpc_make_busy_overlay"),
        egui::Sense::click_and_drag(),
    );

    ctx.request_repaint();

    let painter = ui.painter_at(panel);
    painter.rect_filled(panel, 3.0, egui::Color32::from_black_alpha(158));

    let t = ctx.input(|i| i.time as f32);
    let center = panel.center();
    let dot_r = (45.0_f32 / 13.25).clamp(5.5, 8.25);
    let spread = dot_r * 2.42;
    let speed = 1.75_f32;
    let tau = std::f32::consts::TAU;
    let bounce_h = dot_r * 1.82;
    let color = egui::Color32::from_gray(238);

    for i in -1_i32..=1 {
        let x = center.x + i as f32 * spread;
        let phase = i as f32 * 0.72;
        let bounce = (((t * speed * tau).mul_add(1.12, phase)).sin()).abs() * bounce_h;
        let y = center.y - bounce;
        painter.circle_filled(egui::pos2(x, y), dot_r, color);
    }
}

pub fn prefill_capture_body(make: &mut GrpcMakeWorkbenchState, raw_body: &[u8]) {
    make.hex_input = hex::encode(raw_body);
    make.strip_grpc_framing = true;
    make.auto_peel_gzip = true;
    make.dissect_text = None;
    make.dissect_err = None;
    make.status_hint = None;
    run_dissect(make);
}

pub fn render_grpc_make(ui: &mut egui::Ui, state: &mut GrpcMakeWorkbenchState, ctx: &egui::Context) {
    poll_length_recovery(state, ctx);

    let framed = egui::Frame::none().show(ui, |ui| {
    ui.label(egui::RichText::new(
        "Paste hex or load a .bin. Multi-frame DATA bodies peel in order; slack after the last frame is shown separately.",
    )
    .weak()
    .size(11.0));

    ui.horizontal(|ui| {
        ui.checkbox(
            &mut state.strip_grpc_framing,
            "Expect gRPC length prefix (streaming-friendly peel)",
        );
        ui.checkbox(
            &mut state.auto_peel_gzip,
            "Auto peel raw gzip blobs (standalone .gz protobuf)",
        );
        let chk = ui
            .checkbox(&mut state.prefer_raw_strings, "View as raw strings")
            .on_hover_text(
                "Treat length-delimited fields as escaped text only (no nested { } protobuf walk); useful for messy payloads.",
            );
        if chk.changed() && has_non_empty_hex_input(&state.hex_input) {
            run_dissect(state);
        }
    });

    ui.horizontal(|ui| {
        ui.label("1st-frame declared payload len:");
        ui.add(
            egui::TextEdit::singleline(&mut state.manual_first_grpc_payload_len)
                .desired_width(140.0)
                .hint_text("wire (empty) · or decimal / 0x…"),
        )
        .on_hover_text(
            "Overrides the four big-endian payload-length bytes **only for the first gRPC DATA frame** at the slice start (Dissect wire + Calculate). Leave empty for wire value.",
        );
    });

    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Load .bin…")
            .on_hover_text("Load raw bytes; hex fill + dissect.")
            .clicked()
        {
            state.dissect_err = None;
            state.status_hint = None;
            let file = rfd::FileDialog::new()
                .add_filter("Binary", &["bin"])
                .add_filter("All files", &["*"])
                .pick_file();
            if let Some(path) = file {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        state.hex_input = hex::encode(&bytes);
                        state.status_hint = Some(format!(
                            "Loaded {} ({} bytes).",
                            path.display(),
                            bytes.len()
                        ));
                        run_dissect(state);
                    }
                    Err(e) => state.dissect_err = Some(format!("Read failed: {}", e)),
                }
            }
        }

        if ui.button("Dissect wire").clicked() {
            run_dissect(state);
        }

        let show_recovery = state.strip_grpc_framing && has_non_empty_hex_input(&state.hex_input);
        if show_recovery {
            let calc_label = if state.needs_calculate_length {
                "Calculate length"
            } else {
                "Calculate again"
            };
            if ui
                .add_enabled(
                    !state.length_recovery_busy,
                    egui::Button::new(calc_label),
                )
                .on_hover_text(
                    "Scan for embedded gRPC frame(s), truncated lengths, or better protobuf offset; picks the most structured dissect.",
                )
                .clicked()
            {
                state.dissect_err = None;
                state.status_hint = None;
                spawn_length_recovery_async(state);
            }
        }

        if ui
            .add_enabled(
                state.dissect_text.is_some(),
                egui::Button::new("Save dissect…"),
            )
            .on_hover_text("Save decoded wire text to .txt.")
            .clicked()
        {
            if let Some(ref txt) = state.dissect_text {
                let file = rfd::FileDialog::new()
                    .add_filter("Text", &["txt"])
                    .add_filter("All files", &["*"])
                    .set_file_name("grpc_dissect.txt")
                    .save_file();
                if let Some(path) = file {
                    if let Err(e) = std::fs::write(&path, txt) {
                        state.dissect_err = Some(format!("Save failed: {}", e));
                    } else {
                        state.status_hint = Some(format!("Saved dissect → {}", path.display()));
                    }
                }
            }
        }

        if ui
            .add_enabled(
                state.dissect_text.is_some(),
                egui::Button::new("Copy dissect"),
            )
            .clicked()
        {
            if let Some(ref s) = state.dissect_text {
                ctx.copy_text(s.clone());
            }
        }
        if ui.small_button("Copy hex dump").clicked() {
            if let Ok(b) = parse_hex_loose(&state.hex_input) {
                ctx.copy_text(format_hex_dump(&b, b.len().min(65536)));
            }
        }
        if ui
            .small_button("Clear")
            .on_hover_text("Clear hex, dissect, hints.")
            .clicked()
        {
            state.hex_input.clear();
            state.manual_first_grpc_payload_len.clear();
            state.dissect_text = None;
            state.dissect_err = None;
            state.status_hint = None;
            state.needs_calculate_length = false;
            state.length_recovery_busy = false;
            state.length_recovery_rx = None;
        }
    });

    if let Some(ref hint) = state.status_hint {
        ui.label(
            egui::RichText::new(hint.as_str())
                .color(egui::Color32::from_rgb(170, 200, 255))
                .size(11.0),
        );
    }

    if let Some(ref e) = state.dissect_err {
        ui.label(egui::RichText::new(e).color(egui::Color32::RED));
    }

    ui.separator();
    ui.label(egui::RichText::new("Hex + dissect (scroll)").weak().size(10.5));

    let max_h = scroll_max(ui);
    egui::ScrollArea::vertical()
        .id_source("grpc_make_main_scroll")
        .max_height(max_h)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Hex input").weak());
            ui.add(
                egui::TextEdit::multiline(&mut state.hex_input)
                    .desired_width(ui.available_width())
                    .desired_rows(8)
                    .code_editor(),
            );

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            if let Some(ref txt) = state.dissect_text {
                ui.label(egui::RichText::new("Dissected wire").weak());
                ui.add_space(2.0);
                ui.monospace(egui::RichText::new(txt.as_str()).size(11.0));
            }
        });

    }); // Frame::none().show

    if state.length_recovery_busy {
        paint_ldrs_bouncy_overlay(ui, framed.response.rect, ctx);
    }
}
