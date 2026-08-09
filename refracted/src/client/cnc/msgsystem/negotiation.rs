
/// Client-channel `ProtocolVersion` (matches `ClientChannelDescriptor` metadata `<Version>2</Version>`
/// + retail empty-auth ref). Required for SimuCloud handshake because
/// `DedicatedSimuCloudChannelDescriptor` borrows client metadata.
pub const PROTOCOL_VERSION: &[u8] = include_bytes!("frames/client_protocol_version.bin");
/// Full `ProtocolTypeIdMapping` SimpleFrame (typeId=0) for ServerHost negotiation reply.
pub const PROTOCOL_TYPE_ID_MAPPING: &[u8] = include_bytes!("frames/ptm.bin");
pub const LOAD_MAP_ALPHA_TUTORIAL: &[u8] = include_bytes!("frames/load_map.bin");
pub const SERVER_HELLO: &[u8] = include_bytes!("frames/server_hello.bin");
pub const SERVER_READY_TO_START: &[u8] = include_bytes!("frames/server_ready.bin");
pub const PROTOCOL_VERSION_TYPE_ID: u16 = 1;
/// Retail `ClientChannelDescriptor` / Prism ServerHost expected version.
pub const CLIENT_CHANNEL_PROTOCOL_VERSION: u32 = 2;
