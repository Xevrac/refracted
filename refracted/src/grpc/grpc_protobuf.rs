// Protobuf Encoding/Decoding for gRPC
// Implements protobuf wire format encoding

use crate::common::error::{BlazeError, BlazeResult};

/// Encode protobuf varint
pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    while value > 0x7F {
        result.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
    result.push(value as u8);
    result
}

/// Decode protobuf varint
pub fn decode_varint(data: &[u8], start_pos: usize) -> BlazeResult<(u64, usize)> {
    if start_pos >= data.len() {
        return Err(BlazeError::InvalidPacket("Varint decode: out of bounds".to_string()));
    }

    let mut result = 0u64;
    let mut shift = 0;
    let mut pos = start_pos;

    while pos < data.len() && shift < 64 {
        let byte = data[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;

        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
    }

    if shift >= 64 {
        return Err(BlazeError::InvalidPacket("Varint decode: too large".to_string()));
    }

    Ok((result, pos - start_pos))
}

/// Encode protobuf field header (field number + wire type)
pub fn encode_field_header(field_num: u32, wire_type: u32) -> Vec<u8> {
    let header = (field_num << 3) | wire_type;
    encode_varint(header as u64)
}

/// Encode protobuf string field (wire type 2: length-delimited)
pub fn encode_string_field(field_num: u32, value: &str) -> Vec<u8> {
    let data = value.as_bytes();
    let length = encode_varint(data.len() as u64);
    let field_header = encode_field_header(field_num, 2);
    let mut result = Vec::new();
    result.extend_from_slice(&field_header);
    result.extend_from_slice(&length);
    result.extend_from_slice(data);
    result
}

/// Encode protobuf int64 field (wire type 0: varint)
pub fn encode_int64_field(field_num: u32, value: i64) -> Vec<u8> {
    let data = encode_varint(value as u64);
    let field_header = encode_field_header(field_num, 0);
    let mut result = Vec::new();
    result.extend_from_slice(&field_header);
    result.extend_from_slice(&data);
    result
}

/// Encode protobuf uint64 field (wire type 0: varint)
pub fn encode_uint64_field(field_num: u32, value: u64) -> Vec<u8> {
    let data = encode_varint(value);
    let field_header = encode_field_header(field_num, 0);
    let mut result = Vec::new();
    result.extend_from_slice(&field_header);
    result.extend_from_slice(&data);
    result
}

/// Encode protobuf message field (wire type 2: length-delimited)
pub fn encode_message_field(field_num: u32, message: &[u8]) -> Vec<u8> {
    let length = encode_varint(message.len() as u64);
    let field_header = encode_field_header(field_num, 2);
    let mut result = Vec::new();
    result.extend_from_slice(&field_header);
    result.extend_from_slice(&length);
    result.extend_from_slice(message);
    result
}

/// Extract string field from protobuf data
pub fn extract_string_field(data: &[u8], field_num: u32) -> Option<String> {
    let mut pos = 0;
    while pos < data.len() {
        let (field_tag, consumed) = match decode_varint(data, pos) {
            Ok((tag, len)) => (tag, len),
            Err(_) => break,
        };

        pos += consumed;
        let found_field_num = (field_tag >> 3) as u32;
        let wire_type = (field_tag & 0x7) as u32;

        if found_field_num == field_num && wire_type == 2 {
            // Length-delimited field (string)
            let (length, len_consumed) = match decode_varint(data, pos) {
                Ok((len, consumed)) => (len, consumed),
                Err(_) => break,
            };

            pos += len_consumed;
            if pos + length as usize <= data.len() {
                let string_data = &data[pos..pos + length as usize];
                if let Ok(s) = String::from_utf8(string_data.to_vec()) {
                    return Some(s);
                }
            }
            break;
        } else {
            // Skip this field - need to determine its length
            if wire_type == 0 {
                // Varint - skip it
                let (_, varint_len) = decode_varint(data, pos).ok()?;
                pos += varint_len;
            } else if wire_type == 1 {
                // Fixed64 - skip 8 bytes
                if pos + 8 > data.len() {
                    break;
                }
                pos += 8;
            } else if wire_type == 2 {
                // Length-delimited - skip length + data
                let (length, len_consumed) = decode_varint(data, pos).ok()?;
                pos += len_consumed + length as usize;
            } else if wire_type == 3 {
                // Start group - skip until matching end group
                let group_field_num = found_field_num;
                let mut group_depth = 1;
                while pos < data.len() && group_depth > 0 {
                    let (group_tag, tag_len) = decode_varint(data, pos).ok()?;
                    pos += tag_len;
                    let group_field = (group_tag >> 3) as u32;
                    let group_wire = (group_tag & 0x7) as u32;
                    if group_wire == 3 && group_field == group_field_num {
                        group_depth += 1;
                    } else if group_wire == 4 && group_field == group_field_num {
                        group_depth -= 1;
                        if group_depth == 0 {
                            break;
                        }
                    } else if group_wire == 0 {
                        let (_, varint_len) = decode_varint(data, pos).ok()?;
                        pos += varint_len;
                    } else if group_wire == 1 {
                        if pos + 8 > data.len() {
                            break;
                        }
                        pos += 8;
                    } else if group_wire == 2 {
                        let (length, len_consumed) = decode_varint(data, pos).ok()?;
                        pos += len_consumed + length as usize;
                    } else if group_wire == 5 {
                        if pos + 4 > data.len() {
                            break;
                        }
                        pos += 4;
                    } else {
                        // Invalid wire type in group - stop parsing group
                        break;
                    }
                }
            } else if wire_type == 4 {
                // End group - just a tag, no data
                // Already consumed the tag, nothing to skip
            } else if wire_type == 5 {
                // Fixed32 - skip 4 bytes
                if pos + 4 > data.len() {
                    break;
                }
                pos += 4;
            } else {
                // Invalid wire type (only 0-5 are valid) - stop parsing
                break;
            }
        }
    }
    None
}

/// Extract int64 field from protobuf data
pub fn extract_int64_field(data: &[u8], field_num: u32) -> Option<i64> {
    let mut pos = 0;
    while pos < data.len() {
        let (field_tag, consumed) = match decode_varint(data, pos) {
            Ok((tag, len)) => (tag, len),
            Err(_) => break,
        };

        pos += consumed;
        let found_field_num = (field_tag >> 3) as u32;
        let wire_type = (field_tag & 0x7) as u32;

        if found_field_num == field_num && wire_type == 0 {
            // Varint field
            let (value, _varint_len) = decode_varint(data, pos).ok()?;
            return Some(value as i64);
        } else {
            // Skip this field
            if wire_type == 0 {
                let (_, varint_len) = decode_varint(data, pos).ok()?;
                pos += varint_len;
            } else if wire_type == 1 {
                if pos + 8 > data.len() {
                    break;
                }
                pos += 8;
            } else if wire_type == 2 {
                let (length, len_consumed) = decode_varint(data, pos).ok()?;
                pos += len_consumed + length as usize;
            } else if wire_type == 3 {
                // Start group - skip until matching end group
                let group_field_num = found_field_num;
                let mut group_depth = 1;
                while pos < data.len() && group_depth > 0 {
                    let (group_tag, tag_len) = decode_varint(data, pos).ok()?;
                    pos += tag_len;
                    let group_field = (group_tag >> 3) as u32;
                    let group_wire = (group_tag & 0x7) as u32;
                    if group_wire == 3 && group_field == group_field_num {
                        group_depth += 1;
                    } else if group_wire == 4 && group_field == group_field_num {
                        group_depth -= 1;
                        if group_depth == 0 {
                            break;
                        }
                    } else if group_wire == 0 {
                        let (_, varint_len) = decode_varint(data, pos).ok()?;
                        pos += varint_len;
                    } else if group_wire == 1 {
                        if pos + 8 > data.len() {
                            break;
                        }
                        pos += 8;
                    } else if group_wire == 2 {
                        let (length, len_consumed) = decode_varint(data, pos).ok()?;
                        pos += len_consumed + length as usize;
                    } else if group_wire == 5 {
                        if pos + 4 > data.len() {
                            break;
                        }
                        pos += 4;
                    } else {
                        // Invalid wire type in group - stop parsing group
                        break;
                    }
                }
            } else if wire_type == 4 {
                // End group - just a tag, no data
                // Already consumed the tag, nothing to skip
            } else if wire_type == 5 {
                if pos + 4 > data.len() {
                    break;
                }
                pos += 4;
            } else {
                // Invalid wire type (only 0-5 are valid) - stop parsing
                break;
            }
        }
    }
    None
}

