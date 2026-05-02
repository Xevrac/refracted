use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// `azp` / `nexus.cli` for Nexus JWTs. Must match `gatewayClientId` from Blaze
/// `UtilComponent.fetchClientConfig` (default labs: `conf_ch1_release_mp11_labs`).
pub const NEXUS_GATEWAY_CLIENT_ID: &str = "GLACIER_LBGW_BK_OL_SERVER";

/// Session information extracted from JWT token
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user_id: i64,
    pub persona_id: i64,
    pub display_name: String,
    pub email: String,
    pub psid: i32,
    pub ausrc: String,
}

impl SessionInfo {
    /// Create default session info
    pub fn default() -> Self {
        // Use session email if available, otherwise fallback
        use crate::session::get_user_session;
        let session = get_user_session();
        Self {
            user_id: 1012711274866,
            persona_id: 1006276674866,
            display_name: "bf-labs-admin".to_string(),
            email: if !session.email.is_empty() {
                session.email.clone()
            } else {
                "bf-labs-admin@ea.com".to_string()
            },
            psid: 0,
            ausrc: "324320".to_string(),
        }
    }
}

/// Parse JWT token to extract user information
pub fn parse_jwt_token(jwt: &str) -> SessionInfo {
    crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Parsing JWT token for Blaze authentication...");
    
    // JWT format: header.payload.signature
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() < 2 {
        crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m ERROR: Invalid JWT token format - using default session info");
        return SessionInfo::default();
    }

    // Decode payload (second part)
    let payload_b64 = parts[1];
    if let Ok(payload_bytes) = general_purpose::URL_SAFE_NO_PAD.decode(payload_b64) {
        if let Ok(payload_str) = String::from_utf8(payload_bytes) {
            if let Ok(payload_json) = serde_json::from_str::<Value>(&payload_str) {
                // Extract nexus data
                if let Some(nexus) = payload_json.get("nexus").and_then(|n| n.as_object()) {
                    // Extract user ID
                    let uid = nexus
                        .get("uid")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i64>().ok())
                        .unwrap_or(1012711274866);

                    // Extract persona ID from pid or psif[0].id
                    let pid = nexus
                        .get("pid")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i64>().ok())
                        .or_else(|| {
                            nexus
                                .get("psif")
                                .and_then(|v| v.as_array())
                                .and_then(|arr| arr.get(0))
                                .and_then(|v| v.get("id"))
                                .and_then(|v| v.as_u64())
                                .map(|v| v as i64)
                        })
                        .unwrap_or(1006276674866);

                    // Extract display name from psif[0].dis or psif[0].nic
                    let display_name = nexus
                        .get("psif")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.get(0))
                        .and_then(|v| v.get("dis").or_else(|| v.get("nic")))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "bf-labs-admin".to_string());

                    // Extract PSID
                    let psid = nexus
                        .get("psid")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as i32)
                        .unwrap_or(0);

                    // Extract AUSRC (auth source)
                    let ausrc = nexus
                        .get("ausrc")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "324320".to_string());

                    // Extract email from nexus.uif.mail or fallback to session email
                    let email = nexus
                        .get("uif")
                        .and_then(|v| v.get("mail"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| {
                            // Try direct nexus.mail
                            nexus.get("mail")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_else(|| {
                            // Fallback to session email instead of hardcoded value
                            use crate::session::get_user_session;
                            let session = get_user_session();
                            session.email.clone()
                        });

                    let session_info = SessionInfo {
                        user_id: uid,
                        persona_id: pid,
                        display_name: display_name.clone(),
                        email: email.clone(),
                        psid,
                        ausrc: ausrc.clone(),
                    };
                    
                    crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT token parsed successfully - uid={}, pid={}, display_name={}, ausrc={}", uid, pid, display_name, ausrc);
                    return session_info;
                }
            }
        }
    }

    // Fallback to default values
    crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m WARN: Failed to parse JWT token - using default session info");
    SessionInfo::default()
}

/// Generate JWT token for HTTP/gRPC responses
pub fn generate_jwt_token(
    session_id: &str,
    persona_id: &u64,
    player_name: &str,
    client_id: &str,
) -> String {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // JWT Header
    let mut header = HashMap::new();
    header.insert("kid".to_string(), "ff55e5b0-895d-485a-afde-897569b103ee".to_string());
    header.insert("alg".to_string(), "RS256".to_string());
    header.insert("typ".to_string(), "JWT".to_string());

    // JWT Payload
    let mut payload = HashMap::new();
    payload.insert("iss".to_string(), "accounts.ea.com".to_string());
    payload.insert("jti".to_string(), session_id.to_string());
    payload.insert("azp".to_string(), client_id.to_string());
    payload.insert("iat".to_string(), current_time.to_string());
    payload.insert("exp".to_string(), (current_time + 259200).to_string()); // 3 days
    payload.insert("pid".to_string(), persona_id.to_string());
    payload.insert("pname".to_string(), player_name.to_string());

    // Encode header and payload
    let header_json = serde_json::to_string(&header).unwrap();
    let payload_json = serde_json::to_string(&payload).unwrap();

    let header_b64 = general_purpose::STANDARD.encode(header_json.as_bytes());
    let payload_b64 = general_purpose::STANDARD.encode(payload_json.as_bytes());

    // Create a fake but realistic-looking signature
    let signature_input = format!("{}.{}", header_b64, payload_b64);
    let signature_hash = Sha256::digest(signature_input.as_bytes());
    let signature_bytes = [signature_hash.as_slice(); 8].concat(); // 256 bytes
    let signature = general_purpose::STANDARD.encode(signature_bytes);

    format!("{}.{}.{}", header_b64, payload_b64, signature)
}

/// Generate EA-specific JWT token with full nexus structure
pub fn generate_ea_jwt_token(
    session_id: &str,
    persona_id: &u64,
    player_name: &str,
    client_id: &str,
    user_id: &u64,
) -> String {
    crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Generating JWT token for client_id={}, persona_id={}, user_id={}", client_id, persona_id, user_id);
    
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // JWT Header for EA
    let mut header = HashMap::new();
    header.insert("kid", "ea-jwt-key-id".to_string());
    header.insert("alg", "RS256".to_string());
    header.insert("typ", "JWT".to_string());

    // Build full nexus structure
    let mut nexus = HashMap::new();
    
    // rsvd object
    let mut rsvd = HashMap::new();
    rsvd.insert("efplty", Value::String("13".to_string()));
    nexus.insert("rsvd", Value::Object(serde_json::Map::from_iter(
        rsvd.into_iter().map(|(k, v)| (k.to_string(), v))
    )));
    
    nexus.insert("cli", Value::String(client_id.to_string()));
    nexus.insert("prd", Value::String("5lkt".to_string()));
    nexus.insert("sco", Value::String("dp.friends.platforms.ea offline dp.client.default signin".to_string()));
    nexus.insert("pid", Value::String(persona_id.to_string()));
    nexus.insert("pty", Value::String("NUCLEUS".to_string()));
    nexus.insert("uid", Value::String(user_id.to_string()));
    
    // PSID - use session psid if available, otherwise calculate from persona_id
    // This ensures consistency with LSX session data
    use crate::session::get_user_session;
    let session = get_user_session();
    let psid_value = if session.psid != 0 {
        session.psid as u64
    } else {
        // Fallback: calculate from persona_id if session psid is 0
        (*persona_id as u64) % 1000000000
    };
    nexus.insert("psid", Value::Number(psid_value.into()));
    
    // Generate device ID (deterministic based on user_id and persona_id)
    let dvid_seed = (*user_id as u64).wrapping_mul(31).wrapping_add(*persona_id as u64);
    let dvid = format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", 
        (dvid_seed >> 32) as u32,
        (dvid_seed >> 16) as u16,
        dvid_seed as u16,
        ((dvid_seed >> 48) & 0xFFFF) as u16,
        dvid_seed);
    nexus.insert("dvid", Value::String(dvid));
    nexus.insert("pltyp", Value::String("PC".to_string()));
    nexus.insert("pnid", Value::String("EA".to_string()));
    nexus.insert("dpid", Value::String("PC".to_string()));
    nexus.insert("stps", Value::String("OFF".to_string()));
    nexus.insert("udg", Value::Bool(false));
    nexus.insert("cnty", Value::String("1".to_string()));
    nexus.insert("ausrc", Value::String("324320".to_string()));
    
    // ipgeo object
    let mut ipgeo = HashMap::new();
    ipgeo.insert("ip", Value::String("*.*.*.*".to_string()));
    ipgeo.insert("cty", Value::String("AU".to_string()));
    ipgeo.insert("reg", Value::String("New South Wales".to_string()));
    ipgeo.insert("cit", Value::String("Sydney".to_string()));
    ipgeo.insert("isp", Value::String("DMIT".to_string()));
    ipgeo.insert("lat", Value::String("3.0515".to_string()));
    ipgeo.insert("lgt", Value::String("-11.2707".to_string()));
    ipgeo.insert("tz", Value::String("-7".to_string()));
    nexus.insert("ipgeo", Value::Object(serde_json::Map::from_iter(
        ipgeo.into_iter().map(|(k, v)| (k.to_string(), v))
    )));
    
    // uif object
    let mut uif = HashMap::new();
    uif.insert("udg", Value::Bool(false));
    uif.insert("cty", Value::String("AU".to_string()));
    uif.insert("lan", Value::String("en".to_string()));
    uif.insert("sta", Value::String("ACTIVE".to_string()));
    uif.insert("ano", Value::Bool(false));
    uif.insert("age", Value::Number(23.into()));
    uif.insert("agp", Value::String("ADULT".to_string()));
    nexus.insert("uif", Value::Object(serde_json::Map::from_iter(
        uif.into_iter().map(|(k, v)| (k.to_string(), v))
    )));
    
    // psif array - persona information
    let mut psif_entry = HashMap::new();
    psif_entry.insert("id", Value::Number((*persona_id as u64).into()));
    psif_entry.insert("ns", Value::String("cem_ea_id".to_string()));
    psif_entry.insert("dis", Value::String(player_name.to_string()));
    psif_entry.insert("nic", Value::String(player_name.to_string()));
    nexus.insert("psif", Value::Array(vec![
        Value::Object(serde_json::Map::from_iter(
            psif_entry.into_iter().map(|(k, v)| (k.to_string(), v))
        ))
    ]));
    
    // enc - encrypted data (empty for now)
    nexus.insert("enc", Value::String("".to_string()));

    // JWT Payload for EA with full nexus structure
    let mut payload = HashMap::new();
    payload.insert("iss", "accounts.ea.com".to_string());
    payload.insert("jti", session_id.to_string());
    payload.insert("azp", client_id.to_string());
    payload.insert("iat", current_time.to_string());
    payload.insert("exp", (current_time + 259200).to_string()); // 3 days
    payload.insert("ver", "1".to_string());
    
    // Convert nexus HashMap to JSON Value
    let nexus_value = Value::Object(serde_json::Map::from_iter(
        nexus.into_iter().map(|(k, v)| (k.to_string(), v))
    ));
    
    // Build payload as JSON Value to properly serialize nested structures
    let mut payload_value = serde_json::Map::new();
    payload_value.insert("iss".to_string(), Value::String("accounts.ea.com".to_string()));
    payload_value.insert("jti".to_string(), Value::String(session_id.to_string()));
    payload_value.insert("azp".to_string(), Value::String(client_id.to_string()));
    payload_value.insert("iat".to_string(), Value::Number(current_time.into()));
    payload_value.insert("exp".to_string(), Value::Number((current_time + 259200).into()));
    payload_value.insert("ver".to_string(), Value::Number(1.into()));
    payload_value.insert("nexus".to_string(), nexus_value);

    // Encode header and payload
    let header_json = serde_json::to_string(&header).unwrap_or_else(|_| "{}".to_string());
    let payload_json = serde_json::to_string(&Value::Object(payload_value)).unwrap_or_else(|_| "{}".to_string());

    let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(header_json.as_bytes());
    let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    // Create a realistic-looking signature (not cryptographically valid but looks right)
    let signature_data = format!("{}.{}", header_b64, payload_b64);
    let signature_bytes = Sha256::digest(signature_data.as_bytes());
    let signature = general_purpose::URL_SAFE_NO_PAD.encode(&signature_bytes);

    // Combine into JWT
    format!("{}.{}.{}", header_b64, payload_b64, signature)
}

/// Generate refresh token JWT for EA authentication
pub fn generate_refresh_token_jwt(
    session_id: &str,
    _persona_id: &u64,
    _player_name: &str,
    client_id: &str,
    user_id: &u64,
) -> String {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // JWT Header for refresh token (different kid for refresh tokens)
    let mut header = HashMap::new();
    header.insert("kid", "d8ae8876-e20e-4e9e-b5c7-795d7a791149".to_string());
    header.insert("alg", "RS256".to_string());
    header.insert("typ", "JWT".to_string());

    // Refresh token payload - simpler structure, longer expiration
    let mut payload_value = serde_json::Map::new();
    payload_value.insert("iss".to_string(), Value::String("accounts.ea.com".to_string()));
    payload_value.insert("jti".to_string(), Value::String(format!("RT_{}", session_id)));
    payload_value.insert("azp".to_string(), Value::String(client_id.to_string()));
    payload_value.insert("iat".to_string(), Value::Number(current_time.into()));
    payload_value.insert("exp".to_string(), Value::Number((current_time + 2592000).into())); // 30 days
    payload_value.insert("ver".to_string(), Value::Number(1.into()));
    
    // Nexus structure for refresh token
    let mut nexus = HashMap::new();
    nexus.insert("uid", Value::String(user_id.to_string()));
    // enc field contains encrypted data (simplified for refresh token)
    nexus.insert("enc", Value::String("".to_string()));
    
    payload_value.insert("nexus".to_string(), Value::Object(serde_json::Map::from_iter(
        nexus.into_iter().map(|(k, v)| (k.to_string(), v))
    )));

    // Encode header and payload
    let header_json = serde_json::to_string(&header).unwrap_or_else(|_| "{}".to_string());
    let payload_json = serde_json::to_string(&Value::Object(payload_value)).unwrap_or_else(|_| "{}".to_string());

    let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(header_json.as_bytes());
    let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    // Create a realistic-looking signature
    let signature_data = format!("{}.{}", header_b64, payload_b64);
    let signature_bytes = Sha256::digest(signature_data.as_bytes());
    let signature = general_purpose::URL_SAFE_NO_PAD.encode(&signature_bytes);

    // Combine into JWT
    format!("{}.{}.{}", header_b64, payload_b64, signature)
}

