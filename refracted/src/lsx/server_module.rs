use aes::{
    cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit},
    Aes128,
};
use hex;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use tracing::error;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LsxServer {
    port: u16,
}

impl LsxServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn ports_from_config(p: &crate::common::game::ServicePorts) -> Vec<(u16, String)> {
        vec![(p.lsx, "LSX".into())]
    }

    pub fn start(&self) {
        use socket2::{Domain, Socket, Type};
        use std::net::SocketAddr;
        
        let addr: SocketAddr = match format!("127.0.0.1:{}", self.port).parse() {
            Ok(a) => a,
            Err(_) => {
                error!("Invalid address format for LSX server");
                return;
            }
        };
        
        // Create socket with SO_REUSEADDR
        let socket = match Socket::new(Domain::IPV4, Type::STREAM, None) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create socket for LSX server: {}", e);
                return;
            }
        };
        
        // Set SO_REUSEADDR to allow port reuse
        if let Err(e) = socket.set_reuse_address(true) {
            error!("Failed to set SO_REUSEADDR on LSX server socket: {}", e);
            return;
        }
        
        // Bind socket
        if let Err(e) = socket.bind(&addr.into()) {
            error!("Failed to bind LSX server to port {}: {}", self.port, e);
            return;
        }
        
        // Listen on socket
        if let Err(e) = socket.listen(128) {
            error!("Failed to listen on LSX server socket: {}", e);
            return;
        }
        
        // Convert to std::net::TcpListener
        let listener: std::net::TcpListener = socket.into();
        // LSX server started (logged by startup progress)

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let peer_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
                    crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m New connection accepted from {}", peer_addr);
                    thread::spawn(move || {
                        Self::handle_client(stream);
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    crate::debug_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle TCP client connection and perform LSX authentication handshake
    fn handle_client(mut stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        let peer_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
        crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Establishing secure connection to client");
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m handle_client entered for {}", peer_addr);

        let start_key = "cacf897a20b6d612ad0c05e011df52bb";
        let challenge = format!(
            r#"<LSX><Event sender="EALS"><Challenge build="release" key="{}" version="10,5,30,15625" /></Event></LSX>"#,
            start_key
        );

        let mut challenge_bytes = challenge.as_bytes().to_vec();
        challenge_bytes.push(0);

        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Sending challenge to client (size: {})", challenge_bytes.len());
        if let Err(e) = stream.write_all(&challenge_bytes) {
            crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Failed to send challenge: {}\x1b[0m", e);
            crate::debug_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Failed to send challenge: {}", e);
            return;
        }
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Challenge sent, waiting for response");

        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Reading challenge response");
        let tcp_string = match Self::read_tcp_string(&mut stream) {
            Ok(s) => {
                crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Challenge response received (size: {})", s.len());
                s
            },
            Err(_) => {
                crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Failed to read challenge response\x1b[0m");
                crate::debug_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Failed to read challenge response");
                return;
            }
        };

        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Parsing challenge response");
        let part_array: Vec<&str> = tcp_string.split('"').collect();
        if part_array.len() < 8 {
            crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Invalid challenge response format\x1b[0m");
            crate::debug_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Invalid challenge response format (parts: {})", part_array.len());
            return;
        }

        let content_id = Self::extract_content_id(&tcp_string);
        let version = Self::extract_version(&tcp_string);
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Extracted content_id: {}, version: {}", content_id, version);

        let (response, key) = match version.as_str() {
            "2" => (part_array[7], part_array[9]),
            _ => (part_array[5], part_array[7]),
        };
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Extracted response and key from challenge");

        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Verifying challenge response");
        if !Self::check_challenge_response(response, start_key) {
            crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Challenge verification failed\x1b[0m");
            crate::debug_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Challenge verification failed");
            return;
        }
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Challenge verification successful");

        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Generating challenge response");
        let new_response = Self::make_challenge_response(key);

        // Calculate seed from first two characters of hex string (uses string characters, not decoded bytes)
        let seed = if new_response.len() >= 2 {
            let first_byte = new_response.as_bytes()[0] as u16;
            let second_byte = new_response.as_bytes()[1] as u16;
            ((first_byte << 8) | second_byte) as u16
        } else {
            0
        };
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Calculated encryption seed: {}", seed);

        let challenge_accepted = format!(
            r#"<LSX><Response id="{}" sender="EALS"><ChallengeAccepted response="{}" /></Response></LSX>"#,
            part_array[3], new_response
        );
        let mut buffer = challenge_accepted.as_bytes().to_vec();
        buffer.push(0);

        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Sending ChallengeAccepted response (size: {})", buffer.len());
        if let Err(e) = stream.write_all(&buffer) {
            crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Failed to send challenge accepted: {}\x1b[0m", e);
            crate::debug_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Failed to send ChallengeAccepted: {}", e);
            return;
        }
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m ChallengeAccepted sent successfully");

        crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Secure connection established");
        crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Authenticating user");

        Self::handle_message_loop(stream, seed, content_id);
    }

    /// Read TCP string
    /// Reads until null terminator
    fn read_tcp_string(stream: &mut TcpStream) -> Result<String, std::io::Error> {
        let mut buffer = Vec::new();
        let mut byte_buf = [0u8; 1];

        stream.set_read_timeout(Some(std::time::Duration::from_secs(3600)))?;

        loop {
            match stream.read(&mut byte_buf) {
                Ok(0) => {
                    // Connection closed - only return error if we haven't read anything
                    if buffer.is_empty() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "Connection closed",
                        ));
                    }
                    break;
                }
                Ok(_) => {
                    let b = byte_buf[0];
                    if b == 0 {
                        break; // Null terminator
                    }
                    buffer.push(b);
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // Timeout or would block - if we have data, return it
                    if !buffer.is_empty() {
                        break;
                    }
                    return Err(std::io::Error::new(e.kind(), "Timeout reading"));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    /// Main message loop
    /// Handles encrypted LSX communication after challenge/response
    fn handle_message_loop(mut stream: TcpStream, seed: u16, content_id: String) {
        let peer_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
        crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m handle_message_loop entered for {} (seed: {}, content_id: {})", peer_addr, seed, content_id);
        let mut auth_confirmed = false;
        let mut username: Option<String> = None;
        
        loop {
            crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Waiting to read encrypted message");
            let data = match Self::read_tcp_string(&mut stream) {
                Ok(s) => {
                    crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Received encrypted message (size: {})", s.len());
                    s
                },
                Err(_) => {
                    if !auth_confirmed {
                        crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Connection lost\x1b[0m");
                    }
                    crate::debug_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Connection lost or read error");
                    return;
                }
            };

            crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Decrypting message with seed {}", seed);
            let decrypted_data = Self::lsx_decrypt_bf4(&data, seed);
            crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Decrypted message (size: {})", decrypted_data.len());

            // Capture request for Inspector
            if !decrypted_data.is_empty() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                
                // Extract method from XML (e.g., <GetProfile>, <QueryEntitlements>, etc.)
                let method = if decrypted_data.starts_with('<') {
                    if let Some(end) = decrypted_data.find('>') {
                        decrypted_data[1..end].to_string()
                    } else {
                        "UNKNOWN".to_string()
                    }
                } else {
                    "UNKNOWN".to_string()
                };
                
                use crate::core::inspector::inspector_module::{capture_lsx, CapturedLsx, LsxDirection};
                capture_lsx(CapturedLsx {
                    timestamp,
                    direction: LsxDirection::ClientToServer,
                    method: method.clone(),
                    path: "/lsx".to_string(),
                    host: "localhost".to_string(),
                    headers: Vec::new(),
                    body_size: decrypted_data.len(),
                    body: decrypted_data.as_bytes().to_vec(),
                    status_code: None,
                });
            }

            // Empty messages are valid (keepalive/handshake) - they return empty responses
            let response_data = if !decrypted_data.is_empty() {
                crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Processing non-empty request");
                // Handle request and get response
                let response = Self::lsx_request_handle_for_bfv(&decrypted_data, &content_id);
                crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Request handled, response size: {}", response.len());
                
                // Log important operations and extract user info from GetProfile
                if decrypted_data.contains("GetProfile") {
                    if username.is_none() {
                        crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Retrieving user profile");
                        if let Some((extracted_username, persona_id, user_id)) = Self::extract_user_info_from_profile(&response) {
                            username = Some(extracted_username.clone());
                            crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m User '{}' authenticated (user_id={}, persona_id={})", extracted_username, user_id, persona_id);
                            
                            // Get email from current profile
                            use crate::common::user_profile::{get_profiles, save_profile};
                            let mut profiles = get_profiles();
                            let email = profiles.profiles
                                .get(&profiles.current_profile)
                                .map(|p| p.email.clone())
                                .unwrap_or_else(|| "".to_string());
                            
                            // Store user session for JWT generation
                            use crate::session::{set_user_session, UserSession};
                            set_user_session(UserSession {
                                jwt_token: None,
                                user_id,
                                persona_id,
                                display_name: extracted_username.clone(),
                                email: email.clone(),
                                psid: 0, // PSID will be calculated from persona_id in JWT
                                update_network_info_count: 0,
                                hwfg: 0,
                                network_exip_ip: None,
                                network_inip_ip: None,
                                network_exip_port: None,
                                network_inip_port: None,
                                network_bps: None,
                                next_message_id: 1160000,
                            });
                            
                            // Update current user profile with LSX data if it matches
                            if let Some(current_profile) = profiles.profiles.get_mut(&profiles.current_profile) {
                                // Update profile with LSX data (preserve email)
                                current_profile.username = extracted_username.clone();
                                current_profile.user_id = user_id;
                                current_profile.persona_id = persona_id;
                                current_profile.display_name = extracted_username.clone();
                                // Email is preserved from profile, not overwritten
                                // Save updated profile
                                if let Err(e) = save_profile(&profiles.current_profile, current_profile.clone()) {
                                    crate::console_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m Failed to update profile: {}", e);
                                }
                            }
                            
                            crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m User session stored for JWT generation");
                        } else if let Some(extracted_username) = Self::extract_username_from_profile(&response) {
                            username = Some(extracted_username.clone());
                            crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m User '{}' authenticated (using defaults)", extracted_username);
                        }
                    } else {
                        crate::debug_println!(
                            "\x1b[38;2;255;215;0m[LSX]\x1b[0m GetProfile ({} bytes XML)",
                            response.len()
                        );
                    }
                } else if decrypted_data.contains("QueryEntitlements") {
                    crate::debug_println!(
                        "\x1b[38;2;255;215;0m[LSX]\x1b[0m QueryEntitlements request handled ({} bytes XML sent)",
                        response.len()
                    );
                } else if decrypted_data.contains("GetAuthCode") {
                    crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Generating authorization code");
                } else if decrypted_data.contains("QueryContent")
                    && !decrypted_data.contains("QueryContentResponse")
                {
                    if response.is_empty() {
                        crate::console_println!("\x1b[38;2;255;150;150m[LSX]\x1b[0m QueryContent: empty response (check routing)");
                    } else {
                        crate::debug_println!(
                            "\x1b[38;2;255;215;0m[LSX]\x1b[0m Querying content ({} bytes XML)",
                            response.len()
                        );
                    }
                } else if decrypted_data.contains("QueryFriends") {
                    crate::debug_println!(
                        "\x1b[38;2;255;215;0m[LSX]\x1b[0m Querying friends list ({} bytes XML)",
                        response.len()
                    );
                } else if decrypted_data.contains("QueryOffers") && !decrypted_data.contains("QueryOffersResponse") {
                    crate::debug_println!(
                        "\x1b[38;2;255;215;0m[LSX]\x1b[0m Query offers ({} bytes XML)",
                        response.len()
                    );
                } else if decrypted_data.contains("SetPresence") && decrypted_data.contains("UserId=") {
                    crate::debug_println!(
                        "\x1b[38;2;255;215;0m[LSX]\x1b[0m Set presence ({} bytes XML)",
                        response.len()
                    );
                }
                
                response
            } else {
                crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Empty request (keepalive), sending empty response");
                String::new() // Empty request -> empty response
            };

            // Capture response for Inspector
            if !response_data.is_empty() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64();
                
                // Extract method from request to match response
                let method = if !decrypted_data.is_empty() && decrypted_data.starts_with('<') {
                    if let Some(end) = decrypted_data.find('>') {
                        decrypted_data[1..end].to_string()
                    } else {
                        "UNKNOWN".to_string()
                    }
                } else {
                    "KEEPALIVE".to_string()
                };
                
                use crate::core::inspector::inspector_module::{capture_lsx, CapturedLsx, LsxDirection};
                capture_lsx(CapturedLsx {
                    timestamp,
                    direction: LsxDirection::ServerToClient,
                    method: format!("{}_Response", method),
                    path: "/lsx".to_string(),
                    host: "localhost".to_string(),
                    headers: Vec::new(),
                    body_size: response_data.len(),
                    body: response_data.as_bytes().to_vec(),
                    status_code: Some(200),
                });
            }

            crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Encrypting response with seed {}", seed);
            let encrypted_data = Self::lsx_encrypt_bf4(&response_data, seed);
            crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Response encrypted (size: {})", encrypted_data.len());

            let mut buffer = encrypted_data.as_bytes().to_vec();
            buffer.push(0);

            crate::debug_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Sending encrypted response (size: {})", buffer.len());
            if let Err(e) = stream.write_all(&buffer) {
                if !auth_confirmed {
                    crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Write error: {}\x1b[0m", e);
                }
                return;
            }

            // Make sure data is sent
            if let Err(e) = stream.flush() {
                if !auth_confirmed {
                    crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m \x1b[38;2;255;150;150mAuthentication failed: Flush error: {}\x1b[0m", e);
                }
                return;
            }
            
            // After first successful message exchange, authentication is confirmed
            if !auth_confirmed && !decrypted_data.is_empty() {
                // First non-empty message means auth is working
                crate::console_println!("\x1b[38;2;255;215;0m[LSX]\x1b[0m Authentication successful");
                auth_confirmed = true;
            }
            
            // If we got username from GetProfile and haven't logged it yet, do so now
            if auth_confirmed && username.is_some() && decrypted_data.contains("GetProfile") {
                // Username was just extracted, but we already confirmed auth
                // The "Retrieving user profile" message above will show the username extraction
            }
        }
    }
    
    /// Extract user information from GetProfile response
    fn extract_user_info_from_profile(response: &str) -> Option<(String, u64, u64)> {
        // Extract Persona, PersonaId, and UserId from GetProfile response XML
        let mut persona = None;
        let mut persona_id = None;
        let mut user_id = None;
        
        // Extract Persona="..."
        if let Some(start) = response.find("Persona=\"") {
            if let Some(end) = response[start + 9..].find("\"") {
                let extracted = response[start + 9..start + 9 + end].to_string();
                if !extracted.is_empty() {
                    persona = Some(extracted);
                }
            }
        }
        
        // Extract PersonaId="..."
        if let Some(start) = response.find("PersonaId=\"") {
            if let Some(end) = response[start + 11..].find("\"") {
                if let Ok(pid) = response[start + 11..start + 11 + end].parse::<u64>() {
                    persona_id = Some(pid);
                }
            }
        }
        
        // Extract UserId="..."
        if let Some(start) = response.find("UserId=\"") {
            if let Some(end) = response[start + 8..].find("\"") {
                if let Ok(uid) = response[start + 8..start + 8 + end].parse::<u64>() {
                    user_id = Some(uid);
                }
            }
        }
        
        if let (Some(p), Some(pid), Some(uid)) = (persona, persona_id, user_id) {
            Some((p, pid, uid))
        } else {
            None
        }
    }
    
    /// Extract username from GetProfile response (legacy method for compatibility)
    fn extract_username_from_profile(response: &str) -> Option<String> {
        if let Some((username, _, _)) = Self::extract_user_info_from_profile(response) {
            Some(username)
        } else {
            Some("Xevrac".to_string())
        }
    }

    fn extract_content_id(xml: &str) -> String {
        if let Some(start) = xml.find("<ContentId>") {
            if let Some(end) = xml[start..].find("</ContentId>") {
                return xml[start + 11..start + end].to_string();
            }
        }
        if let Some(start) = xml.find("<MasterTitleId>") {
            let inner = &xml[start + "<MasterTitleId>".len()..];
            if let Some(end) = inner.find("</MasterTitleId>") {
                return inner[..end].to_string();
            }
        }
        "16426154".to_string() // Default
    }

    fn master_title_id_from_request(request: &str) -> Option<String> {
        if let Some(start) = request.find("<MasterTitleId>") {
            let inner = &request[start + "<MasterTitleId>".len()..];
            if let Some(end) = inner.find("</MasterTitleId>") {
                let id = inner[..end].trim();
                if !id.is_empty() {
                    return Some(id.to_string());
                }
            }
        }
        None
    }

    /// First `id="..."` after `<Request ` — stable when attribute order differs from the quote-split heuristic.
    fn extract_lsx_request_id<'a>(request: &'a str, fallback: &'a str) -> &'a str {
        if let Some(start) = request.find("<Request ") {
            let slice = &request[start..];
            if let Some(pos) = slice.find("id=\"") {
                let rest = &slice[pos + 4..];
                if let Some(end) = rest.find('"') {
                    return &rest[..end];
                }
            }
        }
        fallback
    }

    fn extract_version(xml: &str) -> String {
        if let Some(start) = xml.find("version=\"") {
            if let Some(end) = xml[start + 9..].find("\"") {
                return xml[start + 9..start + 9 + end].to_string();
            }
        }
        "3".to_string() // Default
    }

    /// Check challenge response
    fn check_challenge_response(response: &str, key: &str) -> bool {
        let fixed_key = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let response_bytes = Self::get_byte_array(response);

        if response_bytes.is_empty() {
            return false;
        }

        let decrypted = Self::decrypt_aes_ecb_pkcs7(&response_bytes, &fixed_key);
        decrypted == key
    }

    /// Generate challenge response by encrypting the key
    fn make_challenge_response(key: &str) -> String {
        let fixed_key = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let key_bytes = key.as_bytes();

        Self::encrypt_aes_ecb_pkcs7(key_bytes, &fixed_key)
    }

    /// Decrypt LSX BF4 encrypted data using seed-derived key
    fn lsx_decrypt_bf4(data: &str, seed: u16) -> String {
        if data.is_empty() {
            return String::new();
        }

        let key = Self::get_lsx_key(seed);
        let bytes = Self::get_byte_array(data);

        if bytes.is_empty() {
            return String::new();
        }

        Self::decrypt_aes_ecb_pkcs7(&bytes, &key)
    }

    /// Encrypt LSX BF4 data using seed-derived key
    /// Returns empty string for empty input (essential for keepalive messages)
    fn lsx_encrypt_bf4(data: &str, seed: u16) -> String {
        if data.is_empty() {
            return String::new();
        }

        let key = Self::get_lsx_key(seed);
        let bytes = data.as_bytes();

        Self::encrypt_aes_ecb_pkcs7(bytes, &key)
    }

    /// Generate LSX encryption key from seed using linear congruential generator
    fn get_lsx_key(seed: u16) -> Vec<u8> {
        let mut rand_seed = 7u32;

        rand_seed = rand_seed.wrapping_mul(214013).wrapping_add(2531011);
        let rand_result = ((rand_seed >> 16) & 65535) as i32;

        rand_seed = (rand_result + seed as i32) as u32;

        let mut key = Vec::with_capacity(16);
        for _ in 0..16 {
            rand_seed = rand_seed.wrapping_mul(214013).wrapping_add(2531011);
            let rand_val = ((rand_seed >> 16) & 65535) as u8;
            key.push(rand_val);
        }

        key
    }

    /// Convert hex string to byte array, filtering out invalid characters
    fn get_byte_array(data: &str) -> Vec<u8> {
        let data_lower = data.to_lowercase();
        let source = "0123456789abcdef";
        let mut filtered = String::new();

        // Filter only valid hex characters
        for c in data_lower.chars() {
            if source.contains(c) {
                filtered.push(c);
            }
        }

        // Must be even length
        if filtered.len() % 2 != 0 {
            return Vec::new();
        }

        let mut result = Vec::new();
        for i in (0..filtered.len()).step_by(2) {
            if let Ok(byte) = u8::from_str_radix(&filtered[i..i + 2], 16) {
                result.push(byte);
            }
        }

        result
    }

    /// Decrypt data using AES-128 ECB mode with PKCS7 padding
    fn decrypt_aes_ecb_pkcs7(bytes: &[u8], key: &[u8]) -> String {
        if bytes.is_empty() || key.len() != 16 {
            return String::new();
        }

        let aes_key = GenericArray::from_slice(key);
        let cipher = Aes128::new(aes_key);

        // Decrypt in 16-byte blocks (ECB mode with PKCS7 padding)
        let mut decrypted = bytes.to_vec();
        for i in (0..decrypted.len()).step_by(16) {
            if i + 16 <= decrypted.len() {
                let mut block = *GenericArray::from_slice(&decrypted[i..i + 16]);
                cipher.decrypt_block(&mut block);
                decrypted[i..i + 16].copy_from_slice(&block);
            }
        }

        // Remove PKCS7 padding
        if !decrypted.is_empty() {
            let padding_len = decrypted[decrypted.len() - 1] as usize;
            if padding_len > 0 && padding_len <= 16 && padding_len <= decrypted.len() {
                decrypted.truncate(decrypted.len() - padding_len);
            }
        }

        String::from_utf8_lossy(&decrypted).to_string()
    }

    /// Encrypt data using AES-128 ECB mode with PKCS7 padding
    fn encrypt_aes_ecb_pkcs7(bytes: &[u8], key: &[u8]) -> String {
        if key.len() != 16 {
            return String::new();
        }

        // Add PKCS7 padding
        let block_size = 16;
        let padding_len = block_size - (bytes.len() % block_size);
        let mut padded = bytes.to_vec();
        for _ in 0..padding_len {
            padded.push(padding_len as u8);
        }

        let aes_key = GenericArray::from_slice(key);
        let cipher = Aes128::new(aes_key);

        // Encrypt in 16-byte blocks (ECB mode)
        let mut encrypted = Vec::new();
        for chunk in padded.chunks(16) {
            if chunk.len() == 16 {
                let mut block = *GenericArray::from_slice(chunk);
                cipher.encrypt_block(&mut block);
                encrypted.extend_from_slice(&block);
            }
        }

        hex::encode(encrypted)
    }

    /// Handle LSX request and return appropriate XML response
    fn lsx_request_handle_for_bfv(request: &str, handshake_content_id: &str) -> String {
        if request.is_empty() {
            return String::new();
        }

        let content_id = Self::master_title_id_from_request(request)
            .unwrap_or_else(|| handshake_content_id.to_string());

        // Route before split-by-quote: some clients reorder attributes so `part_array[4]` is not the expected `"><…` token.
        if request.contains("QueryContent") && !request.contains("QueryContentResponse") {
            let id = Self::extract_lsx_request_id(request, "0");
            return match content_id.as_str() {
                "16426154" => Self::query_content_16426154_response(id),
                _ => Self::query_content_generic_response(id),
            };
        }
        if request.contains("<QueryEntitlements") && request.contains("UserId=") {
            let id = Self::extract_lsx_request_id(request, "0");
            return match content_id.as_str() {
                "198387" => Self::query_entitlements_fc25_response(id),
                "16426154" => Self::query_entitlements_16426154_response(id),
                "196736_beta" => Self::query_entitlements_beta_response(id),
                _ => Self::query_entitlements_titanfall2_response(id),
            };
        }
        if request.contains("<GetProfile") && request.contains("index=") {
            let id = Self::extract_lsx_request_id(request, "0");
            return Self::get_profile_response(id);
        }
        if request.contains("<QueryOffers") && request.contains("UserId=") {
            let id = Self::extract_lsx_request_id(request, "0");
            return Self::query_offers_response(id);
        }
        if request.contains("<SetPresence") && request.contains("UserId=") {
            let id = Self::extract_lsx_request_id(request, "0");
            return Self::set_presence_response(id);
        }

        let part_array: Vec<&str> = request.split('"').collect();
        if part_array.len() < 4 {
            return String::new();
        }

        let id = Self::extract_lsx_request_id(request, part_array[3]);
        let request_type = if part_array.len() > 4 {
            part_array[4]
        } else {
            ""
        };

        match request_type {
            "><GetConfig version=" => Self::get_config_response(id),
            "><GetAuthCode ClientId=" | "><GetAuthCode UserId=" => Self::get_auth_code_response(id),
            "><GetBlockList version=" => Self::get_block_list_response(id),
            "><GetGameInfo GameInfoId=" => {
                if part_array.len() > 5 {
                    match part_array[5] {
                        "FREETRIAL" => Self::get_game_info_freetrial_response(id),
                        "UPTODATE" => Self::get_game_info_uptodate_response(id),
                        "INSTALLED_LANGUAGE" => Self::get_game_info_installed_language_response(id),
                        _ => Self::get_game_info_default_response(id),
                    }
                } else {
                    Self::get_game_info_default_response(id)
                }
            }
            "><GetInternetConnectedState version=" => {
                Self::get_internet_connected_state_response(id)
            }
            "><GetPresence UserId=" => Self::get_presence_response_xmpp(id),
            "><GetProfile index=" => Self::get_profile_response(id),
            "><QueryImage ImageId=" => Self::query_image_response(id),
            "><RequestLicense UserId=" => Self::request_license_response(id),
            "><GetSetting SettingId=" => {
                if part_array.len() > 5 {
                    match part_array[5] {
                        "ENVIRONMENT" => Self::get_setting_environment_response(id),
                        "IS_IGO_AVAILABLE" | "IS_IGO_ENABLED" => Self::get_setting_igo_response(id),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                }
            }
            "><SetPresence UserId=" => Self::set_presence_response(id),
            "><QueryFriends UserId=" => Self::query_friends_response(id),
            "><QueryPresence UserId=" => Self::query_presence_response(id),
            "><GetAllGameInfo version=" => Self::get_all_game_info_response(id),
            "><IsProgressiveInstallationAvailable ItemId=" => {
                Self::is_progressive_installation_available_response(id)
            }
            "><QueryEntitlements UserId=" => match content_id.as_str() {
                "198387" => Self::query_entitlements_fc25_response(id),
                "16426154" => Self::query_entitlements_16426154_response(id),
                "196736_beta" => Self::query_entitlements_beta_response(id),
                _ => Self::query_entitlements_titanfall2_response(id),
            },
            "><QueryOffers UserId=" => Self::query_offers_response(id),
            "><SetDownloaderUtilization Utilization=" => {
                Self::set_downloader_utilization_response(id)
            }
            "><QueryChunkStatus ItemId=" => Self::query_chunk_status_response(id),
            "><GetPresenceVisibility UserId=" => Self::get_presence_visibility_response(id),
            "><GetWalletBalance UserId=" => Self::get_wallet_balance_response(id),
            "><GetSettings version=" => Self::get_settings_response(id),
            _ => {
                if request.contains("QueryContent") && !request.contains("QueryContentResponse") {
                    let id = Self::extract_lsx_request_id(request, "0");
                    match content_id.as_str() {
                        "16426154" => Self::query_content_16426154_response(id),
                        _ => Self::query_content_generic_response(id),
                    }
                } else {
                    String::new()
                }
            }
        }
    }

    // ============================================================================
    // Response Functions
    // ============================================================================

    fn get_config_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetConfigResponse>
            <Service Facility="SDK" Name="EbisuSDK" />
            <Service Facility="PROFILE" Name="EbisuSDK" />
            <Service Facility="PRESENCE" Name="XMPP" />
            <Service Facility="FRIENDS" Name="XMPP" />
            <Service Facility="COMMERCE" Name="Commerce" />
            <Service Facility="RECENTPLAYER" Name="EbisuSDK" />
            <Service Facility="IGO" Name="EbisuSDK" />
            <Service Facility="MISC" Name="EbisuSDK" />
            <Service Facility="LOGIN" Name="EALS" />
            <Service Facility="UTILITY" Name="Utility" />
            <Service Facility="XMPP" Name="XMPP" />
            <Service Facility="CHAT" Name="XMPP" />
            <Service Facility="IGO_EVENT" Name="EbisuSDK" />
            <Service Facility="EALS_EVENTS" Name="EALS" />
            <Service Facility="LOGIN_EVENT" Name="EbisuSDK" />
            <Service Facility="INVITE_EVENT" Name="XMPP" />
            <Service Facility="PROFILE_EVENT" Name="EbisuSDK" />
            <Service Facility="PRESENCE_EVENT" Name="XMPP" />
            <Service Facility="FRIENDS_EVENT" Name="XMPP" />
            <Service Facility="COMMERCE_EVENT" Name="Commerce" />
            <Service Facility="CHAT_EVENT" Name="XMPP" />
            <Service Facility="DOWNLOAD_EVENT" Name="EbisuSDK" />
            <Service Facility="PERMISSION" Name="EbisuSDK" />
            <Service Facility="RESOURCES" Name="EbisuSDK" />
            <Service Facility="BLOCKED_USERS" Name="EbisuSDK" />
            <Service Facility="BLOCKED_USER_EVENT" Name="EbisuSDK" />
            <Service Facility="GET_USERID" Name="EbisuSDK" />
            <Service Facility="ONLINE_STATUS_EVENT" Name="EbisuSDK" />
            <Service Facility="ACHIEVEMENT" Name="EbisuSDK" />
            <Service Facility="ACHIEVEMENT_EVENT" Name="EbisuSDK" />
            <Service Facility="BROADCAST_EVENT" Name="EbisuSDK" />
            <Service Facility="PROGRESSIVE_INSTALLATION" Name="PI" />
            <Service Facility="PROGRESSIVE_INSTALLATION_EVENT" Name="PI" />
            <Service Facility="CONTENT" Name="EbisuSDK" />
        </GetConfigResponse>
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_auth_code_response(id: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Generate a realistic-looking authorization code
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Format: AC_<timestamp>_<random_hex>
        let auth_code = format!(
            "AC_EA_GLACIER_{}_{:x}",
            current_time,
            current_time % 0xFFFFFF
        );

        format!(
            r#"<LSX>
    <Response id="{}" sender="Utility">
        <AuthCode value="{}" />
    </Response>
</LSX>"#,
            id, auth_code
        )
    }

    fn get_block_list_response(id: &str) -> String {
        format!(
            r#"<LSX>
  <Response id="{}" sender="EbisuSDK">
        <GetBlockListResponse Return="Success" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_game_info_default_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetGameInfoResponse
            GameInfo="ar_SA,en_US,ko_KR,zh_CN,zh_TW,de_DE,es_ES,es_MX,fr_FR,it_IT,ja_JP,pl_PL,pt_BR,ru_RU" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_game_info_freetrial_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetGameInfoResponse GameInfo="false" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_game_info_uptodate_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetGameInfoResponse GameInfo="true" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_game_info_installed_language_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetGameInfoResponse GameInfo="zh_CN" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_presence_response_xmpp(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="XMPP">
        <GetPresenceResponse TitleId="" Title="" Presence="INGAME" RichPresence="Battlefield 4"
            SessionId="" Group="" MultiplayerId="" GamePresence="" GroupId="" UserId="0" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn query_image_response(id: &str) -> String {
        format!(
            r#"<LSX>
  <Response id="{}" sender="EbisuSDK">
    <QueryImageResponse Result="0">
      <Image ImageId="AvatarId" Width="256" Height="256"
          ResourcePath="AvatarId" />
    </QueryImageResponse>
  </Response>
</LSX>"#,
            id
        )
    }

    fn request_license_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response sender="EbisuSDK" id="{}">
        <RequestLicenseResponse License="LicenseKey" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_internet_connected_state_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="Utility">
        <InternetConnectedState connected="1" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_profile_response(id: &str) -> String {
        // Get current user profile for dynamic values
        use crate::common::user_profile::get_current_profile;
        let profile = get_current_profile();
        
        // PersonaId (PID): Large numeric ID for the persona
        // Persona (DSNM): Display name string (not a number!)
        // UserId (UID): Large numeric user ID
        format!(
            r#"<LSX>
  <Response id="{}" sender="EbisuSDK">
    <GetProfileResponse Country="US" GeoCountry="US" PersonaId="{}" IsTrialSubscriber="false" SubscriberLevel="2" Persona="{}" UserId="{}" CommerceCountry="US" CommerceCurrency="USD" AvatarId="" IsSubscriber="true" UserIndex="0" IsUnderAge="false" IsSteamSubscriber="false"/>
  </Response>
</LSX>"#,
            id,
            profile.persona_id,
            profile.display_name,
            profile.user_id
        )
    }

    fn get_setting_environment_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetSettingResponse Setting="production" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_setting_igo_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetSettingResponse Setting="false" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn set_presence_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="XMPP">
        <ErrorSuccess Description="" Code="0" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn query_friends_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="XMPP">
        <QueryFriendsResponse>
            <Friend RichPresence="LSXEmu by Xevrac" AvatarId="AvatarId" UserId="108447993" Group="" Title="LSXEmu by Xevrac" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" Xevrac" PersonaId="10076847991" State="MUTUAL" MultiplayerId="196216" GroupId="" Presence="INGAME" />
            <Friend RichPresence="Dev007" AvatarId="AvatarId" UserId="10115084479" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" Dev007" PersonaId="1367686647992" State="MUTUAL" MultiplayerId="196216" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="NotAnNPC" AvatarId="AvatarId" UserId="10115084480" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" NotAnNPC" PersonaId="1367686647993" State="MUTUAL" MultiplayerId="196217" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="SniperWolf" AvatarId="AvatarId" UserId="10115084481" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" SniperWolf" PersonaId="1367686647994" State="MUTUAL" MultiplayerId="196218" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="TechGuru" AvatarId="AvatarId" UserId="10115084482" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" TechGuru" PersonaId="1367686647995" State="MUTUAL" MultiplayerId="196219" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="NexusCore" AvatarId="AvatarId" UserId="10115084483" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" NexusCore" PersonaId="1367686647996" State="MUTUAL" MultiplayerId="196220" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="RogueFox" AvatarId="AvatarId" UserId="10115084484" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" RogueFox" PersonaId="1367686647997" State="MUTUAL" MultiplayerId="196221" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="EchoSix" AvatarId="AvatarId" UserId="10115084485" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" EchoSix" PersonaId="1367686647998" State="MUTUAL" MultiplayerId="196222" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="DeltaNine" AvatarId="AvatarId" UserId="10115084486" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" DeltaNine" PersonaId="1367686647999" State="MUTUAL" MultiplayerId="196223" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="Zenith" AvatarId="AvatarId" UserId="10115084487" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" Zenith" PersonaId="1367686648000" State="MUTUAL" MultiplayerId="196224" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="IronHawk" AvatarId="AvatarId" UserId="10115084488" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" IronHawk" PersonaId="1367686648001" State="MUTUAL" MultiplayerId="196225" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="RaptorX" AvatarId="AvatarId" UserId="10115084489" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" RaptorX" PersonaId="1367686648002" State="MUTUAL" MultiplayerId="196226" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="ShadowLink" AvatarId="AvatarId" UserId="10115084490" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" ShadowLink" PersonaId="1367686648003" State="MUTUAL" MultiplayerId="196227" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="ByteKnight" AvatarId="AvatarId" UserId="10115084491" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" ByteKnight" PersonaId="1367686648004" State="MUTUAL" MultiplayerId="196228" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="Vortex" AvatarId="AvatarId" UserId="10115084492" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" Vortex" PersonaId="1367686648005" State="MUTUAL" MultiplayerId="196229" GroupId="" Presence="JOINABLE" />
            <Friend RichPresence="CrimsonAce" AvatarId="AvatarId" UserId="10115084493" Group="" Title="Battlefield Labs" TitleId="Origin.OFR.50.0004152" GamePresence="" Persona=" CrimsonAce" PersonaId="1367686648006" State="MUTUAL" MultiplayerId="196230" GroupId="" Presence="JOINABLE" />
        </QueryFriendsResponse>
    </Response>
</LSX>"#,
            id
        )
    }

    fn query_presence_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="XMPP">
        <QueryPresenceResponse>
            <Presence UserId="10010" State="ONLINE" />
        </QueryPresenceResponse>
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_all_game_info_response(id: &str) -> String {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <GetAllGameInfoResponse DisplayName="Battlefield Labs" InstalledVersion="1.0.382.13608" AvailableVersion="1.0.382.13608" UpToDate="true" FullGameReleased="true" FullGameReleaseDate="2010-01-01T00:00:00" FullGamePurchased="true" FreeTrial="false" Expiration="0000-00-00T00:00:00" EntitlementSource="ORIGIN" MaxGroupSize="16" Languages="en_US,ja_JP,zh_CN" InstalledLanguage="en_US" HasExpiration="false" SystemTime="{}"/>
    </Response>
</LSX>"#,
            id, now
        )
    }

    fn is_progressive_installation_available_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <IsProgressiveInstallationAvailableResponse Available="true" />
    </Response>
</LSX>"#,
            id
        )
    }

    fn query_content_16426154_response(id: &str) -> String {
        format!(
            r#"<LSX>
  <Response sender="EbisuSDK" id="{}">
    <QueryContentResponse>
      <Game state="READY_TO_PLAY" installedVersion="1.0.366.6303" displayName="Battlefield Labs - Content 2" contentID="Origin.OFR.50.0005994" progressValue="1" availableVersion="1.0.366.6303"/>
      <Game state="READY_TO_PLAY" installedVersion="1.0.366.6303" displayName="Battlefield Labs - Content 1" contentID="Origin.OFR.50.0005993" progressValue="1" availableVersion="1.0.366.6303"/>
      <Game state="READY_TO_PLAY" installedVersion="1.0.366.6303" displayName="Battlefield Labs - Content 3" contentID="Origin.OFR.50.0005995" progressValue="1" availableVersion="1.0.366.6303"/>
    </QueryContentResponse>
  </Response>
</LSX>"#,
            id
        )
    }

    fn query_content_generic_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <QueryContentResponse>
            <Content Gamestate="PLAYING" progressValue="0" contentID="Origin.OFR.50.0004657"
                installedVersion="0" availableVersion="1.0.57.44284" displayName="Battlefield 1" />
            <Content Gamestate="READY_TO_PLAY" progressValue="0" contentID="Origin.OFR.50.0000557"
                installedVersion="1.0.57.44284" availableVersion="1.0.57.44284"
                displayName="Battlefield 1" />
        </QueryContentResponse>
    </Response>
</LSX>"#,
            id
        )
    }

    fn query_entitlements_16426154_response(id: &str) -> String {
        use crate::common::user_profile::get_current_profile;
        let uid = get_current_profile().user_id;
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <QueryEntitlementsResponse>
            <Entitlement Expiration="0000-00-00T00:00:00" ItemId="Origin.OFR.50.0005997" LastModifiedDate="0000-00-00T00:00:00" GrantDate="2025-05-15T20:11:00" Version="0" UseCount="0" Group="LABSPC" Type="DEFAULT" EntitlementTag="bflabs_marker01" EntitlementId="{}" ResourceId=""/>
            <Entitlement Expiration="0000-00-00T00:00:00" ItemId="Origin.OFR.50.0005995" LastModifiedDate="0000-00-00T00:00:00" GrantDate="2025-05-15T20:11:00" Version="0" UseCount="0" Group="LABSPC" Type="DEFAULT" EntitlementTag="bflabs_content003" EntitlementId="{}" ResourceId=""/>
            <Entitlement Expiration="0000-00-00T00:00:00" ItemId="Origin.OFR.50.0005993" LastModifiedDate="0000-00-00T00:00:00" GrantDate="2025-05-15T20:11:00" Version="0" UseCount="0" Group="LABSPC" Type="DEFAULT" EntitlementTag="bflabs_content001" EntitlementId="{}" ResourceId=""/>
            <Entitlement Expiration="0000-00-00T00:00:00" ItemId="Origin.OFR.50.0005994" LastModifiedDate="0000-00-00T00:00:00" GrantDate="2025-05-15T20:11:00" Version="0" UseCount="0" Group="LABSPC" Type="DEFAULT" EntitlementTag="bflabs_content002" EntitlementId="{}" ResourceId=""/>
            <Entitlement Expiration="0000-00-00T00:00:00" ItemId="Origin.OFR.50.0006023" LastModifiedDate="0000-00-00T00:00:00" GrantDate="2025-05-15T20:11:00" Version="0" UseCount="0" Group="LABSPC" Type="ONLINE_ACCESS" EntitlementTag="LABS_ONLINE_ACCESS" EntitlementId="{}" ResourceId=""/>
        </QueryEntitlementsResponse>
    </Response>
</LSX>"#,
            id, uid, uid, uid, uid, uid
        )
    }

    fn query_entitlements_titanfall2_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="Commerce">
        <QueryEntitlementsResponse>
            <Entitlements EntitlementGrantDate="2019-02-13T15:30:00" ResourceId="" UseCount="0"
                EntitlementId="1012783049032" Expiration="0000-00-00T00:00:00" Type="DEFAULT"
                Source="ORIGIN" LastModifiedDate="2019-02-13T15:30:00" Group="Titanfall2PC"
                ItemId="Origin.OFR.50.0002300" EntitlementTag="TITANFALL2_JUMPSTARTERKIT"
                Version="0" />
            <Entitlements EntitlementGrantDate="2019-02-13T15:30:00" ResourceId="" UseCount="0"
                EntitlementId="1012782849032" Expiration="0000-00-00T00:00:00" Type="DEFAULT"
                Source="ORIGIN" LastModifiedDate="2019-02-13T15:30:00" Group="Titanfall2PC"
                ItemId="Origin.OFR.50.0002268"
                EntitlementTag="TITANFALL2_DLC7_UNDERGROUNDR201CARBINE" Version="0" />
            <Entitlements EntitlementGrantDate="2019-02-13T15:30:00" ResourceId="" UseCount="0"
                EntitlementId="1012782649032" Expiration="0000-00-00T00:00:00" Type="DEFAULT"
                Source="ORIGIN" LastModifiedDate="2019-02-13T15:30:00" Group="Titanfall2PC"
                ItemId="Origin.OFR.50.0001466" EntitlementTag="TITANFALL2_DELUXE_CONTENT"
                Version="0" />
            <Entitlements EntitlementGrantDate="2019-02-13T15:30:00" ResourceId="" UseCount="0"
                EntitlementId="1012782449032" Expiration="0000-00-00T00:00:00" Type="DEFAULT"
                Source="ORIGIN" LastModifiedDate="2019-02-13T15:30:00" Group="Titanfall2PC"
                ItemId="Origin.OFR.50.0001455" EntitlementTag="ONLINE_ACCESS" Version="0" />
            <Entitlements EntitlementGrantDate="2016-12-05T04:18:00" ResourceId="" UseCount="0"
                EntitlementId="1010989849032" Expiration="0000-00-00T00:00:00" Type="DEFAULT"
                Source="ORIGIN" LastModifiedDate="2016-12-05T04:18:00" Group="Titanfall2PC"
                ItemId="Origin.OFR.50.0001475" EntitlementTag="TRIAL_ONLINE_ACCESS" Version="0" />
            <Entitlements EntitlementGrantDate="2016-12-05T04:18:00" ResourceId="" UseCount="0"
                EntitlementId="1010989649032" Expiration="0000-00-00T00:00:00" Type="DEFAULT"
                Source="ORIGIN" LastModifiedDate="2016-12-05T04:18:00" Group="Titanfall2PC"
                ItemId="Origin.OFR.50.0001475" EntitlementTag="TITANFALL2_PUBLIC_TRIAL" Version="0" />
        </QueryEntitlementsResponse>
    </Response>
</LSX>"#,
            id
        )
    }

    fn query_entitlements_fc25_response(id: &str) -> String {
        format!(
            r#"<LSX>
  <Response sender="Commerce" id="{}">
    <QueryEntitlementsResponse>
      <Entitlement EntitlementTag="ONLINE_ACCESS" Expiration="0000-00-00T00:00:00" LastModifiedDate="2024-10-19T06:55:00" GrantDate="2024-10-19T06:55:00" ResourceId="" EntitlementId="1022487313462" ItemId="Origin.OFR.50.0005506" Source="STEAM" Type="DEFAULT" Version="0" UseCount="0" Group="FC25PC"/>
      <Entitlement EntitlementTag="FC25PC_FULLGAME" Expiration="0000-00-00T00:00:00" LastModifiedDate="2024-10-19T06:55:00" GrantDate="2024-10-19T06:55:00" ResourceId="" EntitlementId="1022487113462" ItemId="Origin.OFR.50.0005506" Source="STEAM" Type="DEFAULT" Version="0" UseCount="0" Group="FC25PC"/>
      <Entitlement EntitlementTag="FC25PC_BASEGAME" Expiration="0000-00-00T00:00:00" LastModifiedDate="2024-10-19T06:55:00" GrantDate="2024-10-19T06:55:00" ResourceId="" EntitlementId="1022486913462" ItemId="Origin.OFR.50.0005391" Source="STEAM" Type="DEFAULT" Version="0" UseCount="0" Group="FC25PC"/>
    </QueryEntitlementsResponse>
  </Response>
</LSX>"#,
            id
        )
    }

    fn query_entitlements_beta_response(id: &str) -> String {
        format!(
            r#"<LSX>
  <Response id="{}" sender="EbisuSDK">
    <QueryEntitlementsResponse>
      <Entitlement ResourceId="" LastModifiedDate="0000-00-00T00:00:00" Version="0" EntitlementId="1023125274866" UseCount="0" GrantDate="2025-08-04T15:05:00" Type="ONLINE_ACCESS" Group="GLACIERPC" Expiration="0000-00-00T00:00:00" ItemId="Origin.OFR.50.0005901" EntitlementTag="beta_online_access"/>
      <Entitlement ResourceId="" LastModifiedDate="0000-00-00T00:00:00" Version="0" EntitlementId="1023110274866" UseCount="0" GrantDate="2025-08-02T08:45:00" Type="DEFAULT" Group="GLACIERPC" Expiration="0000-00-00T00:00:00" ItemId="Origin.OFR.50.0006014" EntitlementTag="beta_earlyaccess"/>
    </QueryEntitlementsResponse>
  </Response>
</LSX>"#,
            id
        )
    }

    fn query_offers_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="Commerce">
        <QueryOffersResponse>
            <Offer Currency="EUR" bHidden="false" UseEndDate="0000-00-00T00:00:00"
                bIsDiscounted="false" ImageId="" Description="Titanfall 2: Ultimate Edition"
                LocalizedOriginalPrice="€4.99" GameDistributionSubType="" InventorySold="-1"
                Name="Titanfall 2: Ultimate Edition" Price="4.99" InventoryCap="-1" OriginalPrice="4.99"
                PlayableDate="2016-11-29T23:45:00" DownloadSize="0"
                DownloadDate="2016-11-29T23:45:00" LocalizedPrice="€4.99" bCanPurchase="true"
                InventoryAvailable="-1" PurchaseDate="0000-00-00T00:00:00" bIsOwned="false"
                Type="Extra Content" OfferId="Origin.OFR.50.0001873" />
        </QueryOffersResponse>
    </Response>
</LSX>"#,
            id
        )
    }

    fn set_downloader_utilization_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="EbisuSDK">
        <SetDownloaderUtilizationResponse />
    </Response>
</LSX>"#,
            id
        )
    }

    fn query_chunk_status_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response id="{}" sender="PI">
        <QueryChunkStatusResponse>
            <ChunkStatus ChunkETA="0" TotalETA="0" Type="REQUIRED" Progress="1" Size="32639396202"
                ChunkId="0" Name="0" ItemId="Origin.OFR.50.0001455" State="INSTALLED" />
            <ChunkStatus ChunkETA="0" TotalETA="0" Type="RECOMMENDED" Progress="1"
                Size="35636260230" ChunkId="1" Name="1" ItemId="Origin.OFR.50.0001455"
                State="INSTALLED" />
        </QueryChunkStatusResponse>
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_presence_visibility_response(id: &str) -> String {
        format!(
            r#"<LSX>
    <Response sender="XMPP" id="{}">
        <GetPresenceVisibilityResponse Visible="false"/>
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_wallet_balance_response(id: &str) -> String {
        format!(
            r#"<LSX>
  <Response sender="Commerce" id="{}">
    <GetWalletBalanceResponse Balance="0"/>
    </Response>
</LSX>"#,
            id
        )
    }

    fn get_settings_response(id: &str) -> String {
        format!(
            r#"<LSX>
  <Response id="{}" sender="EbisuSDK">
    <GetSettingsResponse Environment="production" IsIGOAvailable="false" IsTelemetryEnabled="false" IsManualOffline="false" Language="en_US" IsIGOEnabled="false"/>
    </Response>
</LSX>"#,
            id
        )
    }
}
