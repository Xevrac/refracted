//! In-game news proxy: the shell reads `/cnc/news/{realm}` and `/cnc/news-image?u=` from this host,
//! and Refracted fetches the Aurora site (`CNC_NEWS_URL`) over HTTPS. EAWebKit cannot complete the
//! site's TLS handshake, and the playtest feed key (`CNC_NEWS_DEV_KEY`) never reaches the client.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::http::handlers::handlers_module::HttpResponse;

const FEED_TTL: Duration = Duration::from_secs(30);
const IMAGE_TTL: Duration = Duration::from_secs(600);
const FETCH_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_FEED_BYTES: u64 = 512 * 1024;
const MAX_IMAGE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CACHED_IMAGES: usize = 64;

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

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn site_base() -> Option<String> {
    let base = env_non_empty("CNC_NEWS_URL")?;
    let lower = base.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return None;
    }
    Some(base.trim_end_matches('/').to_string())
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 3;
            }
            b'%' => return None,
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    query?
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| percent_decode(v))
}

fn error_response(status: u16, message: &str) -> HttpResponse {
    let body = serde_json::json!({ "error": message }).to_string().into_bytes();
    let mut response = HttpResponse::new(status, "application/json", body);
    response
        .headers
        .insert("Cache-Control".to_string(), "no-store".to_string());
    response
}

fn log(message: String) {
    crate::console_println!("\x1b[38;2;100;200;255m[CNC]\x1b[0m news proxy: {message}");
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
fn cached_fetch(
    key: &str,
    url: &str,
    ttl: Duration,
    max_bytes: u64,
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
            if !map.contains_key(key) && key.starts_with("image:") {
                let images = map.keys().filter(|k| k.starts_with("image:")).count();
                if images >= MAX_CACHED_IMAGES {
                    let oldest = map
                        .iter()
                        .filter(|(k, _)| k.starts_with("image:"))
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

/// GET /cnc/news/{prod|dev}
pub fn handle_feed(realm: &str) -> HttpResponse {
    if realm != "prod" && realm != "dev" {
        return error_response(404, "unknown realm");
    }
    let Some(base) = site_base() else {
        return error_response(503, "news_url is not configured");
    };
    let mut url = format!("{base}/api/shell/news/{realm}");
    if realm == "dev" {
        if let Some(key) = env_non_empty("CNC_NEWS_DEV_KEY") {
            url.push_str("?key=");
            url.push_str(&percent_encode(&key));
        }
    }

    match cached_fetch(&format!("feed:{realm}"), &url, FEED_TTL, MAX_FEED_BYTES, |ct| {
        ct.eq_ignore_ascii_case("application/json")
    }) {
        Ok((_, body)) => {
            let mut response = HttpResponse::new(200, "application/json", body);
            response
                .headers
                .insert("Cache-Control".to_string(), "no-cache, no-store, must-revalidate".to_string());
            response
        }
        Err((status, message)) => error_response(status, &message),
    }
}

/// GET /cnc/news-image?u={url}; only images under the configured site are proxied.
pub fn handle_image(query: Option<&str>) -> HttpResponse {
    let Some(base) = site_base() else {
        return error_response(503, "news_url is not configured");
    };
    let Some(url) = query_param(query, "u") else {
        return error_response(400, "missing u");
    };
    let allowed = url
        .get(..base.len() + 1)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&format!("{base}/")));
    if !allowed || url.contains("..") {
        return error_response(403, "image is not on the news site");
    }

    match cached_fetch(&format!("image:{url}"), &url, IMAGE_TTL, MAX_IMAGE_BYTES, |ct| {
        ct.to_ascii_lowercase().starts_with("image/")
    }) {
        Ok((content_type, body)) => {
            let mut response = HttpResponse::new(200, &content_type, body);
            response
                .headers
                .insert("Cache-Control".to_string(), "public, max-age=600".to_string());
            response
        }
        Err((status, message)) => error_response(status, &message),
    }
}
