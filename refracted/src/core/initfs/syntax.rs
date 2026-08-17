//! Syntax highlighting for initfs payloads (Frostbite cfg/lua, JSON, XML, hex).

use super::payload::ViewMode;
use egui::{text::LayoutJob, Color32, FontId, TextFormat};

#[derive(Clone, Copy)]
pub struct Palette {
    pub text: Color32,
    pub quote: Color32,
    pub comment: Color32,
    pub disabled: Color32,
    pub squote: Color32,
    pub bracket: Color32,
    pub value: Color32,
    pub hex_offset: Color32,
    pub hex_byte: Color32,
    pub hex_ascii: Color32,
    pub hex_zero: Color32,
    pub hex_high: Color32,
    pub json_key: Color32,
}

impl Palette {
    pub fn for_dark(dark: bool) -> Self {
        if dark {
            Self {
                text: Color32::from_rgb(245, 245, 245),
                quote: Color32::from_rgb(86, 156, 214),
                comment: Color32::from_rgb(87, 166, 74),
                disabled: Color32::from_rgb(180, 50, 50),
                squote: Color32::from_rgb(206, 145, 120),
                bracket: Color32::from_rgb(200, 180, 80),
                value: Color32::from_rgb(245, 245, 245),
                hex_offset: Color32::from_rgb(120, 120, 160),
                hex_byte: Color32::from_rgb(220, 220, 220),
                hex_ascii: Color32::from_rgb(130, 180, 130),
                hex_zero: Color32::from_rgb(70, 70, 70),
                hex_high: Color32::from_rgb(220, 100, 100),
                json_key: Color32::from_rgb(86, 156, 214),
            }
        } else {
            Self {
                text: Color32::from_rgb(30, 30, 30),
                quote: Color32::from_rgb(86, 156, 214),
                comment: Color32::from_rgb(87, 166, 74),
                disabled: Color32::from_rgb(180, 50, 50),
                squote: Color32::from_rgb(206, 145, 120),
                bracket: Color32::from_rgb(160, 130, 40),
                value: Color32::from_rgb(20, 20, 20),
                hex_offset: Color32::from_rgb(100, 100, 160),
                hex_byte: Color32::from_rgb(30, 30, 30),
                hex_ascii: Color32::from_rgb(60, 120, 60),
                hex_zero: Color32::from_rgb(180, 180, 180),
                hex_high: Color32::from_rgb(180, 40, 40),
                json_key: Color32::from_rgb(0, 80, 160),
            }
        }
    }
}

fn fmt(color: Color32, font: FontId, italics: bool) -> TextFormat {
    TextFormat {
        font_id: font,
        color,
        italics,
        ..Default::default()
    }
}

fn append(job: &mut LayoutJob, text: &str, color: Color32, font: &FontId, italics: bool) {
    job.append(text, 0.0, fmt(color, font.clone(), italics));
}

fn looks_json(text: &str) -> bool {
    let t = text.trim_start();
    t.starts_with('{') || t.starts_with('[')
}

fn looks_xml(text: &str) -> bool {
    text.trim_start().starts_with('<')
}

fn is_ini_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.ends_with(".ini") || n.ends_with(".inf")
}

pub fn highlight(text: &str, name: &str, mode: ViewMode, dark: bool, font: FontId) -> LayoutJob {
    let pal = Palette::for_dark(dark);
    match mode {
        ViewMode::Hex => highlight_hex(text, &pal, &font, false),
        ViewMode::HexText => highlight_hex(text, &pal, &font, true),
        ViewMode::Text => {
            let low = name.to_ascii_lowercase();
            if low.ends_with(".json")
                || (looks_json(text)
                    && !looks_like_cfg(text)
                    && !low.ends_with(".lua")
                    && !low.ends_with(".cfg")
                    && !low.ends_with(".txt"))
            {
                return highlight_json(text, &pal, &font);
            }
            if looks_xml(text)
                || low.ends_with(".xml")
                || low.ends_with(".dbmanifest")
            {
                return highlight_xml(text, &pal, &font);
            }
            highlight_cfg(text, &pal, &font, is_ini_name(name))
        }
    }
}

fn looks_like_cfg(text: &str) -> bool {
    text.lines().take(20).any(|l| {
        let t = l.trim();
        t.contains('.') && t.split_whitespace().next().map(|w| w.contains('.')).unwrap_or(false)
    })
}

fn highlight_cfg(text: &str, pal: &Palette, font: &FontId, is_ini: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    job.wrap.break_anywhere = false;

    // Block comments --[=[ ... ]=] spanning lines: pre-scan ranges.
    let block_ranges = find_lua_block_comments(text);

    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();
        if overlaps_block(&block_ranges, line_start, offset) {
            // Whole line (or remainder) inside a block comment — still split around the range.
            paint_line_with_blocks(line, line_start, &block_ranges, pal, font, is_ini, &mut job);
            continue;
        }
        paint_cfg_line(line, pal, font, is_ini, &mut job);
    }
    if job.sections.is_empty() {
        append(&mut job, text, pal.text, font, false);
    }
    job
}

fn find_lua_block_comments(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'[' {
            let mut eq = 0usize;
            let mut p = i + 3;
            while p < bytes.len() && bytes[p] == b'=' {
                eq += 1;
                p += 1;
            }
            if p < bytes.len() && bytes[p] == b'[' {
                let close = {
                    let mut needle = Vec::from(b"]");
                    needle.extend(std::iter::repeat(b'=').take(eq));
                    needle.push(b']');
                    text[p + 1..]
                        .as_bytes()
                        .windows(needle.len())
                        .position(|w| w == needle.as_slice())
                        .map(|rel| p + 1 + rel + needle.len())
                };
                let end = close.unwrap_or(bytes.len());
                out.push((i, end));
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn overlaps_block(ranges: &[(usize, usize)], start: usize, end: usize) -> bool {
    ranges.iter().any(|&(a, b)| a < end && b > start)
}

fn paint_line_with_blocks(
    line: &str,
    line_start: usize,
    blocks: &[(usize, usize)],
    pal: &Palette,
    font: &FontId,
    is_ini: bool,
    job: &mut LayoutJob,
) {
    let mut i = 0usize;
    while i < line.len() {
        let abs = line_start + i;
        if let Some(&(_a, b)) = blocks.iter().find(|&&(a, b)| a <= abs && abs < b) {
            let local_end = (b - line_start).min(line.len());
            append(job, &line[i..local_end], pal.comment, font, true);
            i = local_end;
        } else {
            let next_block = blocks
                .iter()
                .filter(|&&(a, _)| a > abs)
                .map(|r| r.0 - line_start)
                .min()
                .unwrap_or(line.len());
            let chunk_end = next_block.min(line.len());
            paint_cfg_line(&line[i..chunk_end], pal, font, is_ini, job);
            i = chunk_end;
        }
    }
}

fn paint_cfg_line(line: &str, pal: &Palette, font: &FontId, is_ini: bool, job: &mut LayoutJob) {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    if indent_len > 0 {
        append(job, &line[..indent_len], pal.text, font, false);
    }
    let body = trimmed.trim_end_matches(['\n', '\r']);
    let nl = &trimmed[body.len()..];

    let is_comment_line = body.starts_with("--") || body.starts_with("//") || body.starts_with('#');
    if is_comment_line {
        let rest = if body.starts_with("--") {
            &body[2..]
        } else if body.starts_with("//") {
            &body[2..]
        } else {
            &body[1..]
        };
        let cmd = rest.trim_start();
        let looks_disabled_cmd = cmd
            .split_whitespace()
            .next()
            .map(|w| is_dotted_ident(w))
            .unwrap_or(false);
        if looks_disabled_cmd {
            append(job, body, pal.disabled, font, true);
        } else {
            append(job, body, pal.comment, font, true);
        }
        if !nl.is_empty() {
            append(job, nl, pal.text, font, false);
        }
        return;
    }

    if is_ini {
        paint_quoted(body, pal, font, job);
        if !nl.is_empty() {
            append(job, nl, pal.text, font, false);
        }
        return;
    }

    // command.with.dots   value   -- comment
    if let Some((cmd, rest)) = split_dotted_command(body) {
        append(job, cmd, pal.text, font, false);
        let rest_trim = rest.trim_start();
        let ws_len = rest.len() - rest_trim.len();
        if ws_len > 0 {
            append(job, &rest[..ws_len], pal.text, font, false);
        }
        let (value, inline_cmt) = split_inline_comment(rest_trim);
        if !value.is_empty() {
            paint_value(value, pal, font, job);
        }
        if !inline_cmt.is_empty() {
            append(job, inline_cmt, pal.comment, font, true);
        }
        if !nl.is_empty() {
            append(job, nl, pal.text, font, false);
        }
        return;
    }

    paint_quoted(body, pal, font, job);
    if !nl.is_empty() {
        append(job, nl, pal.text, font, false);
    }
}

fn is_dotted_ident(w: &str) -> bool {
    let mut parts = w.split('.');
    let first = match parts.next() {
        Some(s) if !s.is_empty() && s.chars().next().unwrap().is_ascii_alphabetic() => s,
        _ => return false,
    };
    if !first.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    let mut n = 0;
    for p in parts {
        if p.is_empty() || !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
        n += 1;
    }
    n >= 1
}

fn split_dotted_command(body: &str) -> Option<(&str, &str)> {
    let token_end = body
        .find(|c: char| c.is_whitespace())
        .unwrap_or(body.len());
    let cmd = &body[..token_end];
    if is_dotted_ident(cmd) {
        Some((cmd, &body[token_end..]))
    } else {
        None
    }
}

fn split_inline_comment(s: &str) -> (&str, &str) {
    for (idx, _) in s.char_indices() {
        let rest = &s[idx..];
        if rest.starts_with("--") || rest.starts_with("//") || rest.starts_with('#') {
            // Don't split if inside quotes — cheap scan of prefix.
            if odd_unescaped_quotes(&s[..idx]) {
                continue;
            }
            return s.split_at(idx);
        }
    }
    (s, "")
}

fn odd_unescaped_quotes(s: &str) -> bool {
    let mut n = 0u32;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next();
            continue;
        }
        if c == '"' {
            n += 1;
        }
    }
    n % 2 == 1
}

fn paint_value(value: &str, pal: &Palette, font: &FontId, job: &mut LayoutJob) {
    let mut i = 0usize;
    let bytes = value.as_bytes();
    while i < value.len() {
        if bytes[i] == b'"' {
            let end = scan_dq(value, i);
            append(job, &value[i..end], pal.quote, font, false);
            i = end;
            continue;
        }
        if bytes[i] == b'\'' {
            let end = scan_sq(value, i);
            append(job, &value[i..end], pal.squote, font, false);
            i = end;
            continue;
        }
        if bytes[i] == b'[' && !is_lua_long_open(bytes, i) {
            if let Some(end) = scan_bracket(value, i) {
                append(job, &value[i..end], pal.bracket, font, false);
                i = end;
                continue;
            }
        }
        let next = (i + 1..value.len())
            .find(|&j| bytes[j] == b'"' || bytes[j] == b'\'' || bytes[j] == b'[')
            .unwrap_or(value.len());
        append(job, &value[i..next], pal.value, font, false);
        i = next;
    }
}

fn paint_quoted(s: &str, pal: &Palette, font: &FontId, job: &mut LayoutJob) {
    paint_value(s, pal, font, job);
}

fn scan_dq(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return i + 1;
        }
        i += 1;
    }
    s.len()
}

fn scan_sq(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'\'' || bytes[i] == b'\n' {
            return if bytes[i] == b'\'' { i + 1 } else { i };
        }
        i += 1;
    }
    s.len()
}

fn is_lua_long_open(bytes: &[u8], i: usize) -> bool {
    if i + 1 >= bytes.len() || bytes[i] != b'[' {
        return false;
    }
    let mut p = i + 1;
    while p < bytes.len() && bytes[p] == b'=' {
        p += 1;
    }
    p < bytes.len() && bytes[p] == b'['
}

fn scan_bracket(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if start > 0 {
        let prev = bytes[start - 1];
        if !(prev == b' ' || prev == b'\t' || prev == b'[' || prev == b',' || prev == b'(') {
            // Allow start of string.
            if start != 0 && !bytes[start - 1].is_ascii_whitespace() && prev != b'[' {
                // still allow if start of value
            }
        }
    }
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            b'\n' | b'\r' => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

fn highlight_json(text: &str, pal: &Palette, font: &FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < text.len() {
        if bytes[i] == b'"' {
            let end = scan_dq(text, i);
            let after = text[end..].trim_start_matches(|c: char| c.is_whitespace());
            let color = if after.starts_with(':') {
                pal.json_key
            } else {
                pal.squote
            };
            append(&mut job, &text[i..end], color, font, false);
            i = end;
            continue;
        }
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            let end = text[i..].find('\n').map(|n| i + n).unwrap_or(text.len());
            append(&mut job, &text[i..end], pal.comment, font, true);
            i = end;
            continue;
        }
        let ch = bytes[i] as char;
        if ch.is_ascii_digit() || ch == '-' {
            let mut j = i + 1;
            while j < bytes.len() {
                let c = bytes[j];
                if !(c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'+' || c == b'-')
                {
                    break;
                }
                j += 1;
            }
            append(&mut job, &text[i..j], pal.bracket, font, false);
            i = j;
            continue;
        }
        if text[i..].starts_with("true") || text[i..].starts_with("null") {
            append(&mut job, &text[i..i + 4], pal.bracket, font, false);
            i += 4;
            continue;
        }
        if text[i..].starts_with("false") {
            append(&mut job, &text[i..i + 5], pal.bracket, font, false);
            i += 5;
            continue;
        }
        let next = (i + 1..text.len())
            .find(|&j| {
                bytes[j] == b'"'
                    || (bytes[j] == b'/' && j + 1 < bytes.len() && bytes[j + 1] == b'/')
                    || (bytes[j] as char).is_ascii_digit()
                    || bytes[j] == b'-'
                    || text[j..].starts_with("true")
                    || text[j..].starts_with("false")
                    || text[j..].starts_with("null")
            })
            .unwrap_or(text.len());
        append(&mut job, &text[i..next], pal.text, font, false);
        i = next;
    }
    job
}

fn highlight_xml(text: &str, pal: &Palette, font: &FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < text.len() {
        if text[i..].starts_with("<!--") {
            let end = text[i + 4..]
                .find("-->")
                .map(|n| i + 4 + n + 3)
                .unwrap_or(text.len());
            append(&mut job, &text[i..end], pal.comment, font, true);
            i = end;
            continue;
        }
        if bytes[i] == b'<' {
            let end = text[i..].find('>').map(|n| i + n + 1).unwrap_or(text.len());
            append(&mut job, &text[i..end], pal.quote, font, false);
            i = end;
            continue;
        }
        if bytes[i] == b'"' {
            let end = scan_dq(text, i);
            append(&mut job, &text[i..end], pal.squote, font, false);
            i = end;
            continue;
        }
        let next = (i + 1..text.len())
            .find(|&j| bytes[j] == b'<' || bytes[j] == b'"')
            .unwrap_or(text.len());
        append(&mut job, &text[i..next], pal.text, font, false);
        i = next;
    }
    job
}

fn highlight_hex(text: &str, pal: &Palette, font: &FontId, with_ascii: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY;
    job.wrap.break_anywhere = false;
    for (li, line) in text.split_inclusive('\n').enumerate() {
        if li == 0 {
            append(&mut job, line, pal.hex_offset, font, false);
            continue;
        }
        let body = line.trim_end_matches(['\n', '\r']);
        let nl = &line[body.len()..];
        if body.len() >= 8 && body.as_bytes()[..8].iter().all(|b| b.is_ascii_hexdigit()) {
            append(&mut job, &body[..8], pal.hex_offset, font, false);
            let rest = &body[8..];
            if with_ascii {
                if let Some(bar) = rest.find('|') {
                    paint_hex_bytes(&rest[..bar], pal, font, &mut job);
                    append(&mut job, &rest[bar..], pal.hex_ascii, font, false);
                } else {
                    paint_hex_bytes(rest, pal, font, &mut job);
                }
            } else {
                paint_hex_bytes(rest, pal, font, &mut job);
            }
        } else {
            append(&mut job, body, pal.hex_byte, font, false);
        }
        if !nl.is_empty() {
            append(&mut job, nl, pal.text, font, false);
        }
    }
    job
}

fn paint_hex_bytes(s: &str, pal: &Palette, font: &FontId, job: &mut LayoutJob) {
    let mut i = 0usize;
    let b = s.as_bytes();
    while i < s.len() {
        if i + 2 <= s.len() && b[i].is_ascii_hexdigit() && b[i + 1].is_ascii_hexdigit() {
            let tok = &s[i..i + 2];
            let color = match tok {
                "00" => pal.hex_zero,
                "FF" => pal.hex_high,
                _ => pal.hex_byte,
            };
            append(job, tok, color, font, false);
            i += 2;
            continue;
        }
        append(job, &s[i..i + 1], pal.hex_byte, font, false);
        i += 1;
    }
}
