use crate::common::error::{BlazeError, BlazeResult};
use std::collections::HashMap;

/// Represents a TDF value type
#[derive(Debug, Clone)]
pub enum TdfValue {
    Integer(i64),
    String(String),
    Blob(Vec<u8>),
    Struct(HashMap<String, TdfValue>),
    List(Vec<TdfValue>),
    Map(HashMap<String, TdfValue>),
    Float(f32),
    Time(i64),
    Null,
}

/// Represents a TDF field with tag and value
#[derive(Debug, Clone)]
pub struct TdfField {
    pub tag: String,
    pub value: TdfValue,
}

/// Represents a TDF tree node for UI display
#[derive(Debug, Clone)]
pub struct TdfTreeNode {
    pub name: String,
    pub tag: String,
    pub value_type: String,
    pub value_display: String,
    pub children: Vec<TdfTreeNode>,
    pub raw_value: Option<TdfValue>, // Store raw value for detailed view
}

impl TdfTreeNode {
    pub fn new(name: String, tag: String, value_type: String, value_display: String) -> Self {
        Self {
            name,
            tag,
            value_type,
            value_display,
            children: Vec::new(),
            raw_value: None,
        }
    }
}

/// TDF Tree Parser
pub struct TdfTreeParser;

impl TdfTreeParser {
    /// Check if a string appears to be binary data (mostly non-printable characters)
    fn is_likely_binary(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        
        let bytes = s.as_bytes();
        let mut non_printable = 0;
        let mut invalid_utf8 = 0;
        
        for &byte in bytes {
            // Check if byte is printable ASCII (0x20-0x7E) or common whitespace
            if byte < 0x20 && byte != 0x09 && byte != 0x0A && byte != 0x0D {
                non_printable += 1;
            } else if byte > 0x7E {
                // Non-ASCII - check if it's valid UTF-8 continuation
                // For simplicity, count high bytes as potentially binary
                if byte > 0x7F {
                    invalid_utf8 += 1;
                }
            }
        }
        
        // If more than 30% of bytes are non-printable or invalid UTF-8, consider it binary
        let total_bytes = bytes.len();
        let suspicious_bytes = non_printable + invalid_utf8;
        suspicious_bytes * 10 > total_bytes * 3
    }
    
    /// Format a string value, detecting binary data and displaying as hex if needed
    fn format_string_value(s: &str, raw_bytes: &[u8]) -> String {
        // Check if this looks like binary data
        if Self::is_likely_binary(s) || raw_bytes.len() > 0 && Self::is_likely_binary_bytes(raw_bytes) {
            // Display as hex
            let hex_str: String = raw_bytes.iter()
                .take(256) // Limit to first 256 bytes for display
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            
            if raw_bytes.len() > 256 {
                format!("[BINARY DATA - {} bytes]\n{}...", raw_bytes.len(), hex_str)
            } else {
                format!("[BINARY DATA - {} bytes]\n{}", raw_bytes.len(), hex_str)
            }
        } else {
            // Normal string - return as-is
            s.to_string()
        }
    }
    
    /// Check if raw bytes appear to be binary data
    fn is_likely_binary_bytes(bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }
        
        let mut non_printable = 0;
        for &byte in bytes {
            // Count non-printable ASCII (excluding common whitespace)
            if byte < 0x20 && byte != 0x09 && byte != 0x0A && byte != 0x0D {
                non_printable += 1;
            }
        }
        
        // If more than 30% are non-printable, consider it binary
        non_printable * 10 > bytes.len() * 3
    }
    
    /// Decode TDF tag from 32-bit integer to 4-character label
    fn decode_tag_from_u32(tag_u32: u32) -> String {
        // Extract the 3 tag bytes from the upper 24 bits of the u32
        // Format: tag_u32 = [tag_byte0][tag_byte1][tag_byte2][type_byte] as BE uint32
        // We want the upper 24 bits: [tag_byte0][tag_byte1][tag_byte2]
        let tag_bytes = [
            ((tag_u32 >> 24) & 0xFF) as u8,
            ((tag_u32 >> 16) & 0xFF) as u8,
            ((tag_u32 >> 8) & 0xFF) as u8,
        ];
        
        // Use the same decoding logic as TdfEncoder::decode_tag
        let mut res = [0u8; 4];
        
        // Byte 0 reconstruction (from tag_bytes[0] bits)
        res[0] |= (tag_bytes[0] & 0x80) >> 1;
        res[0] |= (tag_bytes[0] & 0x40) >> 2;
        res[0] |= (tag_bytes[0] & 0x30) >> 2;
        res[0] |= (tag_bytes[0] & 0x0C) >> 2;
        
        // Byte 1 reconstruction (from tag_bytes[0] and tag_bytes[1] bits)
        res[1] |= (tag_bytes[0] & 0x02) << 5;
        res[1] |= (tag_bytes[0] & 0x01) << 4;
        res[1] |= (tag_bytes[1] & 0xF0) >> 4; // bits 7-4 → bits 3-0
        
        // Byte 2 reconstruction (from tag_bytes[1] and tag_bytes[2] bits)
        res[2] |= (tag_bytes[1] & 0x08) << 3;
        res[2] |= (tag_bytes[1] & 0x04) << 2;
        res[2] |= (tag_bytes[1] & 0x03) << 2;
        res[2] |= (tag_bytes[2] & 0xC0) >> 6;
        
        // Byte 3 reconstruction (from tag_bytes[2] bits)
        res[3] |= (tag_bytes[2] & 0x20) << 1;
        res[3] |= tag_bytes[2] & 0x1F;
        
        // Convert null bytes to spaces (0x20)
        for i in 0..4 {
            if res[i] == 0 {
                res[i] = 0x20;
            }
        }
        
        // Convert to string and trim trailing spaces
        let result = String::from_utf8_lossy(&res).trim_end().to_string();
        
        // TDF tags are encoded from 3 bytes, so they should decode to at most 4 characters
        // (before any hex escaping). If we get more, something is wrong.
        if result.len() > 4 {
            // This shouldn't happen, but if it does, return hex representation
            return format!("{:02X}{:02X}{:02X}", tag_bytes[0], tag_bytes[1], tag_bytes[2]);
        }
        
        // Validate that the decoded tag is reasonable - TDF tags should be mostly printable ASCII
        // If we get too many non-printable characters, this is likely data bytes being read as tags
        let non_printable_count = result.chars().filter(|c| {
            let byte = *c as u32;
            !c.is_ascii() || byte < 0x20 || byte > 0x7E
        }).count();
        
        // If more than 1 character is non-printable, this is likely invalid (data bytes as tags)
        // TDF tags should be mostly uppercase letters, numbers, and spaces
        if non_printable_count > 1 {
            // Return hex representation of the raw tag bytes instead
            format!("{:02X}{:02X}{:02X}", tag_bytes[0], tag_bytes[1], tag_bytes[2])
        } else if non_printable_count > 0 {
            // Some non-printable chars - replace them with hex escapes
            // Note: This may make the tag longer than 4 chars, but that's okay for display
            let mut cleaned = String::new();
            for c in result.chars() {
                let byte = c as u32;
                if c.is_ascii() && byte >= 0x20 && byte <= 0x7E {
                    cleaned.push(c);
                } else if byte <= 0xFF {
                    cleaned.push_str(&format!("\\x{:02X}", byte as u8));
                } else {
                    cleaned.push_str(&format!("\\u{:04X}", byte));
                }
            }
            cleaned
        } else {
            result
        }
    }
    
    /// Parse TDF data into a tree structure
    pub fn parse_packet(data: &[u8]) -> BlazeResult<Vec<TdfTreeNode>> {
        // Safety: return empty tree for empty data
        if data.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut nodes = Vec::new();
        let mut pos = 0;
        let max_iterations = data.len().min(10000); // Prevent infinite loops
        let mut iterations = 0;
        let mut last_pos = 0;
        let mut stuck_count = 0;
        
        while pos < data.len() && iterations < max_iterations {
            iterations += 1;
            
            // Safety: detect if we're stuck (not making progress)
            if pos == last_pos {
                stuck_count += 1;
                if stuck_count > 10 {
                    // We've been stuck for 10 iterations, break out
                    break;
                }
            } else {
                stuck_count = 0;
            }
            last_pos = pos;
            
            match Self::parse_field(data, &mut pos) {
                Ok(Some(node)) => {
                    nodes.push(node);
                    // Limit number of top-level nodes
                    if nodes.len() >= 1000 {
                        break;
                    }
                }
                Ok(None) => break, // End of data
                Err(_e) => {
                    // If we can't parse, try to skip and continue
                    if pos < data.len() {
                        pos += 1;
                        continue;
                    }
                    // If we're at the end, just return what we have
                    break;
                }
            }
        }
        
        Ok(nodes)
    }
    
    /// Parse a single TDF field
    fn parse_field(data: &[u8], pos: &mut usize) -> BlazeResult<Option<TdfTreeNode>> {
        // Safety check: ensure we have enough data
        if *pos >= data.len() {
            return Ok(None);
        }
        
        // Need at least 4 bytes: tag (4 bytes as uint32) + type (1 byte)
        // Actually, TDF stores tag as 32-bit BE int, but only uses bytes 1-3
        // Format: [tag_byte0][tag_byte1][tag_byte2][type_byte]
        // Safety: explicit bounds check to prevent panics
        if data.len() < *pos + 4 {
            return Ok(None);
        }
        
        // Additional safety check - ensure we can safely access all 4 bytes
        let remaining = data.len().saturating_sub(*pos);
        if remaining < 4 {
            return Ok(None);
        }
        
        // Read tag as 32-bit big-endian integer
        // Tag is stored as: (Head & 0xFFFFFF00), type is (Head & 0xFF)
        // Format: [tag_byte0][tag_byte1][tag_byte2][type_byte] as BE uint32
        // Use get() to safely access bytes and avoid panics
        let tag_bytes = match (
            data.get(*pos),
            data.get(*pos + 1),
            data.get(*pos + 2),
            data.get(*pos + 3),
        ) {
            (Some(&b0), Some(&b1), Some(&b2), Some(&b3)) => [b0, b1, b2, b3],
            _ => return Ok(None), // Not enough bytes
        };
        let tag_u32 = u32::from_be_bytes(tag_bytes);
        
        // Extract tag (upper 24 bits) and type (lower 8 bits)
        let tag_value = tag_u32 & 0xFFFFFF00;
        let type_byte = (tag_u32 & 0xFF) as u8;
        
        // Decode tag to 4-character label
        // The tag_value is a 32-bit integer where the lower 8 bits are 0
        // and the upper 24 bits contain the 3-byte encoded tag
        let tag = Self::decode_tag_from_u32(tag_value);
        
        // TDF tags are encoded from 3 bytes, so they should decode to at most 4 characters
        // If we get a tag longer than 4 characters, we're definitely reading data as tags
        if tag.len() > 4 {
            return Ok(None);
        }
        
        // Validate that this looks like a real tag, not data bytes
        // Real TDF tags are mostly uppercase letters, numbers, and spaces
        // If the decoded tag has too many non-printable characters, we're likely reading data as tags
        let non_printable_count = tag.chars().filter(|c| {
            let byte = *c as u32;
            !c.is_ascii() || byte < 0x20 || byte > 0x7E
        }).count();
        
        // Also check if type byte is reasonable (0x0-0xB are valid TDF types)
        // If both type is invalid AND tag has non-printable chars, definitely reading data as tags
        let invalid_type = type_byte > 0xB;
        
        if invalid_type && non_printable_count > 1 {
            // Definitely reading data as tags - return early to prevent cascading errors
            return Ok(None);
        }
        
        // If tag has more than 1 non-printable char, it's likely invalid even with valid type
        // TDF tags should be mostly uppercase letters, numbers, and spaces
        if non_printable_count > 1 {
            // Likely reading data as tags - return early
            return Ok(None);
        }
        
        // Additional check: if the tag has only 1-2 printable chars, it's very suspicious
        let printable_count = tag.chars().filter(|c| {
            let byte = *c as u32;
            c.is_ascii() && byte >= 0x20 && byte <= 0x7E
        }).count();
        if printable_count <= 2 && non_printable_count > 0 {
            // Too few printable chars with non-printable ones - likely data bytes
            return Ok(None);
        }
        
        *pos += 4;
        
            let tag_clone = tag.clone();
            let (node, _consumed) = match type_byte {
            0x0 => Self::parse_integer_varint(data, &tag_clone, pos)?, // INTEGER (varint)
            0x1 => Self::parse_string(data, &tag_clone, pos)?,
            0x2 => Self::parse_blob(data, &tag_clone, pos)?,
            0x3 => Self::parse_struct(data, &tag_clone, pos)?, // STRUCT (no length prefix, ends with 0x00)
            0x4 => Self::parse_list(data, &tag_clone, pos)?,
            0x5 => Self::parse_map(data, &tag_clone, pos)?,
            0x6 => Self::parse_union(data, &tag_clone, pos)?, // UNION
            0x7 => Self::parse_int64(data, &tag_clone, pos)?,
            0x8 => Self::parse_blob(data, &tag_clone, pos)?, // BLOB (alternative)
            // Type 0x09 is Blaze **OBJECT_ID** (3× varint): matches [`TdfEncoder::encode_object_id`] and [`TdfEncoder::skip_field`] (not IEEE float).
            0x9 => Self::parse_object_id_raw(data, &tag_clone, pos)?,
            0xA => Self::parse_time(data, &tag_clone, pos)?,
            0xB => Self::parse_struct(data, &tag_clone, pos)?, // STRUCT (without length, just null terminator)
            _ => {
                // Unknown type, try to skip
                if let Ok(skipped) = super::tdf_module::TdfEncoder::skip_field(&data[*pos..], type_byte) {
                    *pos += skipped;
                    return Ok(Some(TdfTreeNode::new(
                        tag_clone.clone(),
                        tag_clone,
                        format!("Unknown(0x{:02x})", type_byte),
                        "Unknown type".to_string(),
                    )));
                }
                return Err(BlazeError::TdfEncoding(format!("Unknown type byte: 0x{:02x}", type_byte)));
            }
        };
        
        // Note: All parse functions modify *pos internally, so we don't need to add consumed
        // The consumed value is returned for information, but pos is already advanced
        Ok(Some(node))
    }
    
    
    fn parse_string(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        // Safety: ensure we have enough data and pos is valid
        if *pos < 4 {
            return Err(BlazeError::TdfEncoding("Invalid position for string parsing".to_string()));
        }
        if *pos - 4 >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for string".to_string()));
        }
        let start_pos = *pos - 4; // decode_string expects tag+type at start
        match super::tdf_module::TdfEncoder::decode_string(&data[start_pos..]) {
            Ok((_, value, consumed)) => {
                *pos = start_pos + consumed; // Advance position to after the string
                
                // Get raw bytes for binary detection
                // The string data starts after: tag(4) + varint_length
                let string_data_start = start_pos + 4;
                let (_, varint_len) = super::tdf_module::TdfEncoder::decode_varint(&data[string_data_start..])
                    .unwrap_or((0, 1)); // Fallback if varint decode fails
                let string_bytes_start = string_data_start + varint_len;
                let string_bytes_end = *pos - 1; // -1 to exclude null terminator
                let raw_bytes = if string_bytes_end > string_bytes_start && string_bytes_end <= data.len() {
                    &data[string_bytes_start..string_bytes_end]
                } else {
                    value.as_bytes()
                };
                
                let formatted_value = Self::format_string_value(&value, raw_bytes);
                // For empty strings, explicitly show "(empty)" instead of blank
                let display_value = if value.is_empty() {
                    "(empty)".to_string()
                } else if formatted_value.starts_with("[BINARY") {
                    format!("[BINARY - {} bytes]", raw_bytes.len())
                } else {
                    value.clone()
                };
                let node = TdfTreeNode::new(
                    format!("{}: {}", tag, display_value),
                    tag.to_string(),
                    "STRING".to_string(),
                    if value.is_empty() {
                        "(empty)".to_string()
                    } else {
                        formatted_value
                    },
                );
                Ok((node, consumed - 4)) // Return consumed minus the 4 bytes we already advanced past
            }
            Err(e) => Err(e),
        }
    }
    
    /// Parse a string without tag+type header (used in maps/lists)
    /// Format: varint length + string bytes + null terminator
    fn parse_string_raw(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        let start_pos = *pos;
        
        // Read length (varint) - includes null terminator
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for string length".to_string()));
        }
        
        let (length, varint_len) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        let length = length as usize;
        *pos += varint_len;
        
        // Safety: ensure we have enough data
        if length == 0 || length > data.len() - *pos {
            return Err(BlazeError::TdfEncoding(format!("Invalid string length {}", length)));
        }
        
        // Read string value (length includes null terminator)
        let str_end = *pos + length - 1; // -1 to exclude null terminator
        if str_end > data.len() {
            return Err(BlazeError::TdfEncoding("String extends beyond data bounds".to_string()));
        }
        
        let raw_bytes = &data[*pos..str_end];
        let value = String::from_utf8_lossy(raw_bytes).to_string();
        *pos += length; // Advance past string including null terminator
        
        let formatted_value = Self::format_string_value(&value, raw_bytes);
        // For empty strings, explicitly show "(empty)" instead of blank
        let display_value = if value.is_empty() {
            "(empty)".to_string()
        } else if formatted_value.starts_with("[BINARY") {
            format!("[BINARY - {} bytes]", raw_bytes.len())
        } else {
            value.clone()
        };
        let name = if tag.is_empty() {
            display_value.clone()
        } else {
            format!("{}: {}", tag, display_value)
        };
        let node = TdfTreeNode::new(
            name,
            tag.to_string(),
            "STRING".to_string(),
            if value.is_empty() {
                "(empty)".to_string()
            } else {
                formatted_value
            },
        );
        
        Ok((node, *pos - start_pos))
    }
    
    fn parse_int64(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        if data.len() - *pos < 8 {
            return Err(BlazeError::TdfEncoding("Not enough data for int64".to_string()));
        }
        
        // Safety: use get() to safely access bytes and avoid panics
        let int_bytes = match (
            data.get(*pos),
            data.get(*pos + 1),
            data.get(*pos + 2),
            data.get(*pos + 3),
            data.get(*pos + 4),
            data.get(*pos + 5),
            data.get(*pos + 6),
            data.get(*pos + 7),
        ) {
            (Some(&b0), Some(&b1), Some(&b2), Some(&b3), Some(&b4), Some(&b5), Some(&b6), Some(&b7)) => {
                [b0, b1, b2, b3, b4, b5, b6, b7]
            }
            _ => return Err(BlazeError::TdfEncoding("Not enough data for int64".to_string())),
        };
        let value = i64::from_be_bytes(int_bytes);
        *pos += 8; // Advance position internally
        
        let node = TdfTreeNode::new(
            format!("{}: 0x{:X} ({})", tag, value, value),
            tag.to_string(),
            "INT64".to_string(),
            format!("0x{:X} ({})", value, value),
        );
        
        Ok((node, 8))
    }
    
    #[allow(dead_code)] // Legacy: type 0x09 is OBJECT_ID on the wire; floats use a different subtype in this codebase.
    fn parse_float(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        if data.len() - *pos < 4 {
            return Err(BlazeError::TdfEncoding("Not enough data for float".to_string()));
        }
        
        // Safety: use get() to safely access bytes and avoid panics
        let float_bytes = match (
            data.get(*pos),
            data.get(*pos + 1),
            data.get(*pos + 2),
            data.get(*pos + 3),
        ) {
            (Some(&b0), Some(&b1), Some(&b2), Some(&b3)) => [b0, b1, b2, b3],
            _ => return Err(BlazeError::TdfEncoding("Not enough data for float".to_string())),
        };
        let value = f32::from_be_bytes(float_bytes);
        *pos += 4; // Advance position internally
        
        let node = TdfTreeNode::new(
            format!("{}: {}", tag, value),
            tag.to_string(),
            "FLOAT".to_string(),
            value.to_string(),
        );
        
        Ok((node, 4))
    }
    
    fn parse_time(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        if data.len() - *pos < 8 {
            return Err(BlazeError::TdfEncoding("Not enough data for time".to_string()));
        }
        
        // Safety: use get() to safely access bytes and avoid panics
        let time_bytes = match (
            data.get(*pos),
            data.get(*pos + 1),
            data.get(*pos + 2),
            data.get(*pos + 3),
            data.get(*pos + 4),
            data.get(*pos + 5),
            data.get(*pos + 6),
            data.get(*pos + 7),
        ) {
            (Some(&b0), Some(&b1), Some(&b2), Some(&b3), Some(&b4), Some(&b5), Some(&b6), Some(&b7)) => {
                [b0, b1, b2, b3, b4, b5, b6, b7]
            }
            _ => return Err(BlazeError::TdfEncoding("Not enough data for time".to_string())),
        };
        let value = i64::from_be_bytes(time_bytes);
        *pos += 8; // Advance position internally
        
        let node = TdfTreeNode::new(
            format!("{}: {}", tag, value),
            tag.to_string(),
            "TIME".to_string(),
            value.to_string(),
        );
        
        Ok((node, 8))
    }
    
    fn parse_blob(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        match super::tdf_module::TdfEncoder::skip_field(&data[*pos..], 0x2) {
            Ok(skipped) => {
                // Safety: ensure we don't read past the end of data
                let blob_end = (*pos + skipped).min(data.len());
                let blob_data = &data[*pos..blob_end];
                let hex_dump = Self::format_hex_preview(blob_data, 32);
                *pos += skipped; // Advance position internally
                let node = TdfTreeNode::new(
                    format!("{}: BLOB ({} bytes)", tag, skipped),
                    tag.to_string(),
                    "BLOB".to_string(),
                    format!("Length: {} bytes\n{}", skipped, hex_dump),
                );
                Ok((node, skipped))
            }
            Err(e) => Err(e),
        }
    }
    
    /// Parse a BLOB without tag+type header (used in lists)
    /// Format: varint length + data bytes
    fn parse_blob_raw(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        let start_pos = *pos;
        
        // Read length (varint)
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for blob length".to_string()));
        }
        
        let (length, varint_len) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        let length = length as usize;
        *pos += varint_len;
        
        // Safety: ensure we have enough data
        if length > data.len() - *pos {
            return Err(BlazeError::TdfEncoding(format!("Invalid blob length {}", length)));
        }
        
        // Read blob data
        let blob_data = &data[*pos..*pos + length];
        let hex_dump = Self::format_hex_preview(blob_data, 32);
        *pos += length; // Advance past blob data
        
        let display_value = if blob_data.len() > 32 {
            format!("[BLOB - {} bytes]\n{}...", blob_data.len(), hex_dump)
        } else {
            format!("[BLOB - {} bytes]\n{}", blob_data.len(), hex_dump)
        };
        
        let name = if tag.is_empty() {
            format!("BLOB ({} bytes)", blob_data.len())
        } else {
            format!("{}: BLOB ({} bytes)", tag, blob_data.len())
        };
        
        let node = TdfTreeNode::new(
            name,
            tag.to_string(),
            "BLOB".to_string(),
            display_value,
        );
        
        Ok((node, *pos - start_pos))
    }
    
    /// Parse an ObjectId without tag+type header (used in lists)
    /// Format: 3 varints (component, type, id)
    fn parse_object_id_raw(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        let start_pos = *pos;
        
        // Read component (varint)
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for ObjectId component".to_string()));
        }
        let (component, varint_len1) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        *pos += varint_len1;
        
        // Read type (varint)
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for ObjectId type".to_string()));
        }
        let (obj_type, varint_len2) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        *pos += varint_len2;
        
        // Read id (varint)
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for ObjectId id".to_string()));
        }
        let (id, varint_len3) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        *pos += varint_len3;
        
        let display_value = format!("component=0x{:X} ({}) type=0x{:X} ({}) id=0x{:X} ({})", 
            component, component, obj_type, obj_type, id, id as i64);
        
        let name = if tag.is_empty() {
            format!("ObjectId(0x{:X}, 0x{:X}, 0x{:X})", component, obj_type, id)
        } else {
            format!("{}: ObjectId(0x{:X}, 0x{:X}, 0x{:X})", tag, component, obj_type, id)
        };
        
        let node = TdfTreeNode::new(
            name,
            tag.to_string(),
            "OBJECT_ID".to_string(),
            display_value,
        );
        
        Ok((node, *pos - start_pos))
    }
    
    /// Parse struct with length prefix (type 0x3)
    #[allow(dead_code)]
    fn parse_struct_with_length(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        let start_pos = *pos;
        
        // Read length (varint) - includes null terminator
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for struct length".to_string()));
        }
        
        let (struct_length, varint_len) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        let struct_length = struct_length as usize;
        *pos += varint_len;
        
        // Safety: ensure we have enough data
        if *pos + struct_length > data.len() {
            return Err(BlazeError::TdfEncoding(format!("Struct length {} exceeds available data", struct_length)));
        }
        
        // Parse struct content - length includes null terminator
        // We'll parse fields until we hit the null terminator (which should be at struct_length - 1 from struct start)
        let struct_start = *pos;
        let struct_end = struct_start + struct_length;
        
        // Parse fields from struct data
        let mut children = Vec::new();
        let mut has_startswith2 = false;
        
        // Check for optional 0x02 prefix
        if struct_start < data.len() && data[struct_start] == 0x02 {
            has_startswith2 = true;
            *pos += 1;
        }
        
        // Parse fields until we hit 0x00 terminator or reach struct_end
        while *pos < struct_end && *pos < data.len() {
            // Check for null terminator
            if data[*pos] == 0x00 {
                *pos += 1;
                break;
            }
            
            // Skip 0x02 if it appears (it's just a marker)
            if data[*pos] == 0x02 {
                has_startswith2 = true;
                *pos += 1;
                continue;
            }
            
            // Need at least 4 bytes for a field
            if struct_end - *pos < 4 || data.len() - *pos < 4 {
                break;
            }
            
            // Parse the next field
            match Self::parse_field(data, pos) {
                Ok(Some(child)) => {
                    children.push(child);
                    if children.len() >= 1000 {
                        break; // Limit to prevent memory issues
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    // Try to find null terminator within struct bounds
                    let search_end = struct_end.min(data.len());
                    if let Some(null_pos) = data[*pos..search_end].iter().position(|&b| b == 0x00) {
                        *pos += null_pos + 1;
                        break;
                    }
                    break;
                }
            }
        }
        
        // Ensure we're at struct_end (after null terminator)
        *pos = struct_end;
        
        let struct_name = if has_startswith2 {
            format!("{}: STRUCT (starts with 0x02)", tag)
        } else {
            format!("{}: STRUCT", tag)
        };
        
        let value_display = if children.is_empty() {
            "Empty struct".to_string()
        } else if children.len() <= 10 {
            let mut display = format!("Struct with {} fields:\n\n", children.len());
            for (i, child) in children.iter().enumerate() {
                display.push_str(&format!("{}. {} ({})\n", i + 1, child.tag, child.value_type));
            }
            display
        } else {
            format!("Struct with {} fields:\n\nShowing first 10:\n\n{}", 
                children.len(),
                children.iter().take(10).enumerate()
                    .map(|(i, c)| format!("{}. {} ({})", i + 1, c.tag, c.value_type))
                    .collect::<Vec<_>>()
                    .join("\n"))
        };
        
        let node = TdfTreeNode {
            name: struct_name,
            tag: tag.to_string(),
            value_type: "STRUCT".to_string(),
            value_display,
            children,
            raw_value: None,
        };
        
        let total_consumed = *pos - start_pos;
        Ok((node, total_consumed))
    }
    
    /// Parse struct without length prefix (type 0xB) - just reads until null terminator
    fn parse_struct(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        
        let start_pos = *pos;
        let mut has_startswith2 = false;
        
        // Safety: prevent infinite loops and ensure we don't read past end
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for struct".to_string()));
        }
        
        let max_iterations = data.len().saturating_sub(*pos).min(1000);
        let mut iterations = 0;
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: usize = 10;
        
        // Check for optional 0x02 prefix
        if *pos < data.len() && data[*pos] == 0x02 {
            has_startswith2 = true;
            *pos += 1;
        }
        
        // Parse fields until we hit 0x00 terminator
        let mut children = Vec::new();
        let mut last_pos = *pos;
        let mut stuck_count = 0;
        
        while *pos < data.len() && iterations < max_iterations {
            iterations += 1;
            
            // Safety: detect if we're stuck (not making progress)
            if *pos == last_pos {
                stuck_count += 1;
                if stuck_count > 5 {
                    // We've been stuck for 5 iterations, likely hit invalid data or missing terminator
                    // Try to find a null terminator nearby
                    let search_start = (*pos).saturating_sub(10);
                    let search_end = (*pos + 20).min(data.len());
                    if let Some(null_pos) = data[search_start..search_end].iter().position(|&b| b == 0x00) {
                        *pos = search_start + null_pos + 1;
                        break;
                    }
                    // If no null terminator found, break to prevent infinite loop
                    break;
                }
            } else {
                stuck_count = 0;
            }
            last_pos = *pos;
            
            // Safety: bounds check before accessing
            if *pos >= data.len() {
                break;
            }
            
            // Check for null terminator
            if data[*pos] == 0x00 {
                *pos += 1;
                break;
            }
            
            // Skip 0x02 if it appears (it's just a marker, not a field)
            if data[*pos] == 0x02 {
                has_startswith2 = true;
                *pos += 1;
                continue;
            }
            
            // Safety: need at least 4 bytes for a field
            if data.len() - *pos < 4 {
                // Not enough data for a field, likely end of struct
                break;
            }
            
            // Parse the next field
            match Self::parse_field(data, pos) {
                Ok(Some(child)) => {
                    children.push(child);
                    consecutive_errors = 0; // Reset error count on success
                    // Limit number of children to prevent memory issues
                    if children.len() >= 1000 {
                        break;
                    }
                }
                Ok(None) => break, // End of data
                Err(_e) => {
                    consecutive_errors += 1;
                    // If we have too many consecutive errors, likely hit invalid data
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        // Try to find null terminator to end struct gracefully
                        let search_start = *pos;
                        let search_end = (*pos + 50).min(data.len());
                        if let Some(null_pos) = data[search_start..search_end].iter().position(|&b| b == 0x00) {
                            *pos = search_start + null_pos + 1;
                            break;
                        }
                        // If we can't find terminator and have too many errors, break
                        break;
                    }
                    // If we can't parse, don't skip bytes - that causes out-of-sync issues
                    // Instead, try to find the next null terminator and end the struct
                    let search_start = *pos;
                    let search_end = (*pos + 100).min(data.len());
                    if let Some(null_pos) = data[search_start..search_end].iter().position(|&b| b == 0x00) {
                        *pos = search_start + null_pos + 1;
                        break;
                    }
                    // If no null terminator found, break to prevent reading data as tags
                    break;
                }
            }
        }
        
        let total_consumed = *pos - start_pos;
        
        let struct_name = if has_startswith2 {
            format!("{}: STRUCT (starts with 0x02)", tag)
        } else {
            format!("{}: STRUCT", tag)
        };
        
        // Create a more detailed value display for structs
        let value_display = if children.is_empty() {
            "Empty struct".to_string()
        } else if children.len() <= 10 {
            // Show all fields for small structs
            let mut display = format!("Struct with {} fields:\n\n", children.len());
            for (i, child) in children.iter().enumerate() {
                display.push_str(&format!("{}. {} ({})\n", i + 1, child.tag, child.value_type));
            }
            display
        } else {
            // Show summary for large structs
            format!("Struct with {} fields:\n\nShowing first 10:\n\n{}", 
                children.len(),
                children.iter().take(10).enumerate()
                    .map(|(i, c)| format!("{}. {} ({})", i + 1, c.tag, c.value_type))
                    .collect::<Vec<_>>()
                    .join("\n"))
        };
        
        let node = TdfTreeNode {
            name: struct_name,
            tag: tag.to_string(),
            value_type: "STRUCT".to_string(),
            value_display,
            children,
            raw_value: None,
        };
        
        Ok((node, total_consumed))
    }
    
    fn parse_list(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        if data.len() - *pos < 2 {
            return Err(BlazeError::TdfEncoding("Not enough data for list".to_string()));
        }
        
        let start_pos = *pos;
        // Safety: use get() to safely access bytes and avoid panics
        let item_type = match data.get(*pos) {
            Some(&t) => t,
            None => return Err(BlazeError::TdfEncoding("Not enough data for list item type".to_string())),
        };
        *pos += 1;
        
        let (list_len, varint_len) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        *pos += varint_len;
        
        // Safety: limit list length to prevent excessive parsing or out-of-bounds access
        let list_len = list_len.min(10000) as usize;
        
        let mut children = Vec::new();
        for _i in 0..list_len {
            // Safety: check if we have enough data remaining before parsing each item
            if data.len() - *pos < 1 {
                break; // Not enough data for another item
            }
            // Use empty tag for list items - the value_display will show the actual value
            let entry_tag = "";
            let child_result = match item_type {
                0x0 => Self::parse_integer_varint(data, entry_tag, pos),
                0x1 => Self::parse_string_raw(data, entry_tag, pos), // Strings in lists don't have tag+type
                0x2 => Self::parse_blob_raw(data, entry_tag, pos), // BLOBs in lists don't have tag+type
                0x3 => Self::parse_struct(data, entry_tag, pos), // Structs in lists (no length prefix)
                0x6 => Self::parse_union(data, entry_tag, pos), // List<NetworkAddress> etc.
                0x9 => Self::parse_object_id_raw(data, entry_tag, pos), // ObjectIds in lists (3 varints: component, type, id)
                _ => {
                    // Try to skip
                    if let Ok(skipped) = super::tdf_module::TdfEncoder::skip_field(&data[*pos..], item_type) {
                        // Safety: ensure skipped doesn't exceed data bounds
                        let actual_skip = skipped.min(data.len() - *pos);
                        *pos += actual_skip;
                        Ok((
                            TdfTreeNode::new(
                                format!("Unknown(0x{:02x})", item_type),
                                "".to_string(),
                                format!("Unknown(0x{:02x})", item_type),
                                format!("Unknown type 0x{:02x} (skipped {} bytes)", item_type, actual_skip),
                            ),
                            0,
                        ))
                    } else {
                        // Can't skip - create an "Unknown" node with error info instead of breaking
                        // This helps debug by showing that an item exists but couldn't be parsed
                        Ok((
                            TdfTreeNode::new(
                                format!("Unknown(0x{:02x})", item_type),
                                "".to_string(),
                                format!("Unknown(0x{:02x})", item_type),
                                format!("Unknown type 0x{:02x} - Could not skip field", item_type),
                            ),
                            0,
                        ))
                    }
                }
            };
            
            let (mut child, _consumed) = match child_result {
                Ok(node) => node,
                Err(e) => {
                    // Error parsing item - create an "Unknown" placeholder instead of breaking
                    // This helps debug issues by showing that an item exists but couldn't be parsed
                    let bytes_skipped = if let Ok(skipped) = super::tdf_module::TdfEncoder::skip_field(&data[*pos..], item_type) {
                        let actual_skip = skipped.min(data.len() - *pos);
                        *pos += actual_skip;
                        actual_skip
                    } else {
                        // If we can't skip, try to advance by at least 1 byte to avoid infinite loops
                        // This is a fallback - ideally skip_field should work
                        if *pos < data.len() {
                            *pos += 1;
                            1
                        } else {
                            0
                        }
                    };
                    let error_msg = format!("Parse error: {:?}", e);
                    (
                        TdfTreeNode::new(
                            format!("Unknown(0x{:02x}) - {}", item_type, error_msg),
                            "".to_string(),
                            format!("Unknown(0x{:02x})", item_type),
                            format!("Unknown - Failed to parse: {:?}", e),
                        ),
                        bytes_skipped,
                    )
                }
            };
            
            // For list items, use value_display as the name (removes "Entry #" prefix)
            if entry_tag.is_empty() {
                child.name = child.value_display.clone();
            }
            children.push(child);
            // Note: All parse functions (parse_integer_varint, parse_string_raw, parse_struct) 
            // modify *pos internally, so we don't need to add consumed again
        }
        
        let node = TdfTreeNode {
            name: format!("{}: LIST ({} items)", tag, list_len),
            tag: tag.to_string(),
            value_type: "LIST".to_string(),
            value_display: if list_len == 0 {
                "Empty list".to_string()
            } else if list_len <= 10 {
                format!("List with {} items:\n\n{}", 
                    list_len,
                    children.iter().take(10).enumerate()
                        .map(|(i, c)| format!("{}. {} ({})", i + 1, c.value_display, c.value_type))
                        .collect::<Vec<_>>()
                        .join("\n"))
            } else {
                format!("List with {} items:\n\nShowing first 10:\n\n{}\n\n... ({} more items)", 
                    list_len,
                    children.iter().take(10).enumerate()
                        .map(|(i, c)| format!("{}. {} ({})", i + 1, c.value_display, c.value_type))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    list_len - 10)
            },
            children,
            raw_value: None,
        };
        
        Ok((node, *pos - start_pos))
    }
    
    fn parse_integer_varint(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        match super::tdf_module::TdfEncoder::decode_varint(&data[*pos..]) {
            Ok((value, consumed)) => {
                *pos += consumed; // Advance position internally for consistency
                let value_display = format!("0x{:X} ({})", value, value as i64);
                let name = if tag.is_empty() {
                    value_display.clone()
                } else {
                    format!("{}: {}", tag, value_display)
                };
                let node = TdfTreeNode::new(
                    name,
                    tag.to_string(),
                    "INTEGER".to_string(),
                    value_display,
                );
                Ok((node, consumed))
            }
            Err(e) => Err(e),
        }
    }
    
    fn parse_map(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        if data.len() - *pos < 3 {
            return Err(BlazeError::TdfEncoding("Not enough data for map".to_string()));
        }
        
        let start_pos = *pos;
        // Safety: use get() to safely access bytes and avoid panics
        let key_type = match data.get(*pos) {
            Some(&k) => k,
            None => return Err(BlazeError::TdfEncoding("Not enough data for map key type".to_string())),
        };
        let val_type = match data.get(*pos + 1) {
            Some(&v) => v,
            None => return Err(BlazeError::TdfEncoding("Not enough data for map value type".to_string())),
        };
        *pos += 2;
        
        let (map_len, varint_len) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        *pos += varint_len;
        
        // Safety: limit map length to prevent excessive parsing or out-of-bounds access
        let map_len = map_len.min(10000) as usize;
        
        let mut children = Vec::new();
        for _i in 0..map_len {
            // Safety: check if we have enough data remaining before parsing each entry
            if data.len() - *pos < 4 {
                break; // Not enough data for another entry
            }
            // Parse key - wrap in error handling to prevent panics
            let key_result = match key_type {
                0x0 => Self::parse_integer_varint(data, "Key", pos),
                0x1 => Self::parse_string_raw(data, "Key", pos), // Strings in maps don't have tag+type
                _ => {
                    if let Ok(skipped) = super::tdf_module::TdfEncoder::skip_field(&data[*pos..], key_type) {
                        // Safety: ensure skipped doesn't exceed data bounds
                        let actual_skip = skipped.min(data.len() - *pos);
                        *pos += actual_skip;
                        Ok((
                            TdfTreeNode::new(
                                "Key".to_string(),
                                "Key".to_string(),
                                "Unknown".to_string(),
                                "Unknown".to_string(),
                            ),
                            0,
                        ))
                    } else {
                        break; // Can't skip, break out of loop
                    }
                }
            };
            
            let (key_node, _key_consumed) = match key_result {
                Ok(node) => node,
                Err(_) => break, // Error parsing key, break out of loop
            };
            // Note: parse functions modify *pos internally, so we don't need to add key_consumed
            // The consumed value is for information only
            
            // Safety: check bounds before parsing value
            if data.len() - *pos < 1 {
                break; // Not enough data for value
            }
            
            // Parse value - wrap in error handling to prevent panics
            let val_result = match val_type {
                0x0 => Self::parse_integer_varint(data, "Value", pos),
                0x1 => Self::parse_string_raw(data, "Value", pos), // Strings in maps don't have tag+type
                0x3 => Self::parse_struct(data, "Value", pos), // Structs in maps (no length prefix, ends with 0x00)
                0xC => Self::parse_generic_type_value(data, "Value", pos), // SCNA GenericType payload
                _ => {
                    if let Ok(skipped) = super::tdf_module::TdfEncoder::skip_field(&data[*pos..], val_type) {
                        // Safety: ensure skipped doesn't exceed data bounds
                        let actual_skip = skipped.min(data.len() - *pos);
                        *pos += actual_skip;
                        Ok((
                            TdfTreeNode::new(
                                "Value".to_string(),
                                "Value".to_string(),
                                "Unknown".to_string(),
                                "Unknown".to_string(),
                            ),
                            0,
                        ))
                    } else {
                        break; // Can't skip, break out of loop
                    }
                }
            };
            
            let (val_node, _val_consumed) = match val_result {
                Ok(node) => node,
                Err(_) => break, // Error parsing value, break out of loop
            };
            // Note: parse functions modify *pos internally, so we don't need to add val_consumed
            // The consumed value is for information only
            
            let entry_node = TdfTreeNode {
                name: format!("{} = {}", key_node.value_display, val_node.value_display),
                tag: key_node.value_display.clone(),
                value_type: "MAP_ENTRY".to_string(),
                value_display: format!("{} = {}", key_node.value_display, val_node.value_display),
                children: vec![key_node, val_node],
                raw_value: None,
            };
            children.push(entry_node);
        }
        
        let node = TdfTreeNode {
            name: format!("{}: MAP ({} entries)", tag, map_len),
            tag: tag.to_string(),
            value_type: "MAP".to_string(),
            value_display: if map_len == 0 {
                "Empty map".to_string()
            } else if map_len <= 10 {
                format!("Map with {} entries:\n\n{}", 
                    map_len,
                    children.iter().take(10).enumerate()
                        .map(|(i, c)| format!("{}. {}", i + 1, c.name))
                        .collect::<Vec<_>>()
                        .join("\n"))
            } else {
                format!("Map with {} entries:\n\nShowing first 10:\n\n{}\n\n... ({} more entries)", 
                    map_len,
                    children.iter().take(10).enumerate()
                        .map(|(i, c)| format!("{}. {}", i + 1, c.name))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    map_len - 10)
            },
            children,
            raw_value: None,
        };
        
        Ok((node, *pos - start_pos))
    }

    fn parse_generic_type_value(
        data: &[u8],
        name: &str,
        pos: &mut usize,
    ) -> BlazeResult<(TdfTreeNode, usize)> {
        let start_pos = *pos;
        if data.len().saturating_sub(*pos) < 3 {
            return Err(BlazeError::TdfEncoding(
                "Not enough data for GenericType".to_string(),
            ));
        }

        let is_present = data[*pos] != 0;
        *pos += 1;
        let (tdf_id, tdf_id_len) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        *pos += tdf_id_len;

        let value_type = *data
            .get(*pos)
            .ok_or_else(|| BlazeError::TdfEncoding("GenericType missing value type".to_string()))?;
        *pos += 1;

        let mut children = Vec::new();
        children.push(TdfTreeNode::new(
            "IsPresent".to_string(),
            "PRES".to_string(),
            "BOOL".to_string(),
            is_present.to_string(),
        ));
        children.push(TdfTreeNode::new(
            format!("TdfId: 0x{:08X} ({})", tdf_id, tdf_id),
            "TDFI".to_string(),
            "INTEGER".to_string(),
            format!("0x{:08X} ({})", tdf_id, tdf_id),
        ));

        let value_node = match value_type {
            0x0 => Self::parse_integer_varint(data, "VALU", pos)?.0,
            0x1 => Self::parse_string_raw(data, "VALU", pos)?.0,
            0x2 => Self::parse_blob_raw(data, "VALU", pos)?.0,
            0x3 => Self::parse_struct(data, "VALU", pos)?.0,
            0x4 => Self::parse_list(data, "VALU", pos)?.0,
            0x5 => Self::parse_map(data, "VALU", pos)?.0,
            0x6 => Self::parse_union(data, "VALU", pos)?.0,
            0x7 => Self::parse_int64(data, "VALU", pos)?.0,
            0x8 => Self::parse_blob(data, "VALU", pos)?.0,
            0x9 => Self::parse_object_id_raw(data, "VALU", pos)?.0,
            0xA => Self::parse_time(data, "VALU", pos)?.0,
            0xB => Self::parse_struct(data, "VALU", pos)?.0,
            _ => {
                if let Ok(skipped) = super::tdf_module::TdfEncoder::skip_field(&data[*pos..], value_type) {
                    *pos += skipped.min(data.len().saturating_sub(*pos));
                }
                TdfTreeNode::new(
                    format!("VALU: unsupported type 0x{:02X}", value_type),
                    "VALU".to_string(),
                    "UNKNOWN".to_string(),
                    format!("type=0x{:02X}", value_type),
                )
            }
        };
        children.push(value_node);

        if data.get(*pos) == Some(&0x00) {
            *pos += 1;
        }

        let node = TdfTreeNode {
            name: format!(
                "{}: GenericType(TdfId=0x{:08X}, Type=0x{:02X})",
                name, tdf_id, value_type
            ),
            tag: name.to_string(),
            value_type: "GENERIC_TYPE".to_string(),
            value_display: format!(
                "IsPresent={}, TdfId=0x{:08X} ({})",
                is_present, tdf_id, tdf_id
            ),
            children,
            raw_value: None,
        };

        Ok((node, *pos - start_pos))
    }
    
    fn parse_union(data: &[u8], tag: &str, pos: &mut usize) -> BlazeResult<(TdfTreeNode, usize)> {
        let start_pos = *pos;
        
        // Read active member index (varint)
        if *pos >= data.len() {
            return Err(BlazeError::TdfEncoding("Not enough data for union active member index".to_string()));
        }
        
        let (active_index, varint_len) = super::tdf_module::TdfEncoder::decode_varint(&data[*pos..])?;
        *pos += varint_len;
        
        // Parse the active member value
        // The value is encoded as a tagged TDF field (tag + type + value)
        // For ADDR, it's typically a STRUCT with tag "VALU"
        // We'll parse it as a regular TDF field
        let mut children = Vec::new();
        
        // Parse the next field (which is the union's active member value)
        let value_display = match Self::parse_field(data, pos) {
            Ok(Some(member_node)) => {
                let member_type = member_node.value_type.clone();
                children.push(member_node);
                format!("Active member index: {} ({})", active_index, member_type)
            }
            Ok(None) => {
                // End of data or couldn't parse
                format!("Active member index: {} (no value or parse error)", active_index)
            }
            Err(e) => {
                // Parse error - try to skip the field
                if let Ok(skipped) = super::tdf_module::TdfEncoder::skip_field(&data[*pos..], 0x3) {
                    // Try skipping as struct (most common for unions)
                    let actual_skip = skipped.min(data.len() - *pos);
                    *pos += actual_skip;
                    format!("Active member index: {} (skipped {} bytes, parse error: {:?})", 
                        active_index, actual_skip, e)
                } else {
                    format!("Active member index: {} (parse error: {:?})", active_index, e)
                }
            }
        };
        
        let node = TdfTreeNode {
            name: format!("{}: UNION (active member: {})", tag, active_index),
            tag: tag.to_string(),
            value_type: "UNION".to_string(),
            value_display,
            children,
            raw_value: None,
        };
        
        Ok((node, *pos - start_pos))
    }
    
    fn format_hex_preview(data: &[u8], max_bytes: usize) -> String {
        let preview = if data.len() > max_bytes {
            &data[..max_bytes]
        } else {
            data
        };
        
        preview
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

