//! Client-channel `ProtocolVersion` (SimuCloud handshake).
//!
//! Wire shape matches retail `ClientNetWrapper`: version 2 + empty auth token ref
//! (not null)

use super::wire::{SimpleFrame, WireWriter};

pub const PROTOCOL_VERSION_TYPE_ID: u16 = 1;
/// Retail `ClientChannelDescriptor` / Prism ServerHost expected version.
pub const CLIENT_CHANNEL_PROTOCOL_VERSION: u32 = 2;

/// Client-channel `ProtocolVersion` (matches `ClientChannelDescriptor` metadata
/// `<Version>2</Version>` + retail empty-auth ref). Required for SimuCloud handshake
/// because `DedicatedSimuCloudChannelDescriptor` borrows client metadata.
pub fn encode_protocol_version() -> SimpleFrame {
    let mut w = WireWriter::new();
    w.write_u32(CLIENT_CHANNEL_PROTOCOL_VERSION);
    // Present (non-null) empty `byte[]` auth token: marker + u16 length 0.
    // Retail payloadLen=7 (`02 00 00 00 01 00 00`).
    w.write_u8(1);
    w.write_u16(0);
    SimpleFrame {
        type_id: PROTOCOL_VERSION_TYPE_ID,
        payload: w.into_bytes(),
    }
}

pub fn encode_protocol_version_frame() -> Vec<u8> {
    encode_protocol_version().write()
}

#[cfg(test)]
pub fn frames_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/client/cnc/msgsystem/frames")
}

/// Load a locally generated dump fixture. Returns `None` when the file is absent
/// so dump-parity tests can skip on a clean clone.
#[cfg(test)]
pub fn read_frame_dump(file_name: &str) -> Option<Vec<u8>> {
    std::fs::read(frames_dir().join(file_name)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_is_retail_client_shape() {
        let frame = encode_protocol_version();
        assert_eq!(frame.type_id, PROTOCOL_VERSION_TYPE_ID);
        assert_eq!(frame.payload.len(), 7);
        assert_eq!(
            u32::from_le_bytes(frame.payload[0..4].try_into().unwrap()),
            CLIENT_CHANNEL_PROTOCOL_VERSION
        );
        assert_eq!(&frame.payload[4..], &[1, 0, 0]);
        assert_eq!(encode_protocol_version_frame().len(), 13);
    }

    #[test]
    fn protocol_version_matches_retail_dump_when_present() {
        let Some(dump) = read_frame_dump("client_protocol_version.bin") else {
            eprintln!("skip dump-parity: frames/client_protocol_version.bin not present");
            return;
        };
        assert_eq!(encode_protocol_version_frame(), dump);
    }
}
