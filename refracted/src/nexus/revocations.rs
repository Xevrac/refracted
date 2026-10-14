//! Nexus revocation feed: active bans and revoked tickets, polled from Nexus every minute.
//! Logins never wait on Nexus; a realm that cannot reach it keeps the last list it had.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use parking_lot::Mutex;

use crate::common::app_env::NexusRealmParams;

const INTERVAL: Duration = Duration::from_secs(60);
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Ban {
    pub user_id: i64,
    /// Unix seconds; `None` = permanent.
    pub until: Option<i64>,
    #[serde(default)]
    pub discord_id: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct FeedDto {
    #[serde(default)]
    bans: Vec<Ban>,
    #[serde(default)]
    revoked: Vec<String>,
}

#[derive(Default)]
struct Feed {
    bans: HashMap<i64, Ban>,
    revoked: HashSet<String>,
    loaded: bool,
}

static FEED: Mutex<Option<Feed>> = Mutex::new(None);

/// Fetch once now, then keep polling on a background thread.
pub fn start(params: &NexusRealmParams) {
    if params.nexus_url.is_empty() || params.realm.is_empty() || params.realm_key.is_empty() {
        crate::nexus::log_nexus_to_blaze(
            "Revocations: realm, nexus_url or realm_key unset; bans and sign-outs will not reach this realm",
        );
        return;
    }
    static STARTED: std::sync::Once = std::sync::Once::new();
    let params = params.clone();
    STARTED.call_once(move || {
        // Empty sentinel: the first result is always logged.
        let mut last_error = poll(&params, Some(String::new()));
        let _ = std::thread::Builder::new()
            .name("nexus-revocations".into())
            .spawn(move || loop {
                std::thread::sleep(INTERVAL);
                last_error = poll(&params, last_error.take());
            });
    });
}

/// Logs only changes of state, not every failed minute.
fn poll(params: &NexusRealmParams, last_error: Option<String>) -> Option<String> {
    match fetch(params) {
        Ok(dto) => {
            let count = (dto.bans.len(), dto.revoked.len());
            replace(dto);
            if last_error.is_some() {
                crate::nexus::log_nexus_to_blaze(format!(
                    "Revocations: {} bans, {} revoked sessions from {}",
                    count.0, count.1, params.nexus_url
                ));
            }
            None
        }
        Err(e) => {
            if last_error.as_deref() != Some(e.as_str()) {
                crate::nexus::log_nexus_to_blaze(format!(
                    "Revocations: fetch failed ({e}); keeping the last list"
                ));
            }
            Some(e)
        }
    }
}

fn fetch(params: &NexusRealmParams) -> Result<FeedDto, String> {
    let url = format!("{}/realm/v1/revocations", params.nexus_url);
    let body = ureq::get(&url)
        .timeout(TIMEOUT)
        .set("Authorization", &format!("Bearer {}", params.realm_key))
        .set("X-Realm", &params.realm)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&body).map_err(|e| format!("bad feed json: {e}"))
}

fn replace(dto: FeedDto) {
    *FEED.lock() = Some(Feed {
        bans: dto.bans.into_iter().map(|b| (b.user_id, b)).collect(),
        revoked: dto.revoked.into_iter().collect(),
        loaded: true,
    });
}

/// Active ban: `Some(None)` permanent, `Some(Some(unix))` until then, `None` not banned.
pub fn active_ban(user_id: i64) -> Option<Option<i64>> {
    let now = chrono::Utc::now().timestamp();
    let guard = FEED.lock();
    let ban = guard.as_ref()?.bans.get(&user_id)?;
    match ban.until {
        Some(until) if until <= now => None,
        until => Some(until),
    }
}

pub fn ban_discord_id(user_id: i64) -> Option<String> {
    FEED.lock().as_ref()?.bans.get(&user_id)?.discord_id.clone()
}

pub fn is_revoked(jti: &str) -> bool {
    FEED.lock().as_ref().is_some_and(|f| f.loaded && f.revoked.contains(jti))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_answers_bans_and_revocations() {
        let now = chrono::Utc::now().timestamp();
        replace(FeedDto {
            bans: vec![
                Ban { user_id: 1, until: None, discord_id: Some("42".into()) },
                Ban { user_id: 2, until: Some(now + 600), discord_id: None },
                Ban { user_id: 3, until: Some(now - 1), discord_id: None },
            ],
            revoked: vec!["gone".into()],
        });
        assert_eq!(active_ban(1), Some(None));
        assert_eq!(active_ban(2), Some(Some(now + 600)));
        assert_eq!(active_ban(3), None);
        assert_eq!(active_ban(4), None);
        assert_eq!(ban_discord_id(1).as_deref(), Some("42"));
        assert!(is_revoked("gone"));
        assert!(!is_revoked("live"));
    }
}
