use crate::common::error::{BlazeError, BlazeResult};
use crate::blaze::server::BlazeProtocolServer;
use crate::web::server::WebProtocolServer;
use crate::lsx::LsxServer;
use crate::qos::QosProtocolServer;
use crate::rtm::RtmProtocolServer;
use crate::session::get_user_session;
use chrono::{Datelike, Utc};
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub struct BlazeServer {
    host: String,
    running: Arc<AtomicBool>,
    pub shutdown_tx: broadcast::Sender<()>,
    ssl_context: Option<Arc<ServerConfig>>,
    // Auth data from packet capture (for future Blaze protocol implementation)
    #[allow(dead_code)]
    auth_token: String,
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    user_id: u64,
    #[allow(dead_code)]
    persona_id: u64,
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    player_name: String,
    #[allow(dead_code)]
    steam_id: String,
    // Server instances
    #[allow(dead_code)]
    servers: HashMap<String, Arc<ServerConfig>>,
}

impl BlazeServer {
    /// Create new Blaze Server instance
    pub async fn new(host: String) -> BlazeResult<Self> {
        let ssl_context = Self::create_tls_config().ok().map(Arc::new);
        let (shutdown_tx, _) = broadcast::channel(16);

        // Get user session data from LSX authentication pipeline
        // If not authenticated yet, defaults will be used
        let session = get_user_session();
        
        Ok(Self {
            host,
            running: Arc::new(AtomicBool::new(false)),
            shutdown_tx,
            ssl_context,
            // Auth data from LSX authentication session
            auth_token: session.jwt_token.clone().unwrap_or_default(),
            session_id: format!("{:x}_{}", session.user_id, session.persona_id),
            user_id: session.user_id,
            persona_id: session.persona_id,
            username: session.display_name.clone(),
            player_name: session.display_name.clone(),
            steam_id: format!("76561198{:014}", session.user_id), // Convert user_id to Steam ID format
            servers: HashMap::new(),
        })
    }

    /// Create TLS configuration
    fn create_tls_config() -> BlazeResult<ServerConfig> {
        info!("Generating new self-signed certificates");
        let (cert, key) = Self::generate_self_signed_cert()?;

        if let Err(e) = Self::save_certificates(&cert, &key) {
            warn!("Failed to save certificates: {}", e);
        }

        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .map_err(|e| BlazeError::Tls(e))?;

        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Ok(config)
    }

    /// Generate self-signed certificate
    fn generate_self_signed_cert() -> BlazeResult<(Certificate, PrivateKey)> {
        use rcgen::{
            Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, KeyPair,
            KeyUsagePurpose, SanType, PKCS_ECDSA_P256_SHA256,
        };

        let mut params = CertificateParams::default();

        // TLS server end-entity: Schannel / newer clients often require explicit EKU (serverAuth).
        params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

        // Set certificate subject
        params
            .distinguished_name
            .push(DnType::CommonName, "*.ea.com");

        // Set subject alternative names to cover all EA endpoints we emulate
        let mut subject_alt_names = vec![
            SanType::DnsName("localhost".to_string()),
            SanType::IpAddress("127.0.0.1".parse().unwrap()),
            SanType::DnsName("*.ea.com".to_string()),
            SanType::DnsName("*.grpc.ea.com".to_string()),
            SanType::DnsName("*.social.ea.com".to_string()),
            SanType::DnsName("*.data.ea.com".to_string()),
            SanType::DnsName("*.tnt-ea.com".to_string()),
            SanType::DnsName("*.blazeredirector.ea.com".to_string()),
            SanType::DnsName("*.gosredirector.ea.com".to_string()),
            SanType::DnsName("*.ops.dice.se".to_string()),
            SanType::DnsName("*.dice.se".to_string()),
            SanType::DnsName("gcs.ea.com".to_string()),
            SanType::DnsName("accounts.grpc.ea.com".to_string()),
            SanType::DnsName("gateway.grpc.ea.com".to_string()),
            SanType::DnsName("api.k.social.ea.com".to_string()),
            SanType::DnsName("freeform-river.data.ea.com".to_string()),
            SanType::DnsName("update.layer.ea.com".to_string()),
            SanType::DnsName("collector.errors.ea.com".to_string()),
            SanType::DnsName("spring25.client.blazeredirector.ea.com".to_string()),
            SanType::DnsName("spring18.gosredirector.ea.com".to_string()),
            SanType::DnsName("ext-127-0-0-1.blaze.ea.com".to_string()),
            SanType::DnsName("rtm.tnt-ea.com".to_string()),
            SanType::DnsName("pn.tnt-ea.com".to_string()),
            SanType::DnsName("stats.gameservices.ea.com".to_string()),
            SanType::DnsName("leaderboards.gameservices.ea.com".to_string()),
            SanType::DnsName("leaderboards-api-ext.leaderboards.ea.com".to_string()),
            SanType::DnsName("qoscoordinator.gameservices.ea.com".to_string()),
            SanType::DnsName("tos.ea.com".to_string()),
            SanType::DnsName("reports.tools.gos.ea.com".to_string()),
            SanType::DnsName("tools.gos.ea.com".to_string()),
            SanType::DnsName("*.tools.gos.ea.com".to_string()),
        ];
        subject_alt_names.dedup();
        params.subject_alt_names = subject_alt_names;

        // Validity anchored to launch: small skew allowance + horizon from current calendar year (no fixed expiry).
        const HORIZON_YEARS: i32 = 30;
        let today = Utc::now().date_naive();
        let start = today - chrono::Duration::days(1);
        let not_after_year = (today.year() + HORIZON_YEARS).min(9999);
        params.not_before = rcgen::date_time_ymd(start.year(), start.month() as u8, start.day() as u8);
        params.not_after = rcgen::date_time_ymd(not_after_year, 12, 31);

        // Generate key pair
        let key_pair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256)?;
        params.key_pair = Some(key_pair);

        // Create certificate
        let cert = Certificate::from_params(params)?;

        // Convert to rustls format
        let cert_der = cert.serialize_der()?;
        let key_der = cert.serialize_private_key_der();

        Ok((Certificate(cert_der), PrivateKey(key_der)))
    }

    /// Save certificates to files
    fn save_certificates(cert: &Certificate, key: &PrivateKey) -> BlazeResult<()> {
        use std::fs;

        // Convert DER to PEM format with proper line wrapping
        use base64::{engine::general_purpose, Engine as _};
        let cert_b64 = general_purpose::STANDARD.encode(&cert.0);
        let cert_pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
            Self::wrap_base64(&cert_b64)
        );

        let key_b64 = general_purpose::STANDARD.encode(&key.0);
        let key_pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
            Self::wrap_base64(&key_b64)
        );

        fs::write("cert.pem", cert_pem)?;
        fs::write("key.pem", key_pem)?;

        info!("Certificates saved to cert.pem and key.pem");
        Ok(())
    }

    /// Wrap base64 string to 64 characters per line
    fn wrap_base64(input: &str) -> String {
        input
            .chars()
            .collect::<Vec<_>>()
            .chunks(64)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if a port is available
    pub fn check_port_available(host: &str, port: u16) -> bool {
        use std::net::TcpListener;
        match TcpListener::bind(format!("{}:{}", host, port)) {
            Ok(listener) => {
                // Port is available - drop the listener immediately
                drop(listener);
                true
            }
            Err(_e) => {
                // Port is in use or bind failed for another reason
                // On Windows, common errors:
                // - ErrorKind::AddrInUse (10048) - port already in use
                // - ErrorKind::PermissionDenied - need admin for ports < 1024
                false
            }
        }
    }

    /// Ports required for the **currently selected game** (`games.json` → `service_ports` + enabled services).
    pub fn get_required_ports() -> Vec<(u16, String)> {
        crate::client::aggregated_required_ports()
    }

    /// Check all required ports and return list of ports in use
    pub fn check_all_ports(host: &str) -> Vec<(u16, String)> {
        let mut ports_in_use = Vec::new();
        for (port, name) in Self::get_required_ports() {
            if !Self::check_port_available(host, port) {
                ports_in_use.push((port, name));
            }
        }
        ports_in_use
    }

    /// Start the Blaze Server with all services
    pub async fn start_emulator(&mut self) -> BlazeResult<()> {
        use crate::common::startup_progress::{finish_startup_progress, log_startup_progress, start_startup_progress};
        
        self.running.store(true, Ordering::SeqCst);
        
        // Start startup progress mode (suppresses intermediate messages)
        start_startup_progress();

        // Setup signal handling for graceful shutdown
        self.setup_signal_handlers().await?;

        // Initialize and start all protocol servers with progress updates
        log_startup_progress("Starting Refracted service layers...");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let flags = crate::client::ServiceFlags::from_current_game();
        let ports = crate::common::game::current_service_ports();

        if flags.web {
            log_startup_progress("Starting Web server..");
            self.start_web_services(&ports).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        if flags.blaze {
            log_startup_progress("Starting Blaze..");
            self.start_blaze_services(&ports).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        if flags.lsx || flags.qos || flags.rtm {
            log_startup_progress("Starting services..");
            self.start_specialized_services(&flags, &ports).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        finish_startup_progress();
        
        info!("Services have started successfully.");
        info!("Ready to accept client connections..");

        // Wait for shutdown signal
        self.wait_for_shutdown().await;

        // Graceful shutdown
        self.shutdown().await;

        Ok(())
    }

    /// Setup signal handlers for graceful shutdown
    async fn setup_signal_handlers(&self) -> BlazeResult<()> {
        let shutdown_tx = self.shutdown_tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};

                let mut sigterm = signal(SignalKind::terminate()).unwrap();
                let mut sigint = signal(SignalKind::interrupt()).unwrap();

                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM, initiating graceful shutdown...");
                    }
                    _ = sigint.recv() => {
                        info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
                    }
                }
            }

            #[cfg(windows)]
            {
                use tokio::signal::windows::ctrl_c;

                match ctrl_c() {
                    Ok(mut ctrl_c) => {
                        ctrl_c.recv().await;
                        info!("Received Ctrl+C, initiating graceful shutdown...");
                    }
                    Err(e) => {
                        error!("Failed to setup Ctrl+C handler: {}", e);
                        return;
                    }
                }
            }

            // Signal shutdown
            running.store(false, Ordering::SeqCst);
            let _ = shutdown_tx.send(());
        });

        Ok(())
    }

    /// Wait for shutdown signal
    async fn wait_for_shutdown(&self) {
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Wait for shutdown signal
        let _ = shutdown_rx.recv().await;
    }

    /// Graceful shutdown of all services
    async fn shutdown(&self) {
        info!("Shutting down Blaze Server...");

        // Set running to false to stop accepting new connections
        self.running.store(false, Ordering::SeqCst);

        // Give connections time to close gracefully
        info!("Waiting for active connections to close...");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Additional wait to ensure all ports are released
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        info!("Blaze Server shutdown complete");
    }

    /// Check if the emulator is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Request graceful shutdown
    pub fn request_shutdown(&self) {
        info!("Shutdown requested");
        self.running.store(false, Ordering::SeqCst);
        let _ = self.shutdown_tx.send(());
    }

    async fn start_web_services(
        &self,
        ports: &crate::common::game::ServicePorts,
    ) -> BlazeResult<()> {
        let web_server = WebProtocolServer::new(self.host.clone(), self.ssl_context.clone());
        web_server.start_web_servers(ports).await?;
        Ok(())
    }

    async fn start_blaze_services(
        &self,
        ports: &crate::common::game::ServicePorts,
    ) -> BlazeResult<()> {
        let blaze_server = BlazeProtocolServer::new(
            self.host.clone(),
            self.ssl_context.clone(),
            self.running.clone(),
        );
        blaze_server.start_blaze_servers(ports).await?;
        Ok(())
    }

    async fn start_specialized_services(
        &self,
        flags: &crate::client::ServiceFlags,
        ports: &crate::common::game::ServicePorts,
    ) -> BlazeResult<()> {
        if flags.qos {
            let qos_server = QosProtocolServer::new(self.host.clone(), self.ssl_context.clone());
            qos_server.start_qos_server(ports).await?;
        }

        if flags.rtm {
            let rtm_server = RtmProtocolServer::new(self.host.clone());
            rtm_server.start_rtm_server(ports).await?;
        }

        if flags.lsx {
            let lsx_server = LsxServer::new(ports.lsx);
            std::thread::spawn(move || {
                lsx_server.start();
            });
        }

        Ok(())
    }
}
