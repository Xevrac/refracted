//! In-game news proxy: the shell reads `/cnc/news/{realm}` and `/cnc/news-image?u=` from this host,
//! and Refracted fetches the Aurora site (`CNC_NEWS_URL`) over HTTPS. EAWebKit cannot complete the
//! site's TLS handshake, and the playtest feed key (`CNC_NEWS_DEV_KEY`) never reaches the client.

use std::time::Duration;

use super::remote_fetch::{cached_fetch, error_response};
use crate::http::handlers::handlers_module::HttpResponse;

const FEED_TTL: Duration = Duration::from_secs(30);
const IMAGE_TTL: Duration = Duration::from_secs(600);
const MAX_FEED_BYTES: u64 = 512 * 1024;
const MAX_IMAGE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CACHED_IMAGES: usize = 64;

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

    match cached_fetch(&format!("news-feed:{realm}"), &url, FEED_TTL, MAX_FEED_BYTES, None, |ct| {
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

    match cached_fetch(&format!("news-image:{url}"), &url, IMAGE_TTL, MAX_IMAGE_BYTES, Some(MAX_CACHED_IMAGES), |ct| {
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
