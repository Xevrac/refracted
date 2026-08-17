//! Initfs load/save: obfuscation detection, XOR/MEA unwrap, AES-128-CBC, re-pack.

use super::db::{parse_db_object, write_db_object, DbValue};
use std::path::Path;

pub const BODY_START: usize = 0x22C;
pub const KEY_OFFSET: usize = 0x128;
pub const KEY_LEN: usize = 0x101; // 257
pub const DEFAULT_AES_KEY: [u8; 16] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
];

const MAGIC_BF3: u32 = 0x00CED100;
const MAGIC_PVZ: u32 = 0x01CED100;
const MAGIC_NULL: u32 = 0x03CED100;
const MEA_TAIL_KEY: &[u8] = b"@e!adnXd$^!rfOsrDyIrI!xVgHeA!6Vc";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeobfuscatorType {
    Pvz,
    Mea,
    Da,
    Bf3,
    Null,
}

impl DeobfuscatorType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pvz => "PVZ",
            Self::Mea => "MEA",
            Self::Da => "DA",
            Self::Bf3 => "BF3",
            Self::Null => "Null",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedInitfs {
    pub path: std::path::PathBuf,
    pub kind: DeobfuscatorType,
    pub had_encrypted: bool,
    pub aes_key: Option<[u8; 16]>,
    /// Original file bytes (needed to copy the obfuscation header on save).
    pub original: Vec<u8>,
    /// Decrypted/deobfuscated payload list (or object) the editor mutates.
    pub root: DbValue,
}

fn magic_le(data: &[u8]) -> Option<u32> {
    if data.len() < 4 {
        return None;
    }
    Some(u32::from_le_bytes(data[0..4].try_into().unwrap()))
}

fn xor_pvz_in_place(data: &mut [u8], key: &[u8]) {
    let n = key.len();
    if n == 0 {
        return;
    }
    for (i, b) in data.iter_mut().enumerate() {
        *b ^= key[i % n];
    }
}

fn xor_bf3_in_place(data: &mut [u8], key: &[u8]) {
    let n = key.len();
    if n == 0 {
        return;
    }
    for (i, b) in data.iter_mut().enumerate() {
        *b ^= key[i % n] ^ 0x7B;
    }
}

fn pvz_key(src: &[u8]) -> Result<Vec<u8>, String> {
    if src.len() < KEY_OFFSET + KEY_LEN {
        return Err("file too small for PVZ key table".into());
    }
    let mut key = src[KEY_OFFSET..KEY_OFFSET + KEY_LEN].to_vec();
    for b in &mut key {
        *b ^= 0x7B;
    }
    Ok(key)
}

fn bf3_key(src: &[u8]) -> Result<Vec<u8>, String> {
    if src.len() < KEY_OFFSET + KEY_LEN {
        return Err("file too small for BF3 key table".into());
    }
    Ok(src[KEY_OFFSET..KEY_OFFSET + KEY_LEN].to_vec())
}

/// AES-128-CBC, IV = key (InitfsTools / OpenSSL default for this format).
pub fn aes_decrypt(buffer: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, String> {
    use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
    let mut buf = buffer.to_vec();
    let pt = Aes128CbcDec::new(key.into(), key.into())
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| format!("AES decrypt failed (bad key or padding): {e}"))?;
    Ok(pt.to_vec())
}

pub fn aes_encrypt(plain: &[u8], key: &[u8; 16]) -> Vec<u8> {
    use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
    let mut buf = vec![0u8; plain.len() + 16];
    buf[..plain.len()].copy_from_slice(plain);
    let n = Aes128CbcEnc::new(key.into(), key.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plain.len())
        .expect("PKCS7 encrypt")
        .len();
    buf.truncate(n);
    buf
}

fn try_parse_after(kind: DeobfuscatorType, raw: &[u8]) -> Result<DbValue, String> {
    let body = deobfuscate_body(kind, raw)?;
    parse_db_object(&body)
}

fn deobfuscate_body(kind: DeobfuscatorType, raw: &[u8]) -> Result<Vec<u8>, String> {
    match kind {
        DeobfuscatorType::Bf3 => {
            if raw.len() < BODY_START {
                return Err("file too small for BF3 header".into());
            }
            let key = bf3_key(raw)?;
            let mut body = raw[BODY_START..].to_vec();
            xor_bf3_in_place(&mut body, &key);
            Ok(body)
        }
        DeobfuscatorType::Pvz | DeobfuscatorType::Da => {
            if raw.len() < BODY_START {
                return Err("file too small for PVZ/DA header".into());
            }
            let key = pvz_key(raw)?;
            let mut body = raw[BODY_START..].to_vec();
            xor_pvz_in_place(&mut body, &key);
            Ok(body)
        }
        DeobfuscatorType::Null => {
            if let Some(m) = magic_le(raw) {
                if (m == MAGIC_PVZ || m == MAGIC_NULL) && raw.len() > BODY_START {
                    return Ok(raw[BODY_START..].to_vec());
                }
            }
            Ok(raw.to_vec())
        }
        DeobfuscatorType::Mea => mea_deobfuscate_body(raw),
    }
}

fn rotate_left_u32(value: i32, count: i32) -> i32 {
    let nbits = 32;
    let count = count.rem_euclid(nbits);
    if count == 0 {
        return value;
    }
    let u = value as u32;
    ((u << count) | (u >> (nbits - count))) as i32
}

fn mea_deobfuscate_block(buffer: &mut [u8], offset: usize, count: usize) {
    let mut a: i32 = 1_172_968_056;
    let mut z = 0i32;
    for i in 0..count {
        let idx = offset + i;
        let b = (buffer[idx] as i32)
            ^ (a + ((a >> 8) & 0xFF) + (a >> 16) + ((a >> 24) & 0xFF));
        buffer[idx] = b as u8;
        let c = rotate_left_u32(a, b & 0x1F);
        let packed = b | ((b | ((b | (b << 8)) << 8)) << 8);
        a = rotate_left_u32(packed.wrapping_add(c), 1);
        if z > 16 {
            a = a.wrapping_mul(2);
            z = 0;
        }
        z += 1;
    }
}

fn mix4(buf: &[u8], i0: usize) -> u32 {
    let mut t = buf[i0] as u32 ^ (2u32.wrapping_mul(buf[i0] as u32));
    t = buf[i0 + 1] as u32 ^ (2u32.wrapping_mul((buf[i0 + 1] as u32).wrapping_add(t)));
    t = buf[i0 + 2] as u32 ^ (2u32.wrapping_mul((buf[i0 + 2] as u32).wrapping_add(t)));
    buf[i0 + 3] as u32 ^ (2u32.wrapping_mul((buf[i0 + 3] as u32).wrapping_add(t)))
}

fn mea_deobfuscate_body(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.len() < 36 {
        return Err("file too small for MEA trailer".into());
    }
    let tail = &raw[raw.len() - 36..];
    let obf_size = i32::from_le_bytes(tail[0..4].try_into().unwrap());
    if obf_size <= 0 || (obf_size as usize) > raw.len() {
        return Err("invalid MEA obfuscation size".into());
    }
    if &tail[4..36] != MEA_TAIL_KEY {
        // Trailer key mismatch: treat as a CED100 skip if possible.
        if matches!(magic_le(raw), Some(MAGIC_PVZ | MAGIC_NULL)) && raw.len() > BODY_START {
            return Ok(raw[BODY_START..].to_vec());
        }
        return Ok(raw.to_vec());
    }

    let obf_size = obf_size as usize;
    let start = raw.len() - obf_size;
    let mut buf = raw[start..].to_vec();
    if buf.len() < 410 {
        return Err("MEA trailer too small".into());
    }

    let unknown = u16::from_le_bytes(buf[392..394].try_into().unwrap()) as i16;
    let mut tmp_a = 0u32;
    if unknown != 0 {
        for i in 0..unknown as usize {
            let c = buf[410 + i] as u32;
            tmp_a = c ^ (2u32.wrapping_mul(c.wrapping_add(tmp_a)));
        }
    }

    let mut tmp_b = 0u32;
    tmp_b = tmp_b.wrapping_add(mix4(&buf, 402));
    tmp_b = tmp_b.wrapping_add(mix4(&buf, 0));
    tmp_b = tmp_b.wrapping_add((buf[391] as u32) ^ (2u32.wrapping_mul(buf[391] as u32)));
    tmp_b = tmp_b.wrapping_add(tmp_a);
    tmp_b = tmp_b.wrapping_add(mix4(&buf, 394));

    let mut sub_total_a = mix4(&buf, 406);
    sub_total_a = sub_total_a.wrapping_add(tmp_b);

    let mut sub_total_b = 0u32;
    for i in 0..129 {
        let x = buf[(i * 3 + 5) - 1] as u32;
        let y = buf[i * 3 + 5] as u32;
        let z = buf[(i * 3 + 5) + 1] as u32;
        sub_total_b = z
            ^ (2u32.wrapping_mul(
                z.wrapping_add(y ^ (2u32.wrapping_mul(y.wrapping_add(x ^ (2u32.wrapping_mul(x.wrapping_add(sub_total_b))))))),
            ));
    }
    let total = sub_total_b.wrapping_add(sub_total_a);

    if unknown != 0 {
        mea_deobfuscate_block(&mut buf, 410, unknown as usize);
    }
    mea_deobfuscate_block(&mut buf, 394, 4);
    mea_deobfuscate_block(&mut buf, 0, 4);
    mea_deobfuscate_block(&mut buf, 402, 4);
    mea_deobfuscate_block(&mut buf, 406, 4);
    mea_deobfuscate_block(&mut buf, 4, 387);

    let magic = u32::from_le_bytes(buf[0..4].try_into().unwrap());
    let obf_type = buf[4];
    let initial = (buf[5] as u32 ^ total) as u8;

    let data_end = raw.len() - obf_size;
    let mut work = raw[..data_end].to_vec();
    let body_start = {
        let tmp_magic = magic_le(&work).unwrap_or(0);
        if tmp_magic == MAGIC_PVZ || tmp_magic == MAGIC_NULL {
            BODY_START.min(work.len())
        } else {
            0
        }
    };

    if magic == 4 {
        if obf_type != 2 {
            return Err(format!("MEA obfuscation method {obf_type} is not implemented"));
        }
        let mut current = initial;
        for (i, b) in work.iter_mut().enumerate() {
            let orig = *b;
            *b = current ^ orig;
            current = (orig ^ initial).wrapping_sub(i as u8);
        }
        Ok(work[body_start..].to_vec())
    } else if body_start > 0 {
        Ok(work[body_start..].to_vec())
    } else {
        Ok(work)
    }
}

pub fn auto_detect(raw: &[u8], path_hint: &str) -> DeobfuscatorType {
    let _ = path_hint;
    let magic = magic_le(raw).unwrap_or(0);
    if magic == MAGIC_BF3 {
        return DeobfuscatorType::Bf3;
    }
    if magic == MAGIC_PVZ || magic == MAGIC_NULL {
        let peek_n = raw.len().min(0x1000);
        if raw[..peek_n].windows(9).any(|w| w == b"encrypted") {
            return DeobfuscatorType::Null;
        }
        if raw.len() >= KEY_OFFSET + 257 {
            let key_slice = &raw[KEY_OFFSET..KEY_OFFSET + 257];
            if key_slice.iter().any(|b| *b != 0) {
                if try_parse_after(DeobfuscatorType::Pvz, raw)
                    .ok()
                    .and_then(|o| o.as_list().map(|l| !l.is_empty()))
                    .unwrap_or(false)
                {
                    return DeobfuscatorType::Pvz;
                }
            }
        }
    }
    if raw.len() >= 36 {
        let tail = &raw[raw.len() - 36..];
        if &tail[4..36] == MEA_TAIL_KEY {
            return DeobfuscatorType::Mea;
        }
    }
    if try_parse_after(DeobfuscatorType::Da, raw)
        .ok()
        .and_then(|o| o.as_list().map(|l| !l.is_empty()))
        .unwrap_or(false)
    {
        return DeobfuscatorType::Da;
    }
    DeobfuscatorType::Null
}

fn unwrap_encrypted(
    obj: DbValue,
    fallback: [u8; 16],
    extra_keys: &[[u8; 16]],
) -> Result<(DbValue, bool, Option<[u8; 16]>), String> {
    let Some(blob) = obj.field("encrypted").and_then(|v| v.as_bytes()) else {
        return Ok((obj, false, None));
    };
    let blob = blob.to_vec();
    let mut candidates = Vec::new();
    candidates.push(fallback);
    for k in extra_keys {
        if !candidates.contains(k) {
            candidates.push(*k);
        }
    }
    let mut last_err = String::new();
    for key in candidates {
        match aes_decrypt(&blob, &key) {
            Ok(plain) => match parse_db_object(&plain) {
                Ok(inner) => return Ok((inner, true, Some(key))),
                Err(e) => last_err = e,
            },
            Err(e) => last_err = e,
        }
    }
    Err(format!(
        "AES-wrapped initfs: none of the supplied keys worked ({last_err})"
    ))
}

pub fn load_initfs(
    path: &Path,
    extra_keys: &[[u8; 16]],
    prompted_key: Option<[u8; 16]>,
) -> Result<LoadedInitfs, String> {
    let raw = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if raw.is_empty() {
        return Err("file is empty".into());
    }
    let kind = auto_detect(&raw, &path.to_string_lossy());
    let obj = try_parse_after(kind, &raw)?;
    let mut keys = extra_keys.to_vec();
    if let Some(k) = prompted_key {
        keys.insert(0, k);
    }
    let (mut root, had_encrypted, aes_key) = unwrap_encrypted(obj, DEFAULT_AES_KEY, &keys)?;
    if root.is_object() && root.has_field("$file") {
        root = DbValue::List(vec![root]);
    }
    Ok(LoadedInitfs {
        path: path.to_path_buf(),
        kind,
        had_encrypted,
        aes_key,
        original: raw,
        root,
    })
}

fn header_bytes(original: &[u8], kind: DeobfuscatorType) -> Vec<u8> {
    match kind {
        DeobfuscatorType::Bf3 | DeobfuscatorType::Pvz | DeobfuscatorType::Da => {
            original.get(..BODY_START.min(original.len())).unwrap_or(&[]).to_vec()
        }
        DeobfuscatorType::Null | DeobfuscatorType::Mea => {
            if matches!(magic_le(original), Some(MAGIC_PVZ | MAGIC_NULL | MAGIC_BF3)) {
                original.get(..BODY_START.min(original.len())).unwrap_or(&[]).to_vec()
            } else {
                Vec::new()
            }
        }
    }
}

pub fn save_initfs(loaded: &LoadedInitfs, dest: &Path) -> Result<(), String> {
    let header = header_bytes(&loaded.original, loaded.kind);
    let mut out = Vec::new();

    if loaded.had_encrypted {
        let key = loaded.aes_key.unwrap_or(DEFAULT_AES_KEY);
        if key.len() != 16 {
            return Err("AES key must be 16 bytes".into());
        }
        let plain = write_db_object(&loaded.root);
        let encrypted = aes_encrypt(&plain, &key);
        let wrapper = DbValue::Object(vec![("encrypted".into(), DbValue::Bytes(encrypted))]);
        out.extend_from_slice(&header);
        out.extend_from_slice(&write_db_object(&wrapper));
    } else if loaded.kind == DeobfuscatorType::Bf3 {
        let mut body = write_db_object(&loaded.root);
        let key = bf3_key(&loaded.original)?;
        xor_bf3_in_place(&mut body, &key);
        out.extend_from_slice(&header);
        out.extend_from_slice(&body);
    } else if loaded.kind == DeobfuscatorType::Pvz || loaded.kind == DeobfuscatorType::Da {
        let mut body = write_db_object(&loaded.root);
        let key = pvz_key(&loaded.original)?;
        xor_pvz_in_place(&mut body, &key);
        out.extend_from_slice(&header);
        out.extend_from_slice(&body);
    } else {
        // Null / MEA: copy original header, write plaintext body (InitfsTools behaviour).
        out.extend_from_slice(&header);
        out.extend_from_slice(&write_db_object(&loaded.root));
    }

    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
        }
    }
    std::fs::write(dest, out).map_err(|e| format!("write {}: {e}", dest.display()))
}

pub fn parse_aes_key_hex(s: &str) -> Result<[u8; 16], String> {
    let hex: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if hex.len() != 32 {
        return Err("AES key must be 32 hex characters (16 bytes)".into());
    }
    let bytes = hex::decode(&hex).map_err(|e| format!("invalid hex: {e}"))?;
    let mut key = [0u8; 16];
    key.copy_from_slice(&bytes);
    Ok(key)
}

pub fn format_aes_key(key: &[u8; 16]) -> String {
    hex::encode_upper(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::initfs::db::write_db_object;

    #[test]
    fn aes_roundtrip() {
        let key = DEFAULT_AES_KEY;
        let pt = b"hello initfs aes pad test!!";
        let ct = aes_encrypt(pt, &key);
        let back = aes_decrypt(&ct, &key).unwrap();
        assert_eq!(back, pt);
    }

    #[test]
    fn detect_plain_list() {
        let file = DbValue::Object(vec![
            ("name".into(), DbValue::String("a.cfg".into())),
            ("payload".into(), DbValue::Bytes(b"x".to_vec())),
            ("length".into(), DbValue::Int(1)),
        ]);
        let entry = DbValue::Object(vec![
            ("name".into(), DbValue::String("a.cfg".into())),
            ("$file".into(), file),
        ]);
        let root = DbValue::List(vec![entry]);
        let bytes = write_db_object(&root);
        assert_eq!(auto_detect(&bytes, "initfs"), DeobfuscatorType::Null);
        let parsed = parse_db_object(&bytes).unwrap();
        assert!(parsed.as_list().is_some());
    }
}
