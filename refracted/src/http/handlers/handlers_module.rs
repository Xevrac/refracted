use crate::common::error::BlazeResult;
use crate::core::inspector::inspector_module::*;
use crate::grpc::grpc_handler::*;
use crate::grpc::grpc_body_decode_capture;
use crate::grpc::grpc_frame::*;
use crate::grpc::grpc_protobuf::*;
use crate::jwt::{
    generate_ea_jwt_token, generate_jwt_token, generate_refresh_token_jwt, NEXUS_GATEWAY_CLIENT_ID,
};
use indexmap::IndexMap;
use parking_lot::Mutex as CompactGrpcMutex;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// HTTP request handler for EA services
#[derive(Clone)]
pub struct HttpHandler {
    // Auth data from packet capture
    #[allow(dead_code)]
    auth_token: String,
    session_id: String,
    #[allow(dead_code)]
    user_id: u64,
    persona_id: u64,
    #[allow(dead_code)]
    username: String,
    player_name: String,
    #[allow(dead_code)]
    steam_id: String,
    jwt_token: String,
    #[allow(dead_code)]
    access_token: String,
    #[allow(dead_code)]
    refresh_token: String,
}

struct GrpcRequestEntry {
    count: u32,
}

struct GrpcRequestLogState {
    entries: HashMap<String, GrpcRequestEntry>,
}

fn grpc_request_log_state() -> &'static CompactGrpcMutex<GrpcRequestLogState> {
    static STATE: OnceLock<CompactGrpcMutex<GrpcRequestLogState>> = OnceLock::new();
    STATE.get_or_init(|| {
        CompactGrpcMutex::new(GrpcRequestLogState {
            entries: HashMap::new(),
        })
    })
}

fn milestone_once_flags() -> &'static Mutex<HashSet<&'static str>> {
    static FLAGS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    FLAGS.get_or_init(|| Mutex::new(HashSet::new()))
}

impl HttpHandler {
    /// Clear per-endpoint gRPC counters when a new HTTP/2 connection is accepted (fresh TLS session).
    pub fn flush_grpc_compact_log_on_new_http2_connection() {
        grpc_request_log_state().lock().entries.clear();
    }

    fn log_grpc_request_compact(&self, method: &str, path: &str, host: &str, _body_len: usize) {
        let normalized_path = path.split('?').next().unwrap_or(path);
        let display_path = if normalized_path.len() > 80 {
            format!("{}...", &normalized_path[..77])
        } else {
            normalized_path.to_string()
        };
        let key = format!("{} {} (host: {})", method, normalized_path, host);
        let line = format!("{} {} (host: {})", method, display_path, host);

        let count = {
            let mut state = grpc_request_log_state().lock();
            let e = state
                .entries
                .entry(key.clone())
                .or_insert(GrpcRequestEntry { count: 0 });
            e.count = e.count.saturating_add(1);
            e.count
        };

        let ansi = format!(
            "\x1b[38;2;0;200;255m[gRPC]\x1b[0m {} \x1b[38;2;140;140;140mx{}\x1b[0m",
            line,
            count.max(1)
        );
        crate::core::console::push_grpc_compact_upsert(key, &ansi);
    }

    fn log_milestone_once(flag: &'static str, message: &str) {
        let Ok(mut flags) = milestone_once_flags().lock() else {
            return;
        };
        if !flags.insert(flag) {
            return;
        }
        drop(flags);
        crate::console_println!("\x1b[38;2;255;215;0m[MILESTONE]\x1b[0m {}", message);
    }

    fn load_photon_bundle_runtime(requested_name: Option<&str>) -> Option<Vec<u8>> {
        let data_js_dir = crate::client::labs::photon_js_runtime_dir();

        if let Some(name) = requested_name {
            let requested_path = data_js_dir.join(name);
            if let Ok(bytes) = std::fs::read(requested_path) {
                return Some(bytes);
            }
        }

        let entries = std::fs::read_dir(&data_js_dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with("photon-bundle-") && name.ends_with(".js") {
                if let Ok(bytes) = std::fs::read(path) {
                    return Some(bytes);
                }
            }
        }
        None
    }

    fn extract_query_param<'a>(path: &'a str, key: &str) -> Option<&'a str> {
        let (_, query) = path.split_once('?')?;
        for pair in query.split('&') {
            let (k, v) = pair.split_once('=')?;
            if k == key {
                return Some(v);
            }
        }
        None
    }

    pub fn new() -> Self {
        let auth_token =
            "000000030d1fbb88_zBW34oENCBoLOxxYHf6NRG63u1k8BPIAM$H10*JpXmc=".to_string();
        let session_id =
            "000000030d1fbb88_zBW34oENCBoLOxxYHf6NRG63u1k8BPIAM$H10*JpXmc=".to_string();
        let user_id = 1012711274866;
        let persona_id = 1016820078927;
        let username = "SickSir#3YG6QzrzjmMDCGH".to_string();
        let player_name = "Xevrac".to_string();
        let steam_id = "76561198036565655".to_string();

        let jwt_token = generate_jwt_token(&session_id, &persona_id, &player_name, "GLACIER_LABS_STEAM_CLIENT");
        let access_token = generate_jwt_token(&session_id, &persona_id, &player_name, "GLACIER_LABS_STEAM_CLIENT");
        let refresh_token = format!("RT_{}", session_id);

        Self {
            auth_token,
            session_id,
            user_id,
            persona_id,
            username,
            player_name,
            steam_id,
            jwt_token,
            access_token,
            refresh_token,
        }
    }


    /// Check if a path is a GetAuthForToken request
    fn is_get_auth_for_token(&self, path: &str) -> bool {
        path.contains("GetAuthForToken")
            || path.contains("AuthServiceServicer")
            || path.contains("AuthService/GetAuth")
            || path.contains("AuthService:GetAuth")
            || path.contains("/eadp.nexus.connect.grpc.v1.AuthService/GetAuthForToken")
            || path.contains("eadp.nexus.connect.grpc.v1.AuthService")
            || (path.contains("AuthService") && (path.contains("GetAuth") || path.contains("Token")))
            || (path.contains("Auth") && path.contains("Token") && path.contains("Get"))
    }

    /// Santiago `ClientAuthentication:viaAuthCode`.
    fn is_via_auth_code_path(&self, path: &str) -> bool {
        path.contains("viaAuthCode")
    }

    /// Check if a request is a gRPC request
    fn is_grpc_request(&self, host: &str, path: &str) -> bool {
        host.contains("grpc.ea.com")
            || host.contains(".dice.se")
            || path.contains("/eadp.")
            || path.contains("/santiago.")
    }

    /// Route request to appropriate handler based on domain
    fn route_by_domain(
        &self,
        host: &str,
        path: &str,
        method: &str,
        body: &[u8],
    ) -> Option<BlazeResult<HttpResponse>> {
        // Priority 1: GetAuthForToken on any domain (critical for authentication flow)
        if self.is_get_auth_for_token(path) {
            return Some(self.handle_accounts_grpc(path, method, body, host));
        }

        // Santiago auth on any Dice ops gateway (santiago-prod-mp-cgw, eventprod-mp-cgw, bflabs, …)
        if host.contains(".dice.se") && self.is_via_auth_code_path(path) {
            crate::console_println!(
                "\x1b[38;2;255;215;0m[gRPC]\x1b[0m viaAuthCode (Dice gateway): {}",
                path
            );
            return Some(self.handle_santiago_auth_via_auth_code(body));
        }

        // Priority 2: Domain-specific routing
        if host == "gcs.ea.com" {
            return Some(self.handle_gcs(path, method, body));
        }

        // Check for GrantTokenByAuthorizationCode on ANY host (critical for auth flow)
        if path.contains("GrantTokenByAuthorizationCode") || path.contains("TokenServiceServicer") {
            return Some(self.handle_accounts_grpc(path, method, body, host));
        }

        if host.contains("accounts.grpc.ea.com") {
            return Some(self.handle_accounts_grpc(path, method, body, host));
        }

        if host.contains("gateway.grpc.ea.com") {
            if self.is_via_auth_code_path(path) {
                crate::console_println!(
                    "\x1b[38;2;255;215;0m[gRPC]\x1b[0m viaAuthCode (gateway.grpc.ea.com): {}",
                    path
                );
                return Some(self.handle_santiago_auth_via_auth_code(body));
            }
            return Some(self.handle_gateway_grpc(path, method, body));
        }

        if host.contains("bflabs-prod-gt-cgw.ops.dice.se") {
            return Some(self.handle_santiago_services(path, method, body));
        }

        if host.contains("bflabs-prod-eventbridge.ops.dice.se") {
            return Some(self.handle_eventbridge(path, method, body));
        }

        if host.contains("collector.errors.ea.com") {
            return Some(self.handle_collector_errors(path, method, body));
        }

        if host.contains("reports.tools.gos.ea.com") || host.contains("tools.gos.ea.com") {
            return Some(self.handle_collector_errors(path, method, body));
        }

        if host.contains("api.k.social.ea.com") {
            return Some(self.handle_social_api(path, method, body));
        }

        if host.contains("data.ea.com") || host.contains("freeform-river.data.ea.com") {
            return Some(self.handle_telemetry(path, method, body));
        }

        if host.contains("qoscoordinator.gameservices.ea.com") {
            return Some(self.handle_qos_coordinator(path, method, body));
        }

        if host.contains("update.layer.ea.com") {
            return Some(self.handle_update_layer(path, method, body));
        }

        if host.contains("tos.ea.com") {
            return Some(self.handle_tos(path, method, body));
        }

        // Redirector handling (multiple patterns)
        if host.contains("redirector") {
            if host.contains("spring25") {
                return Some(self.handle_spring25_redirector(path, method, body));
            } else if host.contains("spring18") || host.contains("gosredirector") {
                return Some(self.handle_spring18_redirector(path, method, body));
            }
        }

        None
    }

    /// Handle HTTP request based on host and path
    pub fn handle_request(
        &self,
        host: &str,
        path: &str,
        method: &str,
        body: &[u8],
    ) -> BlazeResult<HttpResponse> {
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m handle_request entered - Method: {}, Path: {}, Host: {}, Body size: {}", method, path, host, body.len());

        // Handle HTTP/2 connection preface (PRI *)
        if method == "PRI" && path == "*" {
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m HTTP/2 connection preface detected, returning empty response");
            return Ok(HttpResponse::new(
                200,
                "application/json",
                b"{}".to_vec(),
            ));
        }

        // Log gRPC requests
        if self.is_grpc_request(host, path) || host.contains("accounts.grpc.ea.com") || host.contains(".dice.se") || host.contains("gateway.grpc.ea.com") {
            if self.is_get_auth_for_token(path) {
                self.log_grpc_request_compact(method, path, host, body.len());
                crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m GetAuthForToken request routed to accounts handler");
            } else {
                self.log_grpc_request_compact(method, path, host, body.len());
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m gRPC request detected, routing by domain");
            }
        }

        // Capture request for inspector
        let is_grpc = self.is_grpc_request(host, path) || host.contains(".grpc.") || path.contains("/grpc.");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        
        if is_grpc {
            // Capture gRPC request
            let cap = grpc_body_decode_capture(body);

            let grpc_capture = CapturedGrpc {
                capture_seq: 0,
                timestamp,
                direction: GrpcDirection::ClientToServer,
                method: method.to_string(),
                path: path.to_string(),
                host: host.to_string(),
                headers: Vec::new(), // Will be captured from request if available
                body_size: body.len(),
                body: body.to_vec(),
                protobuf_data: cap.protobuf_chunks.first().cloned(),
                protobuf_chunks: cap.protobuf_chunks,
                is_compressed: cap.any_frame_was_compressed,
                grpc_status: None,
            };
            capture_grpc(grpc_capture);
        } else {
            // Capture HTTP request
            let http_capture = CapturedHttp {
                capture_seq: 0,
                timestamp,
                direction: HttpDirection::ClientToServer,
                method: method.to_string(),
                path: path.to_string(),
                host: host.to_string(),
                headers: Vec::new(), // Will be captured from request if available
                body_size: body.len(),
                body: body.to_vec(),
                status_code: None,
            };
            capture_http(http_capture);
        }

        if let Some(response) = crate::client::cnc::try_handle_cnc_post(method, path, body) {
            response.capture_response(method, path, host, is_grpc);
            return Ok(response);
        }

        if let Some(response) = crate::client::cnc::try_handle_http_request(method, path) {
            response.capture_response(method, path, host, is_grpc);
            return Ok(response);
        }

        let full_url = format!("https://{}{}", host, path);
        if let Some(resp) = crate::client::labs::try_load_captured_response(&full_url, body) {
            let is_grpc = self.is_grpc_request(host, path)
                || host.contains(".grpc.")
                || path.contains("/grpc.");
            resp.capture_response(method, path, host, is_grpc);
            return Ok(resp);
        } else if host.contains(".ops.dice.se")
            && (path.contains("ClientLocalization/getTranslations")
                || path.contains("UnifiedMessaging/fetchActions")
                || path.contains("ClientMenu/getScheduledMenu")
                || path.contains("ClientMenu/getMenuUpdates")
                || path.contains("ClientMenu/getStoreMenu"))
        {
            crate::console_println!(
                "\x1b[38;2;255;200;120m[gRPC]\x1b[0m Labs capture miss: {}{}",
                host,
                path
            );
        }

        // Try domain-based routing first
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Attempting domain-based routing");
        if let Some(result) = self.route_by_domain(host, path, method, body) {
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Domain-based routing matched, returning result");
            
            // Capture response and return
            let response = result?;
            response.capture_response(method, path, host, is_grpc);
            
            return Ok(response);
        }
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Domain-based routing did not match");

        // Fallback: Check for unhandled gRPC requests (especially GetAuthForToken)
        if self.is_grpc_request(host, path) && self.is_get_auth_for_token(path) {
            crate::debug_println!(
                "\x1b[38;2;255;215;0m[gRPC]\x1b[0m GetAuthForToken on unhandled host={} path={}",
                host, path
            );
            let result = self.handle_accounts_grpc(path, method, body, host)?;
            result.capture_response(method, path, host, true);
            return Ok(result);
        }

        if self.is_grpc_request(host, path) && self.is_via_auth_code_path(path) {
            crate::console_println!(
                "\x1b[38;2;255;215;0m[gRPC]\x1b[0m viaAuthCode (fallback host): {} {}",
                host,
                path
            );
            let result = self.handle_santiago_auth_via_auth_code(body)?;
            result.capture_response(method, path, host, true);
            return Ok(result);
        }
        
        // CRITICAL: Catch any request that might be GetAuthForToken but didn't match earlier
        // This is a last resort catch-all for authentication requests
        if (host.contains("accounts") || host.contains("grpc") || host.contains("auth")) 
            && (path.contains("Auth") || path.contains("Token") || path.contains("GetAuth")) {
            crate::console_println!(
                "\x1b[38;2;255;165;0m[gRPC-CATCHALL]\x1b[0m Potential auth request on host: {} path: {} (body: {} bytes)",
                host, path, body.len()
            );
            // Try to handle it as GetAuthForToken
            if self.is_get_auth_for_token(path) || (path.contains("Auth") && path.contains("Token")) {
                crate::console_println!(
                    "\x1b[38;2;255;215;0m[gRPC-CATCHALL]\x1b[0m Treating as GetAuthForToken: {} {}",
                    host, path
                );
                let result = self.handle_accounts_grpc(path, method, body, host)?;
                result.capture_response(method, path, host, true);
                return Ok(result);
            }
        }

        // Default response for unhandled requests
        let response = HttpResponse::new(
            200,
            "application/json",
            b"{\"status\":\"ok\"}".to_vec(),
        );
        response.capture_response(method, path, host, is_grpc);
        Ok(response)
    }

// This is alt. instance info for response for client request. Testing shows no behaviour changes between either.
//
//     /// Handle spring25 redirector requests
//     /// Returns server instance info that the client uses to get the Blaze server address
//     fn handle_spring25_redirector(
//         &self,
//         path: &str,
//         _method: &str,
//         _body: &[u8],
//     ) -> BlazeResult<HttpResponse> {
//         if path.contains("getServerInstance") {
//             let response_body = r#"<?xml version="1.0" encoding="UTF-8"?>
// <serverinstanceinfo>
//     <address member="0">
//         <valu>
//             <hostname>ext-127-0-0-1.blaze.ea.com</hostname>
//             <ip>66835934</ip>
//             <port>10042</port>
//         </valu>
//     </address>
//     <secure>1</secure>
//     <trialservicename></trialservicename>
//     <defaultdnsaddress>0</defaultdnsaddress>
// </serverinstanceinfo>"#;

//             let mut headers = HashMap::new();
//             headers.insert("X-BLAZE-COMPONENT".to_string(), "redirector".to_string());
//             headers.insert(
//                 "X-BLAZE-COMMAND".to_string(),
//                 "getServerInstance".to_string(),
//             );
//             headers.insert("X-BLAZE-SEQNO".to_string(), "0".to_string());
//             headers.insert("Connection".to_string(), "close".to_string());

//             Ok(HttpResponse::new_with_headers(
//                 200,
//                 "application/xml",
//                 response_body.as_bytes().to_vec(),
//                 headers,
//             ))
//         } else {
//             // Return 404 for other spring25 redirector requests
//             Ok(HttpResponse::new(
//                 404,
//                 "text/html",
//                 b"<html><body>Negative caching 404</html>".to_vec(),
//             ))
//         }
//     }

//     /// Handle spring18 redirector requests
//     fn handle_spring18_redirector(
//         &self,
//         path: &str,
//         _method: &str,
//         _body: &[u8],
//     ) -> BlazeResult<HttpResponse> {
//         if path.contains("getServerInstance") {
//             let response_body = r#"<?xml version="1.0" encoding="UTF-8"?>
// <serverinstanceinfo>
//     <address member="0">
//         <valu>
//             <hostname>ext-127-0-0-1.blaze.ea.com</hostname>
//             <ip>66835934</ip>
//             <port>10040</port>
//         </valu>
//     </address>
//     <secure>1</secure>
//     <trialservicename></trialservicename>
//     <defaultdnsaddress>0</defaultdnsaddress>
// </serverinstanceinfo>"#;

//             let mut headers = HashMap::new();
//             headers.insert("X-BLAZE-COMPONENT".to_string(), "redirector".to_string());
//             headers.insert(
//                 "X-BLAZE-COMMAND".to_string(),
//                 "getServerInstance".to_string(),
//             );
//             headers.insert("X-BLAZE-SEQNO".to_string(), "0".to_string());
//             headers.insert("Connection".to_string(), "close".to_string());

//             Ok(HttpResponse::new_with_headers(
//                 200,
//                 "application/xml",
//                 response_body.as_bytes().to_vec(),
//                 headers,
//             ))
//         } else {
//             Ok(HttpResponse::new(
//                 404,
//                 "text/html",
//                 b"<html><body>Negative caching 404</html>".to_vec(),
//             ))
//         }
//     }

    /// Handle spring25 redirector requests
    /// Returns server instance info that the client uses to get the Blaze server address
    fn handle_spring25_redirector(
        &self,
        path: &str,
        _method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {
        if path.contains("getServerInstance") {
            let blaze_main = crate::common::game::current_service_ports().blaze_main;
            let response_body = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<serverinstanceinfo>
    <address member="0">
        <valu>
            <hostname>127.0.0.1</hostname>
            <ip>2130706433</ip>
            <port>{blaze_main}</port>
        </valu>
    </address>
    <secure>1</secure>
    <trialservicename></trialservicename>
    <defaultdnsaddress>0</defaultdnsaddress>
</serverinstanceinfo>"#
            );

            let mut headers = HashMap::new();
            headers.insert("X-BLAZE-COMPONENT".to_string(), "redirector".to_string());
            headers.insert(
                "X-BLAZE-COMMAND".to_string(),
                "getServerInstance".to_string(),
            );
            headers.insert("X-BLAZE-SEQNO".to_string(), "0".to_string());
            headers.insert("Connection".to_string(), "close".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/xml",
                response_body.as_bytes().to_vec(),
                headers,
            ))
        } else {
            // Return 404 for other spring25 redirector requests
            Ok(HttpResponse::new(
                404,
                "text/html",
                b"<html><body>Negative caching 404</html>".to_vec(),
            ))
        }
    }

    /// Handle spring18 redirector requests
    fn handle_spring18_redirector(
        &self,
        path: &str,
        _method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {
        if path.contains("getServerInstance") {
            let blaze_main = crate::common::game::current_service_ports().blaze_main;
            let response_body = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<serverinstanceinfo>
    <address member="0">
        <valu>
            <hostname>127.0.0.1</hostname>
            <ip>2130706433</ip>
            <port>{blaze_main}</port>
        </valu>
    </address>
    <secure>0</secure>
    <trialservicename></trialservicename>
    <defaultdnsaddress>0</defaultdnsaddress>
</serverinstanceinfo>"#
            );

            let mut headers = HashMap::new();
            headers.insert("X-BLAZE-COMPONENT".to_string(), "redirector".to_string());
            headers.insert(
                "X-BLAZE-COMMAND".to_string(),
                "getServerInstance".to_string(),
            );
            headers.insert("X-BLAZE-SEQNO".to_string(), "0".to_string());
            headers.insert("Connection".to_string(), "close".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/xml",
                response_body.as_bytes().to_vec(),
                headers,
            ))
        } else {
            Ok(HttpResponse::new(
                404,
                "text/html",
                b"<html><body>Negative caching 404</html>".to_vec(),
            ))
        }
    }

    /// Handle GCS (Global Configuration Service) requests
    fn handle_gcs(&self, _path: &str, _method: &str, _body: &[u8]) -> BlazeResult<HttpResponse> {
        // Working clients GET .../application_id/.../device_id/<id> — must not 404 here or boot stalls waiting on GCS.
        let config = self.build_gcs_response();
        Ok(HttpResponse::new(200, "application/json", config))
    }

    /// Build GCS response
    fn build_gcs_response(&self) -> Vec<u8> {
        // Use IndexMap to maintain insertion order for consistent DNS lookup order
        // This ensures the client encounters hostnames in the correct order
        let mut config = IndexMap::new();

        // Order 1-3: Basic services (collector, gcs, freeform-river handled by DNS redirection, not GCS)
        // Instrumentation service - comes early
        config.insert(
            "eadp.instrumentation.service".to_string(),
            "https://freeform-river.data.ea.com".to_string(),
        );
        
        // Order 4: update.layer.ea.com - must come BEFORE accounts.grpc
        config.insert(
            "eadp.update.layer".to_string(),
            "https://update.layer.ea.com".to_string(),
        );
        
        // Order 5: accounts.grpc.ea.com - comes AFTER update.layer
        config.insert("eadp.nexus.connect.grpc.v1".to_string(), "https://accounts.grpc.ea.com".to_string());
        // AuthService for GetAuthForToken ("AuthServiceServicer:GetAuthForToken")
        config.insert("eadp.nexus.connect.grpc.v1.AuthService".to_string(), "https://accounts.grpc.ea.com".to_string());
        config.insert(
            "eadp.identity.v1.AuthenticationService".to_string(),
            "https://accounts.grpc.ea.com".to_string(),
        );
        config.insert(
            "eadp.identity.v1.TokenService".to_string(),
            "https://accounts.grpc.ea.com".to_string(),
        );
        
        // Order 6: spring25 redirector - comes AFTER accounts.grpc
        config.insert("eadp.redirector.hostname".to_string(), "spring25.client.blazeredirector.ea.com".to_string());
        config.insert("eadp.redirector.primary".to_string(), "spring25.client.blazeredirector.ea.com".to_string());
        config.insert("eadp.redirector.fallback".to_string(), "spring18.gosredirector.ea.com".to_string());

        // New keys the game is looking for
        config.insert("eadp.friends.notifications".to_string(), "https://api.k.social.ea.com".to_string());
        config.insert("eadp.friends.v1".to_string(), "https://api.k.social.ea.com".to_string());
        config.insert("eadp.social.presence.v1".to_string(), "https://api.k.social.ea.com".to_string());

        // Keep the other services too
        config.insert("eadp.identity.v1.IdentityService".to_string(), "https://127.0.0.1:443".to_string());
        // Duplicate TokenService URL omitted here: `TokenService` is already set above; repeating the key would change client discovery order.
        config.insert(
            "eadp.social.presence.v1.PresenceService".to_string(),
            "https://api.k.social.ea.com".to_string(),
        );
        config.insert(
            "eadp.social.friends.v1.FriendsService".to_string(),
            "https://api.k.social.ea.com".to_string(),
        );
        config.insert(
            "eadp.feeds.reader.v1.ReaderService".to_string(),
            "https://api.k.social.ea.com".to_string(),
        );
        config.insert(
            "eadp.errors.v1.CollectorService".to_string(),
            "https://collector.errors.ea.com".to_string(),
        );
        config.insert(
            "eadp.friends.v2.FriendsNotificationsService".to_string(),
            "https://api.k.social.ea.com".to_string(),
        );

        config.insert("eadp.social.privacy.v1".to_string(), "https://api.k.social.ea.com".to_string());
        config.insert("eadp.feeds.reader.v1".to_string(), "https://api.k.social.ea.com".to_string());
        config.insert("eadp.chat.tcp".to_string(), "ssl://chat.ea.com:8095".to_string());
        config.insert("eadp.playercard.v1".to_string(), "https://api.k.social.ea.com".to_string());
        config.insert("eadp.identity".to_string(), "https://accounts.ea.com".to_string());
        config.insert("eadp.identity.proxy".to_string(), "https://gateway.ea.com/proxy".to_string());
        config.insert("eadp.auth.account".to_string(), "https://signin.ea.com/".to_string());
        config.insert("eadp.stats".to_string(), "https://stats.gameservices.ea.com:11000".to_string());
        config.insert(
            "eadp.leaderboards".to_string(),
            "https://leaderboards.gameservices.ea.com:11000".to_string(),
        );
        config.insert(
            "eadp.leaderboards.v2".to_string(),
            "https://leaderboards-api-ext.leaderboards.ea.com:443".to_string(),
        );
        config.insert("eadp.pushnotification".to_string(), "https://pn.tnt-ea.com".to_string());
        config.insert("eadp.realtimemessaging".to_string(), "https://rtm.tnt-ea.com:9000".to_string());
        config.insert("eadp.authentication.useJwtToken".to_string(), "false".to_string());
        config.insert(
            "eadp.candi.offer.service".to_string(),
            "https://gateway.grpc.ea.com:443".to_string(),
        );
        config.insert(
            "eadp.candi.catalog.service".to_string(),
            "https://gateway.grpc.ea.com:443".to_string(),
        );
        config.insert(
            "eadp.candi.entitlement.v2.service".to_string(),
            "https://gateway.grpc.ea.com:443".to_string(),
        );
        config.insert("eadp.pin".to_string(), "https://pin-river-grpc.data.ea.com:443".to_string());
        config.insert(
            "eadp.experimentation.grouping.v1".to_string(),
            "https://experimentation-grpc.data.ea.com".to_string(),
        );

        // Convert IndexMap to JSON, maintaining insertion order
        serde_json::to_string_pretty(&config).unwrap().into_bytes()
    }

    /// Handle accounts gRPC requests
    fn handle_accounts_grpc(
        &self,
        path: &str,
        method: &str,
        body: &[u8],
        host: &str,
    ) -> BlazeResult<HttpResponse> {
        // Log all accounts gRPC requests for debugging
        crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m POST {} (body size: {} bytes)", path, body.len());
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m [ACCOUNTS-gRPC] Path: '{}', Body: {} bytes, First 100 bytes: {:?}", 
            path, body.len(), 
            if body.len() > 100 { format!("{:?}", &body[..100]) } else { format!("{:?}", body) });
        
        // Path formats:
        //   - TokenServiceServicer:GrantTokenByAuthorizationCode
        //   - /eadp.identity.v1.TokenService/GrantTokenByAuthorizationCode
        //   - /TokenService/GrantTokenByAuthorizationCode
        let is_grant_token = path.contains("GrantTokenByAuthorizationCode")
            || path.contains("TokenServiceServicer")
            || (path.contains("TokenService") && (path.contains("GrantToken") || path.contains("AuthorizationCode")))
            || (path.contains("Token") && path.contains("AuthorizationCode") && path.contains("Grant"));
        
        if is_grant_token {
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m *** GrantTokenByAuthorizationCode Detected: {} ***", path);
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m GrantTokenByAuthorizationCode handler entered, body size: {}", body.len());

            let build_profile = crate::session::session_module::get_build_profile();
            let (token_response, grpc_response, response_headers) = if build_profile
                == crate::session::session_module::BuildProfile::LabsAlpha
            {
                // Legacy Labs builds are stricter about grant payload/framing.
                let token_response = self.build_token_grant_response();
                let mut headers = HashMap::new();
                headers.insert("grpc-status".to_string(), "0".to_string());
                let grpc_response = self.wrap_grpc_response(&token_response);
                (token_response, grpc_response, headers)
            } else {
                // EA token exchange request - signed JWT path.
                let token_response = self.build_ea_token_grant_response();
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Building gRPC frame with compression");
                let mut request_headers = HashMap::new();
                request_headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
                let (grpc_response, response_headers) = build_grpc_response(&token_response, &request_headers)
                    .unwrap_or_else(|e| {
                        crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to build gRPC response: {}, using fallback", e);
                        (self.wrap_grpc_response(&token_response), self.create_grpc_auth_headers())
                    });
                (token_response, grpc_response, response_headers)
            };
            
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT token generated successfully!");
            crate::debug_println!(
                "\x1b[38;2;0;200;255m[gRPC]\x1b[0m Token response ready (payload: {}, frame: {})",
                token_response.len(),
                grpc_response.len()
            );

            let response = HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                response_headers,
            );
            response.capture_response(method, path, host, true);
            return Ok(response);
        }
        
        // Path formats: 
        //   - /eadp.nexus.connect.grpc.v1.AuthService/GetAuthForToken
        //   - /eadp.nexus.connect.grpc.v1.AuthService:GetAuthForToken
        //   - AuthServiceServicer:GetAuthForToken
        //   - Any path containing "Auth" and "Token" (catch-all for variations)
        // NOTE: Must NOT match GrantTokenByAuthorizationCode (already handled above)
        let is_get_auth_for_token = (path.contains("GetAuthForToken") && !path.contains("GrantTokenByAuthorizationCode"))
            || (path.contains("AuthServiceServicer") && !path.contains("TokenServiceServicer"))
            || path.contains("AuthService/GetAuth")
            || path.contains("AuthService:GetAuth")
            || path.contains("eadp.nexus.connect.grpc.v1.AuthService")
            || (path.contains("AuthService") && (path.contains("GetAuth") || path.contains("Token")) && !path.contains("GrantToken"));
        
        if is_get_auth_for_token {
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m GetAuthForToken detected: {}", path);
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m GetAuthForToken handler entered, body size: {}", body.len());
            
            // Parse gRPC frame and extract JWT token from request body.
            // Try multiple extraction methods.
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m Parsing gRPC frame from request body");
            let jwt_token = if let Ok((_, protobuf_data)) = parse_grpc_frame(body) {
                crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m gRPC frame parsed, extracting JWT from protobuf");
                // Only accept values that look like JWTs; field 6 may contain non-token values.
                extract_string_field(&protobuf_data, 6)
                    .filter(|candidate| self.looks_like_jwt(candidate))
                    .or_else(|| {
                        self.extract_protobuf_nested_string(&protobuf_data, 1, 1)
                            .filter(|candidate| self.looks_like_jwt(candidate))
                    })
                    .unwrap_or_else(|| self.extract_jwt_from_request(&protobuf_data))
            } else {
                crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to parse gRPC frame, using old extraction method");
                self.extract_jwt_from_request(body)
            };
            
            if jwt_token.is_empty() {
                crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m WARN: No JWT token found in GetAuthForToken request - using session state");
                crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Request body (first 200 bytes): {:?}", &body[..body.len().min(200)]);
                crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m JWT extraction failed, will use session state");
            } else {
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted for GetAuthForToken (len={})", jwt_token.len());
            }
            
            // GetAuthForTokenResponse uses repeated Code messages (nic:UserID:PersonalID), not the alternate UUID+JWT struct layout.
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m Building GetAuthForToken response (Code list format)");
            if !jwt_token.is_empty() {
                crate::debug_println!(
                    "\x1b[38;2;0;200;255m[gRPC]\x1b[0m GetAuthForToken JWT present (len={}), building Code response",
                    jwt_token.len()
                );
            }
            let auth_response = self.build_get_auth_for_token_response(if jwt_token.is_empty() {
                None
            } else {
                Some(jwt_token.as_str())
            });
            
            let build_profile = crate::session::session_module::get_build_profile();
            let (grpc_response, response_headers) = if build_profile
                == crate::session::session_module::BuildProfile::LabsAlpha
            {
                // Older Labs clients are sensitive to compressed auth frames.
                let mut headers = HashMap::new();
                headers.insert("grpc-status".to_string(), "0".to_string());
                (self.wrap_grpc_response(&auth_response), headers)
            } else {
                crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m Building gRPC frame with compression");
                let mut request_headers = HashMap::new();
                request_headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
                build_grpc_response(&auth_response, &request_headers).unwrap_or_else(|e| {
                    crate::debug_println!(
                        "\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to build gRPC response: {}, using fallback",
                        e
                    );
                    (self.wrap_grpc_response(&auth_response), self.create_grpc_auth_headers())
                })
            };
            
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GetAuthForToken response: {} bytes", grpc_response.len());
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m GetAuthForToken response ready (size: {})", grpc_response.len());
            let response = HttpResponse::new_with_headers(200, "application/grpc", grpc_response, response_headers);
            response.capture_response(method, path, host, true);
            return Ok(response);
        }
        
        if path.contains("GetAuthForSteamClient")
            || path.contains("Login")
            || path.contains("Authenticate")
        {
            // Build JWT token response
            let auth_data = self.build_auth_response_for_steam_client();
            let grpc_response = self.wrap_grpc_response(&auth_data);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else if path.contains("GrantTokenByAuthorizationCode") && !path.contains("TokenServiceServicer") {
            crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m WARN: GrantTokenByAuthorizationCode matched in fallback (path: {})", path);
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GrantTokenByAuthorizationCode called..");
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Generating JWT token for Blaze auth..");
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GrantTokenByAuthorizationCode handler entered, body size: {}", body.len());

            // EA token exchange request - needs signed JWT
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Building EA token grant response");
            let token_response = self.build_ea_token_grant_response();
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Building gRPC frame with compression");
            let mut request_headers = HashMap::new();
            request_headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
            let (grpc_response, response_headers) = build_grpc_response(&token_response, &request_headers)
                .unwrap_or_else(|e| {
                    crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to build gRPC response: {}, using fallback", e);
                    (self.wrap_grpc_response(&token_response), self.create_grpc_auth_headers())
                });
            
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT token generated successfully!");
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Token response ready (size: {})", grpc_response.len());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                response_headers,
            ))
        } else if path.contains("viaAuthCode") || 
                (path.contains("ClientAuthentication") && path.contains("viaAuthCode")) {
            crate::console_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m viaAuthCode on accounts host: {}", path);
            // Use the same handler as gateway_grpc
            return self.handle_santiago_auth_via_auth_code(body);
        }
        else if path.contains("GrantTokenByRefreshToken") {
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GrantTokenByRefreshToken — issuing tokens (same envelope as auth code)");
            let token_response = self.build_ea_token_grant_response();
            let mut request_headers = HashMap::new();
            request_headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
            let (grpc_response, response_headers) = build_grpc_response(&token_response, &request_headers)
                .unwrap_or_else(|e| {
                    crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m GrantTokenByRefreshToken build_grpc_response failed: {}, fallback", e);
                    (self.wrap_grpc_response(&token_response), self.create_grpc_auth_headers())
                });
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GrantTokenByRefreshToken response size: {}", grpc_response.len());
            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                response_headers,
            ))
        } else if path.contains("DeleteToken") {
            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m DeleteToken — acknowledged");
            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());
            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                self.wrap_grpc_response(&[]),
                headers,
            ))
        } else if path.contains("GetTokenInfo") {
            // Token validation request
            let build_profile = crate::session::session_module::get_build_profile();
            let token_info_response = if build_profile
                == crate::session::session_module::BuildProfile::LabsAlpha
            {
                // Legacy clients accept an empty success payload here.
                Vec::new()
            } else {
                self.build_token_info_response()
            };
            let grpc_response = self.wrap_grpc_response(&token_info_response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else {
            // Check if this might be GetAuthForToken with a different path format
            // Uses "AuthServiceServicer:GetAuthForToken"
            // It might be under a different service path like "AuthService" or "nexus"
            if path.contains("Auth") && (path.contains("GetAuth") || path.contains("Token")) {
                crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m Auth request matched fallback handler: {}", path);
                // Parse gRPC frame and extract JWT
                let jwt_token = if let Ok((_, protobuf_data)) = parse_grpc_frame(body) {
                    extract_string_field(&protobuf_data, 6)
                        .filter(|candidate| self.looks_like_jwt(candidate))
                        .or_else(|| {
                            self.extract_protobuf_nested_string(&protobuf_data, 1, 1)
                                .filter(|candidate| self.looks_like_jwt(candidate))
                        })
                        .unwrap_or_else(|| self.extract_jwt_from_request(&protobuf_data))
                } else {
                    self.extract_jwt_from_request(body)
                };
                
                let auth_response = self.build_get_auth_for_token_response(if jwt_token.is_empty() {
                    None
                } else {
                    Some(jwt_token.as_str())
                });
                
                let mut request_headers = HashMap::new();
                request_headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
                let (grpc_response, response_headers) = build_grpc_response(&auth_response, &request_headers)
                    .unwrap_or_else(|_| (self.wrap_grpc_response(&auth_response), self.create_grpc_auth_headers()));
                let response = HttpResponse::new_with_headers(200, "application/grpc", grpc_response, response_headers);
                response.capture_response(method, path, host, true);
                return Ok(response);
            }
            
            // Generic gRPC response for unhandled services
            crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Unhandled accounts gRPC request: {}", path);
            let response = b"{\"status\":\"ok\"}";
            let grpc_response = self.wrap_grpc_response(response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        }
    }

    /// Handle gateway gRPC requests
    fn handle_gateway_grpc(
        &self,
        path: &str,
        _method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {

        if path.contains("BatchGetPlayerCards") {
            // eadp.playercard.v1.PlayerCardService/BatchGetPlayerCards
            // Return empty player cards list
            let response = self.build_player_cards_response();
            let grpc_response = self.wrap_grpc_response(&response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else {
            // Generic gateway response
            let response = b"{\"status\":\"ok\",\"services\":[\"game\",\"social\",\"commerce\"]}";
            let grpc_response = self.wrap_grpc_response(response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        }
    }

    /// Handle social API requests
    fn handle_social_api(
        &self,
        path: &str,
        _method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {
        let path_lower = path.to_ascii_lowercase();
        if path.contains("ConnectToPresenceSession") {
            // Build presence session response
            let response = self.build_presence_session_response();
            let grpc_response = self.wrap_grpc_response(&response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else if path.contains("ListFriends") {
            // Build friends list response
            let response = self.build_list_friends_response();
            let grpc_response = self.wrap_grpc_response(&response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else if path_lower.contains("privacy") {
            let response = b"{\"status\":\"ok\",\"privacy\":{\"available\":true,\"policyVersion\":\"placeholder\",\"consentRequired\":false}}";
            let grpc_response = self.wrap_grpc_response(response);
            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());
            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else {
            // Generic social response
            let response = b"{}";
            let grpc_response = self.wrap_grpc_response(response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        }
    }

    /// Handle telemetry requests
    fn handle_telemetry(
        &self,
        path: &str,
        _method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {

        if path.contains("fetch-experiment-grouping") {
            let response =
                r#"{"experiments":{"igli":{"group":"control"},"igli#40xg":{"group":"enabled"}}}"#;
            Ok(HttpResponse::new(
                200,
                "application/json",
                response.as_bytes().to_vec(),
            ))
        } else if path.contains("genericEvents") {
            // Handle generic events telemetry - return proper gRPC response
            let response = self.build_generic_events_response();
            let grpc_response = self.wrap_grpc_response(&response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else {
            Ok(HttpResponse::new(
                200,
                "application/json",
                b"{\"status\":\"ok\"}".to_vec(),
            ))
        }
    }

    /// Handle QoS coordinator requests
    /// QoS coordinator can be accessed via HTTP/HTTPS for health checks or configuration
    /// The actual QoS protocol (TCP) is handled by the TCP server on port 3659
    /// May also receive gRPC requests for health checks or service discovery
    fn handle_qos_coordinator(
        &self,
        path: &str,
        method: &str,
        body: &[u8],
    ) -> BlazeResult<HttpResponse> {
        crate::console_println!("\x1b[38;2;80;200;120m[QoS]\x1b[0m {} {} (body size: {} bytes)", method, path, body.len());
        
        // Check if this is a gRPC request (common gRPC paths or content-type)
        if path.contains("/grpc.") || path.contains("/Grpc") || path.contains("Health") || path.contains("Check") {
            // Handle gRPC health check requests
            crate::console_println!("\x1b[38;2;80;200;120m[QoS]\x1b[0m Handling gRPC request for QoS coordinator");
            let grpc_response = self.wrap_grpc_response(&[]);
            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());
            headers.insert("grpc-encoding".to_string(), "gzip".to_string());
            headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
            return Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ));
        }
        
        // QoS coordinator HTTP endpoints - return appropriate responses
        if path == "/" || path.is_empty() || path == "/health" || path == "/status" {
            // Health check endpoint
            let response = r#"{"status":"ok","qos":"available","port":3659}"#;
            Ok(HttpResponse::new(
                200,
                "application/json",
                response.as_bytes().to_vec(),
            ))
        } else if path.contains("config") || path.contains("qos") {
            // Configuration endpoint
            let response = r#"{"qos":{"port":3659,"protocol":"tcp","available":true}}"#;
            Ok(HttpResponse::new(
                200,
                "application/json",
                response.as_bytes().to_vec(),
            ))
        } else {
            // Default response for any other paths (including potential gRPC paths we don't recognize)
            // Try to detect if it might be gRPC based on path structure
            if path.contains("/") && path.split("/").count() > 2 {
                // Looks like a gRPC path (e.g., /service/method)
                crate::console_println!("\x1b[38;2;80;200;120m[QoS]\x1b[0m Treating as gRPC request (path structure)");
                let grpc_response = self.wrap_grpc_response(&[]);
                let mut headers = HashMap::new();
                headers.insert("grpc-status".to_string(), "0".to_string());
                headers.insert("grpc-encoding".to_string(), "gzip".to_string());
                headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
                Ok(HttpResponse::new_with_headers(
                    200,
                    "application/grpc",
                    grpc_response,
                    headers,
                ))
            } else {
                // Regular HTTP response
                let response = r#"{"status":"ok","qos":"available"}"#;
                Ok(HttpResponse::new(
                    200,
                    "application/json",
                    response.as_bytes().to_vec(),
                ))
            }
        }
    }

    /// Handle update.layer.ea.com requests
    fn handle_update_layer(
        &self,
        path: &str,
        method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {
        if method == "GET" && path.contains("bundle") {
            if path.contains("context=Photon") || path.contains("jsVersion=photon-bundle-") {
                let requested_name = Self::extract_query_param(path, "jsVersion")
                    .filter(|v| v.starts_with("photon-bundle-") && v.ends_with(".js"));
                if let Some(runtime_bundle) = Self::load_photon_bundle_runtime(requested_name) {
                    crate::console_println!(
                        "\x1b[38;2;120;220;180m[HTTP]\x1b[0m update.layer Photon bundle served from data/client/labs/js ({} bytes)",
                        runtime_bundle.len()
                    );
                    return Ok(HttpResponse::new(
                        200,
                        "application/javascript",
                        runtime_bundle,
                    ));
                }
                crate::console_println!(
                    "\x1b[38;2;255;180;120m[HTTP]\x1b[0m update.layer Photon bundle missing in data/client/labs/js, serving placeholder"
                );
            }
            Ok(HttpResponse::new(
                200,
                "application/javascript",
                b"void 0;\n".to_vec(),
            ))
        } else {
            Ok(HttpResponse::new(
                200,
                "application/json",
                b"{\"status\":\"ok\"}".to_vec(),
            ))
        }
    }

    /// Handle tos.ea.com requests (Terms of Service)
    fn handle_tos(
        &self,
        path: &str,
        method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {
        let path_lower = path.to_ascii_lowercase();
        if method == "GET" {
            let body = if path_lower.contains("webprivacy") || path_lower.contains("privacy") {
                "<html><body><h1>Privacy Notice</h1><p>Placeholder privacy content for offline emulator.</p></body></html>"
            } else if path_lower.contains("webterms") || path_lower.contains("terms") {
                "<html><body><h1>Terms of Service</h1><p>Placeholder terms content for offline emulator.</p></body></html>"
            } else if path_lower.contains("prfa") {
                "<html><body><h1>Parental Controls</h1><p>Placeholder PRFA content for offline emulator.</p></body></html>"
            } else {
                "<html><body><h1>Legal</h1><p>Placeholder legal content for offline emulator.</p></body></html>"
            };
            Ok(HttpResponse::new(
                200,
                "text/html",
                body.as_bytes().to_vec(),
            ))
        } else {
            Ok(HttpResponse::new(
                200,
                "application/json",
                b"{\"status\":\"ok\",\"legal\":\"placeholder\"}".to_vec(),
            ))
        }
    }

    /// Build auth response for Steam client
    fn build_auth_response_for_steam_client(&self) -> Vec<u8> {
        // The JWT token goes in field 1 of the inner AuthData message
        let auth_data = self.encode_protobuf_string(1, &self.jwt_token);

        // Wrap the auth data in field 1 of the outer response
        self.encode_protobuf_message(1, &auth_data)
    }
    
    /// Extract JWT token from GetAuthForToken request body.
    /// Request format: field 1 (fields) contains a message with field 1 (value) containing the JWT string.
    fn extract_jwt_from_request(&self, body: &[u8]) -> String {
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m extract_jwt_from_request entered, body size: {}", body.len());
        if body.is_empty() {
            crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Request body is empty");
            return String::new();
        }
        
        // Try field 1 -> field 1 first.
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Trying to extract JWT from field 1->1 (fields.value)");
        if let Some(jwt) = self.extract_protobuf_nested_string(body, 1, 1) {
            if self.looks_like_jwt(&jwt) {
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted from field 1->1 (fields.value)");
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted successfully from field 1->1 (length: {})", jwt.len());
                return jwt;
            }
        }
        
        // Try field 6 -> field 1 (alternative format)
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Trying to extract JWT from field 6->1");
        if let Some(jwt) = self.extract_protobuf_nested_string(body, 6, 1) {
            if self.looks_like_jwt(&jwt) {
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted from field 6->1");
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted successfully from field 6->1 (length: {})", jwt.len());
                return jwt;
            }
        }
        
        // Try direct field 1 string (in case it's not nested)
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Trying to extract JWT from direct field 1");
        if let Some(jwt) = self.extract_protobuf_string_field(body, 1) {
            if self.looks_like_jwt(&jwt) {
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted from direct field 1");
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted successfully from direct field 1 (length: {})", jwt.len());
                return jwt;
            }
        }
        
        // Fallback: search for JWT-like strings in the body
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Trying string search for JWT token");
        for i in 0..body.len().saturating_sub(10) {
            if body[i] == b'e' && body[i+1] == b'y' && body[i+2] == b'J' {
                let mut end = i + 3;
                while end < body.len() && body[end] != 0 && body[end] >= 32 && body[end] < 127 {
                    end += 1;
                }
                if end - i > 100 {
                    if let Ok(jwt) = String::from_utf8(body[i..end].to_vec()) {
                        if self.looks_like_jwt(&jwt) {
                            crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT extracted via string search");
                            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT found via string search at offset {} (length: {})", i, jwt.len());
                            return jwt;
                        }
                    }
                }
            }
        }
        
        crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m JWT extraction failed - no valid JWT found in request body");
        String::new()
    }

    fn looks_like_jwt(&self, value: &str) -> bool {
        value.starts_with("eyJ")
            && value.len() > 100
            && value.split('.').count() >= 3
    }
    
    /// Extract a nested string field from protobuf (outer_field contains inner_field containing string)
    fn extract_protobuf_nested_string(&self, body: &[u8], outer_field: u32, inner_field: u32) -> Option<String> {
        if body.is_empty() {
            return None;
        }
        
        let mut pos = 0;
        while pos < body.len() {
            if pos + 1 >= body.len() {
                break;
            }
            
            let field_tag = body[pos];
            let field_num = (field_tag >> 3) as u32;
            let wire_type = field_tag & 0x7;
            
            if field_num == outer_field && wire_type == 2 {
                // Length-delimited field (message)
                pos += 1;
                if pos >= body.len() {
                    break;
                }
                
                // Decode varint length
                let mut length = 0u64;
                let mut shift = 0;
                let mut len_pos = pos;
                
                while len_pos < body.len() && shift < 64 {
                    let byte = body[len_pos];
                    length |= ((byte & 0x7F) as u64) << shift;
                    len_pos += 1;
                    
                    if (byte & 0x80) == 0 {
                        break;
                    }
                    shift += 7;
                }
                
                if len_pos + length as usize <= body.len() {
                    let nested_data = &body[len_pos..len_pos + length as usize];
                    // Now extract inner_field from nested_data
                    return self.extract_protobuf_string_field(nested_data, inner_field);
                }
                
                pos = len_pos + length as usize;
            } else {
                // Skip this field
                pos += 1;
                if wire_type == 2 {
                    // Length-delimited, skip length and data
                    pos += 1;
                    let mut length = 0u64;
                    let mut shift = 0;
                    let mut len_pos = pos;
                    
                    while len_pos < body.len() && shift < 64 {
                        let byte = body[len_pos];
                        length |= ((byte & 0x7F) as u64) << shift;
                        len_pos += 1;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                        shift += 7;
                    }
                    
                    pos = len_pos + length as usize;
                } else if wire_type == 0 {
                    // Varint, skip it
                    pos += 1;
                    while pos < body.len() && (body[pos] & 0x80) != 0 {
                        pos += 1;
                    }
                    pos += 1;
                } else {
                    pos += 1;
                }
            }
        }
        
        None
    }
    
    /// Extract a direct string field from protobuf (not nested)
    fn extract_protobuf_string_field(&self, body: &[u8], field_num: u32) -> Option<String> {
        if body.is_empty() {
            return None;
        }
        
        let mut pos = 0;
        while pos < body.len() {
            if pos + 1 >= body.len() {
                break;
            }
            
            let field_tag = body[pos];
            let field_num_found = (field_tag >> 3) as u32;
            let wire_type = field_tag & 0x7;
            
            if field_num_found == field_num && wire_type == 2 {
                // Length-delimited field (string)
                pos += 1;
                if pos >= body.len() {
                    break;
                }
                
                // Decode varint length
                let mut length = 0u64;
                let mut shift = 0;
                let mut len_pos = pos;
                
                while len_pos < body.len() && shift < 64 {
                    let byte = body[len_pos];
                    length |= ((byte & 0x7F) as u64) << shift;
                    len_pos += 1;
                    
                    if (byte & 0x80) == 0 {
                        break;
                    }
                    shift += 7;
                }
                
                if len_pos + length as usize <= body.len() {
                    let string_data = &body[len_pos..len_pos + length as usize];
                    if let Ok(s) = String::from_utf8(string_data.to_vec()) {
                        return Some(s);
                    }
                }
                
                pos = len_pos + length as usize;
            } else {
                // Skip this field
                pos += 1;
                if wire_type == 2 {
                    // Length-delimited, skip length and data
                    let mut length = 0u64;
                    let mut shift = 0;
                    let mut len_pos = pos;
                    
                    while len_pos < body.len() && shift < 64 {
                        let byte = body[len_pos];
                        length |= ((byte & 0x7F) as u64) << shift;
                        len_pos += 1;
                        
                        if (byte & 0x80) == 0 {
                            break;
                        }
                        shift += 7;
                    }
                    
                    pos = len_pos + length as usize;
                } else if wire_type == 0 {
                    // Varint, skip it
                    while pos < body.len() && (body[pos] & 0x80) != 0 {
                        pos += 1;
                    }
                    pos += 1;
                } else {
                    pos += 1;
                }
            }
        }
        
        None
    }
    
    /// Build GetAuthForToken response (`nic:UserID:PersonalID` tokens in `Code` entries).
    fn build_get_auth_for_token_response(&self, jwt_token: Option<&str>) -> Vec<u8> {
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m build_get_auth_for_token_response entered, jwt_token provided: {}", jwt_token.is_some());
        use crate::session::get_user_session;
        use base64::{engine::general_purpose, Engine as _};
        use serde_json::Value;
        
        // Extract user info from JWT following jwt-nexus-tokn-rsp format
        let (nic, user_id, persona_id) = if let Some(jwt) = jwt_token {
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Parsing JWT token to extract user info");
            // Decode JWT: decoded = jwt.decode(request.fields.value, options={"verify_signature": False})
            let parts: Vec<&str> = jwt.split('.').collect();
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT split into {} parts", parts.len());
            if parts.len() >= 2 {
                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Decoding JWT payload (base64)");
                if let Ok(payload_bytes) = general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
                    crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Payload decoded, parsing JSON");
                    if let Ok(payload_str) = String::from_utf8(payload_bytes) {
                        if let Ok(payload_json) = serde_json::from_str::<Value>(&payload_str) {
                            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JSON parsed, extracting nexus object");
                            if let Some(nexus) = payload_json.get("nexus").and_then(|n| n.as_object()) {
                                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Nexus object found, extracting nic, user_id, persona_id");
                                // nic = decoded['nexus']['psif'][0]['nic']
                                let nic_val = nexus
                                    .get("psif")
                                    .and_then(|v| v.as_array())
                                    .and_then(|arr| arr.get(0))
                                    .and_then(|v| v.get("nic"))
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| {
                                        let session = get_user_session();
                                        session.display_name.clone()
                                    });
                                
                                // UserID = decoded['nexus']['psif'][0]['id']  (persona ID from psif)
                                let user_id_val = nexus
                                    .get("psif")
                                    .and_then(|v| v.as_array())
                                    .and_then(|arr| arr.get(0))
                                    .and_then(|v| v.get("id"))
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or_else(|| {
                                        let session = get_user_session();
                                        session.persona_id
                                    });
                                
                                // PersonalID = decoded['nexus']['pid']  (persona ID from root)
                                // Note: pid can be a string or number in the JWT
                                let persona_id_val = nexus
                                    .get("pid")
                                    .and_then(|v| {
                                        // Try as string first (common format)
                                        if let Some(s) = v.as_str() {
                                            s.parse::<u64>().ok()
                                        } else {
                                            // Fallback to number
                                            v.as_u64()
                                        }
                                    })
                                    .unwrap_or_else(|| {
                                        let session = get_user_session();
                                        session.persona_id
                                    });
                                
                                crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Extracted from JWT - nic: {}, user_id: {}, persona_id: {}", nic_val, user_id_val, persona_id_val);
                                (nic_val, user_id_val, persona_id_val)
                            } else {
                                crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Nexus object not found in JWT, using session fallback");
                                // Fallback to session
                                let session = get_user_session();
                                (session.display_name, session.persona_id, session.persona_id)
                            }
                        } else {
                            crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to parse JWT payload as JSON, using session fallback");
                            // Fallback to session
                            let session = get_user_session();
                            (session.display_name, session.persona_id, session.persona_id)
                        }
                    } else {
                        crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to decode JWT payload from UTF-8, using session fallback");
                        // Fallback to session
                        let session = get_user_session();
                        (session.display_name, session.persona_id, session.persona_id)
                    }
                } else {
                    crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to base64 decode JWT payload, using session fallback");
                    // Fallback to session
                    let session = get_user_session();
                    (session.display_name, session.persona_id, session.persona_id)
                }
            } else {
                crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m JWT has insufficient parts (< 2), using session fallback");
                // Fallback to session
                let session = get_user_session();
                (session.display_name, session.persona_id, session.persona_id)
            }
        } else {
            crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m No JWT token provided, using session state");
            // No JWT provided, use session state
            let session = get_user_session();
            (session.display_name, session.persona_id, session.persona_id)
        };
        
        // Build token: "nic:UserID:PersonalID"
        let token = format!("{}:{}:{}", nic, user_id, persona_id);
        let token_url = format!("http://127.0.0.1/success?code={}", token);
        
        crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GetAuthForToken: {}", token);
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Built token: {} (nic: {}, user_id: {}, persona_id: {})", token, nic, user_id, persona_id);
        
        // Older Labs clients expect the legacy UUID+JWT shape.
        let build_profile = crate::session::session_module::get_build_profile();
        if build_profile == crate::session::session_module::BuildProfile::LabsAlpha {
            // Legacy clients expect the account UID in user-id fields.
            let legacy_user_id = jwt_token
                .and_then(|jwt| {
                    let parts: Vec<&str> = jwt.split('.').collect();
                    if parts.len() < 2 {
                        return None;
                    }
                    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
                        .decode(parts[1])
                        .ok()?;
                    let payload_str = String::from_utf8(payload_bytes).ok()?;
                    let payload_json = serde_json::from_str::<serde_json::Value>(&payload_str).ok()?;
                    payload_json
                        .get("nexus")
                        .and_then(|n| n.get("uid"))
                        .and_then(|v| {
                            if let Some(s) = v.as_str() {
                                s.parse::<u64>().ok()
                            } else {
                                v.as_u64()
                            }
                        })
                })
                .unwrap_or_else(|| {
                    let session = get_user_session();
                    session.user_id
                });
            let legacy_jwt = jwt_token
                .filter(|v| !v.is_empty())
                .unwrap_or(&self.jwt_token);
            let legacy_response = build_get_auth_for_token_protobuf(
                "cbdede63-d594-49aa-bbc1-1f86f6f2507b",
                legacy_user_id,
                persona_id,
                &legacy_user_id.to_string(),
                &persona_id.to_string(),
                legacy_jwt,
                "0.17.1",
            );
            crate::debug_println!(
                "\x1b[38;2;0;200;255m[gRPC]\x1b[0m Using legacy GetAuthForToken payload (labs-legacy), size={} bytes",
                legacy_response.len()
            );
            return legacy_response;
        }

        // Build protobuf response where field 1 (repeated) contains Code messages,
        // each with field 1 (token string).
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Building protobuf response with two Code messages");
        let mut response = Vec::new();
        
        // First Code message: field 1 (repeated), contains Code { field 1: token }
        let mut code1_msg = Vec::new();
        code1_msg.extend_from_slice(&self.encode_protobuf_string(1, &token));
        response.extend_from_slice(&self.encode_protobuf_message(1, &code1_msg));
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m First Code message added (token: {})", token);
        
        // Second Code message: field 1 (repeated), contains Code { field 1: token_url }
        let mut code2_msg = Vec::new();
        code2_msg.extend_from_slice(&self.encode_protobuf_string(1, &token_url));
        response.extend_from_slice(&self.encode_protobuf_message(1, &code2_msg));
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Second Code message added (token_url: {})", token_url);
        
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GetAuthForToken response built: token={}, response_size={} bytes", token, response.len());
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m GetAuthForToken protobuf response complete (size: {} bytes)", response.len());
        
        response
    }

    /// Build token grant response (Steam flow)
    #[allow(dead_code)]
    fn build_token_grant_response(&self) -> Vec<u8> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_in_seconds = 86400; // 1 day
        let refresh_expires_in_seconds = 2592000; // 30 days

        let expires_at_utc = current_time + expires_in_seconds;
        let refresh_token_expires_at_utc = current_time + refresh_expires_in_seconds;

        // Build the inner token data (field 1)
        let mut token_data = Vec::new();
        token_data.extend_from_slice(&self.encode_protobuf_string(1, &self.access_token));
        token_data.extend_from_slice(&self.encode_protobuf_string(2, "Bearer"));
        token_data.extend_from_slice(&self.encode_protobuf_int64(3, expires_in_seconds as i64));

        let refresh_token_value = format!("RT_VALID_{}_{}", self.session_id, current_time);
        token_data.extend_from_slice(&self.encode_protobuf_string(4, &refresh_token_value));
        token_data.extend_from_slice(&self.encode_protobuf_string(5, "offline openid"));
        token_data
            .extend_from_slice(&self.encode_protobuf_int64(6, refresh_expires_in_seconds as i64));

        // Build the complete response
        let mut response = Vec::new();
        response.extend_from_slice(&self.encode_protobuf_message(1, &token_data));
        response.extend_from_slice(&self.encode_protobuf_int64(2, expires_at_utc as i64));
        response
            .extend_from_slice(&self.encode_protobuf_int64(3, refresh_token_expires_at_utc as i64));

        response
    }

    /// Build EA token grant response with signed JWT
    fn build_ea_token_grant_response(&self) -> Vec<u8> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_in_seconds = 86400; // 1 day
        let refresh_expires_in_seconds = 2592000; // 30 days

        let expires_at_utc = current_time + expires_in_seconds;
        let refresh_token_expires_at_utc = current_time + refresh_expires_in_seconds;

        // Get user session from LSX authentication (or use defaults)
        use crate::session::{get_user_session, set_user_session};
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Retrieving user session for JWT generation");
        let session = get_user_session();
        let persona_id = session.persona_id;
        let user_id = session.user_id;
        let display_name = session.display_name.clone();
        
        crate::console_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Using session from LSX auth - user_id={}, persona_id={}, display_name={}", user_id, persona_id, display_name);
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Session data - user_id={}, persona_id={}, display_name={}", user_id, persona_id, display_name);
        
        // Generate EA-specific JWT token with real user data from LSX
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Generating EA JWT token for client_id={}, persona_id={}, user_id={}", NEXUS_GATEWAY_CLIENT_ID, persona_id, user_id);
        let ea_jwt_token = generate_ea_jwt_token(&self.session_id, &persona_id, &display_name, NEXUS_GATEWAY_CLIENT_ID, &user_id);
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m EA JWT token generated (length: {})", ea_jwt_token.len());
        
        // Generate refresh token JWT
        let refresh_token_jwt = generate_refresh_token_jwt(&self.session_id, &persona_id, &display_name, NEXUS_GATEWAY_CLIENT_ID, &user_id);
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m Refresh token JWT generated (length: {})", refresh_token_jwt.len());
        
        let mut session = get_user_session();
        session.jwt_token = Some(ea_jwt_token.clone());
        set_user_session(session);
        crate::debug_println!("\x1b[38;2;0;200;255m[gRPC]\x1b[0m JWT token stored in session");

        // Build the inner token data (field 1) - EA flow
        let mut token_data = Vec::new();
        token_data.extend_from_slice(&self.encode_protobuf_string(1, &ea_jwt_token));
        token_data.extend_from_slice(&self.encode_protobuf_string(2, "Bearer"));
        token_data.extend_from_slice(&self.encode_protobuf_int64(3, expires_in_seconds as i64));

        // Use JWT refresh token instead of plain string
        token_data.extend_from_slice(&self.encode_protobuf_string(4, &refresh_token_jwt));
        token_data.extend_from_slice(&self.encode_protobuf_string(5, "offline openid"));
        token_data
            .extend_from_slice(&self.encode_protobuf_int64(6, refresh_expires_in_seconds as i64));

        // Build the complete response
        let mut response = Vec::new();
        response.extend_from_slice(&self.encode_protobuf_message(1, &token_data));
        response.extend_from_slice(&self.encode_protobuf_int64(2, expires_at_utc as i64));
        response
            .extend_from_slice(&self.encode_protobuf_int64(3, refresh_token_expires_at_utc as i64));

        response
    }


    /// Build token info response
    fn build_token_info_response(&self) -> Vec<u8> {
        // Create the innermost 'Persona' message
        let mut persona_data = Vec::new();
        persona_data.extend_from_slice(&self.encode_protobuf_int64(1, self.persona_id as i64));
        persona_data.extend_from_slice(&self.encode_protobuf_string(2, &self.player_name));
        persona_data.extend_from_slice(&self.encode_protobuf_int64(3, 1));

        // Create the middle message that wraps the 'Persona' message
        let mut middle_data = Vec::new();
        middle_data.extend_from_slice(&self.encode_protobuf_message(2, &persona_data));

        // Wrap the middle message in the standard gRPC outer message
        self.encode_protobuf_message(1, &middle_data)
    }

    /// Build presence session response
    fn build_presence_session_response(&self) -> Vec<u8> {
        let presence_session_id = format!("presence_{}", self.session_id);
        let session_data = self.encode_protobuf_string(1, &presence_session_id);
        self.encode_protobuf_message(1, &session_data)
    }

    /// Build list friends response
    fn build_list_friends_response(&self) -> Vec<u8> {
        // Field 1 (result) contains an empty message
        self.encode_protobuf_message(1, &[])
    }

    /// Create standard gRPC headers for all gRPC responses
    fn create_grpc_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());
        headers.insert("grpc-encoding".to_string(), "gzip".to_string()); // gzip response body
        headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
        headers
    }

    /// Create standard gRPC headers
    fn create_grpc_auth_headers(&self) -> HashMap<String, String> {
        self.create_grpc_headers()
    }

    /// Wrap response in gRPC frame (legacy - use build_grpc_response from grpc module)
    fn wrap_grpc_response(&self, data: &[u8]) -> Vec<u8> {
        // Use new gRPC frame builder (no compression for legacy compatibility)
        build_grpc_frame(data, false).unwrap_or_else(|_| {
            // Fallback to old format if new one fails
            let mut response = Vec::new();
            response.push(0); // No compression
            response.extend_from_slice(&(data.len() as u32).to_be_bytes());
            response.extend_from_slice(data);
            response
        })
    }

    /// Encode protobuf string field
    fn encode_protobuf_string(&self, field_num: u32, value: &str) -> Vec<u8> {
        let data = value.as_bytes();
        let length = self.encode_varint(data.len() as u64);
        let field_header = self.encode_field_header(field_num, 2); // Wire type 2 for length-delimited
        let mut result = Vec::new();
        result.extend_from_slice(&field_header);
        result.extend_from_slice(&length);
        result.extend_from_slice(data);
        result
    }

    /// Encode protobuf int64 field
    fn encode_protobuf_int64(&self, field_num: u32, value: i64) -> Vec<u8> {
        let data = self.encode_varint(value as u64);
        let field_header = self.encode_field_header(field_num, 0); // Wire type 0 for varint
        let mut result = Vec::new();
        result.extend_from_slice(&field_header);
        result.extend_from_slice(&data);
        result
    }

    /// Encode protobuf message field
    fn encode_protobuf_message(&self, field_num: u32, message: &[u8]) -> Vec<u8> {
        let length = self.encode_varint(message.len() as u64);
        let field_header = self.encode_field_header(field_num, 2); // Wire type 2 for length-delimited
        let mut result = Vec::new();
        result.extend_from_slice(&field_header);
        result.extend_from_slice(&length);
        result.extend_from_slice(message);
        result
    }

    /// Encode protobuf field header
    fn encode_field_header(&self, field_num: u32, wire_type: u32) -> Vec<u8> {
        let header = (field_num << 3) | wire_type;
        self.encode_varint(header as u64)
    }

    /// Encode varint
    fn encode_varint(&self, mut value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        while value > 0x7F {
            result.push(((value & 0x7F) | 0x80) as u8);
            value >>= 7;
        }
        result.push(value as u8);
        result
    }
    
    /// Generate a UUID v4 format string
    fn generate_uuid(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.gen();
        
        // Format as UUID v4: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        // Version 4: set version bits (bits 12-15 of time_hi_and_version to 0100)
        // Variant: set bits 6-7 of clock_seq_hi_and_reserved to 10
        let mut uuid_bytes = bytes;
        uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x40; // Version 4
        uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80; // Variant 10
        
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            u32::from_be_bytes([uuid_bytes[0], uuid_bytes[1], uuid_bytes[2], uuid_bytes[3]]),
            u16::from_be_bytes([uuid_bytes[4], uuid_bytes[5]]),
            u16::from_be_bytes([uuid_bytes[6], uuid_bytes[7]]),
            u16::from_be_bytes([uuid_bytes[8], uuid_bytes[9]]),
            u64::from_be_bytes([0, 0, uuid_bytes[10], uuid_bytes[11], uuid_bytes[12], uuid_bytes[13], uuid_bytes[14], uuid_bytes[15]])
        )
    }
    

    /// Labs gateway service paths (gRPC-style over HTTPS for Battlefield Labs routing).
    fn handle_santiago_services(
        &self,
        path: &str,
        _method: &str,
        body: &[u8],
    ) -> BlazeResult<HttpResponse> {

        if path.contains("santiago.client.authentication.ClientAuthentication/viaAuthCode") || 
           path.contains("client.authentication.ClientAuthentication/viaAuthCode") {
            self.handle_santiago_auth_via_auth_code(body)
        } else if path.contains("santiago.client.schedule.ClientSchedule/getConfig") {
            // Matches getConfig and getConfigs (substring)
            self.handle_santiago_schedule_configs()
        } else if path.contains("santiago.client.clientstorage.ClientStorage/UnsetUserBits") {
            self.handle_santiago_set_user_bits()
        } else if path.contains("santiago.client.clientstorage.ClientStorage/SetUserBits") {
            self.handle_santiago_set_user_bits()
        } else if path.contains("santiago.client.playercard.ClientPlayerCard/getPlayerCard") {
            self.handle_santiago_get_player_card()
        } else if path.contains("santiago.client.localization.ClientLocalization/getTranslations") {
            self.handle_santiago_get_translations()
        } else if path.contains("santiago.client.licenses.ClientLicenses/getActiveLicenses") {
            self.handle_santiago_get_active_licenses()
        } else if path.contains("santiago.client.inventory.ClientInventory/getLicenseSources") {
            self.handle_santiago_get_license_sources()
        } else if path.contains("santiago.client.following.ClientFollowing/ListFollowedCreators") {
            self.handle_santiago_list_followed_creators()
        } else if path.contains("santiago.client.unifiedmessaging.UnifiedMessaging/fetchActions") {
            Self::log_milestone_once("menu_ready", "Menu flow detected (actions fetched)");
            self.handle_santiago_fetch_actions()
        } else if path.contains("santiago.client.following.ClientFollowing/IsFollowableCreator") {
            self.handle_santiago_is_followable_creator()
        } else if path.contains("santiago.client.play.ClientPlay/GetBlueprint") {
            Self::log_milestone_once("match_join", "Match flow detected (blueprint requested)");
            self.handle_santiago_get_blueprint()
        } else if path.contains("santiago.client.ban.ClientBan/ListBannedPlayers") {
            self.handle_santiago_list_banned_players()
        } else if path.contains("santiago.client.menu.ClientMenu/getScheduledMenu")
            || path.contains("santiago.client.menu.ClientMenu/getMenuUpdates")
        {
            Self::log_milestone_once("menu_ready", "Menu flow detected (menu requested)");
            self.handle_santiago_get_scheduled_menu()
        } else if path.contains("santiago.client.play.ClientPlay/GetRegionFilterMappings") {
            self.handle_santiago_get_region_filter_mappings()
        } else if path.contains("santiago.client.xpmodifiers.ClientXpModifiers/getMenuXpModifiers")
        {
            self.handle_santiago_get_menu_xp_modifiers()
        } else if path.contains("santiago.client.inventory.ClientInventory/getPlayerInventoryV2") {
            self.handle_santiago_get_player_inventory_v2()
        } else if path.contains("santiago.client.xpmodifiers.ClientXpModifiers/getXpModifiers") {
            self.handle_santiago_get_xp_modifiers()
        } else if path.contains("santiago.client.inventory.ClientInventory/getPlayerWallets") {
            self.handle_santiago_get_player_wallets()
        } else if path.contains("santiago.client.store.ClientStore/getOffers") {
            self.handle_santiago_get_offers()
        } else if path.contains("santiago.client.inventory.ClientInventory/getPotentialItemSources")
        {
            self.handle_santiago_get_potential_item_sources()
        } else if path.contains("santiago.client.rank.ClientRank/getRankConfigurations") {
            self.handle_santiago_get_rank_configurations()
        } else if path.contains("santiago.client.rank.ClientRank/getOffers") {
            self.handle_santiago_get_rank_offers()
        } else if path.contains("santiago.client.gameevent.ClientGameEvent/getGameEvents") {
            Self::log_milestone_once("match_join", "Match flow detected (game events requested)");
            self.handle_santiago_get_game_events()
        } else if path.contains("santiago.client.rank.ClientRank/getAllRanks") {
            self.handle_santiago_get_all_ranks()
        } else if path.contains("santiago.client.loadout.ClientLoadout/getLoadouts") {
            self.handle_santiago_get_loadouts()
        } else if path.contains("santiago.client.stats.ClientStats/getStats") {
            self.handle_santiago_get_stats()
        } else if path.contains("santiago.client.quest.ClientQuest/listProgressable") {
            self.handle_santiago_list_progressable()
        } else if path.contains("santiago.client.rank.ClientRank/getRankStatus") {
            self.handle_santiago_get_rank_status()
        } else if path.contains("santiago.client.quest.ClientQuest/getDefinitions") {
            self.handle_santiago_get_quest_definitions()
        } else if path
            .contains("santiago.client.first_party.ClientFirstPartyCommerce/getStoreCatalog")
        {
            self.handle_santiago_get_store_catalog()
        } else if path.contains("santiago.client.store.ClientStore/triggerRewards") {
            self.handle_santiago_trigger_rewards()
        } else if path.contains("santiago.client.store.ClientStore/getStoreMenu")
            || path.contains("santiago.client.menu.ClientMenu/getStoreMenu")
        {
            Self::log_milestone_once("menu_ready", "Menu ready (store menu requested)");
            self.handle_santiago_store_menu()
        } else if path.contains("santiago.client.xpmodifiers.ClientXpModifiers/getXpModifiers") {
            self.handle_santiago_xp_modifiers()
        } else if path.contains("santiago.client.inventory.ClientInventory/getInventories") {
            self.handle_santiago_inventories()
        } else if path
            .contains("santiago.client.inventory.ClientInventory/getInventoryNotifications")
        {
            self.handle_santiago_inventory_notifications()
        } else if path.contains("santiago.client.rank.ClientRank/getRankUpNotifications") {
            self.handle_santiago_rank_up_notifications()
        } else if path.contains("santiago.client.play.ClientPlay/getScheduledCollections") {
            self.handle_santiago_scheduled_collections()
        } else if path.contains("santiago.client.communitygames.CommunityGames/listFollowedHosts") {
            self.handle_santiago_followed_hosts()
        } else if path
            .contains("santiago.client.communitygames.CommunityGames/getScheduledBlueprints")
        {
            self.handle_santiago_scheduled_blueprints()
        } else if path.contains("santiago.client.store.ClientTrial/getTrialInfo") {
            self.handle_santiago_trial_info()
        } else {
            // Generic Santiago response for unhandled services
            crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Unhandled Santiago service: {}", path);
            let response = b"{\"status\":\"ok\"}";
            let grpc_response = self.wrap_grpc_response(response);
            let headers = self.create_grpc_headers();

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        }
    }

    // Santiago service handlers
    fn handle_santiago_auth_via_auth_code(&self, body: &[u8]) -> BlazeResult<HttpResponse> {
        // Parse gRPC frame if present
        let protobuf_body = if let Ok((_, data)) = parse_grpc_frame(body) {
            crate::debug_println!("\x1b[38;2;255;215;0m[gRPC]\x1b[0m viaAuthCode: gRPC frame parsed");
            data
        } else {
            crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m viaAuthCode: Failed to parse gRPC frame, using body as-is");
            body.to_vec()
        };
        
        // Parse request: field 1 = authcode, field 11 = deviceId
        let device_id = extract_string_field(&protobuf_body, 11)
            .or_else(|| self.extract_protobuf_string_field(&protobuf_body, 11))
            .unwrap_or_else(|| "85151234".to_string());
        
        // Get current user session
        use crate::session::get_user_session;
        let session = get_user_session();
        
        // Build protobuf response matching viaAuthCode.txt format:
        // Field 1: UUID token
        // Field 3: User info message { field 1: user_id, field 2: persona_id, field 3: status }
        // Field 5: Device info message { field 1: deviceId, field 2: deviceId }
        let mut response = Vec::new();
        
        // Field 1: UUID token
        let uuid_token = self.generate_uuid();
        response.extend_from_slice(&encode_string_field(1, &uuid_token));
        
        // Field 3: User info message
        let mut user_info = Vec::new();
        user_info.extend_from_slice(&encode_string_field(1, &session.user_id.to_string()));
        user_info.extend_from_slice(&encode_string_field(2, &session.persona_id.to_string()));
        user_info.extend_from_slice(&encode_string_field(3, "1"));
        response.extend_from_slice(&encode_message_field(3, &user_info));
        
        // Field 5: Device info message
        let mut device_info = Vec::new();
        device_info.extend_from_slice(&encode_string_field(1, &device_id));
        device_info.extend_from_slice(&encode_string_field(2, &device_id));
        response.extend_from_slice(&encode_message_field(5, &device_info));
        
        // Build gRPC frame with compression
        let mut request_headers = HashMap::new();
        request_headers.insert("grpc-accept-encoding".to_string(), "gzip".to_string());
        let (grpc_response, response_headers) = build_grpc_response(&response, &request_headers)
            .unwrap_or_else(|e| {
                crate::debug_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m Failed to build gRPC response: {}, using fallback", e);
                (self.wrap_grpc_response(&response), self.create_grpc_auth_headers())
            });

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            response_headers,
        ))
    }

    fn handle_santiago_schedule_configs(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"configs\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_set_user_bits(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_player_card(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_translations(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"translations\":{},\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_active_licenses(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"licenses\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_license_sources(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"sources\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_list_followed_creators(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"creators\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_fetch_actions(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"actions\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_is_followable_creator(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"followable\":false,\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_blueprint(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"blueprint\":{},\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_list_banned_players(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"bannedPlayers\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_scheduled_menu(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"menu\":{},\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_region_filter_mappings(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"mappings\":{},\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_menu_xp_modifiers(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"modifiers\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_player_inventory_v2(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"inventory\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_xp_modifiers(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"modifiers\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_player_wallets(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"wallets\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_offers(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"offers\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_potential_item_sources(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"sources\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_rank_configurations(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"configurations\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_rank_offers(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"offers\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_game_events(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"events\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_all_ranks(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"ranks\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_loadouts(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"loadouts\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_stats(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"stats\":{},\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_list_progressable(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"progressable\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_rank_status(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_quest_definitions(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"definitions\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_get_store_catalog(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"catalog\":{},\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_trigger_rewards(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"rewards\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    /// Handle collector errors service
    fn handle_collector_errors(
        &self,
        path: &str,
        _method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {

        if path.contains("SubmitBootSession") {
            // Build proper protobuf response for SubmitBootSession
            let response = self.build_submit_boot_session_response();
            let grpc_response = self.wrap_grpc_response(&response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else if path.contains("SubmitCrashReport") {
            // Build proper protobuf response for SubmitCrashReport
            let response = self.build_submit_crash_report_response();
            let grpc_response = self.wrap_grpc_response(&response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else {
            // Generic error reporting service response
            let response = b"{\"status\":\"ok\"}";
            let grpc_response = self.wrap_grpc_response(response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        }
    }

    /// Handle event bridge service
    fn handle_eventbridge(
        &self,
        path: &str,
        _method: &str,
        _body: &[u8],
    ) -> BlazeResult<HttpResponse> {

        if path.contains("clientEvents") {
            // Build proper protobuf response for clientEvents
            let response = self.build_client_events_response();
            let grpc_response = self.wrap_grpc_response(&response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        } else {
            // Generic event bridge service response
            let response = b"{\"status\":\"ok\",\"events\":[]}";
            let grpc_response = self.wrap_grpc_response(response);

            let mut headers = HashMap::new();
            headers.insert("grpc-status".to_string(), "0".to_string());

            Ok(HttpResponse::new_with_headers(
                200,
                "application/grpc",
                grpc_response,
                headers,
            ))
        }
    }

    /// Build SubmitBootSession response - returns empty gRPC response
    fn build_submit_boot_session_response(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Build SubmitCrashReport response - returns empty gRPC response
    fn build_submit_crash_report_response(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Build clientEvents response
    fn build_client_events_response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Field 1: success (bool) - indicate the events were processed
        response.extend_from_slice(&self.encode_protobuf_bool(1, true));

        // Field 2: events_processed (int32) - number of events processed
        response.extend_from_slice(&self.encode_protobuf_int64(2, 1));

        // Field 3: timestamp (int64) - when events were processed
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        response.extend_from_slice(&self.encode_protobuf_int64(3, timestamp));

        response
    }

    /// Encode protobuf bool field
    fn encode_protobuf_bool(&self, field_num: u32, value: bool) -> Vec<u8> {
        let data = self.encode_varint(if value { 1 } else { 0 });
        let field_header = self.encode_field_header(field_num, 0); // Wire type 0 for varint
        let mut result = Vec::new();
        result.extend_from_slice(&field_header);
        result.extend_from_slice(&data);
        result
    }

    // Additional Santiago service handlers
    fn handle_santiago_store_menu(&self) -> BlazeResult<HttpResponse> {
        // Build a more realistic store menu response with proper protobuf structure
        let response = self.build_santiago_store_menu_response();
        let grpc_response = self.wrap_grpc_response(&response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_xp_modifiers(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"modifiers\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_inventories(&self) -> BlazeResult<HttpResponse> {
        let response = self.build_santiago_inventory_response();
        let grpc_response = self.wrap_grpc_response(&response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_inventory_notifications(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"notifications\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_rank_up_notifications(&self) -> BlazeResult<HttpResponse> {
        let response = self.build_santiago_rank_response();
        let grpc_response = self.wrap_grpc_response(&response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_scheduled_collections(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"collections\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_followed_hosts(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"hosts\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_scheduled_blueprints(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"blueprints\":[],\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    fn handle_santiago_trial_info(&self) -> BlazeResult<HttpResponse> {
        let response = b"{\"trial\":{\"active\":false},\"status\":\"ok\"}";
        let grpc_response = self.wrap_grpc_response(response);

        let mut headers = HashMap::new();
        headers.insert("grpc-status".to_string(), "0".to_string());

        Ok(HttpResponse::new_with_headers(
            200,
            "application/grpc",
            grpc_response,
            headers,
        ))
    }

    /// Build generic events response for telemetry
    fn build_generic_events_response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Field 1: success (bool) - indicate events were received
        response.extend_from_slice(&self.encode_protobuf_bool(1, true));

        // Field 2: events_received (int32) - number of events received
        response.extend_from_slice(&self.encode_protobuf_int64(2, 1));

        // Field 3: session_id (string) - session identifier
        response.extend_from_slice(&self.encode_protobuf_string(3, &self.session_id));

        // Field 4: timestamp (int64) - when events were received
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        response.extend_from_slice(&self.encode_protobuf_int64(4, timestamp));

        response
    }

    // Response builders for more realistic protobuf structures
    fn build_santiago_store_menu_response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Field 1: menu_items (repeated message)
        // For now, return empty list but with proper structure
        response.extend_from_slice(&self.encode_protobuf_message(1, &[]));

        // Field 2: status (string)
        response.extend_from_slice(&self.encode_protobuf_string(2, "active"));

        // Field 3: last_updated (int64 timestamp)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        response.extend_from_slice(&self.encode_protobuf_int64(3, timestamp));

        response
    }

    fn build_santiago_inventory_response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Field 1: inventories (repeated message)
        response.extend_from_slice(&self.encode_protobuf_message(1, &[]));

        // Field 2: total_count (int64)
        response.extend_from_slice(&self.encode_protobuf_int64(2, 0));

        // Field 3: last_sync (int64 timestamp)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        response.extend_from_slice(&self.encode_protobuf_int64(3, timestamp));

        response
    }

    fn build_santiago_rank_response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Field 1: ranks (repeated message)
        response.extend_from_slice(&self.encode_protobuf_message(1, &[]));

        // Field 2: current_rank (int64)
        response.extend_from_slice(&self.encode_protobuf_int64(2, 1));

        // Field 3: total_xp (int64)
        response.extend_from_slice(&self.encode_protobuf_int64(3, 0));

        response
    }

    /// Build player cards response - returns empty list of player cards
    fn build_player_cards_response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Field 1: player_cards (repeated message) - empty list
        response.extend_from_slice(&self.encode_protobuf_message(1, &[]));

        // Field 2: total_count (int64)
        response.extend_from_slice(&self.encode_protobuf_int64(2, 0));

        // Field 3: last_updated (int64 timestamp)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        response.extend_from_slice(&self.encode_protobuf_int64(3, timestamp));

        response
    }
}

/// HTTP response structure
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub content_type: String,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
}

impl HttpResponse {
    pub fn new(status_code: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status_code,
            content_type: content_type.to_string(),
            body,
            headers: HashMap::new(),
        }
    }

    /// Capture this response for inspector
    pub fn capture_response(&self, method: &str, path: &str, host: &str, is_grpc: bool) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        
        if is_grpc {
            let grpc_status = self.headers.get("grpc-status").cloned();
            let cap = grpc_body_decode_capture(&self.body);

            let grpc_response = CapturedGrpc {
                capture_seq: 0,
                timestamp,
                direction: GrpcDirection::ServerToClient,
                method: method.to_string(),
                path: path.to_string(),
                host: host.to_string(),
                headers: self.headers.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                body_size: self.body.len(),
                body: self.body.clone(),
                protobuf_data: cap.protobuf_chunks.first().cloned(),
                protobuf_chunks: cap.protobuf_chunks,
                is_compressed: cap.any_frame_was_compressed,
                grpc_status,
            };
            capture_grpc(grpc_response);
        } else {
            let mut response_headers: Vec<(String, String)> =
                self.headers.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            response_headers.push(("Content-Type".to_string(), self.content_type.clone()));
            response_headers.push(("Content-Length".to_string(), self.body.len().to_string()));

            let http_response = CapturedHttp {
                capture_seq: 0,
                timestamp,
                direction: HttpDirection::ServerToClient,
                method: method.to_string(),
                path: path.to_string(),
                host: host.to_string(),
                headers: response_headers,
                body_size: self.body.len(),
                body: self.body.clone(),
                status_code: Some(self.status_code),
            };
            capture_http(http_response);
        }
    }

    pub fn new_with_headers(
        status_code: u16,
        content_type: &str,
        body: Vec<u8>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            status_code,
            content_type: content_type.to_string(),
            body,
            headers,
        }
    }
}
