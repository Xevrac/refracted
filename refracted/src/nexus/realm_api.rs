//! Read-only realm API for the Nexus frontend: match reporting and live games.
//! Every route needs `Authorization: Bearer <realm_key>`; with no key configured the API is off.

use crate::http::HttpResponse;

const PREFIX: &str = "/nexus/v1/";
const MAX_ROWS: u32 = 200;

/// `None` when `path` is not a realm API route.
pub fn try_handle(method: &str, path: &str, authorization: Option<&str>) -> Option<HttpResponse> {
    let (route, query) = path.split_once('?').unwrap_or((path, ""));
    let route = route.strip_prefix(PREFIX)?;
    let key = crate::common::app_env::current_app_env()
        .map(|env| env.nexus.realm_key)
        .unwrap_or_default();
    if key.is_empty() || !bearer_matches(authorization, &key) {
        return Some(HttpResponse::new(404, "text/plain", b"Not Found".to_vec()));
    }
    if method != "GET" {
        return Some(HttpResponse::new(405, "text/plain", b"Method Not Allowed".to_vec()));
    }
    let result = match route.trim_end_matches('/') {
        "stats/totals" => crate::nexus::identity::report_totals().map(|t| serde_json::json!(t)),
        "stats/personas" => {
            let limit = query_param(query, "limit")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(100)
                .clamp(1, MAX_ROWS);
            crate::nexus::identity::report_rows(None, limit).map(|rows| serde_json::json!({ "personas": rows }))
        }
        "live-games" => Ok(serde_json::json!({
            "generated_at": chrono::Utc::now().timestamp(),
            "games": crate::client::cnc::game_state::live_games_snapshot(),
        })),
        other => match other.strip_prefix("stats/personas/").and_then(|id| id.parse::<i64>().ok()) {
            Some(pid) => crate::nexus::identity::report_rows(Some(pid), 1)
                .map(|rows| serde_json::json!({ "persona": rows.into_iter().next() })),
            None => return Some(HttpResponse::new(404, "text/plain", b"Not Found".to_vec())),
        },
    };
    Some(match result {
        Ok(body) => HttpResponse::new(200, "application/json", body.to_string().into_bytes()),
        Err(e) => {
            tracing::warn!("realm api {route}: {e}");
            HttpResponse::new(
                503,
                "application/json",
                serde_json::json!({ "error": e }).to_string().into_bytes(),
            )
        }
    })
}

fn bearer_matches(authorization: Option<&str>, key: &str) -> bool {
    let Some(presented) = authorization.and_then(|h| h.trim().strip_prefix("Bearer ")) else {
        return false;
    };
    // Comparing digests keeps the comparison time independent of where the keys differ.
    use sha2::{Digest, Sha256};
    Sha256::digest(presented.trim().as_bytes()) == Sha256::digest(key.as_bytes())
}

fn query_param<'a>(query: &'a str, name: &str) -> Option<&'a str> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_must_match_exactly() {
        assert!(bearer_matches(Some("Bearer s3cret"), "s3cret"));
        assert!(!bearer_matches(Some("Bearer s3cre"), "s3cret"));
        assert!(!bearer_matches(Some("s3cret"), "s3cret"));
        assert!(!bearer_matches(None, "s3cret"));
    }

    #[test]
    fn other_paths_are_not_claimed() {
        assert!(try_handle("GET", "/cnc/online-count", None).is_none());
    }
}
