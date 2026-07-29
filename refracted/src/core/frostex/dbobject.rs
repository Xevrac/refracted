//! Frostbite DbObject binary format (layout.toc, .toc, .sb metadata).

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DbObjectType {
    Eoo = 0x00,
    Array = 0x01,
    Object = 0x02,
    HomoArray = 0x03,
    Null = 0x04,
    ObjectId = 0x05,
    Bool = 0x06,
    String = 0x07,
    Integer = 0x08,
    Long = 0x09,
    VarInt = 0x0A,
    Float = 0x0B,
    Double = 0x0C,
    Timestamp = 0x0D,
    RecordId = 0x0E,
    Guid = 0x0F,
    Sha1 = 0x10,
    Matrix44 = 0x11,
    Vector4 = 0x12,
    Blob = 0x13,
    Attachment = 0x14,
    Timespan = 0x15,
    Unknown(u8),
}

impl From<u8> for DbObjectType {
    fn from(v: u8) -> Self {
        match v & 0x1F {
            0x00 => Self::Eoo,
            0x01 => Self::Array,
            0x02 => Self::Object,
            0x03 => Self::HomoArray,
            0x04 => Self::Null,
            0x05 => Self::ObjectId,
            0x06 => Self::Bool,
            0x07 => Self::String,
            0x08 => Self::Integer,
            0x09 => Self::Long,
            0x0A => Self::VarInt,
            0x0B => Self::Float,
            0x0C => Self::Double,
            0x0D => Self::Timestamp,
            0x0E => Self::RecordId,
            0x0F => Self::Guid,
            0x10 => Self::Sha1,
            0x11 => Self::Matrix44,
            0x12 => Self::Vector4,
            0x13 => Self::Blob,
            0x14 => Self::Attachment,
            0x15 => Self::Timespan,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DbValue {
    Null,
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Guid([u8; 16]),
    Sha1([u8; 20]),
    Blob(Vec<u8>),
    ObjectId([u8; 12]),
    RecordId(u16, u16, u16),
    Timestamp(u64),
    Timespan(i64),
    Vector4([f32; 4]),
    Matrix44([f32; 16]),
    Array(Vec<DbObject>),
    Object(Vec<DbObject>),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct DbObject {
    pub name: String,
    pub value: DbValue,
}

impl DbObject {
    pub fn field(&self, name: &str) -> Option<&DbObject> {
        match &self.value {
            DbValue::Object(fields) | DbValue::Array(fields) => {
                fields.iter().find(|f| f.name == name)
            }
            _ => None,
        }
    }

    pub fn as_object_fields(&self) -> Option<&[DbObject]> {
        match &self.value {
            DbValue::Object(f) | DbValue::Array(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.value {
            DbValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            DbValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match &self.value {
            DbValue::Int(v) => Some(*v as i64),
            DbValue::Long(v) => Some(*v),
            DbValue::Timespan(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_sha1(&self) -> Option<[u8; 20]> {
        match &self.value {
            DbValue::Sha1(s) => Some(*s),
            _ => None,
        }
    }

    pub fn as_guid(&self) -> Option<[u8; 16]> {
        match &self.value {
            DbValue::Guid(g) => Some(*g),
            _ => None,
        }
    }

    pub fn to_pretty(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let name = if self.name.is_empty() {
            String::new()
        } else {
            format!("{}: ", self.name)
        };
        match &self.value {
            DbValue::Object(fields) => {
                let mut out = format!("{pad}{name}{{\n");
                for f in fields {
                    out.push_str(&f.to_pretty(indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            DbValue::Array(fields) => {
                let mut out = format!("{pad}{name}[\n");
                for f in fields {
                    out.push_str(&f.to_pretty(indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{pad}]"));
                out
            }
            other => format!("{pad}{name}{}", fmt_leaf(other)),
        }
    }
}

fn fmt_leaf(v: &DbValue) -> String {
    match v {
        DbValue::Null => "null".into(),
        DbValue::Bool(b) => b.to_string(),
        DbValue::Int(i) => i.to_string(),
        DbValue::Long(i) => i.to_string(),
        DbValue::Float(f) => f.to_string(),
        DbValue::Double(f) => f.to_string(),
        DbValue::String(s) => format!("\"{s}\""),
        DbValue::Guid(g) => format!("guid({})", hex::encode(g)),
        DbValue::Sha1(s) => format!("sha1({})", hex::encode(s)),
        DbValue::Blob(b) => format!("blob[{}]", b.len()),
        DbValue::ObjectId(o) => format!("oid({})", String::from_utf8_lossy(o)),
        DbValue::RecordId(a, b, c) => format!("record({a},{b},{c})"),
        DbValue::Timestamp(t) => format!("ts({t})"),
        DbValue::Timespan(t) => format!("span({t})"),
        DbValue::Vector4(v) => format!("vec4({:?})", v),
        DbValue::Matrix44(_) => "mat44(...)".into(),
        DbValue::Raw(b) => format!("raw[{}]", b.len()),
        DbValue::Array(_) | DbValue::Object(_) => unreachable!(),
    }
}

struct Reader<'a> {
    cur: Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            cur: Cursor::new(data),
        }
    }

    fn pos(&self) -> u64 {
        self.cur.position()
    }

    fn remaining(&self) -> usize {
        self.cur.get_ref().len().saturating_sub(self.pos() as usize)
    }

    fn read_exact(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; n];
        self.cur
            .read_exact(&mut buf)
            .map_err(|e| format!("DbObject read: {e}"))?;
        Ok(buf)
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_i32(&mut self) -> Result<i32, String> {
        let b = self.read_exact(4)?;
        Ok(i32::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let b = self.read_exact(4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_i64(&mut self) -> Result<i64, String> {
        let b = self.read_exact(8)?;
        Ok(i64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_u64(&mut self) -> Result<u64, String> {
        let b = self.read_exact(8)?;
        Ok(u64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_f32(&mut self) -> Result<f32, String> {
        let b = self.read_exact(4)?;
        Ok(f32::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_f64(&mut self) -> Result<f64, String> {
        let b = self.read_exact(8)?;
        Ok(f64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_leb128(&mut self) -> Result<u64, String> {
        let mut result = 0u64;
        let mut shift = 0u32;
        loop {
            let b = self.read_u8()?;
            result |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift > 63 {
                return Err("LEB128 overflow".into());
            }
        }
        Ok(result)
    }

    fn read_cstring(&mut self) -> Result<String, String> {
        let mut bytes = Vec::new();
        loop {
            let b = self.read_u8()?;
            if b == 0 {
                break;
            }
            bytes.push(b);
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

fn parse_one(r: &mut Reader<'_>) -> Result<DbObject, String> {
    let header = r.read_u8()?;
    let ty = DbObjectType::from(header);
    let flags = header >> 5;
    let name = if flags != 0x04 {
        r.read_cstring()?
    } else {
        String::new()
    };

    let value = match ty {
        DbObjectType::Eoo => DbValue::Null,
        DbObjectType::Null => DbValue::Null,
        DbObjectType::Bool => DbValue::Bool(r.read_u8()? != 0),
        DbObjectType::Integer => DbValue::Int(r.read_i32()?),
        DbObjectType::Long => DbValue::Long(r.read_i64()?),
        DbObjectType::Float => DbValue::Float(r.read_f32()?),
        DbObjectType::Double => DbValue::Double(r.read_f64()?),
        DbObjectType::Timestamp => DbValue::Timestamp(r.read_u64()?),
        DbObjectType::VarInt => {
            let val = r.read_leb128()? as i64;
            DbValue::Long((val >> 1) ^ -(val & 1))
        }
        DbObjectType::Timespan => {
            let val = r.read_leb128()? as i64;
            DbValue::Timespan((val >> 1) ^ -(val & 1))
        }
        DbObjectType::String => {
            let len = r.read_leb128()? as usize;
            if len == 0 {
                DbValue::String(String::new())
            } else {
                let data = r.read_exact(len.saturating_sub(1))?;
                let _nul = r.read_u8()?;
                DbValue::String(String::from_utf8_lossy(&data).into_owned())
            }
        }
        DbObjectType::ObjectId => {
            let b = r.read_exact(12)?;
            let mut a = [0u8; 12];
            a.copy_from_slice(&b);
            DbValue::ObjectId(a)
        }
        DbObjectType::RecordId => {
            let a = u16::from_le_bytes(r.read_exact(2)?.try_into().unwrap());
            let b = u16::from_le_bytes(r.read_exact(2)?.try_into().unwrap());
            let c = u16::from_le_bytes(r.read_exact(2)?.try_into().unwrap());
            DbValue::RecordId(a, b, c)
        }
        DbObjectType::Guid => {
            let b = r.read_exact(16)?;
            let mut g = [0u8; 16];
            g.copy_from_slice(&b);
            DbValue::Guid(g)
        }
        DbObjectType::Sha1 | DbObjectType::Attachment => {
            let b = r.read_exact(20)?;
            let mut s = [0u8; 20];
            s.copy_from_slice(&b);
            DbValue::Sha1(s)
        }
        DbObjectType::Vector4 => DbValue::Vector4([
            r.read_f32()?,
            r.read_f32()?,
            r.read_f32()?,
            r.read_f32()?,
        ]),
        DbObjectType::Matrix44 => {
            let mut m = [0f32; 16];
            for slot in &mut m {
                *slot = r.read_f32()?;
            }
            DbValue::Matrix44(m)
        }
        DbObjectType::Blob => {
            let n = r.read_leb128()? as usize;
            DbValue::Blob(r.read_exact(n)?)
        }
        DbObjectType::Array | DbObjectType::Object | DbObjectType::HomoArray => {
            let byte_len = r.read_leb128()? as u64;
            let end = r.pos() + byte_len;
            let mut kids = Vec::new();
            while r.pos() + 1 < end {
                kids.push(parse_one(r)?);
            }
            if r.pos() < end {
                let term = r.read_u8()?;
                if term != 0x00 {
                    return Err(format!(
                        "DbObject container terminator 0x{term:02X} at {}",
                        r.pos()
                    ));
                }
            }
            if matches!(ty, DbObjectType::Array | DbObjectType::HomoArray) {
                DbValue::Array(kids)
            } else {
                DbValue::Object(kids)
            }
        }
        DbObjectType::Unknown(t) => {
            return Err(format!(
                "Unhandled DbObject type 0x{t:02X} at pos {}",
                r.pos()
            ));
        }
    };

    Ok(DbObject { name, value })
}

/// Decrypt IceBloc-style XOR wrappers (`00 D1 CE 00/01/03`) if present.
pub fn unwrap_db_bytes(raw: &[u8]) -> Vec<u8> {
    if raw.len() < 4 {
        return raw.to_vec();
    }
    let magic = &raw[0..4];
    if magic == [0x00, 0xD1, 0xCE, 0x00] || magic == [0x00, 0xD1, 0xCE, 0x01] {
        if raw.len() < 556 {
            return raw.to_vec();
        }
        let mut key = raw[296..556].to_vec();
        for b in &mut key {
            *b ^= 0x7B;
        }
        let encrypted = &raw[556..];
        let mut out = vec![0u8; encrypted.len()];
        for (i, b) in encrypted.iter().enumerate() {
            out[i] = key[i % 257] ^ *b;
        }
        return out;
    }
    if magic == [0x00, 0xD1, 0xCE, 0x03] {
        if raw.len() <= 556 {
            return Vec::new();
        }
        return raw[556..].to_vec();
    }
    raw.to_vec()
}

pub fn parse_db_bytes(data: &[u8]) -> Result<DbObject, String> {
    let unwrapped = unwrap_db_bytes(data);
    if unwrapped.is_empty() {
        return Err("empty DbObject payload".into());
    }
    let mut r = Reader::new(&unwrapped);
    parse_one(&mut r)
}

pub fn load_db_file(path: &Path) -> Result<DbObject, String> {
    let raw = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_db_bytes(&raw)
}

pub fn object_map(obj: &DbObject) -> BTreeMap<String, &DbObject> {
    let mut map = BTreeMap::new();
    if let Some(fields) = obj.as_object_fields() {
        for f in fields {
            if !f.name.is_empty() {
                map.insert(f.name.clone(), f);
            }
        }
    }
    map
}
