//! Payload list helpers: extract, import, text/hex views, line-ending preserve.

use super::db::DbValue;
use crate::core::frostex::ebx::{dump_ebx_text, is_ebx};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Text,
    Hex,
    HexText,
}

impl ViewMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Hex => "Hex",
            Self::HexText => "Hex+ASCII",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Text => Self::Hex,
            Self::Hex => Self::HexText,
            Self::HexText => Self::Text,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListViewMode {
    #[default]
    Names,
    Tree,
    Folder,
}

impl ListViewMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Names => "Names",
            Self::Tree => "Tree",
            Self::Folder => "Folder",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Names => Self::Tree,
            Self::Tree => Self::Folder,
            Self::Folder => Self::Names,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Default,
    Az,
    Za,
    BigSmall,
    SmallBig,
}

#[derive(Debug, Clone)]
pub struct Payload {
    pub name: String,
    pub bytes: Vec<u8>,
    pub text: String,
    pub orig_text: String,
    pub orig_bytes: Vec<u8>,
    pub is_text: bool,
    pub read_only: bool,
}

impl Payload {
    pub fn ext(&self) -> String {
        std::path::Path::new(&self.name)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy().to_ascii_lowercase()))
            .unwrap_or_else(|| "(none)".into())
    }

    pub fn dirty(&self) -> bool {
        self.bytes != self.orig_bytes
    }
}

pub fn is_probably_text(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    let sample = data.len().min(4096);
    let printable = data[..sample]
        .iter()
        .filter(|&&b| (0x20u8..=0x7E).contains(&b) || matches!(b, 0x09 | 0x0A | 0x0D))
        .count();
    printable * 100 / sample >= 80
}

fn is_stripped_manifest(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "stripped_database.dbmanifest" || (n.starts_with("stripped_") && n.ends_with(".dbmanifest"))
}

fn pretty_json(raw: &[u8]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(raw).ok()?;
    serde_json::to_string_pretty(&v).ok()
}

pub fn payload_to_text(name: &str, data: &[u8]) -> (String, bool, bool) {
    let low = name.to_ascii_lowercase();
    if is_stripped_manifest(name) {
        if data.first() == Some(&b'<') {
            return (String::from_utf8_lossy(data).into_owned(), true, true);
        }
        if data.first() == Some(&b'{') {
            return (
                pretty_json(data).unwrap_or_else(|| String::from_utf8_lossy(data).into_owned()),
                true,
                true,
            );
        }
        return (
            format!(
                "[stripped binary dbmanifest: {} bytes]\n\n{}",
                data.len(),
                extract_ascii_strings(data, 4)
            ),
            false,
            true,
        );
    }
    if low.contains("dbmanifest") || low.contains(".xml") {
        return (String::from_utf8_lossy(data).into_owned(), true, false);
    }
    if low.ends_with(".ebx") || is_ebx(data) {
        return (dump_ebx_text(data), false, true);
    }
    if low.ends_with(".json") {
        return (
            pretty_json(data).unwrap_or_else(|| String::from_utf8_lossy(data).into_owned()),
            true,
            false,
        );
    }
    if is_probably_text(data) {
        return (String::from_utf8_lossy(data).into_owned(), true, false);
    }
    (
        format!("[binary payload: {} bytes]", data.len()),
        false,
        true,
    )
}

pub fn extract_ascii_strings(data: &[u8], min_len: usize) -> String {
    let mut out = String::new();
    let mut cur = Vec::new();
    for &b in data {
        if (0x20..=0x7E).contains(&b) {
            cur.push(b);
        } else {
            if cur.len() >= min_len {
                out.push_str(&String::from_utf8_lossy(&cur));
                out.push('\n');
            }
            cur.clear();
        }
    }
    if cur.len() >= min_len {
        out.push_str(&String::from_utf8_lossy(&cur));
        out.push('\n');
    }
    out
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn push_hex_byte(out: &mut String, b: u8) {
    out.push(HEX[(b >> 4) as usize] as char);
    out.push(HEX[(b & 0xF) as usize] as char);
}

pub fn render_hex(data: &[u8]) -> String {
    let mut out = String::new();
    out.push_str("          ");
    for i in 0..16u8 {
        push_hex_byte(&mut out, i);
        out.push(' ');
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out.push('\n');
    if data.is_empty() {
        out.push_str("00000000\n");
        return out;
    }
    for (row, chunk) in data.chunks(16).enumerate() {
        let off = (row * 16) as u32;
        out.push_str(&format!("{off:08X}  "));
        for b in chunk {
            push_hex_byte(&mut out, *b);
            out.push(' ');
        }
        for _ in chunk.len()..16 {
            out.push_str("   ");
        }
        out.push('\n');
    }
    out
}

pub fn render_hex_text(data: &[u8]) -> String {
    let mut out = String::new();
    out.push_str("          ");
    for i in 0..16u8 {
        push_hex_byte(&mut out, i);
        out.push(' ');
    }
    out.push_str("  ");
    for i in 0..16u8 {
        out.push(HEX[i as usize] as char);
    }
    out.push('\n');
    if data.is_empty() {
        out.push_str("00000000                                 |                |\n");
        return out;
    }
    for (row, chunk) in data.chunks(16).enumerate() {
        let off = (row * 16) as u32;
        out.push_str(&format!("{off:08X}  "));
        for b in chunk {
            push_hex_byte(&mut out, *b);
            out.push(' ');
        }
        for _ in chunk.len()..16 {
            out.push_str("   ");
        }
        out.push_str(" |");
        for b in chunk {
            let c = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            out.push(c);
        }
        for _ in chunk.len()..16 {
            out.push(' ');
        }
        out.push_str("|\n");
    }
    out
}

pub fn detect_newline(bytes: &[u8]) -> &'static str {
    if bytes.windows(2).any(|w| w == b"\r\n") {
        "\r\n"
    } else if bytes.contains(&b'\r') {
        "\r"
    } else {
        "\n"
    }
}

pub fn normalize_newlines(text: &str, original: &[u8]) -> Vec<u8> {
    let nl = detect_newline(original);
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    if nl == "\n" {
        unified.into_bytes()
    } else {
        unified.replace('\n', nl).into_bytes()
    }
}

fn file_block(entry: &DbValue) -> Option<&DbValue> {
    entry.field("$file")
}

pub fn entry_name(entry: &DbValue) -> String {
    if let Some(n) = entry.field("name").and_then(|v| v.as_str()) {
        if !n.is_empty() {
            return n.to_string();
        }
    }
    if let Some(n) = file_block(entry)
        .and_then(|f| f.field("name"))
        .and_then(|v| v.as_str())
    {
        if !n.is_empty() {
            return n.to_string();
        }
    }
    String::new()
}

pub fn entry_payload_bytes(entry: &DbValue) -> Vec<u8> {
    file_block(entry)
        .and_then(|f| f.field("payload"))
        .and_then(|v| v.as_bytes())
        .map(|b| b.to_vec())
        .unwrap_or_default()
}

pub fn collect_payloads(root: &DbValue) -> Vec<Payload> {
    let mut out = Vec::new();
    let Some(list) = root.as_list() else {
        return out;
    };
    for (i, entry) in list.iter().enumerate() {
        if !entry.has_field("$file") {
            continue;
        }
        let mut name = entry_name(entry);
        if name.is_empty() {
            name = format!("Payload {i}");
        }
        let bytes = entry_payload_bytes(entry);
        let (text, is_text, read_only) = payload_to_text(&name, &bytes);
        out.push(Payload {
            name,
            orig_bytes: bytes.clone(),
            bytes,
            orig_text: text.clone(),
            text,
            is_text,
            read_only,
        });
    }
    out
}

pub fn apply_payload_bytes(root: &mut DbValue, index: usize, new_bytes: Vec<u8>) -> Result<(), String> {
    let list = root
        .as_list_mut()
        .ok_or_else(|| "initfs root is not a payload list".to_string())?;
    let mut seen = 0usize;
    for entry in list.iter_mut() {
        if !entry.has_field("$file") {
            continue;
        }
        if seen == index {
            if let Some(file) = entry.field_mut("$file") {
                let len = new_bytes.len() as i32;
                file.set_field("payload", DbValue::Bytes(new_bytes));
                if file.has_field("length") {
                    file.set_field("length", DbValue::Int(len));
                }
                return Ok(());
            }
            return Err("payload $file block missing".into());
        }
        seen += 1;
    }
    Err("payload index not found".into())
}

pub fn add_payload(root: &mut DbValue, name: &str, content: &[u8]) -> Result<(), String> {
    let list = root
        .as_list_mut()
        .ok_or_else(|| "initfs root is not a payload list".to_string())?;
    for entry in list.iter() {
        if entry_name(entry).eq_ignore_ascii_case(name) {
            return Err(format!("a payload named '{name}' already exists"));
        }
    }
    let file = DbValue::Object(vec![
        ("name".into(), DbValue::String(name.to_string())),
        ("payload".into(), DbValue::Bytes(content.to_vec())),
        ("length".into(), DbValue::Int(content.len() as i32)),
    ]);
    let entry = DbValue::Object(vec![
        ("name".into(), DbValue::String(name.to_string())),
        ("$file".into(), file),
    ]);
    list.push(entry);
    Ok(())
}

pub fn remove_payload(root: &mut DbValue, index: usize) -> Result<(), String> {
    let list = root
        .as_list_mut()
        .ok_or_else(|| "initfs root is not a payload list".to_string())?;
    let mut seen = 0usize;
    let mut remove_at = None;
    for (i, entry) in list.iter().enumerate() {
        if !entry.has_field("$file") {
            continue;
        }
        if seen == index {
            remove_at = Some(i);
            break;
        }
        seen += 1;
    }
    let i = remove_at.ok_or_else(|| "payload index not found".to_string())?;
    list.remove(i);
    Ok(())
}

pub fn rename_payload(root: &mut DbValue, index: usize, new_name: &str) -> Result<(), String> {
    let list = root
        .as_list_mut()
        .ok_or_else(|| "initfs root is not a payload list".to_string())?;
    let mut seen = 0usize;
    for entry in list.iter_mut() {
        if !entry.has_field("$file") {
            continue;
        }
        if seen == index {
            entry.set_field("name", DbValue::String(new_name.to_string()));
            if let Some(file) = entry.field_mut("$file") {
                file.set_field("name", DbValue::String(new_name.to_string()));
            }
            return Ok(());
        }
        seen += 1;
    }
    Err("payload index not found".into())
}

pub fn platform_from_path(path: &str) -> &'static str {
    let n = path.to_ascii_lowercase();
    if n.contains("win32") || n.contains("win64") {
        "Win32"
    } else if n.contains("ps3") {
        "PS3"
    } else if n.contains("ps4") {
        "PS4"
    } else if n.contains("ps5") {
        "PS5"
    } else if n.contains("xbox360") || n.contains("xenon") {
        "Xenon"
    } else if n.contains("xboxone") || n.contains("durango") {
        "Durango"
    } else if n.contains("nx") || n.contains("switch") {
        "NX"
    } else {
        "Unknown"
    }
}

pub fn safe_export_path(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    let mut dest = root.to_path_buf();
    for part in name.split(['/', '\\']) {
        if part.is_empty() || part == "." || part == ".." {
            continue;
        }
        dest.push(part);
    }
    dest
}
