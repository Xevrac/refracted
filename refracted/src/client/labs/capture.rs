//! Optional replay of captured HTTP responses: `data/{md5}.bin` raw bodies.
//!
//! **Search order:** `REFRACTED_LABS_DATA` (if set), then `{executable_dir}/data/`, then a bundled `labs blaze/data` directory under the crate manifest.
//!
//! **Not replayed here:** account or gateway gRPC hosts. The URL prefixes below select gateway capture replay.
//!
//! Replay targets gRPC-style traffic on configured `*.ops.dice.se` gateway hosts (see constants in this module).

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::OnceLock;

use parking_lot::Mutex;
use tracing::debug;

use crate::http::handlers::HttpResponse;

fn labs_capture_shown_paths() -> &'static Mutex<HashSet<String>> {
    static SHOWN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    SHOWN.get_or_init(|| Mutex::new(HashSet::new()))
}

const BFLABS_PREFIX: &str = "https://bflabs-prod-gt-cgw.ops.dice.se/";
const BFLABS_EVENTBRIDGE_PREFIX: &str = "https://bflabs-prod-eventbridge.ops.dice.se/";

fn canonicalize_gateway_base(url: &str) -> String {
    url.replace(
        "https://eventprod-mp-cgw.ops.dice.se/",
        BFLABS_PREFIX,
    )
    .replace(
        "https://santiago-prod-mp-cgw.ops.dice.se/",
        BFLABS_PREFIX,
    )
    .replace(
        "https://eventprod-eventbridge.ops.dice.se/",
        BFLABS_EVENTBRIDGE_PREFIX,
    )
}

fn strip_query_and_fragment(url: &str) -> &str {
    let end = url
        .find(|c| c == '?' || c == '#')
        .unwrap_or(url.len());
    &url[..end]
}

fn replace_first_client_not_after_santiago(path: &str) -> String {
    const NEEDLE: &str = "client.";
    let mut search = 0usize;
    while let Some(rel) = path[search..].find(NEEDLE) {
        let pos = search + rel;
        let before_ok = pos >= 9 && &path[pos - 9..pos] == "santiago.";
        if !before_ok {
            let mut out = String::with_capacity(path.len() + 10);
            out.push_str(&path[..pos]);
            out.push_str("santiago.client.");
            out.push_str(&path[pos + NEEDLE.len()..]);
            return out;
        }
        search = pos + NEEDLE.len();
    }
    path.to_string()
}

pub fn normalize_url(url: &str) -> String {
    let canonical = canonicalize_gateway_base(url);
    let base_owned = strip_query_and_fragment(&canonical).to_string();
    let base = base_owned.as_str();
    if let Some(rest) = base.strip_prefix(BFLABS_PREFIX) {
        let path_new = if rest.starts_with("client.") && !rest.starts_with("santiago.client.") {
            format!("santiago.{rest}")
        } else {
            replace_first_client_not_after_santiago(rest)
        };
        return format!("{BFLABS_PREFIX}{path_new}");
    }
    base.to_string()
}

const SPECIAL_GRPC_URLS: &[&str] = &[
    "https://bflabs-prod-gt-cgw.ops.dice.se/santiago.client.schedule.ClientSchedule/getConfig",
    "https://eventprod-mp-cgw.ops.dice.se/santiago.client.schedule.ClientSchedule/getConfigs",
    "https://bflabs-prod-gt-cgw.ops.dice.se/santiago.client.schedule.ClientSchedule/getConfigs",
    "https://santiago-prod-mp-cgw.ops.dice.se/santiago.client.schedule.ClientSchedule/getConfigs",
];

fn special_body_tag(body: &[u8]) -> &'static str {
    if body.is_empty() {
        return "other";
    }
    let Ok(s) = std::str::from_utf8(body) else {
        return "other";
    };
    if s.contains("gamesettings") {
        return "gamesettings";
    }
    if s.contains("killswitches") {
        return "killswitches";
    }
    "other"
}

fn capture_resource_filename(url: &str, body: &[u8]) -> String {
    let n = normalize_url(url);
    if SPECIAL_GRPC_URLS.iter().any(|&u| u == n.as_str()) {
        let tag = special_body_tag(body);
        let combined = format!("{n}{tag}");
        let hash = hex::encode(md5::compute(combined.as_bytes()).0);
        format!("{tag}_{hash}.bin")
    } else {
        let hash = hex::encode(md5::compute(n.as_bytes()).0);
        format!("{hash}.bin")
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

pub fn parse_raw_response(raw: &[u8]) -> Option<HttpResponse> {
    let (header_end, sep_len, line_sep) = if let Some(idx) = find_bytes(raw, b"\r\n\r\n") {
        (idx, 4usize, "\r\n")
    } else if let Some(idx) = find_bytes(raw, b"\n\n") {
        (idx, 2usize, "\n")
    } else {
        return None;
    };
    let header_part = std::str::from_utf8(&raw[..header_end]).ok()?;
    let body = raw[header_end + sep_len..].to_vec();

    let mut lines = header_part.split(line_sep);
    let status_line = lines.next()?;
    let mut parts = status_line.split_whitespace();
    let _http = parts.next()?;
    let status_code: u16 = parts.next()?.parse().ok()?;

    let mut headers = HashMap::new();
    for line in lines {
        if let Some(colon) = line.find(": ") {
            let key = line[..colon].to_string();
            let value = line[colon + 2..].to_string();
            headers.insert(key, value);
        } else if let Some(colon) = line.find(':') {
            let key = line[..colon].trim().to_string();
            let value = line[colon + 1..].trim().to_string();
            if !key.is_empty() {
                headers.insert(key, value);
            }
        }
    }

    let content_type = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Some(HttpResponse {
        status_code,
        content_type,
        body,
        headers,
    })
}

fn labs_candidate_data_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(env) = std::env::var("REFRACTED_LABS_DATA") {
        if !env.is_empty() {
            out.push(PathBuf::from(env));
        }
    }
    if let Some(p) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("data")))
    {
        out.push(p);
    }
    let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "ref/SPOILER_bf6 blaze emu/bf6 blaze emu/labs blaze/data",
    );
    if bundled.is_dir() {
        out.push(bundled);
    }
    out
}

fn https_host(full_url: &str) -> Option<&str> {
    let rest = full_url.strip_prefix("https://")?;
    let end = rest
        .find(|c| c == '/' || c == ':')
        .unwrap_or(rest.len());
    Some(rest.get(..end).unwrap_or(rest))
}

fn labs_replay_allowed(full_url: &str) -> bool {
    let Some(host) = https_host(full_url) else {
        return false;
    };
    let h = host.to_ascii_lowercase();
    h.ends_with(".ops.dice.se")
        && (h.contains("bflabs-prod-gt-cgw")
            || h.contains("bflabs-prod-eventbridge")
            || h.contains("eventprod-eventbridge")
            || h.contains("santiago-prod-mp-cgw")
            || h.contains("eventprod-mp-cgw"))
}

pub fn try_load_captured_response(full_url: &str, body: &[u8]) -> Option<HttpResponse> {
    if !labs_replay_allowed(full_url) {
        return None;
    }
    let name = capture_resource_filename(full_url, body);
    for dir in labs_candidate_data_dirs() {
        let path = dir.join(&name);
        let Ok(raw) = std::fs::read(&path) else {
            continue;
        };
        let Some(parsed) = parse_raw_response(&raw) else {
            debug!(
                "Labs capture parse failed for {} ({} bytes)",
                path.display(),
                raw.len()
            );
            continue;
        };
        debug!(
            "Labs capture: served {} ({} bytes body)",
            path.display(),
            parsed.body.len()
        );
        if full_url.contains("ClientLocalization/getTranslations")
            || full_url.contains("UnifiedMessaging/fetchActions")
            || full_url.contains("ClientMenu/getScheduledMenu")
            || full_url.contains("ClientMenu/getMenuUpdates")
            || full_url.contains("ClientMenu/getStoreMenu")
        {
            let key = path.display().to_string();
            if labs_capture_shown_paths().lock().insert(key.clone()) {
                crate::console_println!(
                    "\x1b[38;2;120;220;170m[gRPC]\x1b[0m Labs capture hit: {}",
                    key
                );
            }
        }
        return Some(parsed);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_bflabs_client_prefix() {
        let u = "https://bflabs-prod-gt-cgw.ops.dice.se/client.foo/bar";
        assert_eq!(
            normalize_url(u),
            "https://bflabs-prod-gt-cgw.ops.dice.se/santiago.client.foo/bar"
        );
    }

    #[test]
    fn normalize_inserts_santiago_before_client() {
        let u = "https://bflabs-prod-gt-cgw.ops.dice.se/foo/client.schedule/x";
        let n = normalize_url(u);
        assert_eq!(
            n,
            "https://bflabs-prod-gt-cgw.ops.dice.se/foo/santiago.client.schedule/x"
        );
    }

    #[test]
    fn normalize_skips_santiago_client() {
        let u = "https://bflabs-prod-gt-cgw.ops.dice.se/santiago.client.schedule/getConfigs";
        assert_eq!(normalize_url(u), u);
    }

    #[test]
    fn replay_allowlist_dice_gateways_only() {
        assert!(!labs_replay_allowed(
            "https://accounts.grpc.ea.com/eadp.nexus.connect.grpc.v1.TokenService/GrantTokenByAuthorizationCode"
        ));
        assert!(!labs_replay_allowed("https://gcs.ea.com/application_id/x/device_id/y"));
        assert!(!labs_replay_allowed("https://update.layer.ea.com/bundle?x=1"));
        assert!(!labs_replay_allowed(
            "https://collector.errors.ea.com/eadp.errors.v1.CollectorService/SubmitBootSession"
        ));
        assert!(labs_replay_allowed(
            "https://bflabs-prod-gt-cgw.ops.dice.se/santiago.client.foo/getConfig"
        ));
        assert!(labs_replay_allowed(
            "https://bflabs-prod-eventbridge.ops.dice.se/eventbridge.EventBridge/clientEvents"
        ));
    }
}
