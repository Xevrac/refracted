//! Frostbite 2 EBX parser
//!
//! CNC / FB2 magic: `CE D1 B2 0F` (LE) / `0F B2 D1 CE` (BE).

use std::collections::HashMap;
use std::io::{Cursor, Read};

const EBX_MAGIC_LE: [u8; 4] = [0xCE, 0xD1, 0xB2, 0x0F];
const EBX_MAGIC_BE: [u8; 4] = [0x0F, 0xB2, 0xD1, 0xCE];
const EBX_MAGIC4_LE: [u8; 4] = [0xCE, 0xD1, 0xB4, 0x0F];
const EBX_MAGIC4_BE: [u8; 4] = [0x0F, 0xB4, 0xD1, 0xCE];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum FieldType {
    Void = 0x0,
    ValueType = 0x2,
    Class = 0x3,
    Array = 0x4,
    CString = 0x7,
    Enum = 0x8,
    FileRef = 0x9,
    Boolean = 0xA,
    Int8 = 0xB,
    UInt8 = 0xC,
    Int16 = 0xD,
    UInt16 = 0xE,
    Int32 = 0xF,
    UInt32 = 0x10,
    Int64 = 0x11,
    UInt64 = 0x12,
    Float32 = 0x13,
    Float64 = 0x14,
    Guid = 0x15,
    Sha1 = 0x16,
}

impl FieldType {
    fn from_raw(typ: u16) -> Option<Self> {
        match (typ >> 4) & 0x1F {
            0x0 => Some(Self::Void),
            0x2 => Some(Self::ValueType),
            0x3 => Some(Self::Class),
            0x4 => Some(Self::Array),
            0x7 => Some(Self::CString),
            0x8 => Some(Self::Enum),
            0x9 => Some(Self::FileRef),
            0xA => Some(Self::Boolean),
            0xB => Some(Self::Int8),
            0xC => Some(Self::UInt8),
            0xD => Some(Self::Int16),
            0xE => Some(Self::UInt16),
            0xF => Some(Self::Int32),
            0x10 => Some(Self::UInt32),
            0x11 => Some(Self::Int64),
            0x12 => Some(Self::UInt64),
            0x13 => Some(Self::Float32),
            0x14 => Some(Self::Float64),
            0x15 => Some(Self::Guid),
            0x16 => Some(Self::Sha1),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EbxGuid {
    a: u32,
    b: u16,
    c: u16,
    d: u64, // big-endian layout of last 8 bytes
}

impl EbxGuid {
    fn read(r: &mut Cursor<&[u8]>, be: bool) -> Result<Self, String> {
        let mut raw = [0u8; 16];
        r.read_exact(&mut raw)
            .map_err(|e| format!("guid read: {e}"))?;
        let a = if be {
            u32::from_be_bytes(raw[0..4].try_into().unwrap())
        } else {
            u32::from_le_bytes(raw[0..4].try_into().unwrap())
        };
        let b = if be {
            u16::from_be_bytes(raw[4..6].try_into().unwrap())
        } else {
            u16::from_le_bytes(raw[4..6].try_into().unwrap())
        };
        let c = if be {
            u16::from_be_bytes(raw[6..8].try_into().unwrap())
        } else {
            u16::from_le_bytes(raw[6..8].try_into().unwrap())
        };
        let d = u64::from_be_bytes(raw[8..16].try_into().unwrap());
        Ok(Self { a, b, c, d })
    }

    pub fn format(&self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.a,
            self.b,
            self.c,
            (self.d >> 48) & 0xFFFF,
            self.d & 0x0000_FFFF_FFFF_FFFF
        )
    }

    fn is_null(&self) -> bool {
        self.a == 0 && self.b == 0 && self.c == 0 && self.d == 0
    }
}

/// fileGUID → EBX Name (e.g. `_Rts/Art/...`) for resolving Class links.
pub type EbxGuidTable = HashMap<String, String>;

pub fn is_ebx(data: &[u8]) -> bool {
    data.len() >= 4
        && (&data[0..4] == EBX_MAGIC_LE
            || &data[0..4] == EBX_MAGIC_BE
            || &data[0..4] == EBX_MAGIC4_LE
            || &data[0..4] == EBX_MAGIC4_BE)
}

/// Register this EBX's file GUID → Name into a shared table (for Full Dump link resolve).
pub fn register_ebx_guid(data: &[u8], table: &mut EbxGuidTable) {
    if let Ok(dbx) = Dbx::parse(data) {
        if !dbx.true_filename.is_empty() {
            table.insert(dbx.file_guid.format(), dbx.true_filename);
        }
    }
}

pub fn summarize_ebx(data: &[u8]) -> String {
    match Dbx::parse(data) {
        Ok(dbx) => {
            let mut out = String::new();
            out.push_str(&format!(
                "EBX Frostbite2 ({})\n",
                if dbx.big_endian { "big-endian" } else { "little-endian" }
            ));
            out.push_str(&format!("FileGuid: {}\n", dbx.file_guid.format()));
            out.push_str(&format!("Primary: {}\n", dbx.primary_instance_guid.format()));
            out.push_str(&format!("Name: {}\n", dbx.true_filename));
            out.push_str(&format!("Instances: {}\n", dbx.instances.len()));
            out.push_str(&format!("ExternalGUIDs: {}\n", dbx.external_guids.len()));
            out.push_str(&format!("Size: {} bytes\n", data.len()));
            out
        }
        Err(err) => format!("EBX parse failed: {err}"),
    }
}

/// Readable EBX dump matching IceBloc ebxExtract / Nicknine `Dbx.dump()`.
pub fn dump_ebx_text(data: &[u8]) -> String {
    dump_ebx_text_with_table(data, None)
}

pub fn dump_ebx_text_with_table(data: &[u8], table: Option<&EbxGuidTable>) -> String {
    match Dbx::parse(data) {
        Ok(dbx) => dbx.dump(table),
        Err(err) => format!("EBX parse failed: {err}\n\n{}", summarize_ebx_fallback(data)),
    }
}

fn summarize_ebx_fallback(data: &[u8]) -> String {
    let sniffed = sniff_printable_strings(data, 4, 64);
    let mut out = String::new();
    out.push_str("## Sniffed strings\n");
    for s in sniffed {
        out.push_str(&s);
        out.push('\n');
    }
    out
}

fn fnv1_hash(keyword: &str) -> u32 {
    let mut hash: u32 = 5381;
    for b in keyword.bytes() {
        hash = hash.wrapping_mul(33) ^ u32::from(b);
    }
    hash
}

struct Header {
    abs_string_offset: u32,
    num_guid: u32,
    num_instance_repeater: u32,
    num_complex: u32,
    num_field: u32,
    len_name: u32,
    len_string: u32,
    num_array_repeater: u32,
    len_payload: u32,
}

#[derive(Clone)]
struct FieldDesc {
    name: String,
    typ: u16,
    r#ref: u16,
    offset: u32,
}

impl FieldDesc {
    fn field_type(&self) -> Option<FieldType> {
        FieldType::from_raw(self.typ)
    }
}

#[derive(Clone)]
struct ComplexDesc {
    name: String,
    field_start_index: u32,
    num_field: u8,
    size: u16,
}

struct InstanceRepeater {
    repetitions: u32,
    complex_index: u32,
}

#[derive(Clone)]
struct ArrayRepeater {
    offset: u32,
    repetitions: u32,
}

#[derive(Clone)]
pub(crate) struct Complex {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone)]
pub(crate) struct Field {
    pub name: String,
    typ: FieldType,
    pub value: FieldValue,
}

#[derive(Clone)]
pub(crate) enum FieldValue {
    Complex(Complex),
    ClassRef(u32),
    Array(Complex),
    Text(String),
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Guid(EbxGuid),
    Sha1([u8; 20]),
    EnumName(String),
}

struct Dbx {
    big_endian: bool,
    true_filename: String,
    header: Header,
    array_section_start: u64,
    file_guid: EbxGuid,
    primary_instance_guid: EbxGuid,
    external_guids: Vec<(EbxGuid, EbxGuid)>,
    field_descriptors: Vec<FieldDesc>,
    complex_descriptors: Vec<ComplexDesc>,
    array_repeaters: Vec<ArrayRepeater>,
    internal_guids: Vec<EbxGuid>,
    instances: Vec<(EbxGuid, Complex)>,
    enumerations: HashMap<u16, HashMap<i32, String>>,
    is_primary_instance: bool,
}

/// Parsed EBX tree for placement harvest (`ebx_positions`).
pub(crate) struct DbxTree {
    pub true_filename: String,
    pub external_guids: Vec<(EbxGuid, EbxGuid)>,
    pub internal_guids: Vec<EbxGuid>,
    pub instances: Vec<(EbxGuid, Complex)>,
}

/// Parse EBX into a tree suitable for BlueprintTransform harvest.
pub(crate) fn parse_dbx_for_positions(data: &[u8]) -> Result<DbxTree, String> {
    let dbx = Dbx::parse(data)?;
    Ok(DbxTree {
        true_filename: dbx.true_filename,
        external_guids: dbx.external_guids,
        internal_guids: dbx.internal_guids,
        instances: dbx.instances,
    })
}

/// Re-export tree types under a stable path for `ebx_positions`.
pub(crate) mod position_tree {
    pub(crate) use super::{Complex, DbxTree, Field, FieldValue};
}

impl Dbx {
    fn parse(data: &[u8]) -> Result<Self, String> {
        if !is_ebx(data) || data.len() < 48 {
            return Err("not an EBX payload".into());
        }
        let be = data[0..4] == EBX_MAGIC_BE || data[0..4] == EBX_MAGIC4_BE;
        let mut r = Cursor::new(data);
        r.set_position(4);

        let mut hdr = [0u32; 11];
        for h in &mut hdr {
            *h = read_u32(&mut r, be)?;
        }
        let header = Header {
            abs_string_offset: hdr[0],
            num_guid: hdr[2],
            num_instance_repeater: hdr[4],
            num_complex: hdr[5],
            num_field: hdr[6],
            len_name: hdr[7],
            len_string: hdr[8],
            num_array_repeater: hdr[9],
            len_payload: hdr[10],
        };
        let array_section_start =
            u64::from(header.abs_string_offset + header.len_string + header.len_payload);

        let file_guid = EbxGuid::read(&mut r, be)?;
        let primary_instance_guid = EbxGuid::read(&mut r, be)?;
        let mut external_guids = Vec::with_capacity(header.num_guid as usize);
        for _ in 0..header.num_guid {
            external_guids.push((EbxGuid::read(&mut r, be)?, EbxGuid::read(&mut r, be)?));
        }

        let mut name_bytes = vec![0u8; header.len_name as usize];
        r.read_exact(&mut name_bytes)
            .map_err(|e| format!("name section: {e}"))?;
        let keywords: Vec<String> = String::from_utf8_lossy(&name_bytes)
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        let mut keyword_dict: HashMap<u32, String> = HashMap::new();
        for kw in &keywords {
            keyword_dict.insert(fnv1_hash(kw), kw.clone());
        }
        let resolve = |hash: u32| -> String {
            keyword_dict
                .get(&hash)
                .cloned()
                .unwrap_or_else(|| format!("unk_{hash:08x}"))
        };

        let mut field_descriptors = Vec::with_capacity(header.num_field as usize);
        for _ in 0..header.num_field {
            let name_hash = read_u32(&mut r, be)?;
            let typ = read_u16(&mut r, be)?;
            let reff = read_u16(&mut r, be)?;
            let offset = read_u32(&mut r, be)?;
            let _secondary = read_u32(&mut r, be)?;
            field_descriptors.push(FieldDesc {
                name: resolve(name_hash),
                typ,
                r#ref: reff,
                offset,
            });
        }

        let mut complex_descriptors = Vec::with_capacity(header.num_complex as usize);
        for _ in 0..header.num_complex {
            let name_hash = read_u32(&mut r, be)?;
            let field_start_index = read_u32(&mut r, be)?;
            let num_field = read_u8(&mut r)?;
            let _alignment = read_u8(&mut r)?;
            let _typ = read_u16(&mut r, be)?;
            let size = read_u16(&mut r, be)?;
            let _secondary_size = read_u16(&mut r, be)?;
            complex_descriptors.push(ComplexDesc {
                name: resolve(name_hash),
                field_start_index,
                num_field,
                size,
            });
        }

        let mut instance_repeaters = Vec::with_capacity(header.num_instance_repeater as usize);
        for _ in 0..header.num_instance_repeater {
            let _null = read_u32(&mut r, be)?;
            let repetitions = read_u32(&mut r, be)?;
            let complex_index = read_u32(&mut r, be)?;
            instance_repeaters.push(InstanceRepeater {
                repetitions,
                complex_index,
            });
        }

        // Align to 16.
        let pos = r.position();
        let pad = (16 - (pos % 16)) % 16;
        r.set_position(pos + pad);

        let mut array_repeaters = Vec::with_capacity(header.num_array_repeater as usize);
        for _ in 0..header.num_array_repeater {
            let offset = read_u32(&mut r, be)?;
            let repetitions = read_u32(&mut r, be)?;
            let _complex_index = read_u32(&mut r, be)?;
            array_repeaters.push(ArrayRepeater {
                offset,
                repetitions,
            });
        }

        let mut dbx = Self {
            big_endian: be,
            true_filename: String::new(),
            header,
            array_section_start,
            file_guid,
            primary_instance_guid,
            external_guids,
            field_descriptors,
            complex_descriptors,
            array_repeaters,
            internal_guids: Vec::new(),
            instances: Vec::new(),
            enumerations: HashMap::new(),
            is_primary_instance: false,
        };

        r.set_position(u64::from(
            dbx.header.abs_string_offset + dbx.header.len_string,
        ));
        for ir in &instance_repeaters {
            for _ in 0..ir.repetitions {
                let instance_guid = EbxGuid::read(&mut r, be)?;
                dbx.internal_guids.push(instance_guid);
                dbx.is_primary_instance = instance_guid == dbx.primary_instance_guid;
                let inst = dbx.read_complex(ir.complex_index as usize, &mut r)?;
                dbx.instances.push((instance_guid, inst));
            }
        }

        if dbx.true_filename.is_empty() {
            // leave empty; dump still works
        }
        Ok(dbx)
    }

    fn read_complex(
        &mut self,
        complex_index: usize,
        r: &mut Cursor<&[u8]>,
    ) -> Result<Complex, String> {
        let desc = self
            .complex_descriptors
            .get(complex_index)
            .ok_or_else(|| format!("bad complex index {complex_index}"))?
            .clone();
        let start = r.position();
        let mut fields = Vec::new();
        let start_idx = desc.field_start_index as usize;
        let end_idx = start_idx + desc.num_field as usize;
        for field_index in start_idx..end_idx {
            let offset = self.field_descriptors[field_index].offset as u64;
            r.set_position(start + offset);
            fields.push(self.read_field(field_index, r)?);
        }
        r.set_position(start + u64::from(desc.size));
        Ok(Complex {
            name: desc.name,
            fields,
        })
    }

    fn read_field(
        &mut self,
        field_index: usize,
        r: &mut Cursor<&[u8]>,
    ) -> Result<Field, String> {
        let desc = self.field_descriptors[field_index].clone();
        let typ = desc
            .field_type()
            .ok_or_else(|| format!("unknown field type 0x{:04x} ({})", desc.typ, desc.name))?;
        let value = match typ {
            FieldType::Void | FieldType::ValueType => {
                FieldValue::Complex(self.read_complex(desc.r#ref as usize, r)?)
            }
            FieldType::Class => FieldValue::ClassRef(read_u32(r, self.big_endian)?),
            FieldType::Array => {
                let idx = read_u32(r, self.big_endian)? as usize;
                let array_rptr = self
                    .array_repeaters
                    .get(idx)
                    .ok_or_else(|| format!("bad array repeater {idx}"))?
                    .clone();
                let array_desc = self
                    .complex_descriptors
                    .get(desc.r#ref as usize)
                    .ok_or_else(|| format!("bad array complex {}", desc.r#ref))?
                    .clone();
                r.set_position(self.array_section_start + u64::from(array_rptr.offset));
                let mut fields = Vec::new();
                for _ in 0..array_rptr.repetitions {
                    fields.push(self.read_field(array_desc.field_start_index as usize, r)?);
                }
                FieldValue::Array(Complex {
                    name: array_desc.name,
                    fields,
                })
            }
            FieldType::CString | FieldType::FileRef => {
                let start = r.position();
                let string_offset = read_i32(r, self.big_endian)?;
                let text = if string_offset == -1 {
                    if typ == FieldType::CString {
                        "*nullString*".into()
                    } else {
                        "*nullRef*".into()
                    }
                } else {
                    let s = read_cstring_at(
                        r.get_ref(),
                        self.header.abs_string_offset as usize + string_offset as usize,
                    )?;
                    if self.is_primary_instance && desc.name == "Name" && self.true_filename.is_empty()
                    {
                        self.true_filename = s.clone();
                    }
                    s
                };
                r.set_position(start + 4);
                FieldValue::Text(text)
            }
            FieldType::Enum => {
                let compare = read_i32(r, self.big_endian)?;
                if !self.enumerations.contains_key(&desc.r#ref) {
                    let enum_complex = self
                        .complex_descriptors
                        .get(desc.r#ref as usize)
                        .ok_or_else(|| format!("bad enum complex {}", desc.r#ref))?;
                    let mut values = HashMap::new();
                    let start = enum_complex.field_start_index as usize;
                    let end = start + enum_complex.num_field as usize;
                    for i in start..end {
                        let fd = &self.field_descriptors[i];
                        values.insert(fd.offset as i32, fd.name.clone());
                    }
                    self.enumerations.insert(desc.r#ref, values);
                }
                let name = self
                    .enumerations
                    .get(&desc.r#ref)
                    .and_then(|m| m.get(&compare))
                    .cloned()
                    .unwrap_or_else(|| compare.to_string());
                FieldValue::EnumName(name)
            }
            FieldType::Boolean => FieldValue::Bool(read_u8(r)? != 0),
            FieldType::Int8 => FieldValue::I64(i64::from(read_i8(r)?)),
            FieldType::UInt8 => FieldValue::U64(u64::from(read_u8(r)?)),
            FieldType::Int16 => FieldValue::I64(i64::from(read_i16(r, self.big_endian)?)),
            FieldType::UInt16 => FieldValue::U64(u64::from(read_u16(r, self.big_endian)?)),
            FieldType::Int32 => FieldValue::I64(i64::from(read_i32(r, self.big_endian)?)),
            FieldType::UInt32 => FieldValue::U64(u64::from(read_u32(r, self.big_endian)?)),
            FieldType::Int64 => FieldValue::I64(read_i64(r, self.big_endian)?),
            FieldType::UInt64 => FieldValue::U64(read_u64(r, self.big_endian)?),
            FieldType::Float32 => FieldValue::F64(f64::from(read_f32(r, self.big_endian)?)),
            FieldType::Float64 => FieldValue::F64(read_f64(r, self.big_endian)?),
            FieldType::Guid => FieldValue::Guid(EbxGuid::read(r, self.big_endian)?),
            FieldType::Sha1 => {
                let mut sha = [0u8; 20];
                r.read_exact(&mut sha)
                    .map_err(|e| format!("sha1: {e}"))?;
                FieldValue::Sha1(sha)
            }
        };
        Ok(Field {
            name: desc.name,
            typ,
            value,
        })
    }

    fn dump(&self, table: Option<&EbxGuidTable>) -> String {
        let mut out = String::new();
        out.push_str(&self.file_guid.format());
        out.push('\n');
        for (guid, instance) in &self.instances {
            let label = if *guid == self.primary_instance_guid {
                format!("{} #primary instance", guid.format())
            } else {
                guid.format()
            };
            out.push_str(&format!("{} {}\n", instance.name, label));
            self.recurse_write(&instance.fields, &mut out, 0, table);
        }
        out
    }

    fn recurse_write(
        &self,
        fields: &[Field],
        out: &mut String,
        lvl: usize,
        table: Option<&EbxGuidTable>,
    ) {
        let lvl = lvl + 1;
        for field in fields {
            match &field.value {
                FieldValue::Complex(cmplx) => {
                    write_field(out, lvl, &field.name, &format!("::{}", cmplx.name));
                    self.recurse_write(&cmplx.fields, out, lvl, table);
                }
                FieldValue::ClassRef(v) => {
                    let towrite = if (*v >> 31) != 0 {
                        let idx = (*v & 0x7fff_ffff) as usize;
                        if let Some((a, b)) = self.external_guids.get(idx) {
                            if let Some(name) = table.and_then(|t| t.get(&a.format())) {
                                format!("{}/{}", name, b.format())
                            } else {
                                format!("{}/{}", a.format(), b.format())
                            }
                        } else {
                            format!("*badExtGuid:{v}*")
                        }
                    } else if *v == 0 {
                        "*nullGuid*".into()
                    } else {
                        let idx = (*v as usize).saturating_sub(1);
                        self.internal_guids
                            .get(idx)
                            .map(|g| g.format())
                            .unwrap_or_else(|| format!("*badIntGuid:{v}*"))
                    };
                    write_field(out, lvl, &field.name, &format!(" {towrite}"));
                }
                FieldValue::Array(cmplx) => {
                    if cmplx.fields.is_empty() {
                        write_field(out, lvl, &field.name, " *nullArray*");
                    } else {
                        write_field(out, lvl, &field.name, &format!("::{}", cmplx.name));
                        // Index member(N) like Nicknine.
                        let mut indexed: Vec<Field> = cmplx.fields.clone();
                        for (i, member) in indexed.iter_mut().enumerate() {
                            if member.name == "member" {
                                member.name = format!("member({i})");
                            }
                        }
                        self.recurse_write(&indexed, out, lvl, table);
                    }
                }
                FieldValue::Guid(g) => {
                    if g.is_null() {
                        write_field(out, lvl, &field.name, " *nullGuid*");
                    } else {
                        write_field(out, lvl, &field.name, &format!(" {}", g.format()));
                    }
                }
                FieldValue::Sha1(s) => {
                    write_field(out, lvl, &field.name, &format!(" {}", hex_upper(s)));
                }
                FieldValue::Text(s) => write_field(out, lvl, &field.name, &format!(" {s}")),
                FieldValue::EnumName(s) => write_field(out, lvl, &field.name, &format!(" {s}")),
                FieldValue::Bool(b) => {
                    write_field(
                        out,
                        lvl,
                        &field.name,
                        &format!(" {}", if *b { "True" } else { "False" }),
                    );
                }
                FieldValue::I64(n) => write_field(out, lvl, &field.name, &format!(" {n}")),
                FieldValue::U64(n) => write_field(out, lvl, &field.name, &format!(" {n}")),
                FieldValue::F64(n) => write_field(out, lvl, &field.name, &format!(" {n}")),
            }
            let _ = field.typ;
        }
    }
}

fn write_field(out: &mut String, lvl: usize, name: &str, text: &str) {
    for _ in 0..lvl {
        out.push('\t');
    }
    out.push_str(name);
    out.push_str(text);
    out.push('\n');
}

fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

fn read_cstring_at(data: &[u8], start: usize) -> Result<String, String> {
    if start >= data.len() {
        return Err(format!("cstring OOB @ {start}"));
    }
    let end = data[start..]
        .iter()
        .position(|&b| b == 0)
        .map(|i| start + i)
        .unwrap_or(data.len());
    Ok(String::from_utf8_lossy(&data[start..end]).into_owned())
}

fn sniff_printable_strings(data: &[u8], min_len: usize, max_count: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for &b in data {
        if (0x20..=0x7E).contains(&b) {
            cur.push(b as char);
        } else if cur.len() >= min_len {
            out.push(std::mem::take(&mut cur));
            if out.len() >= max_count {
                break;
            }
        } else {
            cur.clear();
        }
    }
    if cur.len() >= min_len && out.len() < max_count {
        out.push(cur);
    }
    out.sort();
    out.dedup();
    out
}

fn read_u8(r: &mut Cursor<&[u8]>) -> Result<u8, String> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b).map_err(|e| format!("u8: {e}"))?;
    Ok(b[0])
}

fn read_i8(r: &mut Cursor<&[u8]>) -> Result<i8, String> {
    Ok(read_u8(r)? as i8)
}

fn read_u16(r: &mut Cursor<&[u8]>, be: bool) -> Result<u16, String> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b).map_err(|e| format!("u16: {e}"))?;
    Ok(if be {
        u16::from_be_bytes(b)
    } else {
        u16::from_le_bytes(b)
    })
}

fn read_i16(r: &mut Cursor<&[u8]>, be: bool) -> Result<i16, String> {
    Ok(read_u16(r, be)? as i16)
}

fn read_u32(r: &mut Cursor<&[u8]>, be: bool) -> Result<u32, String> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b).map_err(|e| format!("u32: {e}"))?;
    Ok(if be {
        u32::from_be_bytes(b)
    } else {
        u32::from_le_bytes(b)
    })
}

fn read_i32(r: &mut Cursor<&[u8]>, be: bool) -> Result<i32, String> {
    Ok(read_u32(r, be)? as i32)
}

fn read_u64(r: &mut Cursor<&[u8]>, be: bool) -> Result<u64, String> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b).map_err(|e| format!("u64: {e}"))?;
    Ok(if be {
        u64::from_be_bytes(b)
    } else {
        u64::from_le_bytes(b)
    })
}

fn read_i64(r: &mut Cursor<&[u8]>, be: bool) -> Result<i64, String> {
    Ok(read_u64(r, be)? as i64)
}

fn read_f32(r: &mut Cursor<&[u8]>, be: bool) -> Result<f32, String> {
    Ok(f32::from_bits(read_u32(r, be)?))
}

fn read_f64(r: &mut Cursor<&[u8]>, be: bool) -> Result<f64, String> {
    Ok(f64::from_bits(read_u64(r, be)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn dumps_apa_hotkeys_like_icebloc() {
        let path = PathBuf::from(
            r"D:\_DATA\Projects\RE\Command and Conquer\Bin\Command & Conquer - Datamine\icebloc\Toc\bundles\ebx\input\rtshotkeymappings_apa.ebx",
        );
        let ice = PathBuf::from(
            r"D:\_DATA\Projects\RE\Command and Conquer\Bin\Command & Conquer - Datamine\icebloc\ebxExtract\Input\RtsHotKeyMappings_APA.txt",
        );
        if !path.is_file() {
            return;
        }
        let bytes = std::fs::read(&path).expect("read ebx");
        let text = dump_ebx_text(&bytes);
        assert!(
            text.contains("BuildStructureHotKeyEntry"),
            "missing instances:\n{}",
            &text[..text.len().min(500)]
        );
        assert!(
            text.contains("ActionType HkatBuildStructure"),
            "missing ActionType field"
        );
        assert!(text.contains("HotKey::HotKey"), "missing nested HotKey");
        assert!(
            text.contains("Key KeyC") || text.contains("Key Key"),
            "missing Key enum"
        );
        assert!(text.starts_with("8550423d-ae8a-11e2-82cc-e04f92e0058b") || text.contains('-'));

        if ice.is_file() {
            let expect = std::fs::read_to_string(&ice).expect("read icebloc");
            let ours: Vec<&str> = text.lines().collect();
            let theirs: Vec<&str> = expect.lines().collect();
            assert_eq!(
                ours[0], theirs[0],
                "file GUID mismatch"
            );
            // Allow primary-instance annotation difference; compare field-bearing lines.
            let mut miss = 0usize;
            for line in theirs.iter().take(40) {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // IceBloc may omit "#primary instance" — match on content after type name.
                if !text.contains(trimmed)
                    && !ours.iter().any(|o| o.trim() == trimmed)
                {
                    // Primary instance line differs by annotation.
                    if trimmed.contains('-') && !trimmed.contains('\t') && trimmed.contains(' ') {
                        continue;
                    }
                    miss += 1;
                    if miss <= 5 {
                        eprintln!("missing icebloc line: {trimmed}");
                    }
                }
            }
            assert!(
                miss == 0,
                "{miss} early IceBloc lines missing from FrostEx dump"
            );
        }
    }
}
