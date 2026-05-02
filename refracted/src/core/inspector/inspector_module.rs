use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Traffic inspection mode: local emulator captures vs proxied upstream capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorMode {
    /// Capture and inspect traffic handled by Refracted itself.
    Emulator,
    /// Proxy the game client toward upstream services and capture in the middle (research tooling).
    Research,
}

impl InspectorMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            InspectorMode::Emulator => "Emulator",
            InspectorMode::Research => "Research (proxy)",
        }
    }
}

/// Inspector type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorType {
    Blaze,
    Grpc,
    Http,
    Lsx,
}

/// Blaze packet capture structures
#[derive(Debug, Clone)]
pub struct CapturedPacket {
    pub timestamp: f64,
    pub direction: PacketDirection,
    pub component: u16,
    pub command: u16,
    pub msg_num: u32,
    pub msg_type: String,
    pub payload_size: usize,
    pub payload: Vec<u8>,
    pub raw_packet: Vec<u8>, // Full packet including header
    pub command_name: Option<String>,
    pub metadata_size: u16, // Fire2Frame metadata size (for proper payload offset)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketDirection {
    ClientToBlaze,
    BlazeToClient,
}

impl PacketDirection {
    pub fn to_string(&self) -> &'static str {
        match self {
            PacketDirection::ClientToBlaze => "Client->Blaze",
            PacketDirection::BlazeToClient => "Blaze->Client",
        }
    }
}

pub type PacketBuffer = Arc<Mutex<Vec<CapturedPacket>>>;

impl InspectorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InspectorType::Blaze => "Blaze",
            InspectorType::Grpc => "gRPC",
            InspectorType::Http => "HTTP",
            InspectorType::Lsx => "LSX",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolkitWorkbenchMode {
    #[default]
    Listen,
    Make,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolkitMakeTab {
    #[default]
    Blaze,
    Grpc,
}

/// Captured gRPC request/response
#[derive(Debug, Clone)]
pub struct CapturedGrpc {
    pub timestamp: f64,
    pub direction: GrpcDirection,
    pub method: String,
    pub path: String,
    pub host: String,
    pub headers: Vec<(String, String)>,
    pub body_size: usize,
    pub body: Vec<u8>,
    pub protobuf_data: Option<Vec<u8>>, // Legacy: first decompressed protobuf message (if peeling worked)
    /// One protobuf payload per peeled gRPC data frame (`[flag][BE len][data]`…) on HTTP/2 DATA.
    pub protobuf_chunks: Vec<Vec<u8>>,
    pub is_compressed: bool,
    pub grpc_status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcDirection {
    ClientToServer,
    ServerToClient,
}

impl GrpcDirection {
    pub fn to_string(&self) -> &'static str {
        match self {
            GrpcDirection::ClientToServer => "Client->Server",
            GrpcDirection::ServerToClient => "Server->Client",
        }
    }
}

/// Captured HTTP request/response
#[derive(Debug, Clone)]
pub struct CapturedHttp {
    pub timestamp: f64,
    pub direction: HttpDirection,
    pub method: String,
    pub path: String,
    pub host: String,
    pub headers: Vec<(String, String)>,
    pub body_size: usize,
    pub body: Vec<u8>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpDirection {
    ClientToServer,
    ServerToClient,
}

impl HttpDirection {
    pub fn to_string(&self) -> &'static str {
        match self {
            HttpDirection::ClientToServer => "Client->Server",
            HttpDirection::ServerToClient => "Server->Client",
        }
    }
}

/// Captured LSX request/response
#[derive(Debug, Clone)]
pub struct CapturedLsx {
    pub timestamp: f64,
    pub direction: LsxDirection,
    pub method: String,
    pub path: String,
    pub host: String,
    pub headers: Vec<(String, String)>,
    pub body_size: usize,
    pub body: Vec<u8>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LsxDirection {
    ClientToServer,
    ServerToClient,
}

impl LsxDirection {
    pub fn to_string(&self) -> &'static str {
        match self {
            LsxDirection::ClientToServer => "Client->Server",
            LsxDirection::ServerToClient => "Server->Client",
        }
    }
}

pub type GrpcBuffer = Arc<Mutex<Vec<CapturedGrpc>>>;
pub type HttpBuffer = Arc<Mutex<Vec<CapturedHttp>>>;
pub type LsxBuffer = Arc<Mutex<Vec<CapturedLsx>>>;

// Global buffers
static GLOBAL_PACKET_BUFFER: parking_lot::Mutex<Option<PacketBuffer>> = parking_lot::const_mutex(None);
static GLOBAL_GRPC_BUFFER: parking_lot::Mutex<Option<GrpcBuffer>> = parking_lot::const_mutex(None);
static GLOBAL_HTTP_BUFFER: parking_lot::Mutex<Option<HttpBuffer>> = parking_lot::const_mutex(None);
static GLOBAL_LSX_BUFFER: parking_lot::Mutex<Option<LsxBuffer>> = parking_lot::const_mutex(None);

/// Initialize global packet buffer
pub fn init_global_packet_buffer(buffer: PacketBuffer) {
    *GLOBAL_PACKET_BUFFER.lock() = Some(buffer);
}

/// Initialize global gRPC buffer
pub fn init_global_grpc_buffer(buffer: GrpcBuffer) {
    *GLOBAL_GRPC_BUFFER.lock() = Some(buffer);
}

/// Initialize global HTTP buffer
pub fn init_global_http_buffer(buffer: HttpBuffer) {
    *GLOBAL_HTTP_BUFFER.lock() = Some(buffer);
}

/// Initialize global LSX buffer
pub fn init_global_lsx_buffer(buffer: LsxBuffer) {
    *GLOBAL_LSX_BUFFER.lock() = Some(buffer);
}

/// Get global packet buffer
pub fn get_global_packet_buffer() -> Option<PacketBuffer> {
    GLOBAL_PACKET_BUFFER.lock().clone()
}

/// Get global gRPC buffer
pub fn get_global_grpc_buffer() -> Option<GrpcBuffer> {
    GLOBAL_GRPC_BUFFER.lock().clone()
}

/// Get global HTTP buffer
pub fn get_global_http_buffer() -> Option<HttpBuffer> {
    GLOBAL_HTTP_BUFFER.lock().clone()
}

/// Get global LSX buffer
pub fn get_global_lsx_buffer() -> Option<LsxBuffer> {
    GLOBAL_LSX_BUFFER.lock().clone()
}

/// Capture a Blaze packet
pub fn capture_packet(packet: CapturedPacket) {
    if let Some(buffer) = get_global_packet_buffer() {
        let mut buf = buffer.lock();
        buf.push(packet);
        let len = buf.len();
        if len > 1000 {
            let remove_count = len - 1000;
            buf.drain(0..remove_count);
        }
    }
}

/// Capture a gRPC request/response
pub fn capture_grpc(grpc: CapturedGrpc) {
    if let Some(buffer) = get_global_grpc_buffer() {
        let mut buf = buffer.lock();
        buf.push(grpc);
        
        // Keep only last 1000 entries
        let len = buf.len();
        if len > 1000 {
            buf.drain(0..len - 1000);
        }
    }
}

/// Capture an HTTP request/response
pub fn capture_http(http: CapturedHttp) {
    if let Some(buffer) = get_global_http_buffer() {
        let mut buf = buffer.lock();
        buf.push(http);
        
        // Keep only last 1000 entries
        let len = buf.len();
        if len > 1000 {
            buf.drain(0..len - 1000);
        }
    }
}

/// Capture an LSX request/response
pub fn capture_lsx(lsx: CapturedLsx) {
    if let Some(buffer) = get_global_lsx_buffer() {
        let mut buf = buffer.lock();
        buf.push(lsx);
        
        // Keep only last 1000 entries
        let len = buf.len();
        if len > 1000 {
            buf.drain(0..len - 1000);
        }
    }
}

/// Proxy configuration for research mode
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub http_listen_port: u16,
    pub https_listen_port: u16,
    pub grpc_listen_port: u16,
    pub blaze_listen_port: u16,
    pub lsx_listen_port: u16,
    pub target_host: String,
    pub target_http_port: u16,
    pub target_https_port: u16,
    pub target_grpc_port: u16,
    pub target_blaze_port: u16,
    pub target_lsx_port: u16,
    pub enable_http: bool,
    pub enable_https: bool,
    pub enable_grpc: bool,
    pub enable_blaze: bool,
    pub enable_lsx: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            http_listen_port: 80,      // Match emulator HTTP port
            https_listen_port: 443,    // Match emulator HTTPS port
            grpc_listen_port: 443,     // Match emulator gRPC port
            blaze_listen_port: 10042,  // Match emulator Blaze TLS port
            lsx_listen_port: 3216,     // Match emulator LSX port
            target_host: "localhost".to_string(),
            target_http_port: 80,
            target_https_port: 443,
            target_grpc_port: 443,
            target_blaze_port: 10042,
            target_lsx_port: 3216,
            enable_http: true,
            enable_https: true,
            enable_grpc: true,
            enable_blaze: true,
            enable_lsx: true,
        }
    }
}

/// Proxy server state
pub struct ProxyState {
    pub running: Arc<AtomicBool>,
    pub config: Arc<Mutex<ProxyConfig>>,
}

impl ProxyState {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(ProxyConfig::default())),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

// Global proxy state
static GLOBAL_PROXY_STATE: parking_lot::Mutex<Option<Arc<ProxyState>>> = parking_lot::const_mutex(None);

/// Initialize global proxy state
pub fn init_global_proxy_state(state: Arc<ProxyState>) {
    *GLOBAL_PROXY_STATE.lock() = Some(state);
}

/// Get global proxy state
pub fn get_global_proxy_state() -> Option<Arc<ProxyState>> {
    GLOBAL_PROXY_STATE.lock().clone()
}

/// Format bytes as hex dump (shared utility)
pub fn format_hex_dump(data: &[u8], max_bytes: usize) -> String {
    let data = if data.len() > max_bytes {
        &data[..max_bytes]
    } else {
        data
    };
    
    let mut result = String::new();
    let mut ascii = String::new();
    
    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = i * 16;
        result.push_str(&format!("{:08x}  ", offset));
        
        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                result.push(' ');
            }
            result.push_str(&format!("{:02x} ", byte));
            
            // ASCII representation
            let ch = if *byte >= 32 && *byte < 127 {
                *byte as char
            } else {
                '.'
            };
            ascii.push(ch);
        }
        
        // Pad hex if line is incomplete
        if chunk.len() < 16 {
            let padding = 16 - chunk.len();
            let hex_padding = if chunk.len() < 8 {
                (8 - chunk.len()) * 3 + 1 + padding * 3
            } else {
                padding * 3
            };
            result.push_str(&" ".repeat(hex_padding));
        }
        
        result.push_str(" |");
        result.push_str(&ascii);
        result.push_str("|\n");
        ascii.clear();
    }
    
    if data.len() > max_bytes {
        result.push_str(&format!("... ({} more bytes)\n", data.len() - max_bytes));
    }
    
    result
}

