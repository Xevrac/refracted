//! FireFrame Protocol Implementation
//! 
//! This module implements the FireFrame protocol (12-byte header format).
//! This is the older protocol format, not to be confused with Fire2Frame (16-byte header).

use crate::common::error::{BlazeError, BlazeResult};
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

/// FireFrame header structure (12 bytes)
/// Format: PayloadSize(4) + Component(2) + Command(2) + MsgNum(3) + MsgType+UserIndex(1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireFrameHeader {
    pub payload_size: u32,     // 4 bytes - payload size
    pub component: u16,        // 2 bytes - component ID
    pub command: u16,          // 2 bytes - command ID
    pub msg_num: u32,          // 3 bytes - message number (24 bits)
    pub msg_type: u8,          // 3 bits - message type
    pub user_index: u8,        // 5 bits - user index
}

impl FireFrameHeader {
    pub const HEADER_SIZE: usize = 12;
    pub const MSGNUM_MASK: u32 = 0x00ffffff;

    /// Create a new FireFrame header for sending
    pub fn new_send(
        payload_size: u32,
        component: u16,
        command: u16,
        msg_num: u32,
        msg_type: u8,
    ) -> Self {
        Self {
            payload_size,
            component,
            command,
            msg_num: msg_num & Self::MSGNUM_MASK,
            msg_type: msg_type & 0x07, // 3 bits
            user_index: 0,
        }
    }

    /// Parse FireFrame header from bytes
    pub fn from_bytes(data: &[u8]) -> BlazeResult<Self> {
        if data.len() < Self::HEADER_SIZE {
            return Err(BlazeError::InvalidPacket(format!(
                "Header too short: {} bytes",
                data.len()
            )));
        }

        let payload_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let component = u16::from_be_bytes([data[4], data[5]]);
        let command = u16::from_be_bytes([data[6], data[7]]);
        let msg_num = u32::from_be_bytes([0, data[8], data[9], data[10]]);
        let msg_type_user = data[11];
        let msg_type = msg_type_user >> 5;
        let user_index = msg_type_user & 0x1f;

        Ok(Self {
            payload_size,
            component,
            command,
            msg_num,
            msg_type,
            user_index,
        })
    }

    /// Convert header to bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(Self::HEADER_SIZE);

        // Payload size (4 bytes)
        buf.put_u32(self.payload_size);

        // Component (2 bytes)
        buf.put_u16(self.component);

        // Command (2 bytes)
        buf.put_u16(self.command);

        // Message number (3 bytes)
        let msg_num_bytes = self.msg_num.to_be_bytes();
        buf.put_u8(msg_num_bytes[1]);
        buf.put_u8(msg_num_bytes[2]);
        buf.put_u8(msg_num_bytes[3]);

        // Message type (3 bits) + User index (5 bits)
        buf.put_u8((self.msg_type << 5) | (self.user_index & 0x1f));

        buf.freeze()
    }

    /// Get total size including header and payload
    pub fn get_total_size(&self) -> usize {
        Self::HEADER_SIZE + self.payload_size as usize
    }
}

/// Complete FireFrame packet
#[derive(Debug, Clone)]
pub struct FireFramePacket {
    pub header: FireFrameHeader,
    pub payload: Bytes,
}

impl FireFramePacket {
    /// Create a new FireFrame packet for sending
    pub fn new_send(
        component: u16,
        command: u16,
        msg_num: u32,
        msg_type: u8,
        payload: Bytes,
    ) -> Self {
        let header =
            FireFrameHeader::new_send(payload.len() as u32, component, command, msg_num, msg_type);
        Self { header, payload }
    }

    /// Parse complete packet from bytes
    pub fn from_bytes(data: &[u8]) -> BlazeResult<Self> {
        if data.len() < FireFrameHeader::HEADER_SIZE {
            return Err(BlazeError::InvalidPacket(format!(
                "Packet too short: {} bytes",
                data.len()
            )));
        }

        let header = FireFrameHeader::from_bytes(&data[..FireFrameHeader::HEADER_SIZE])?;
        let total_size = header.get_total_size();

        if data.len() < total_size {
            return Err(BlazeError::InvalidPacket(format!(
                "Packet incomplete: expected {} bytes, got {}",
                total_size,
                data.len()
            )));
        }

        let payload = Bytes::copy_from_slice(&data[FireFrameHeader::HEADER_SIZE..total_size]);

        Ok(Self { header, payload })
    }

    /// Convert packet to bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.header.get_total_size());
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.payload);
        buf.freeze()
    }

    /// Get total packet size
    pub fn total_size(&self) -> usize {
        self.header.get_total_size()
    }
}
