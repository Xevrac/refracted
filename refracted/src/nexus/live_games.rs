//! Live games list published to Nexus: one `live_games` row per lobby/match, plus a heartbeat in
//! `settings` so the admin page can tell an empty list from a backend that stopped publishing.
//! Schema is owned by the frontend (`php artisan nexus:migrate`).

use std::time::Duration;

use mysql::prelude::Queryable;
use mysql::params;

use crate::client::cnc::game_state::{live_games_snapshot, LiveGame};

const INTERVAL: Duration = Duration::from_secs(5);
pub const HEARTBEAT_KEY: &str = "live_games_heartbeat";

/// Start the publisher thread. Headless MySQL boot only: a desktop instance must not
/// overwrite the hosted list with its local games.
pub fn start() {
    static STARTED: std::sync::Once = std::sync::Once::new();
    STARTED.call_once(|| {
        let _ = std::thread::Builder::new()
            .name("nexus-live-games".into())
            .spawn(run);
    });
}

fn run() {
    let mut last_error: Option<String> = None;
    loop {
        std::thread::sleep(INTERVAL);
        let Some(store) = crate::nexus::identity::current_identity_store() else {
            continue;
        };
        let result = store.conn().and_then(|mut conn| publish(&mut conn, &live_games_snapshot()));
        match result {
            Ok(()) => {
                if last_error.take().is_some() {
                    crate::nexus::log_blaze_to_nexus("Live games: publishing again");
                }
            }
            Err(e) => {
                if last_error.as_deref() != Some(e.as_str()) {
                    crate::nexus::log_blaze_to_nexus(format!(
                        "Live games: publish failed ({e}); run `php artisan nexus:migrate --force` if live_games is missing"
                    ));
                    last_error = Some(e);
                }
            }
        }
    }
}

fn publish(conn: &mut mysql::PooledConn, games: &[LiveGame]) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut tx = conn
        .start_transaction(mysql::TxOpts::default())
        .map_err(|e| format!("mysql tx: {e}"))?;
    tx.query_drop("DELETE FROM live_games")
        .map_err(|e| format!("mysql live_games clear: {e}"))?;
    for g in games {
        let roster = serde_json::to_string(&g.roster).unwrap_or_else(|_| "[]".into());
        tx.exec_drop(
            "INSERT INTO live_games
                (gid, name, map, map_path, phase, stage, players, humans, spectators, max_players,
                 host_persona, host_name, dedicated_session, region, version, password_protected,
                 match_secs, roster, updated_at)
             VALUES (:gid, :name, :map, :map_path, :phase, :stage, :players, :humans, :spectators,
                 :max_players, :host_persona, :host_name, :dedicated_session, :region, :version,
                 :password_protected, :match_secs, :roster, :updated_at)",
            params! {
                "gid" => g.gid,
                "name" => &g.name,
                "map" => &g.map,
                "map_path" => &g.map_path,
                "phase" => g.phase,
                "stage" => &g.stage,
                "players" => g.players as u32,
                "humans" => g.humans as u32,
                "spectators" => g.spectators as u32,
                "max_players" => g.max_players as u32,
                "host_persona" => g.host_persona,
                "host_name" => g.host_name.as_deref(),
                "dedicated_session" => g.dedicated_session,
                "region" => g.region.as_deref(),
                "version" => g.version.as_deref(),
                "password_protected" => u8::from(g.password_protected),
                "match_secs" => g.match_secs,
                "roster" => roster,
                "updated_at" => &now,
            },
        )
        .map_err(|e| format!("mysql live_games insert gid={}: {e}", g.gid))?;
    }
    tx.exec_drop(
        "INSERT INTO settings (`key`, value, updated_at) VALUES (:k, :v, :now)
         ON DUPLICATE KEY UPDATE value = :v, updated_at = :now",
        params! { "k" => HEARTBEAT_KEY, "v" => games.len().to_string(), "now" => &now },
    )
    .map_err(|e| format!("mysql live_games heartbeat: {e}"))?;
    tx.commit().map_err(|e| format!("mysql tx commit: {e}"))
}
