//! (H476) Country flags for the server browser: `GET /cnc/flag/{cc}.png`.
//!
//! `cc` is a dedicated server's two-letter region (`DedicatedServerEntry::region`, from
//! `-Prism.Region` or the server's Windows region). The image comes from flagcdn.com through
//! [`super::remote_fetch`], since EAWebKit cannot fetch an HTTPS image itself.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use super::remote_fetch::{cached_fetch, error_response};
use crate::http::handlers::handlers_module::HttpResponse;

const FLAG_SOURCE: &str = "https://flagcdn.com/w40/";
const FLAG_TTL: Duration = Duration::from_secs(7 * 24 * 3600);
/// After a failed fetch, the flag is not asked for again for this long.
const FLAG_RETRY_AFTER: Duration = Duration::from_secs(300);
const MAX_FLAG_BYTES: u64 = 64 * 1024;

fn failures() -> &'static Mutex<HashMap<String, Instant>> {
    static FAILED: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    FAILED.get_or_init(|| Mutex::new(HashMap::new()))
}

/// GET /cnc/flag/{cc}.png
pub fn handle_flag(file: &str) -> HttpResponse {
    let cc = file.strip_suffix(".png").unwrap_or(file).to_ascii_lowercase();
    if cc.len() != 2 || !cc.bytes().all(|b| b.is_ascii_lowercase()) {
        return error_response(404, "unknown flag");
    }
    if failures()
        .lock()
        .get(&cc)
        .is_some_and(|at| at.elapsed() < FLAG_RETRY_AFTER)
    {
        return error_response(404, "flag unavailable");
    }
    let url = format!("{FLAG_SOURCE}{cc}.png");
    match cached_fetch(&format!("flag:{cc}"), &url, FLAG_TTL, MAX_FLAG_BYTES, None, |ct| {
        ct.to_ascii_lowercase().starts_with("image/")
    }) {
        Ok((content_type, body)) => {
            failures().lock().remove(&cc);
            let mut response = HttpResponse::new(200, &content_type, body);
            response
                .headers
                .insert("Cache-Control".to_string(), "public, max-age=86400".to_string());
            response
        }
        Err((status, message)) => {
            failures().lock().insert(cc, Instant::now());
            error_response(if status == 404 { 404 } else { 502 }, &message)
        }
    }
}
