//! Fire2Frame Protocol Implementation
//! 
//! This module implements the Fire2Frame protocol (16-byte header format).
//! Not to be confused with the older FireFrame protocol (12-byte header).
//! 
//! Fire2Frame Header Structure (16 bytes):
//! - PayloadSize (4 bytes)
//! - MetadataSize (2 bytes)
//! - Component (2 bytes)
//! - Command (2 bytes)
//! - MsgNum (3 bytes)
//! - MsgType + UserIndex (1 byte)
//! - Options (1 byte)
//! - Reserved (1 byte)

use crate::common::error::{BlazeError, BlazeResult};
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};

/// Fire2Frame message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Message = 0,
    Reply = 1,
    Notification = 2,
    ErrorReply = 3,
    Ping = 4,
    PingReply = 5,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => MessageType::Message,
            1 => MessageType::Reply,
            2 => MessageType::Notification,
            3 => MessageType::ErrorReply,
            4 => MessageType::Ping,
            5 => MessageType::PingReply,
            _ => MessageType::Message, // Default fallback
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            MessageType::Message => "MESSAGE",
            MessageType::Reply => "REPLY",
            MessageType::Notification => "NOTIFICATION",
            MessageType::ErrorReply => "ERROR_REPLY",
            MessageType::Ping => "PING",
            MessageType::PingReply => "PING_REPLY",
        }
    }
}

/// Fire2Frame header structure (16 bytes)
/// Format: PayloadSize(4) + MetadataSize(2) + Component(2) + Command(2) + MsgNum(3) + MsgType+UserIndex(1) + Options(1) + Reserved(1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fire2FrameHeader {
    pub payload_size: u32,     // 4 bytes - payload size
    pub metadata_size: u16,    // 2 bytes - metadata size
    pub component: u16,        // 2 bytes - component ID
    pub command: u16,          // 2 bytes - command ID
    pub msg_num: u32,          // 3 bytes - message number (24 bits)
    pub msg_type: MessageType, // 3 bits - message type
    pub user_index: u32,       // 5 bits - user index
    pub options: u8,           // 1 byte - options
    pub reserved: u8,          // 1 byte - reserved
}

impl Fire2FrameHeader {
    pub const HEADER_SIZE: usize = 16;
    pub const MSGNUM_MASK: u32 = 0x00ffffff;

    /// Create a new Fire2Frame header for sending
    pub fn new_send(
        payload_size: u32,
        component: u16,
        command: u16,
        msg_num: u32,
        msg_type: MessageType,
    ) -> Self {
        Self::new_send_with_options(payload_size, component, command, msg_num, msg_type, 0)
    }

    /// Same as [`Self::new_send`], with explicit `options` (byte 14). Clients that read a 16-bit
    /// message kind at the same offset as the legacy `ushort` QType use `(header[13] << 8) | header[14]`
    /// — e.g. `0x4000` vs `0x4001` for notification subtype 0 vs 1.
    pub fn new_send_with_options(
        payload_size: u32,
        component: u16,
        command: u16,
        msg_num: u32,
        msg_type: MessageType,
        options: u8,
    ) -> Self {
        Self {
            payload_size,
            metadata_size: 0,
            component,
            command,
            msg_num: msg_num & Self::MSGNUM_MASK,
            msg_type,
            user_index: 0,
            options,
            reserved: 0,
        }
    }

    /// Parse Fire2Frame header from bytes
    pub fn from_bytes(data: &[u8]) -> BlazeResult<Self> {
        if data.len() < Self::HEADER_SIZE {
            return Err(BlazeError::InvalidPacket(format!(
                "Header too short: {} bytes",
                data.len()
            )));
        }


        let payload_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let metadata_size = u16::from_be_bytes([data[4], data[5]]);
        let component = u16::from_be_bytes([data[6], data[7]]);
        let command = u16::from_be_bytes([data[8], data[9]]);
        let msg_num = u32::from_be_bytes([0, data[10], data[11], data[12]]);
        let msg_type_user = data[13];
        let msg_type = MessageType::from_u8(msg_type_user >> 5);
        let user_index = (msg_type_user & 0x1f) as u32;
        let options = data[14];
        let reserved = data[15];

        Ok(Self {
            payload_size,
            metadata_size,
            component,
            command,
            msg_num,
            msg_type,
            user_index,
            options,
            reserved,
        })
    }

    /// Convert header to bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(Self::HEADER_SIZE);

        // Payload size (4 bytes)
        buf.put_u32(self.payload_size);

        // Metadata size (2 bytes)
        buf.put_u16(self.metadata_size);

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
        buf.put_u8((self.msg_type.to_u8() << 5) | (self.user_index as u8 & 0x1f));

        // Options
        buf.put_u8(self.options);

        // Reserved
        buf.put_u8(self.reserved);

        buf.freeze()
    }

    /// Get total size including header, metadata, and payload
    pub fn get_total_size(&self) -> usize {
        Self::HEADER_SIZE + self.metadata_size as usize + self.payload_size as usize
    }
}

/// Complete Fire2Frame packet
#[derive(Debug, Clone)]
pub struct Fire2FramePacket {
    pub header: Fire2FrameHeader,
    pub payload: Bytes,
}

impl Fire2FramePacket {
    /// Create a new Fire2Frame packet for sending
    pub fn new_send(
        component: u16,
        command: u16,
        msg_num: u32,
        msg_type: MessageType,
        payload: Bytes,
    ) -> Self {
        Self::new_send_with_options(component, command, msg_num, msg_type, payload, 0)
    }

    pub fn new_send_with_options(
        component: u16,
        command: u16,
        msg_num: u32,
        msg_type: MessageType,
        payload: Bytes,
        options: u8,
    ) -> Self {
        let header = Fire2FrameHeader::new_send_with_options(
            payload.len() as u32,
            component,
            command,
            msg_num,
            msg_type,
            options,
        );
        Self { header, payload }
    }

    /// Parse complete packet from bytes
    pub fn from_bytes(data: &[u8]) -> BlazeResult<Self> {
        if data.len() < Fire2FrameHeader::HEADER_SIZE {
            return Err(BlazeError::InvalidPacket(format!(
                "Packet too short: {} bytes",
                data.len()
            )));
        }

        let header = Fire2FrameHeader::from_bytes(&data[..Fire2FrameHeader::HEADER_SIZE])?;
        let total_size = header.get_total_size();

        if data.len() < total_size {
            return Err(BlazeError::InvalidPacket(format!(
                "Packet incomplete: expected {} bytes, got {}",
                total_size,
                data.len()
            )));
        }

        let payload_start = Fire2FrameHeader::HEADER_SIZE + header.metadata_size as usize;
        let payload = Bytes::copy_from_slice(&data[payload_start..total_size]);

        Ok(Self { header, payload })
    }

    /// Convert packet to bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.header.get_total_size());
        buf.extend_from_slice(&self.header.to_bytes());

        // Add metadata padding if needed
        for _ in 0..self.header.metadata_size {
            buf.put_u8(0);
        }

        buf.extend_from_slice(&self.payload);
        buf.freeze()
    }

    /// Get total packet size
    pub fn total_size(&self) -> usize {
        self.header.get_total_size()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NamedBlazeCommand {
    pub component: u16,
    pub command: u16,
    pub name: &'static str,
}

/// Named commands (same table as listener / logging). Used by toolkit presets and `get_command_name`.
pub const NAMED_BLAZE_COMMANDS: &[NamedBlazeCommand] = &[
    NamedBlazeCommand { component: 0x0009, command: 0x02, name: "Util.ping" },
    NamedBlazeCommand { component: 0x0009, command: 0x07, name: "Util.preAuth" },
    NamedBlazeCommand { component: 0x0009, command: 0x01, name: "Util.fetchClientConfig" },
    NamedBlazeCommand { component: 0x0009, command: 0x08, name: "Util.postAuth" },
    NamedBlazeCommand { component: 0x0009, command: 0x05, name: "Util.getTelemetryServer" },
    NamedBlazeCommand { component: 0x0001, command: 0x0a, name: "Authentication.login" },
    NamedBlazeCommand { component: 0x0001, command: 0x46, name: "Authentication.logout" },
    NamedBlazeCommand { component: 0x7802, command: 0x14, name: "UserSessions.updateNetworkInfo" },
    NamedBlazeCommand { component: 0x7802, command: 0x0c, name: "UserSessions.lookupUser" },
    NamedBlazeCommand { component: 0x7802, command: 0x08, name: "UserSessions.updateHardwareFlags" },
    NamedBlazeCommand { component: 0x7802, command: 0x3c, name: "UserSessions.setClientState" },
    NamedBlazeCommand { component: 0x0004, command: 0x03, name: "GameManager.advanceGameState" },
    NamedBlazeCommand { component: 0x0004, command: 0x11, name: "GameManager.returnDedicatedServerToPool" },
    NamedBlazeCommand { component: 0, command: 0, name: "Disconnect" },
];

/// Command name lookup for debugging — same registry as toolkit presets.
pub fn get_command_name(component: u16, command: u16) -> Option<&'static str> {
    NAMED_BLAZE_COMMANDS
        .iter()
        .find(|n| n.component == component && n.command == command)
        .map(|n| n.name)
}

