//! **Compact Blaze framing**: `u16` payload length (BE) + 10 bytes (component, command, error,
//! qtype, packet_id) + payload.  
//! This matches the FireFrame-mode Blaze TCP loop in [`crate::blaze::server::server_module`]
//! (`handle_blaze_protocol_fireframe`), used wherever that listener shape is enabled.

use crate::blaze::protocol::{Fire2FramePacket, FireFramePacket, MessageType};
use crate::common::error::{BlazeError, BlazeResult};

#[inline]
pub fn pack(
    payload: &[u8],
    component: u16,
    command: u16,
    error: u16,
    qtype: u16,
    packet_id: u16,
) -> BlazeResult<Vec<u8>> {
    if payload.len() > u16::MAX as usize {
        return Err(BlazeError::InvalidPacket(
            "payload length does not fit u16 prefix".into(),
        ));
    }
    let sz = payload.len() as u16;
    let mut out = Vec::with_capacity(12 + payload.len());
    out.extend_from_slice(&sz.to_be_bytes());
    out.extend_from_slice(&component.to_be_bytes());
    out.extend_from_slice(&command.to_be_bytes());
    out.extend_from_slice(&error.to_be_bytes());
    out.extend_from_slice(&qtype.to_be_bytes());
    out.extend_from_slice(&packet_id.to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn qtype_from_fire2frame_msg_type(ty: MessageType) -> u16 {
    match ty {
        MessageType::Notification => 0x2000,
        _ => 0x1000,
    }
}

/// Encode Fire2Frame plaintext wire into compact framing (for listeners that use it).
pub fn from_fire2frame_plain_wire(wire: &[u8]) -> BlazeResult<Vec<u8>> {
    let pkt = Fire2FramePacket::from_bytes(wire)?;
    let payload = pkt.payload;
    let qtype = qtype_from_fire2frame_msg_type(pkt.header.msg_type);
    let packet_id = pkt.header.msg_num as u16;
    pack(
        payload.as_ref(),
        pkt.header.component,
        pkt.header.command,
        0,
        qtype,
        packet_id,
    )
}

fn qtype_from_fireframe_msg_type_byte(msg_ty: u8) -> u16 {
    if msg_ty == 2 {
        0x2000
    } else {
        0x1000
    }
}

/// Map [`FireFramePacket`] (4-byte size / 12-byte header wire format) into compact u16-length framing.
pub fn from_fireframe_library_packet(pkt: &FireFramePacket) -> BlazeResult<Vec<u8>> {
    let payload = pkt.payload.as_ref();
    if payload.len() > u16::MAX as usize {
        return Err(BlazeError::InvalidPacket(
            "FireFrame payload too large for compact u16 length prefix".into(),
        ));
    }
    let qtype = qtype_from_fireframe_msg_type_byte(pkt.header.msg_type);
    let packet_id = pkt.header.msg_num as u16;
    pack(
        payload,
        pkt.header.component,
        pkt.header.command,
        0,
        qtype,
        packet_id,
    )
}

/// Split a validated compact envelope into header fields and payload slice.
pub fn split_compact_envelope(data: &[u8]) -> Option<(u16, u16, u16, u16, u16, &[u8])> {
    if !is_complete_compact_envelope(data) {
        return None;
    }
    let psz = u16::from_be_bytes([data[0], data[1]]) as usize;
    let component = u16::from_be_bytes([data[2], data[3]]);
    let command = u16::from_be_bytes([data[4], data[5]]);
    let error = u16::from_be_bytes([data[6], data[7]]);
    let qtype = u16::from_be_bytes([data[8], data[9]]);
    let packet_id = u16::from_be_bytes([data[10], data[11]]);
    Some((component, command, error, qtype, packet_id, &data[12..12 + psz]))
}

/// True if `data` is at least one complete compact envelope and lengths are consistent.
pub fn is_complete_compact_envelope(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    let payload_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    data.len() == 12 + payload_len
}

/// Pick compact listener bytes: prefer structured parses so Fire2Frame / FireFrame library are not mistaken for raw compact.
pub fn normalize_for_compact_listener(wire: &[u8]) -> BlazeResult<Vec<u8>> {
    if Fire2FramePacket::from_bytes(wire).is_ok() {
        return from_fire2frame_plain_wire(wire);
    }
    if let Ok(pkt) = FireFramePacket::from_bytes(wire) {
        return from_fireframe_library_packet(&pkt);
    }
    if is_complete_compact_envelope(wire) {
        return Ok(wire.to_vec());
    }
    Err(BlazeError::InvalidPacket(
        "not Fire2Frame wire, FireFrame library wire, or compact envelope".into(),
    ))
}
