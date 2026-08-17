//! InitfsTools-compatible Frostbite DbObject reader/writer.
//!
//! Matches `DbReader` / `DbWriter` in InitfsTools 2.15:
//! unnamed values set bit 7 on the type byte; containers are 7-bit size + payload + `0x00`.

use std::io::Cursor;

#[derive(Debug, Clone, PartialEq)]
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
    Bytes(Vec<u8>),
    /// Named fields, insertion order preserved.
    Object(Vec<(String, DbValue)>),
    /// Unnamed list items.
    List(Vec<DbValue>),
}

impl DbValue {
    pub fn object() -> Self {
        Self::Object(Vec::new())
    }

    pub fn list() -> Self {
        Self::List(Vec::new())
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub fn field(&self, name: &str) -> Option<&DbValue> {
        match self {
            Self::Object(fields) => fields
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn field_mut(&mut self, name: &str) -> Option<&mut DbValue> {
        match self {
            Self::Object(fields) => fields
                .iter_mut()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn set_field(&mut self, name: &str, value: DbValue) {
        if let Self::Object(fields) = self {
            if let Some(slot) = fields
                .iter_mut()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
            {
                slot.1 = value;
            } else {
                fields.push((name.to_string(), value));
            }
        }
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.field(name).is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Long(v) => Some(*v as i32),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[DbValue]> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<DbValue>> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, DbValue)]> {
        match self {
            Self::Object(v) => Some(v),
            _ => None,
        }
    }
}

const TY_LIST: u8 = 1;
const TY_OBJECT: u8 = 2;
const TY_BOOL: u8 = 6;
const TY_STRING: u8 = 7;
const TY_INT: u8 = 8;
const TY_LONG: u8 = 9;
const TY_FLOAT: u8 = 11;
const TY_DOUBLE: u8 = 12;
const TY_GUID: u8 = 15;
const TY_SHA1: u8 = 16;
const TY_BYTES: u8 = 19;
const FLAG_UNNAMED: u8 = 0x80;

struct Reader<'a> {
    cur: Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            cur: Cursor::new(data),
        }
    }

    fn remaining(&self) -> usize {
        self.cur.get_ref().len().saturating_sub(self.cur.position() as usize)
    }

    fn read_exact(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let pos = self.cur.position() as usize;
        let buf = self.cur.get_ref();
        if pos + n > buf.len() {
            return Err(format!("DbObject truncated at {pos}, need {n}"));
        }
        let out = buf[pos..pos + n].to_vec();
        self.cur.set_position((pos + n) as u64);
        Ok(out)
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_i32(&mut self) -> Result<i32, String> {
        let b = self.read_exact(4)?;
        Ok(i32::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_i64(&mut self) -> Result<i64, String> {
        let b = self.read_exact(8)?;
        Ok(i64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_f32(&mut self) -> Result<f32, String> {
        let b = self.read_exact(4)?;
        Ok(f32::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_f64(&mut self) -> Result<f64, String> {
        let b = self.read_exact(8)?;
        Ok(f64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_7bit_i32(&mut self) -> Result<i32, String> {
        Ok(self.read_7bit_i64()? as i32)
    }

    fn read_7bit_i64(&mut self) -> Result<i64, String> {
        let mut result = 0i64;
        let mut shift = 0;
        loop {
            let b = self.read_u8()?;
            result |= i64::from(b & 0x7F) << shift;
            if b & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift > 63 {
                return Err("7-bit integer overflow".into());
            }
        }
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

    fn pos(&self) -> u64 {
        self.cur.position()
    }
}

fn read_value(r: &mut Reader<'_>) -> Result<(String, DbValue), String> {
    if r.remaining() == 0 {
        return Ok((String::new(), DbValue::Null));
    }
    let header = r.read_u8()?;
    let ty = header & 0x1F;
    if ty == 0 {
        return Ok((String::new(), DbValue::Null));
    }
    let name = if header & FLAG_UNNAMED == 0 {
        r.read_cstring()?
    } else {
        String::new()
    };

    let value = match ty {
        TY_LIST => {
            let size = r.read_7bit_i64()? as u64;
            let start = r.pos();
            let mut kids = Vec::new();
            while r.pos().saturating_sub(start) < size {
                let (_n, child) = read_value(r)?;
                if matches!(child, DbValue::Null) {
                    break;
                }
                kids.push(child);
            }
            DbValue::List(kids)
        }
        TY_OBJECT => {
            let size = r.read_7bit_i64()? as u64;
            let start = r.pos();
            let mut fields = Vec::new();
            while r.pos().saturating_sub(start) < size {
                let (n, child) = read_value(r)?;
                if matches!(child, DbValue::Null) && n.is_empty() {
                    break;
                }
                fields.push((n, child));
            }
            DbValue::Object(fields)
        }
        TY_BOOL => DbValue::Bool(r.read_u8()? == 1),
        TY_STRING => {
            let len = r.read_7bit_i32()? as usize;
            if len == 0 {
                DbValue::String(String::new())
            } else {
                let raw = r.read_exact(len)?;
                let s = raw
                    .into_iter()
                    .filter(|b| *b != 0)
                    .collect::<Vec<_>>();
                DbValue::String(String::from_utf8_lossy(&s).into_owned())
            }
        }
        TY_INT => DbValue::Int(r.read_i32()?),
        TY_LONG => DbValue::Long(r.read_i64()?),
        TY_FLOAT => DbValue::Float(r.read_f32()?),
        TY_DOUBLE => DbValue::Double(r.read_f64()?),
        TY_GUID => {
            let b = r.read_exact(16)?;
            let mut g = [0u8; 16];
            g.copy_from_slice(&b);
            DbValue::Guid(g)
        }
        TY_SHA1 => {
            let b = r.read_exact(20)?;
            let mut s = [0u8; 20];
            s.copy_from_slice(&b);
            DbValue::Sha1(s)
        }
        TY_BYTES => {
            let n = r.read_7bit_i32()? as usize;
            DbValue::Bytes(r.read_exact(n)?)
        }
        other => return Err(format!("unsupported DbObject type 0x{other:02X}")),
    };
    Ok((name, value))
}

pub fn parse_db_object(data: &[u8]) -> Result<DbValue, String> {
    if data.is_empty() {
        return Err("empty DbObject payload".into());
    }
    let mut r = Reader::new(data);
    let (_name, value) = read_value(&mut r)?;
    if matches!(value, DbValue::Null) {
        return Err("root DbObject was empty/invalid".into());
    }
    Ok(value)
}

fn write_7bit(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn write_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_value(name: &str, value: &DbValue, out: &mut Vec<u8>) {
    let ty = match value {
        DbValue::Null => return,
        DbValue::List(_) => TY_LIST,
        DbValue::Object(_) => TY_OBJECT,
        DbValue::Bool(_) => TY_BOOL,
        DbValue::String(_) => TY_STRING,
        DbValue::Int(_) => TY_INT,
        DbValue::Long(_) => TY_LONG,
        DbValue::Float(_) => TY_FLOAT,
        DbValue::Double(_) => TY_DOUBLE,
        DbValue::Guid(_) => TY_GUID,
        DbValue::Sha1(_) => TY_SHA1,
        DbValue::Bytes(_) => TY_BYTES,
    };
    let flags = if name.is_empty() { FLAG_UNNAMED } else { 0 };
    out.push(flags | ty);
    if !name.is_empty() {
        out.extend_from_slice(name.as_bytes());
        out.push(0);
    }
    match value {
        DbValue::Null => {}
        DbValue::Object(fields) => {
            let mut sub = Vec::new();
            for (k, v) in fields {
                write_value(k, v, &mut sub);
            }
            write_7bit(out, (sub.len() + 1) as u64);
            out.extend_from_slice(&sub);
            out.push(0);
        }
        DbValue::List(items) => {
            let mut sub = Vec::new();
            for v in items {
                write_value("", v, &mut sub);
            }
            write_7bit(out, (sub.len() + 1) as u64);
            out.extend_from_slice(&sub);
            out.push(0);
        }
        DbValue::Bool(b) => out.push(if *b { 1 } else { 0 }),
        DbValue::String(s) => {
            write_7bit(out, (s.len() + 1) as u64);
            out.extend_from_slice(s.as_bytes());
            out.push(0);
        }
        DbValue::Int(v) => write_i32(out, *v),
        DbValue::Long(v) => {
            write_i32(out, (*v & 0xFFFF_FFFF) as i32);
            write_i32(out, (*v >> 32) as i32);
        }
        DbValue::Float(f) => out.extend_from_slice(&f.to_le_bytes()),
        DbValue::Double(d) => out.extend_from_slice(&d.to_le_bytes()),
        DbValue::Guid(g) => out.extend_from_slice(g),
        DbValue::Sha1(s) => out.extend_from_slice(s),
        DbValue::Bytes(b) => {
            write_7bit(out, b.len() as u64);
            out.extend_from_slice(b);
        }
    }
}

pub fn write_db_object(value: &DbValue) -> Vec<u8> {
    let mut out = Vec::new();
    write_value("", value, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_list_with_file() {
        let file = DbValue::Object(vec![
            ("name".into(), DbValue::String("Scripts/test.lua".into())),
            ("payload".into(), DbValue::Bytes(b"print(1)\n".to_vec())),
            ("length".into(), DbValue::Int(9)),
        ]);
        let entry = DbValue::Object(vec![
            ("name".into(), DbValue::String("Scripts/test.lua".into())),
            ("$file".into(), file),
        ]);
        let root = DbValue::List(vec![entry]);
        let bytes = write_db_object(&root);
        let parsed = parse_db_object(&bytes).unwrap();
        assert_eq!(parsed, root);
    }
}
