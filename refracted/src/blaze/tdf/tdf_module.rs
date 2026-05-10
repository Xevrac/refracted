use crate::common::error::{BlazeError, BlazeResult};
use bytes::{BufMut, Bytes, BytesMut};

/// TDF (Tagged Data Format) encoder/decoder
pub struct TdfEncoder;

impl TdfEncoder {
    pub fn make_tag(tag: &str) -> [u8; 3] {
        // Get first 4 characters, pad with null bytes if needed
        let mut label = [0u8; 4];
        let tag_chars: Vec<u8> = tag.chars().take(4).map(|c| c as u8).collect();
        for (i, &byte) in tag_chars.iter().take(4).enumerate() {
            label[i] = byte;
        }
        
        // Encode algorithm
        let mut res = [0u8; 3];
        
        res[0] |= (label[0] & 0x40) << 1; // label[0] bit 6 → res[0] bit 7
        res[0] |= (label[0] & 0x10) << 2; // label[0] bit 4 → res[0] bit 6
        res[0] |= (label[0] & 0x0F) << 2; // label[0] bits 0-3 → res[0] bits 2-5
        res[0] |= (label[1] & 0x40) >> 5; // label[1] bit 6 → res[0] bit 1
        res[0] |= (label[1] & 0x10) >> 4; // label[1] bit 4 → res[0] bit 0
        
        res[1] |= (label[1] & 0x0F) << 4; // label[1] bits 0-3 → res[1] bits 4-7

        res[1] |= (label[2] & 0x40) >> 3; // label[2] bit 6 → res[1] bit 3
        res[1] |= (label[2] & 0x10) >> 2; // label[2] bit 4 → res[1] bit 2
        res[1] |= (label[2] & 0x0C) >> 2; // label[2] bits 2-3 → res[1] bits 0-1
        
        res[2] |= (label[2] & 0x03) << 6; // label[2] bits 0-1 → res[2] bits 6-7
        res[2] |= (label[3] & 0x40) >> 1; // label[3] bit 6 → res[2] bit 5
        res[2] |= label[3] & 0x1F; // label[3] bits 0-4 → res[2] bits 0-4
        
        res
    }

    pub fn encode_varint(value: u64) -> Bytes {
        if value < 0x40 {
            // Single byte for small values
            Bytes::from(vec![value as u8])
        } else {
            // Multi-byte encoding: first byte has 6 bits + continuation bit
            let mut result = Vec::new();
            let mut curbyte = (value & 0x3F) | 0x80;
            result.push(curbyte as u8);
            let mut currshift = value >> 6;

            while currshift >= 0x80 {
                curbyte = (currshift & 0x7F) | 0x80;
                result.push(curbyte as u8);
                currshift >>= 7;
            }
            result.push(currshift as u8); // Final byte without continuation bit
            Bytes::from(result)
        }
    }

    /// Decode a 3-byte encoded tag back to 4-character label
    pub fn decode_tag(buff: &[u8; 3]) -> String {
        // Reconstruct the 4-character label
        let mut res = [0u8; 4];
        
        // Byte 0 reconstruction (from buff[0] bits)
        res[0] |= (buff[0] & 0x80) >> 1;
        res[0] |= (buff[0] & 0x40) >> 2;
        res[0] |= (buff[0] & 0x30) >> 2;
        res[0] |= (buff[0] & 0x0C) >> 2;
        
        // Byte 1 reconstruction (from buff[0] and buff[1] bits)
        res[1] |= (buff[0] & 0x02) << 5;
        res[1] |= (buff[0] & 0x01) << 4;
        res[1] |= (buff[1] & 0xF0) >> 4; // bits 7-4 → bits 3-0
        
        // Byte 2 reconstruction (from buff[1] and buff[2] bits)
        res[2] |= (buff[1] & 0x08) << 3;
        res[2] |= (buff[1] & 0x04) << 2;
        res[2] |= (buff[1] & 0x03) << 2;
        res[2] |= (buff[2] & 0xC0) >> 6;
        
        // Byte 3 reconstruction (from buff[2] bits)
        res[3] |= (buff[2] & 0x20) << 1;
        res[3] |= buff[2] & 0x1F;
        
        // Convert null bytes to spaces (0x20)
        for i in 0..4 {
            if res[i] == 0 {
                res[i] = 0x20;
            }
        }
        
        // Don't trim - preserve trailing spaces as per TDF spec
        let result = String::from_utf8_lossy(&res).to_string();
        // Only trim null bytes at the very end, but keep spaces
        result.trim_end_matches('\0').to_string()
    }

    /// First four bytes of a root TDF field: packed 3-byte tag + low-byte `type` (big-endian u32 layout).
    pub fn root_field_label_and_type(data: &[u8]) -> Option<(String, u8)> {
        if data.len() < 4 {
            return None;
        }
        let u = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let type_byte = (u & 0xFF) as u8;
        if type_byte > 0x0B {
            return None;
        }
        let tb: [u8; 3] = [
            ((u >> 24) & 0xFF) as u8,
            ((u >> 16) & 0xFF) as u8,
            ((u >> 8) & 0xFF) as u8,
        ];
        Some((Self::decode_tag(&tb), type_byte))
    }

    /// Walk root-level fields using [`Self::skip_field`] only (no tree parse). Safer than parse-based
    /// extraction when earlier fields (e.g. maps) confuse [`crate::blaze::tdf::TdfTreeParser`].
    pub fn extract_top_level_field_bytes(data: &[u8], want_tag: &str) -> Option<Vec<u8>> {
        let want = want_tag.trim_end();
        let mut pos = 0usize;
        while pos + 4 <= data.len() {
            match Self::root_field_label_and_type(&data[pos..]) {
                Some((ref tag, type_byte)) => {
                    let start = pos;
                    pos += 4;
                    let skip = match Self::skip_field(&data[pos..], type_byte) {
                        Ok(s) => s,
                        Err(_) => {
                            pos = start.saturating_add(1);
                            continue;
                        }
                    };
                    pos += skip;
                    if tag.trim_end() == want {
                        return Some(data[start..pos].to_vec());
                    }
                }
                None => pos += 1,
            }
        }
        None
    }

    /// Root-level TDF tags without a full tree parse (uses [`Self::skip_field`]).
    /// Tuple: `(tag, type_byte, start_offset, total_field_byte_length)`.
    pub fn scan_root_level_fields(data: &[u8]) -> Vec<(String, u8, usize, usize)> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos + 4 <= data.len() {
            match Self::root_field_label_and_type(&data[pos..]) {
                Some((ref tag, type_byte)) => {
                    let start = pos;
                    pos += 4;
                    let skip = match Self::skip_field(&data[pos..], type_byte) {
                        Ok(s) => s,
                        Err(_) => {
                            pos = start.saturating_add(1);
                            continue;
                        }
                    };
                    let total_len = (pos - start) + skip;
                    pos += skip;
                    out.push((tag.clone(), type_byte, start, total_len));
                }
                None => pos += 1,
            }
        }
        out
    }

    /// Decode a variable-length integer
    pub fn decode_varint(data: &[u8]) -> BlazeResult<(u64, usize)> {
        if data.is_empty() {
            return Err(BlazeError::TdfEncoding("Empty data for varint".to_string()));
        }

        let first_byte = data[0];
        if first_byte < 0x80 {
            // Single byte
            Ok((first_byte as u64, 1))
        } else {
            // Multi-byte
            let mut result = (first_byte & 0x3F) as u64;
            let mut shift = 6;
            let mut pos = 1;

            while pos < data.len() {
                let byte = data[pos];
                result |= ((byte & 0x7F) as u64) << shift;
                pos += 1;

                if byte < 0x80 {
                    break;
                }
                shift += 7;
            }

            Ok((result, pos))
        }
    }

    /// Encode a string with length prefix
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + length + value + null terminator
    pub fn encode_string(tag: &str, value: &str) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);
        let value_bytes = value.as_bytes();

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Format: (tag[0] << 24) | (tag[1] << 16) | (tag[2] << 8) | type_byte
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x1); // TDF.Types.STRING
        
        result.extend_from_slice(&Self::encode_varint((value_bytes.len() + 1) as u64)); // +1 for null terminator
        result.extend_from_slice(value_bytes);
        result.put_u8(0x00); // Null terminator

        result.freeze()
    }

    /// Encode an integer
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + varint value
    pub fn encode_int(tag: &str, value: i32) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x0); // TDF.Types.INTEGER
        
        result.extend_from_slice(&Self::encode_varint(value as u64));

        result.freeze()
    }

    /// 64-bit integer: type **`0x07`** + **8** big-endian bytes (some GameManager notifies use this for ids).
    pub fn encode_int64_be(tag: &str, value: i64) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x07);
        result.put_slice(&value.to_be_bytes());
        result.freeze()
    }

    /// TDF type `0x06`: tag plus a single raw payload byte (used for `ADDR` discriminators, not varint).
    pub fn encode_int_single_byte(tag: &str, value: u8) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x06);
        result.put_u8(value);
        result.freeze()
    }

    /// Encode a long integer using varint encoding
    pub fn encode_long(tag: &str, value: i64) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x0); // TDF.Types.INTEGER
        
        result.extend_from_slice(&Self::encode_varint(value as u64));

        result.freeze()
    }

    /// Encode a boolean
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + value (1 byte)
    pub fn encode_bool(tag: &str, value: bool) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        // Note: Boolean type is typically 0x0 (INTEGER) but stored as 1 byte value
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x0); // TDF.Types.INTEGER (booleans use integer type)
        
        result.put_u8(if value { 1 } else { 0 });

        result.freeze()
    }

    /// Encode binary data (BLOB)
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + length + data
    pub fn encode_binary(tag: &str, value: &[u8]) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x2); // TDF.Types.BLOB
        
        result.extend_from_slice(&Self::encode_varint(value.len() as u64));
        result.extend_from_slice(value);

        result.freeze()
    }

    /// Encode a list of integers
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + item_type byte (0x0 for INTEGER) + count + varint items
    pub fn encode_list(tag: &str, values: &[i32]) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x4); // TDF.Types.LIST
        
        result.put_u8(0x0); // TDF.Types.INTEGER (item type)
        result.extend_from_slice(&Self::encode_varint(values.len() as u64));

        for &value in values {
            // Use varint encoding for integers in lists, not fixed 4-byte
            result.extend_from_slice(&Self::encode_varint(value as u64));
        }

        result.freeze()
    }

    /// List of UTF-8 strings (TDF list subtype `1`). Empty list encodes as zero elements.
    pub fn encode_string_list(tag: &str, values: &[&str]) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x4);
        result.put_u8(0x1);
        result.extend_from_slice(&Self::encode_varint(values.len() as u64));
        for v in values {
            let value_bytes = v.as_bytes();
            result.extend_from_slice(&Self::encode_varint((value_bytes.len() + 1) as u64));
            result.extend_from_slice(value_bytes);
            result.put_u8(0x00);
        }
        result.freeze()
    }

    /// Encode a list of long integers
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + item_type byte + count + varint items
    pub fn encode_long_list(tag: &str, values: &[i64]) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x4); // TDF.Types.LIST
        
        result.put_u8(0x0); // TDF.Types.INTEGER (item type)
        result.extend_from_slice(&Self::encode_varint(values.len() as u64));

        for &value in values {
            // Use varint encoding for integers in lists
            result.extend_from_slice(&Self::encode_varint(value as u64));
        }

        result.freeze()
    }

    /// Encode a map of string to string (deprecated - use encode_string_string_map)
    pub fn encode_map(tag: &str, map: &std::collections::HashMap<String, String>) -> Bytes {
        Self::encode_string_string_map(tag, map)
    }

    /// Encode a map of integer to integer
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + key_type (0x0 for INTEGER) + val_type (0x0 for INTEGER) + map length + key-value pairs
    pub fn encode_int_map(tag: &str, map: &std::collections::HashMap<i32, i32>) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x5); // TDF.Types.MAP

        result.put_u8(0x0); // TDF.Types.INTEGER (key type)
        result.put_u8(0x0); // TDF.Types.INTEGER (value type)
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));

        for (&key, &value) in map {
            // Encode as varints instead of fixed-size integers
            result.extend_from_slice(&Self::encode_varint(key as u64));
            result.extend_from_slice(&Self::encode_varint(value as u64));
        }

        result.freeze()
    }

    /// Blaze TDF type `5` — **double list** (parallel pairs), not a key/value map. Matches `WriteTdfDoubleList`:
    /// tag, type `5`, `subtype1`, `subtype2`, count, then for each index: `list1[i]` then `list2[i]`.
    /// Subtypes: `0` = varint integer, `1` = length-prefixed UTF-8 string + `0x00`.
    pub fn encode_double_list_int_int(tag: &str, list1: &[i64], list2: &[i64]) -> Bytes {
        assert_eq!(list1.len(), list2.len());
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x05);
        result.put_u8(0x00);
        result.put_u8(0x00);
        result.extend_from_slice(&Self::encode_varint(list1.len() as u64));
        for i in 0..list1.len() {
            result.extend_from_slice(&Self::encode_varint(list1[i] as u64));
            result.extend_from_slice(&Self::encode_varint(list2[i] as u64));
        }
        result.freeze()
    }

    /// Double list: string column then integer column (`subtype1` = 1, `subtype2` = 0).
    pub fn encode_double_list_string_int(tag: &str, list1: &[&str], list2: &[i64]) -> Bytes {
        assert_eq!(list1.len(), list2.len());
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x05);
        result.put_u8(0x01);
        result.put_u8(0x00);
        result.extend_from_slice(&Self::encode_varint(list1.len() as u64));
        for i in 0..list1.len() {
            let value_bytes = list1[i].as_bytes();
            result.extend_from_slice(&Self::encode_varint((value_bytes.len() + 1) as u64));
            result.extend_from_slice(value_bytes);
            result.put_u8(0x00);
            result.extend_from_slice(&Self::encode_varint(list2[i] as u64));
        }
        result.freeze()
    }

    /// Integer-to-integer map with stable key order (IndexMap).
    pub fn encode_int_int_map_ordered(tag: &str, map: &indexmap::IndexMap<i32, i32>) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x5);
        result.put_u8(0x0);
        result.put_u8(0x0);
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));
        for (&key, &value) in map {
            result.extend_from_slice(&Self::encode_varint(key as u64));
            result.extend_from_slice(&Self::encode_varint(value as u64));
        }
        result.freeze()
    }

    /// Encode a map of string to integer
    pub fn encode_string_int_map(tag: &str, map: &std::collections::HashMap<String, i32>) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Tag (3 bytes) + type byte (0x5 for MAP) + key_type (0x1 for STRING) + val_type (0x0 for INTEGER) + map length + key-value pairs
        result.extend_from_slice(&tag_encoded);
        result.put_u8(0x5); // TDF.Types.MAP
        result.put_u8(0x1); // TDF.Types.STRING (key type)
        result.put_u8(0x0); // TDF.Types.INTEGER (value type)
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));

        for (key, &value) in map {
            // Key (string) - encode with null terminator
            let key_bytes = key.as_bytes();
            result.extend_from_slice(&Self::encode_varint((key_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(key_bytes);
            result.put_u8(0x00); // Null terminator
            // Value (integer) - use varint
            result.extend_from_slice(&Self::encode_varint(value as u64));
        }

        result.freeze()
    }

    /// Encode a map of string to integer (IndexMap version to preserve order)
    pub fn encode_string_int_map_ordered(tag: &str, map: &indexmap::IndexMap<String, i32>) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Tag (3 bytes) + type byte (0x5 for MAP) + key_type (0x1 for STRING) + val_type (0x0 for INTEGER) + map length + key-value pairs
        result.extend_from_slice(&tag_encoded);
        result.put_u8(0x5); // TDF.Types.MAP
        result.put_u8(0x1); // TDF.Types.STRING (key type)
        result.put_u8(0x0); // TDF.Types.INTEGER (value type)
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));

        for (key, &value) in map {
            // Key (string) - encode with null terminator
            let key_bytes = key.as_bytes();
            result.extend_from_slice(&Self::encode_varint((key_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(key_bytes);
            result.put_u8(0x00); // Null terminator
            // Value (integer) - use varint
            result.extend_from_slice(&Self::encode_varint(value as u64));
        }

        result.freeze()
    }

    /// Encode a struct (nested TDF data)
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + content + null terminator (0x00)
    /// Note: Structs do NOT have a length prefix - they just end with 0x00
    pub fn encode_struct(tag: &str, struct_data: &[u8]) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x3); // TDF.Types.STRUCT
        
        // Note: struct_data may contain 0x00 bytes from nested structs, but we always add
        // a final 0x00 to terminate this struct level
        result.extend_from_slice(struct_data);
        result.put_u8(0x00); // Always add null terminator

        result.freeze()
    }

    /// Encode an ObjectId (3 integers: component, type, id)
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + ObjectId data (3 varints)
    pub fn encode_object_id(tag: &str, component: i32, obj_type: i32, id: i64) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x9); // TDF.Types.OBJECT_ID
        
        result.extend_from_slice(&Self::encode_varint(component as u64));
        result.extend_from_slice(&Self::encode_varint(obj_type as u64));
        result.extend_from_slice(&Self::encode_varint(id as u64));

        result.freeze()
    }

    /// Encode a list of ObjectIds
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + item_type (0x9 for OBJECT_ID) + count + ObjectIds
    pub fn encode_object_id_list(tag: &str, object_ids: &[(i32, i32, i64)]) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Combine tag (3 bytes) and type (1 byte) into 32-bit big-endian integer
        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x4); // TDF.Types.LIST
        
        result.put_u8(0x9); // TDF.Types.OBJECT_ID (item type)
        result.extend_from_slice(&Self::encode_varint(object_ids.len() as u64));

        for &(component, obj_type, id) in object_ids {
            // Encode each ObjectId as 3 varints
            result.extend_from_slice(&Self::encode_varint(component as u64));
            result.extend_from_slice(&Self::encode_varint(obj_type as u64));
            result.extend_from_slice(&Self::encode_varint(id as u64));
        }

        result.freeze()
    }

    /// Encode a map of string to string without a tag (for use inside structs)
    pub fn encode_string_string_map_untagged(
        map: &std::collections::HashMap<String, String>,
    ) -> Bytes {
        let mut result = BytesMut::new();

        // Type byte (0x5 for MAP) + key_type (0x1 for STRING) + val_type (0x1 for STRING) + map length + key-value pairs
        result.put_u8(0x5); // TDF.Types.MAP
        result.put_u8(0x1); // TDF.Types.STRING (key type)
        result.put_u8(0x1); // TDF.Types.STRING (value type)
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));

        for (key, value) in map {
            // Key (string) - encode with null terminator
            let key_bytes = key.as_bytes();
            result.extend_from_slice(&Self::encode_varint((key_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(key_bytes);
            result.put_u8(0x00); // Null terminator
            // Value (string) - encode with null terminator
            let value_bytes = value.as_bytes();
            result.extend_from_slice(&Self::encode_varint((value_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(value_bytes);
            result.put_u8(0x00); // Null terminator
        }

        result.freeze()
    }

    /// Encode a map of string to string
    pub fn encode_string_string_map(
        tag: &str,
        map: &std::collections::HashMap<String, String>,
    ) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x5); // TDF.Types.MAP
        
        // Append the untagged map data (key_type, val_type, length, entries)
        result.extend_from_slice(&Self::encode_string_string_map_untagged(map));

        result.freeze()
    }

    /// Encode a map of string to string (IndexMap version to preserve order)
    /// Format: tag+type as 32-bit BE integer (tag in upper 24 bits, type in lower 8 bits) + key_type (0x1 for STRING) + val_type (0x1 for STRING) + map length + key-value pairs
    pub fn encode_string_string_map_ordered(
        tag: &str,
        map: &indexmap::IndexMap<String, String>,
    ) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Write tag (3 bytes) + type (1 byte) as 4 bytes in big-endian format
        result.put_u8(tag_encoded[0]);
        result.put_u8(tag_encoded[1]);
        result.put_u8(tag_encoded[2]);
        result.put_u8(0x5); // TDF.Types.MAP
        
        result.put_u8(0x1); // TDF.Types.STRING (key type)
        result.put_u8(0x1); // TDF.Types.STRING (value type)
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));

        for (key, value) in map {
            // Key (string) - encode with null terminator
            let key_bytes = key.as_bytes();
            result.extend_from_slice(&Self::encode_varint((key_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(key_bytes);
            result.put_u8(0x00); // Null terminator
            // Value (string) - encode with null terminator
            let value_bytes = value.as_bytes();
            result.extend_from_slice(&Self::encode_varint((value_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(value_bytes);
            result.put_u8(0x00); // Null terminator
        }

        result.freeze()
    }

    /// Encode a map of string to struct
    /// Accepts both HashMap and IndexMap to preserve insertion order when needed
    pub fn encode_string_struct_map(
        tag: &str,
        map: &std::collections::HashMap<String, Vec<u8>>,
    ) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Tag (3 bytes) + type byte (0x5 for MAP) + key_type (0x1 for STRING) + val_type (0x3 for STRUCT) + map length + key-value pairs
        result.extend_from_slice(&tag_encoded);
        result.put_u8(0x5); // TDF.Types.MAP
        result.put_u8(0x1); // TDF.Types.STRING (key type)
        result.put_u8(0x3); // TDF.Types.STRUCT (value type)
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));

        for (key, struct_data) in map {
            // Key (string) - encode with null terminator
            let key_bytes = key.as_bytes();
            result.extend_from_slice(&Self::encode_varint((key_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(key_bytes);
            result.put_u8(0x00); // Null terminator
            // Value (struct data) - always add 0x00 terminator
            result.extend_from_slice(struct_data);
            result.put_u8(0x00); // Always add null terminator for struct
        }

        result.freeze()
    }

    /// Encode a map of string to struct (IndexMap version to preserve order)
    pub fn encode_string_struct_map_ordered(
        tag: &str,
        map: &indexmap::IndexMap<String, Vec<u8>>,
    ) -> Bytes {
        let mut result = BytesMut::new();
        let tag_encoded = Self::make_tag(tag);

        // Tag (3 bytes) + type byte (0x5 for MAP) + key_type (0x1 for STRING) + val_type (0x3 for STRUCT) + map length + key-value pairs
        result.extend_from_slice(&tag_encoded);
        result.put_u8(0x5); // TDF.Types.MAP
        result.put_u8(0x1); // TDF.Types.STRING (key type)
        result.put_u8(0x3); // TDF.Types.STRUCT (value type)
        result.extend_from_slice(&Self::encode_varint(map.len() as u64));

        for (key, struct_data) in map {
            // Key (string) - encode with null terminator
            let key_bytes = key.as_bytes();
            result.extend_from_slice(&Self::encode_varint((key_bytes.len() + 1) as u64)); // +1 for null terminator
            result.extend_from_slice(key_bytes);
            result.put_u8(0x00); // Null terminator
            // Value (struct data) - always add 0x00 terminator
            result.extend_from_slice(struct_data);
            result.put_u8(0x00); // Always add null terminator for struct
        }

        result.freeze()
    }

    /// Decode a string from TDF data
    /// Returns (tag, value, bytes_consumed)
    pub fn decode_string(data: &[u8]) -> BlazeResult<(String, String, usize)> {
        if data.len() < 4 {
            return Err(BlazeError::TdfEncoding("Not enough data for tag".to_string()));
        }

        // Read tag as 32-bit big-endian integer (tag in upper 24 bits, type in lower 8 bits)
        let tag_u32 = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let tag_value = tag_u32 & 0xFFFFFF00;
        let type_byte = (tag_u32 & 0xFF) as u8;
        
        // Decode tag from the 3-byte encoded form
        let tag_bytes = [
            ((tag_value >> 24) & 0xFF) as u8,
            ((tag_value >> 16) & 0xFF) as u8,
            ((tag_value >> 8) & 0xFF) as u8,
        ];
        let tag = Self::decode_tag(&tag_bytes);

        // Read type byte
        if type_byte != 0x1 {
            return Err(BlazeError::TdfEncoding("Expected string type".to_string()));
        }

        // Read length (varint)
        if data.len() < 5 {
            return Err(BlazeError::TdfEncoding("Not enough data for string length".to_string()));
        }
        let (length, varint_len) = Self::decode_varint(&data[4..])?;
        let length = length as usize;
        
        // Safety: ensure we have enough data for the string
        let str_start = 4 + varint_len;
        if str_start >= data.len() {
            return Err(BlazeError::TdfEncoding("Invalid string varint length".to_string()));
        }
        
        // Length must be at least 1 (for null terminator) and not exceed available data
        if length == 0 {
            return Err(BlazeError::TdfEncoding("String length cannot be zero (must include null terminator)".to_string()));
        }
        if length > data.len() - str_start {
            return Err(BlazeError::TdfEncoding("Invalid string length".to_string()));
        }

        // Read string value (length includes null terminator)
        // For empty strings, length = 1 (just null terminator), so str_end = str_start
        let str_end = str_start + length - 1; // -1 to exclude null terminator
        // Safety: bounds check before slicing
        if str_end > data.len() {
            return Err(BlazeError::TdfEncoding("String extends beyond data bounds".to_string()));
        }
        // For empty strings, str_end == str_start, which is valid (empty slice)
        let value = if str_end > str_start {
            String::from_utf8_lossy(&data[str_start..str_end]).to_string()
        } else {
            // Empty string (length = 1, just null terminator)
            String::new()
        };
        
        let bytes_consumed = str_start + length;
        Ok((tag, value, bytes_consumed))
    }

    /// Find and extract an integer field by tag name
    /// This searches through TDF fields recursively, including nested structs
    pub fn find_int_field(data: &[u8], tag: &str) -> Option<i32> {
        Self::find_int_field_recursive(data, tag, 0, 1000)
    }

    /// Compare wire bytes at `pos` with `make_tag(tag)` (TDF tags are 3 packed bytes, not UTF-8).
    #[inline]
    fn wire_tag_matches_at(data: &[u8], pos: usize, tag: &str) -> bool {
        if data.len() < pos + 3 {
            return false;
        }
        // Do not `trim()` — the 4th character (e.g. trailing space in `GID `) is part of the packed label.
        let expected = Self::make_tag(tag);
        data[pos..pos + 3] == expected
    }
    
    /// Recursive helper for find_int_field
    fn find_int_field_recursive(data: &[u8], tag: &str, start_pos: usize, max_depth: usize) -> Option<i32> {
        if max_depth == 0 {
            return None;
        }
        
        let mut pos = start_pos;
        let max_iterations = 1000;
        let mut iterations = 0;
        
        while pos < data.len() && iterations < max_iterations {
            iterations += 1;
            
            if data.len() - pos < 4 {
                break; // Need at least tag (3) + type (1)
            }
            
            let type_byte = data[pos + 3];
            let matches = Self::wire_tag_matches_at(data, pos, tag);
            
            if matches {
                // Found the tag, now decode based on type
                if type_byte == 0x0 {
                    // INTEGER type
                    if let Ok((value, _consumed)) = Self::decode_varint(&data[pos + 4..]) {
                        return Some(value as i32);
                    }
                } else if type_byte == 0x3 {
                    // STRUCT type - recursively search inside
                    if let Ok(struct_data) = Self::decode_struct_data(&data[pos + 4..]) {
                        if let Some(result) = Self::find_int_field_recursive(&struct_data, tag, 0, max_depth - 1) {
                            return Some(result);
                        }
                    }
                } else {
                    // Not an integer, skip it
                    if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                        pos += 3 + skipped;
                        continue;
                    }
                }
            }
            
            // Handle STRUCT type - recursively search inside
            if type_byte == 0x3 {
                if let Ok(struct_data) = Self::decode_struct_data(&data[pos + 4..]) {
                    if let Some(result) = Self::find_int_field_recursive(&struct_data, tag, 0, max_depth - 1) {
                        return Some(result);
                    }
                    // Skip the struct
                    if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                        pos += 3 + skipped;
                        continue;
                    }
                }
            }
            
            // Not the target tag, try to decode and skip
            if type_byte == 0x0 {
                // INTEGER - decode to get length and skip
                if let Ok((_value, varint_len)) = Self::decode_varint(&data[pos + 4..]) {
                    pos += 4 + varint_len;
                    continue;
                }
            } else if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                pos += 3 + skipped;
                continue;
            }
            
            // If we can't skip, advance by minimum amount
            pos += 4;
        }
        
        None
    }

    /// First INTEGER (`0x00` varint) or BIGINT (**`0x07`**, BE `i64`) for `tag` — **linear scan** (works inside null-terminated structs; does not rely on [`Self::decode_struct_data`]).
    pub fn find_long_field(data: &[u8], tag: &str) -> Option<i64> {
        let mut i = 0usize;
        while i + 4 <= data.len() {
            if Self::wire_tag_matches_at(data, i, tag) {
                let tb = data[i + 3];
                if tb == 0x0 {
                    if let Ok((value, _)) = Self::decode_varint(&data[i + 4..]) {
                        return Some(value as i64);
                    }
                } else if tb == 0x07 && i + 12 <= data.len() {
                    if let Ok(eight) = <[u8; 8]>::try_from(&data[i + 4..i + 12]) {
                        return Some(i64::from_be_bytes(eight));
                    }
                }
            }
            i += 1;
        }
        None
    }

    /// All integer values for `tag` in document order (DFS), e.g. two `"IP  "` for EXIP/INIP.
    pub fn find_all_int_fields(data: &[u8], tag: &str) -> Vec<i32> {
        let mut out = Vec::new();
        Self::find_all_int_fields_recursive(data, tag, 0, 1000, &mut out);
        out
    }

    fn find_all_int_fields_recursive(
        data: &[u8],
        tag: &str,
        start_pos: usize,
        max_depth: usize,
        out: &mut Vec<i32>,
    ) {
        if max_depth == 0 {
            return;
        }
        let mut pos = start_pos;
        let mut iterations = 0;
        while pos < data.len() && iterations < 1000 {
            iterations += 1;
            if data.len() - pos < 4 {
                break;
            }
            let type_byte = data[pos + 3];
            if Self::wire_tag_matches_at(data, pos, tag) && type_byte == 0x0 {
                if let Ok((value, consumed)) = Self::decode_varint(&data[pos + 4..]) {
                    out.push(value as i32);
                    pos += 4 + consumed;
                    continue;
                }
            }
            if type_byte == 0x3 {
                if let Ok(struct_data) = Self::decode_struct_data(&data[pos + 4..]) {
                    Self::find_all_int_fields_recursive(&struct_data, tag, 0, max_depth - 1, out);
                }
                if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                    pos += 3 + skipped;
                    continue;
                }
            }
            if type_byte == 0x0 {
                if let Ok((_v, vl)) = Self::decode_varint(&data[pos + 4..]) {
                    pos += 4 + vl;
                    continue;
                }
            } else if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                pos += 3 + skipped;
            } else {
                pos += 4;
            }
        }
    }

    /// Same as [`Self::find_all_int_fields`] but keeps full 32-bit range (e.g. IPv4 as `u32`).
    pub fn find_all_u32_fields(data: &[u8], tag: &str) -> Vec<u32> {
        let mut out = Vec::new();
        Self::find_all_u32_fields_recursive(data, tag, 0, 1000, &mut out);
        out
    }

    fn find_all_u32_fields_recursive(
        data: &[u8],
        tag: &str,
        start_pos: usize,
        max_depth: usize,
        out: &mut Vec<u32>,
    ) {
        if max_depth == 0 {
            return;
        }
        let mut pos = start_pos;
        let mut iterations = 0;
        while pos < data.len() && iterations < 1000 {
            iterations += 1;
            if data.len() - pos < 4 {
                break;
            }
            let type_byte = data[pos + 3];
            if Self::wire_tag_matches_at(data, pos, tag) && type_byte == 0x0 {
                if let Ok((value, consumed)) = Self::decode_varint(&data[pos + 4..]) {
                    out.push(value as u32);
                    pos += 4 + consumed;
                    continue;
                }
            }
            if type_byte == 0x3 {
                if let Ok(struct_data) = Self::decode_struct_data(&data[pos + 4..]) {
                    Self::find_all_u32_fields_recursive(&struct_data, tag, 0, max_depth - 1, out);
                }
                if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                    pos += 3 + skipped;
                    continue;
                }
            }
            if type_byte == 0x0 {
                if let Ok((_v, vl)) = Self::decode_varint(&data[pos + 4..]) {
                    pos += 4 + vl;
                    continue;
                }
            } else if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                pos += 3 + skipped;
            } else {
                pos += 4;
            }
        }
    }

    /// Byte-linear scan for `tag` + INTEGER (`0x00`) + varint — no nested struct decode.
    /// Use when on-wire structs are not length-prefixed the way [`Self::decode_struct_data`] expects.
    /// Also accepts type `0x07` (INT64) + 8 big-endian bytes (some clients use this for addresses).
    pub fn scan_all_u32_fields(data: &[u8], tag: &str) -> Vec<u32> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i + 4 <= data.len() {
            if Self::wire_tag_matches_at(data, i, tag) {
                let tb = data[i + 3];
                if tb == 0x0 {
                    if let Ok((value, consumed)) = Self::decode_varint(&data[i + 4..]) {
                        out.push(value as u32);
                        i += 4 + consumed;
                        continue;
                    }
                } else if tb == 0x7 && i + 12 <= data.len() {
                    if let Ok(eight) = <[u8; 8]>::try_from(&data[i + 4..i + 12]) {
                        out.push(i64::from_be_bytes(eight) as u32);
                        i += 12;
                        continue;
                    }
                }
            }
            i += 1;
        }
        out
    }

    /// Same as [`Self::scan_all_u32_fields`] but collects as `i32` (e.g. PORT).
    pub fn scan_all_int_fields(data: &[u8], tag: &str) -> Vec<i32> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i + 4 <= data.len() {
            if Self::wire_tag_matches_at(data, i, tag) {
                let tb = data[i + 3];
                if tb == 0x0 {
                    if let Ok((value, consumed)) = Self::decode_varint(&data[i + 4..]) {
                        out.push(value as i32);
                        i += 4 + consumed;
                        continue;
                    }
                } else if tb == 0x7 && i + 12 <= data.len() {
                    if let Ok(eight) = <[u8; 8]>::try_from(&data[i + 4..i + 12]) {
                        out.push(i64::from_be_bytes(eight) as i32);
                        i += 12;
                        continue;
                    }
                }
            }
            i += 1;
        }
        out
    }

    /// First STRING (`0x01`) field matching `tag` via linear scan.
    pub fn scan_first_string_field(data: &[u8], tag: &str) -> Option<String> {
        let mut i = 0usize;
        while i + 4 <= data.len() {
            if Self::wire_tag_matches_at(data, i, tag) && data[i + 3] == 0x1 {
                if let Ok((_t, value, _c)) = Self::decode_string(&data[i..]) {
                    return Some(value);
                }
            }
            i += 1;
        }
        None
    }

    /// Find and extract a string field by tag name
    /// This searches through TDF fields recursively, including nested structs
    pub fn find_string_field(data: &[u8], tag: &str) -> Option<String> {
        Self::find_string_field_recursive(data, tag, 0, 1000)
    }
    
    /// Recursive helper for find_string_field
    fn find_string_field_recursive(data: &[u8], tag: &str, start_pos: usize, max_depth: usize) -> Option<String> {
        if max_depth == 0 {
            return None;
        }
        
        let mut pos = start_pos;
        let max_iterations = 1000;
        let mut iterations = 0;
        
        while pos < data.len() && iterations < max_iterations {
            iterations += 1;
            
            if data.len() - pos < 4 {
                break; // Need at least tag (3) + type (1)
            }
            
            let type_byte = data[pos + 3];
            let matches = Self::wire_tag_matches_at(data, pos, tag);
            
            if matches {
                // Found the tag, now decode based on type
                if type_byte == 0x1 {
                    // STRING type
                    if let Ok((_, value, _consumed)) = Self::decode_string(&data[pos..]) {
                        return Some(value);
                    }
                } else if type_byte == 0x3 {
                    // STRUCT type - recursively search inside
                    if let Ok(struct_data) = Self::decode_struct_data(&data[pos + 4..]) {
                        if let Some(result) = Self::find_string_field_recursive(&struct_data, tag, 0, max_depth - 1) {
                            return Some(result);
                        }
                    }
                } else {
                    // Not a string, skip it
                    if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                        pos += 3 + skipped;
                        continue;
                    }
                }
            }
            
            // Handle STRUCT type - recursively search inside
            if type_byte == 0x3 {
                if let Ok(struct_data) = Self::decode_struct_data(&data[pos + 4..]) {
                    if let Some(result) = Self::find_string_field_recursive(&struct_data, tag, 0, max_depth - 1) {
                        return Some(result);
                    }
                    // Skip the struct
                    if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                        pos += 3 + skipped;
                        continue;
                    }
                }
            }
            
            // Not the target tag, try to decode and skip
            if type_byte == 0x1 {
                // STRING - decode to get length and skip
                if let Ok((_, _, consumed)) = Self::decode_string(&data[pos..]) {
                    pos += consumed;
                    continue;
                }
            }
            
            // Try to skip this field
            if let Ok(skipped) = Self::skip_field(&data[pos + 3..], type_byte) {
                pos += 3 + skipped;
            } else {
                // Can't skip, advance by 1 byte and try again
                pos += 1;
            }
        }
        
        None
    }
    
    /// Decode struct data (type 0x3)
    pub fn decode_struct_data(data: &[u8]) -> BlazeResult<Vec<u8>> {
        // Struct format: length (varint) + data + null terminator (0x00)
        let (length, varint_len) = Self::decode_varint(data)?;
        let length = length as usize;

        if length > data.len().saturating_sub(varint_len) {
            return Err(BlazeError::TdfEncoding("Invalid struct length".to_string()));
        }

        let struct_start = varint_len;
        let struct_end = struct_start + length;

        if struct_start >= struct_end {
            return Ok(Vec::new());
        }

        // Exclude trailing null only when it is inside the declared span (never produce struct_start > actual_end).
        let actual_end = if data.get(struct_end - 1) == Some(&0x00) {
            (struct_end - 1).max(struct_start)
        } else {
            struct_end
        };

        Ok(data[struct_start..actual_end].to_vec())
    }

    /// Skip a **null-terminated** struct body (type **`0x03`**) after tag+type: fields until a bare **`0x00`**
    /// end marker. Used by [`Self::skip_field`] and by root walks such as [`Self::extract_top_level_field_bytes`].
    fn skip_null_terminated_struct(data: &[u8]) -> BlazeResult<usize> {
        let mut pos = 0usize;
        while pos < data.len() {
            if data[pos] == 0x00 {
                return Ok(pos + 1);
            }
            if data.len() - pos < 4 {
                return Err(BlazeError::TdfEncoding(
                    "null-terminated struct truncated before field tag".to_string(),
                ));
            }
            let type_byte = data[pos + 3];
            pos += 4;
            let n = if type_byte == 0x3 {
                Self::skip_null_terminated_struct(&data[pos..])?
            } else {
                Self::skip_field(&data[pos..], type_byte)?
            };
            pos += n;
        }
        Err(BlazeError::TdfEncoding(
            "null-terminated struct missing 0x00 terminator".to_string(),
        ))
    }
    
    /// Skip a TDF field based on its type
    pub fn skip_field(data: &[u8], type_byte: u8) -> BlazeResult<usize> {
        match type_byte {
            0x0 => {
                let (_, vl) = Self::decode_varint(data)?;
                Ok(vl)
            }
            0x1 => {
                // STRING
                if data.len() < 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for string length".to_string()));
                }
                let (length, varint_len) = Self::decode_varint(data)?;
                Ok(varint_len + length as usize)
            }
            0x2 => {
                // BLOB - varint length + data
                if data.len() < 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for blob length".to_string()));
                }
                let (length, varint_len) = Self::decode_varint(data)?;
                Ok(varint_len + length as usize)
            }
            0x3 => Self::skip_null_terminated_struct(data),
            0x4 => {
                // LIST
                if data.len() < 2 {
                    return Err(BlazeError::TdfEncoding("Not enough data for list".to_string()));
                }
                let item_type = data[0];
                let (list_len, varint_len) = Self::decode_varint(&data[1..])?;
                let mut offset = 1 + varint_len;
                
                for _ in 0..list_len {
                    if item_type == 0x0 {
                        let (_, vl) = Self::decode_varint(&data[offset..])?;
                        offset += vl;
                    } else if item_type == 0x1 {
                        // STRING
                        let (item_len, item_varint_len) = Self::decode_varint(&data[offset..])?;
                        offset += item_varint_len + item_len as usize;
                    } else if item_type == 0x2 {
                        // BLOB
                        let (item_len, item_varint_len) = Self::decode_varint(&data[offset..])?;
                        offset += item_varint_len + item_len as usize;
                    } else if item_type == 0x3 {
                        while offset < data.len() {
                            if data[offset] == 0x00 {
                                offset += 1;
                                break;
                            }
                            if data.len() - offset < 4 {
                                return Err(BlazeError::TdfEncoding(
                                    "struct list row truncated".to_string(),
                                ));
                            }
                            let vtype = data[offset + 3];
                            offset += 4;
                            offset += Self::skip_field(&data[offset..], vtype)?;
                        }
                    } else if item_type == 0x6 {
                        offset += Self::skip_field(&data[offset..], 0x6)?;
                    } else if item_type == 0x9 {
                        // OBJECT_ID - 3 varints (component, type, id)
                        let (_, varint_len1) = Self::decode_varint(&data[offset..])?;
                        offset += varint_len1;
                        let (_, varint_len2) = Self::decode_varint(&data[offset..])?;
                        offset += varint_len2;
                        let (_, varint_len3) = Self::decode_varint(&data[offset..])?;
                        offset += varint_len3;
                    } else {
                        offset += 4; // Assume fixed size for other types
                    }
                }
                Ok(offset)
            }
            0x5 => {
                // MAP - complex, try to decode minimally
                if data.len() < 3 {
                    return Err(BlazeError::TdfEncoding("Not enough data for map".to_string()));
                }
                let key_type = data[0];
                let val_type = data[1];
                let (map_len, varint_len) = Self::decode_varint(&data[2..])?;
                let mut offset = 2 + varint_len;
                
                // Skip each key-value pair
                for _ in 0..map_len {
                    match key_type {
                        0x0 => {
                            let (_, vl) = Self::decode_varint(&data[offset..])?;
                            offset += vl;
                        }
                        0x1 => {
                            let (key_len, key_varint_len) = Self::decode_varint(&data[offset..])?;
                            offset += key_varint_len + key_len as usize;
                        }
                        _ => {
                            return Err(BlazeError::TdfEncoding(format!(
                                "skip_field MAP unsupported key_type 0x{:02x}",
                                key_type
                            )));
                        }
                    }
                    match val_type {
                        0x0 => {
                            let (_, vl) = Self::decode_varint(&data[offset..])?;
                            offset += vl;
                        }
                        0x1 => {
                            let (val_len, val_varint_len) = Self::decode_varint(&data[offset..])?;
                            offset += val_varint_len + val_len as usize;
                        }
                        0x2 => {
                            let (val_len, val_varint_len) = Self::decode_varint(&data[offset..])?;
                            offset += val_varint_len + val_len as usize;
                        }
                        0x3 => {
                            let (struct_len, struct_varint_len) = Self::decode_varint(&data[offset..])?;
                            offset += struct_varint_len + struct_len as usize;
                        }
                        0x7 => {
                            offset += 8;
                        }
                        _ => {
                            return Err(BlazeError::TdfEncoding(format!(
                                "skip_field MAP unsupported val_type 0x{:02x}",
                                val_type
                            )));
                        }
                    }
                }
                Ok(offset)
            }
            0x6 => {
                // UNION - varint active member index + tagged field value
                if data.len() < 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for union".to_string()));
                }
                // Read active member index
                let (_, varint_len) = Self::decode_varint(data)?;
                let mut offset = varint_len;
                
                // The union value is a tagged field (tag + type + value)
                // We need to skip the tag+type (4 bytes) and then skip the value based on its type
                if data.len() < offset + 4 {
                    return Err(BlazeError::TdfEncoding("Not enough data for union tag+type".to_string()));
                }
                
                let value_type = data[offset + 3]; // Type byte is the 4th byte
                offset += 4; // Skip tag (3 bytes) + type (1 byte)
                
                // Now skip the value based on its type
                match Self::skip_field(&data[offset..], value_type) {
                    Ok(value_skip) => {
                        offset += value_skip;
                        Ok(offset)
                    }
                    Err(_) => {
                        // If we can't skip properly, try to skip as struct (most common for unions)
                        if value_type == 0x3 || value_type == 0xB {
                            // STRUCT - try to find null terminator
                            let search_start = offset;
                            let search_end = (offset + 1000).min(data.len());
                            if let Some(null_pos) = data[search_start..search_end].iter().position(|&b| b == 0x00) {
                                offset += null_pos + 1;
                                Ok(offset)
                            } else {
                                // Can't find terminator, skip a reasonable amount
                                offset += 100;
                                Ok(offset)
                            }
                        } else {
                            // Unknown type, skip a fixed amount
                            offset += 100;
                            Ok(offset)
                        }
                    }
                }
            }
            0x7 => Ok(8), // INT64
            0x8 => {
                // BLOB (alternative encoding) - varint length + data
                if data.len() < 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for blob length".to_string()));
                }
                let (length, varint_len) = Self::decode_varint(data)?;
                Ok(varint_len + length as usize)
            }
            0x9 => {
                // OBJECT_ID - 3 varints (component, type, id)
                if data.len() < 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for ObjectId".to_string()));
                }
                let (_, varint_len1) = Self::decode_varint(data)?;
                if data.len() < varint_len1 + 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for ObjectId type".to_string()));
                }
                let (_, varint_len2) = Self::decode_varint(&data[varint_len1..])?;
                if data.len() < varint_len1 + varint_len2 + 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for ObjectId id".to_string()));
                }
                let (_, varint_len3) = Self::decode_varint(&data[varint_len1 + varint_len2..])?;
                Ok(varint_len1 + varint_len2 + varint_len3)
            }
            0xA => Ok(8), // TIME
            0xB => {
                // STRUCT
                if data.len() < 1 {
                    return Err(BlazeError::TdfEncoding("Not enough data for struct length".to_string()));
                }
                let (struct_len, varint_len) = Self::decode_varint(data)?;
                Ok(varint_len + struct_len as usize)
            }
            0xC => {
                // SCNA GenericType payload
                // Layout: IsPresent(1) + TdfId(varint) + ValueType(1) + Value + optional 0x00 terminator.
                if data.len() < 3 {
                    return Err(BlazeError::TdfEncoding(
                        "Not enough data for GenericType".to_string(),
                    ));
                }
                let mut offset = 0usize;
                offset += 1; // IsPresent
                let (_, id_len) = Self::decode_varint(&data[offset..])?;
                offset += id_len;
                let value_type = *data
                    .get(offset)
                    .ok_or_else(|| BlazeError::TdfEncoding("GenericType missing value type".to_string()))?;
                offset += 1;

                let value_skip = match value_type {
                    0x0 => {
                        let (_, vl) = Self::decode_varint(&data[offset..])?;
                        vl
                    }
                    0x1 | 0x2 => {
                        let (len, vl) = Self::decode_varint(&data[offset..])?;
                        vl + len as usize
                    }
                    0x3 | 0xB => Self::skip_null_terminated_struct(&data[offset..])?,
                    0x4 | 0x5 | 0x6 | 0x7 | 0x8 | 0x9 | 0xA => {
                        Self::skip_field(&data[offset..], value_type)?
                    }
                    _ => {
                        return Err(BlazeError::TdfEncoding(format!(
                            "GenericType unsupported value_type 0x{:02x}",
                            value_type
                        )));
                    }
                };
                offset += value_skip;
                if data.get(offset) == Some(&0x00) {
                    offset += 1;
                }
                Ok(offset)
            }
            _ => Err(BlazeError::TdfEncoding(format!("Unknown type byte: 0x{:02x}", type_byte))),
        }
    }
}
