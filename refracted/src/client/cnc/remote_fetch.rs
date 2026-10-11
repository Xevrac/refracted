//! HTTPS fetches made on behalf of the shell, with an in-memory cache.
//!
//! EAWebKit cannot complete a modern TLS handshake, so anything the shell shows from another site
//! (news feed and images, server browser flags) is fetched here and served from this host.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::http::handlers::handlers_module::HttpResponse;

const FETCH_TIMEOUT: Duration = Duration::from_secs(8);

struct Cached {
    at: Instant,
    content_type: String,
    body: Vec<u8>,
}

fn cache() -> &'static Mutex<HashMap<String, Cached>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Cached>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| ureq::AgentBuilder::new().timeout(FETCH_TIMEOUT).build())
}

/// JSON error body, never cached by the shell.
pub fn error_response(status: u16, message: &str) -> HttpResponse {
    let body = serde_json::json!({ "error": message }).to_string().into_bytes();
    let mut response = HttpResponse::new(status, "application/json", body);
    response
        .headers
        .insert("Cache-Control".to_string(), "no-store".to_string());
    response
}

fn log(message: String) {
    crate::console_println!("\x1b[38;2;100;200;255m[CNC]\x1b[0m remote fetch: {message}");
}

/// GET `url`; returns (status, content type, body) for any HTTP status, Err on transport failure.
fn fetch(url: &str, max_bytes: u64) -> Result<(u16, String, Vec<u8>), String> {
    let response = match agent().get(url).call() {
        Ok(r) | Err(ureq::Error::Status(_, r)) => r,
        Err(e) => return Err(e.to_string()),
    };
    let status = response.status();
    let content_type = response.content_type().to_string();
    let mut body = Vec::new();
    response
        .into_reader()
        .take(max_bytes + 1)
        .read_to_end(&mut body)
        .map_err(|e| e.to_string())?;
    if body.len() as u64 > max_bytes {
        return Err(format!("response larger than {max_bytes} bytes"));
    }
    Ok((status, content_type, body))
}

/// Serves a fresh cache entry, otherwise fetches; on upstream failure falls back to a stale entry.
///
/// `key` is `"<kind>:<id>"`. With `max_of_kind`, the oldest entry of the same kind is dropped
/// before a new one is added once that many are cached.
pub fn cached_fetch(
    key: &str,
    url: &str,
    ttl: Duration,
    max_bytes: u64,
    max_of_kind: Option<usize>,
    accept: fn(&str) -> bool,
) -> Result<(String, Vec<u8>), (u16, String)> {
    if let Some(hit) = cache().lock().get(key) {
        if hit.at.elapsed() < ttl {
            return Ok((hit.content_type.clone(), hit.body.clone()));
        }
    }

    let failure = match fetch(url, max_bytes) {
        Ok((200, content_type, body)) if accept(&content_type) => {
            let mut map = cache().lock();
            if let Some(cap) = max_of_kind {
                let kind = key.split(':').next().unwrap_or(key);
                let prefix = format!("{kind}:");
                if !map.contains_key(key) && map.keys().filter(|k| k.starts_with(&prefix)).count() >= cap {
                    let oldest = map
                        .iter()
                        .filter(|(k, _)| k.starts_with(&prefix))
                        .min_by_key(|(_, v)| v.at)
                        .map(|(k, _)| k.clone());
                    if let Some(oldest) = oldest {
                        map.remove(&oldest);
                    }
                }
            }
            map.insert(
                key.to_string(),
                Cached { at: Instant::now(), content_type: content_type.clone(), body: body.clone() },
            );
            return Ok((content_type, body));
        }
        Ok((200, content_type, _)) => (502, format!("unexpected content type {content_type}")),
        Ok((status, _, _)) => (if status == 404 { 404 } else { 502 }, format!("upstream HTTP {status}")),
        Err(e) => (502, e),
    };

    log(format!("{key} failed: {}", failure.1));
    if let Some(stale) = cache().lock().get(key) {
        return Ok((stale.content_type.clone(), stale.body.clone()));
    }
    Err(failure)
}
