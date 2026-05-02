use bytes::Bytes;

use crate::common::error::BlazeResult;

use super::{Fire2FrameHeader, Fire2FramePacket, MessageType};

#[derive(Debug, Clone, Copy)]
pub enum RedirectorWire {
    FireFrame { packet_id: u16 },
    Fire2Frame { msg_num: u32 },
}

fn protocol_is_fireframe(protocol: &str) -> bool {
    protocol.eq_ignore_ascii_case("fireframe")
}

pub fn find_get_server_instance(protocol: &str, data: &[u8]) -> Option<RedirectorWire> {
    if protocol_is_fireframe(protocol) {
        return find_get_server_instance_fireframe(data);
    }
    find_get_server_instance_fire2(data)
}

fn find_get_server_instance_fireframe(data: &[u8]) -> Option<RedirectorWire> {
    if data.len() < 6 {
        return None;
    }
    if data.len() >= 6 {
        let component = u16::from_be_bytes([data[2], data[3]]);
        let command = u16::from_be_bytes([data[4], data[5]]);
        if component == 0x0005 && command == 0x0001 {
            let packet_id = if data.len() >= 12 {
                u16::from_be_bytes([data[10], data[11]])
            } else {
                0
            };
            return Some(RedirectorWire::FireFrame { packet_id });
        }
    }
    None
}

fn find_get_server_instance_fire2(data: &[u8]) -> Option<RedirectorWire> {
    if data.len() < Fire2FrameHeader::HEADER_SIZE {
        return None;
    }
    for offset in 0..=data.len() - Fire2FrameHeader::HEADER_SIZE {
        let header = match Fire2FrameHeader::from_bytes(
            &data[offset..offset + Fire2FrameHeader::HEADER_SIZE],
        ) {
            Ok(h) => h,
            Err(_) => continue,
        };
        if header.payload_size > 65535 {
            continue;
        }
        if header.component == 0x0005 && header.command == 0x0001 {
            return Some(RedirectorWire::Fire2Frame {
                msg_num: header.msg_num,
            });
        }
    }
    None
}

pub fn build_get_server_instance_reply(frame: RedirectorWire, payload: Bytes) -> BlazeResult<Vec<u8>> {
    let bytes = match frame {
        RedirectorWire::FireFrame { packet_id } => {
            let mut out = Vec::with_capacity(12 + payload.len());
            let size = payload.len() as u16;
            out.extend_from_slice(&size.to_be_bytes());
            out.extend_from_slice(&0x0005u16.to_be_bytes());
            out.extend_from_slice(&0x0001u16.to_be_bytes());
            out.extend_from_slice(&0u16.to_be_bytes()); // error
            out.extend_from_slice(&0x1000u16.to_be_bytes()); // reply qtype
            out.extend_from_slice(&packet_id.to_be_bytes()); // packet id
            out.extend_from_slice(&payload);
            out
        }
        RedirectorWire::Fire2Frame { msg_num } => Fire2FramePacket::new_send(
            0x0005,
            0x0001,
            msg_num,
            MessageType::Reply,
            payload,
        )
        .to_bytes()
        .to_vec(),
    };
    Ok(bytes)
}

