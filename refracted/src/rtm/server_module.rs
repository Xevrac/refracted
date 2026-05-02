use crate::common::error::BlazeResult;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info};

/// RTM Protocol Server - Handles Real-Time Messaging WebSocket communications
pub struct RtmProtocolServer {
    host: String,
}

impl RtmProtocolServer {
    /// Create new RTM protocol server
    pub fn new(host: String) -> Self {
        Self { host }
    }

    pub fn ports_from_config(p: &crate::common::game::ServicePorts) -> Vec<(u16, String)> {
        vec![(p.rtm, "RTM WebSocket".into())]
    }

    /// Start RTM protocol server
    pub async fn start_rtm_server(
        &self,
        ports: &crate::common::game::ServicePorts,
    ) -> BlazeResult<()> {
        let port = ports.rtm;
        let host = self.host.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_rtm_server(host, port).await {
                error!("RTM server error: {}", e);
            }
        });

        Ok(())
    }

    /// Run RTM server
    async fn run_rtm_server(host: String, port: u16) -> BlazeResult<()> {
        let addr = format!("{}:{}", host, port);
        let socket = tokio::net::TcpSocket::new_v4()
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        #[cfg(windows)]
        socket.set_reuseaddr(true)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        socket.bind(addr.parse()
            .map_err(|e| crate::common::error::BlazeError::InvalidPacket(format!("Invalid address: {}", e)))?)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        let listener = socket.listen(128)
            .map_err(|e| crate::common::error::BlazeError::Io(e))?;
        if !crate::common::startup_progress::is_startup_in_progress() {
            info!("RTM server listening on {}", addr);
        }

        loop {
            let (stream, addr) = listener.accept().await?;
            crate::debug_println!("\x1b[38;2;150;255;200m[RTM]\x1b[0m New WebSocket connection accepted on port {} from {}", port, addr);
            tokio::spawn(async move {
                if let Err(e) = Self::handle_rtm_connection(stream, addr).await {
                    error!("RTM connection error: {}", e);
                }
            });
        }
    }

    /// Handle RTM connection
    async fn handle_rtm_connection(mut stream: TcpStream, addr: SocketAddr) -> BlazeResult<()> {
        info!("RTM connection from {}", addr);
        crate::debug_println!("\x1b[38;2;150;255;200m[RTM]\x1b[0m handle_rtm_connection entered for {}", addr);

        // Read RTM request
        crate::debug_println!("\x1b[38;2;150;255;200m[RTM]\x1b[0m Waiting to read WebSocket handshake from {}", addr);
        let mut buffer = vec![0; 4096];
        let n = stream.read(&mut buffer).await?;
        crate::debug_println!("\x1b[38;2;150;255;200m[RTM]\x1b[0m Read {} bytes from {}", n, addr);
        let request = String::from_utf8_lossy(&buffer[..n]);
        crate::debug_println!("\x1b[38;2;150;255;200m[RTM]\x1b[0m Request preview: {}", request.lines().next().unwrap_or(""));

        // Simple RTM handler - send a basic response
        crate::debug_println!("\x1b[38;2;150;255;200m[RTM]\x1b[0m Sending RTM response");
        let response = "RTM Response - WebSocket Ready";
        stream.write_all(response.as_bytes()).await?;
        crate::debug_println!("\x1b[38;2;150;255;200m[RTM]\x1b[0m RTM response sent successfully");

        Ok(())
    }
}
