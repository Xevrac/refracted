use mysql::prelude::Queryable;
use mysql::{params, Pool};

use crate::common::app_env::MysqlParams;

#[derive(Debug, Clone)]
pub struct MatchStats {
    pub persona_id: i64,
    pub wins: i32,
    pub losses: i32,
    pub win_streak: i32,
    pub loss_streak: i32,
    /// General id (RtsGeneral ServerId) and faction of the last reported match.
    pub last_general_id: Option<i64>,
    pub last_faction: Option<String>,
    /// Shortest reported match (any result) and fastest win, in seconds from StartGame.
    pub shortest_match_secs: Option<i32>,
    pub best_win_secs: Option<i32>,
}

/// What GameReporting knows about a match beyond its result (from the CreateGame roster and
/// StartGame). Every field is optional: nothing is recorded that was not seen.
#[derive(Debug, Clone, Default)]
pub struct MatchDetails {
    pub general_id: Option<i64>,
    pub faction: Option<String>,
    pub duration_secs: Option<i32>,
}

/// One reported match for a persona (`persona_match_history`), newest first.
#[derive(Debug, Clone)]
pub struct MatchEntry {
    pub victory: bool,
    /// Unix seconds (UTC).
    pub ended_at: i64,
    pub duration_secs: Option<i32>,
    pub general_id: Option<i64>,
    pub faction: Option<String>,
}

/// Leaderboard row: personas with at least one reported match, by wins, then fewest losses.
#[derive(Debug, Clone)]
pub struct LeaderRow {
    pub rank: i64,
    pub persona_id: i64,
    pub display_name: String,
    pub wins: i32,
    pub losses: i32,
}

/// One persona's totals as the Nexus reporting pages show them.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReportRow {
    pub persona_id: i64,
    pub user_id: Option<i64>,
    pub display_name: Option<String>,
    pub wins: i32,
    pub losses: i32,
    pub win_streak: i32,
    pub loss_streak: i32,
    /// Unix seconds (UTC).
    pub updated_at: Option<i64>,
    pub last_general_id: Option<i64>,
    pub last_faction: Option<String>,
    pub shortest_match_secs: Option<i32>,
    pub best_win_secs: Option<i32>,
}

/// Realm-wide totals.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReportTotals {
    pub personas: i64,
    pub wins: i64,
    pub losses: i64,
}

#[derive(Debug, Clone)]
pub struct BoundSession {
    pub user_id: i64,
    pub persona_id: i64,
    pub email: String,
    pub display_name: String,
    pub expired: bool,
}

#[derive(Clone)]
pub struct IdentityStore {
    pool: Pool,
}

impl IdentityStore {
    pub fn open_mysql(mysql: &MysqlParams) -> Result<Self, String> {
        let (host, port) = mysql.host_port();
        let opts = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(host.clone()))
            .tcp_port(port)
            .user(Some(mysql.user.clone()))
            .pass(Some(mysql.pass.clone()))
            .db_name(Some(mysql.database.clone()))
            .prefer_socket(false);
        let pool = Pool::new(opts).map_err(|e| {
            format!(
                "failed to connect to mysql {}:{}/{} as {}: {e}",
                host, port, mysql.database, mysql.user
            )
        })?;
        Ok(Self { pool })
    }

    pub(crate) fn conn(&self) -> Result<mysql::PooledConn, String> {
        self.pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))
    }

    /// Bring the realm schema up to date. Refuses to boot on failure.
    pub fn migrate(&self) -> Result<Vec<u32>, String> {
        let mut conn = self.conn()?;
        super::super::realm_schema::migrate(&mut conn)
    }

    pub fn persona_count(&self) -> Result<i64, String> {
        self.count("SELECT COUNT(*) FROM personas")
    }

    /// Mirror the Nexus persona a verified ticket names, for leaderboards and chat tags.
    pub fn upsert_persona(&self, persona_id: i64, user_id: i64, display_name: &str, name_tag: &str) -> Result<(), String> {
        let mut conn = self.conn()?;
        conn.exec_drop(
            "INSERT INTO personas (id, user_id, display_name, name_tag, updated_at)
             VALUES (:id, :user_id, :display_name, :name_tag, UTC_TIMESTAMP())
             ON DUPLICATE KEY UPDATE user_id = :user_id, display_name = :display_name,
                name_tag = :name_tag, updated_at = UTC_TIMESTAMP()",
            params! {
                "id" => persona_id,
                "user_id" => user_id,
                "display_name" => display_name,
                "name_tag" => name_tag,
            },
        )
        .map_err(|e| format!("mysql persona mirror: {e}"))
    }

    /// Persona ids whose account carries a chat name tag (`developer` or `playtester`).
    pub fn tagged_personas(&self) -> Result<std::collections::HashMap<i64, String>, String> {
        let mut conn = self.conn()?;
        let rows: Vec<(i64, String)> = conn
            .query_map(
                "SELECT id, name_tag FROM personas WHERE name_tag IN ('developer', 'playtester')",
                |(id, tag)| (id, tag),
            )
            .map_err(|e| format!("mysql name tags: {e}"))?;
        Ok(rows.into_iter().collect())
    }

    pub fn match_stats(&self, persona_id: i64) -> Result<MatchStats, String> {
        let mut conn = self.conn()?;
        #[allow(clippy::type_complexity)]
        let row: Option<(i32, i32, i32, i32, Option<i64>, Option<String>, Option<i32>, Option<i32>)> = conn
            .exec_first(
                "SELECT wins, losses, win_streak, loss_streak,
                        last_general_id, last_faction, shortest_match_secs, best_win_secs
                 FROM persona_match_stats WHERE persona_id = :id",
                params! { "id" => persona_id },
            )
            .map_err(|e| format!("mysql match stats: {e}"))?;
        Ok(match row {
            Some((wins, losses, win_streak, loss_streak, last_general_id, last_faction, shortest, best)) => {
                MatchStats {
                    persona_id,
                    wins,
                    losses,
                    win_streak,
                    loss_streak,
                    last_general_id,
                    last_faction,
                    shortest_match_secs: shortest,
                    best_win_secs: best,
                }
            }
            None => MatchStats {
                persona_id,
                wins: 0,
                losses: 0,
                win_streak: 0,
                loss_streak: 0,
                last_general_id: None,
                last_faction: None,
                shortest_match_secs: None,
                best_win_secs: None,
            },
        })
    }

    pub fn record_match_outcome(
        &self,
        persona_id: i64,
        victory: bool,
        details: &MatchDetails,
    ) -> Result<MatchStats, String> {
        if persona_id <= 0 {
            return Err("persona id is required".into());
        }
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let win_inc: i32 = if victory { 1 } else { 0 };
        let loss_inc: i32 = if victory { 0 } else { 1 };
        let mut conn = self.conn()?;
        let win_secs = if victory { details.duration_secs } else { None };
        // Unknown details (NULL) leave the stored values alone.
        conn.exec_drop(
            "INSERT INTO persona_match_stats
                (persona_id, wins, losses, win_streak, loss_streak, updated_at,
                 last_general_id, last_faction, shortest_match_secs, best_win_secs)
             VALUES (:id, :wins, :losses, :win_streak, :loss_streak, :updated_at,
                 :general, :faction, :secs, :win_secs)
             ON DUPLICATE KEY UPDATE
                wins = wins + :wins,
                losses = losses + :losses,
                win_streak = IF(:victory = 1, win_streak + 1, 0),
                loss_streak = IF(:victory = 1, 0, loss_streak + 1),
                updated_at = :updated_at,
                last_general_id = COALESCE(:general, last_general_id),
                last_faction = COALESCE(:faction, last_faction),
                shortest_match_secs = IF(:secs IS NULL, shortest_match_secs,
                    IF(shortest_match_secs IS NULL OR :secs < shortest_match_secs, :secs, shortest_match_secs)),
                best_win_secs = IF(:win_secs IS NULL, best_win_secs,
                    IF(best_win_secs IS NULL OR :win_secs < best_win_secs, :win_secs, best_win_secs))",
            params! {
                "id" => persona_id,
                "wins" => win_inc,
                "losses" => loss_inc,
                "win_streak" => win_inc,
                "loss_streak" => loss_inc,
                "victory" => win_inc,
                "updated_at" => now.clone(),
                "general" => details.general_id,
                "faction" => details.faction.as_deref(),
                "secs" => details.duration_secs,
                "win_secs" => win_secs,
            },
        )
        .map_err(|e| format!("mysql match record: {e}"))?;
        // History feeds the profile pages only; a failed insert must not lose the record above.
        if let Err(e) = self.append_match_history(persona_id, victory, &now, details) {
            tracing::warn!("match history insert failed persona={persona_id}: {e}");
        }
        self.match_stats(persona_id)
    }

    fn append_match_history(
        &self,
        persona_id: i64,
        victory: bool,
        ended_at: &str,
        details: &MatchDetails,
    ) -> Result<(), String> {
        let mut conn = self.conn()?;
        conn.exec_drop(
            "INSERT INTO persona_match_history
                (persona_id, victory, ended_at, duration_secs, general_id, faction)
             VALUES (:id, :victory, :ended_at, :secs, :general, :faction)",
            params! {
                "id" => persona_id,
                "victory" => if victory { 1 } else { 0 },
                "ended_at" => ended_at,
                "secs" => details.duration_secs,
                "general" => details.general_id,
                "faction" => details.faction.as_deref(),
            },
        )
        .map_err(|e| format!("mysql match history: {e}"))
    }

    /// Newest first, at most `limit` rows.
    pub fn match_history(&self, persona_id: i64, limit: u32) -> Result<Vec<MatchEntry>, String> {
        let mut conn = self.conn()?;
        #[allow(clippy::type_complexity)]
        let rows: Vec<(i32, String, Option<i32>, Option<i64>, Option<String>)> = conn
            .exec_map(
                "SELECT victory, CAST(ended_at AS CHAR), duration_secs, general_id, faction
                 FROM persona_match_history
                 WHERE persona_id = :id
                 ORDER BY ended_at DESC, id DESC
                 LIMIT :limit",
                params! { "id" => persona_id, "limit" => limit },
                |(victory, ended_at, duration_secs, general_id, faction)| {
                    (victory, ended_at, duration_secs, general_id, faction)
                },
            )
            .map_err(|e| format!("mysql match history read: {e}"))?;
        Ok(rows
            .into_iter()
            .filter_map(|(victory, ended_at, duration_secs, general_id, faction)| {
                let at = chrono::NaiveDateTime::parse_from_str(&ended_at, "%Y-%m-%d %H:%M:%S").ok()?;
                Some(MatchEntry {
                    victory: victory != 0,
                    ended_at: at.and_utc().timestamp(),
                    duration_secs,
                    general_id,
                    faction,
                })
            })
            .collect())
    }

    /// Every persona with a reported match, ranked by wins, then fewest losses, then persona id.
    pub fn leaderboard(&self) -> Result<Vec<LeaderRow>, String> {
        let mut conn = self.conn()?;
        let rows: Vec<(i64, String, i32, i32)> = conn
            .query_map(
                "SELECT s.persona_id, p.display_name, s.wins, s.losses
                 FROM persona_match_stats s
                 INNER JOIN personas p ON p.id = s.persona_id
                 WHERE s.wins + s.losses > 0
                 ORDER BY s.wins DESC, s.losses ASC, s.persona_id ASC",
                |(persona_id, display_name, wins, losses)| (persona_id, display_name, wins, losses),
            )
            .map_err(|e| format!("mysql leaderboard: {e}"))?;
        Ok(rows
            .into_iter()
            .enumerate()
            .map(|(i, (persona_id, display_name, wins, losses))| LeaderRow {
                rank: i as i64 + 1,
                persona_id,
                display_name,
                wins,
                losses,
            })
            .collect())
    }

    pub fn report_totals(&self) -> Result<ReportTotals, String> {
        let mut conn = self.conn()?;
        let row: Option<(i64, i64, i64)> = conn
            .query_first(
                "SELECT COUNT(*), CAST(COALESCE(SUM(wins), 0) AS SIGNED), CAST(COALESCE(SUM(losses), 0) AS SIGNED)
                 FROM persona_match_stats",
            )
            .map_err(|e| format!("mysql report totals: {e}"))?;
        let (personas, wins, losses) = row.unwrap_or((0, 0, 0));
        Ok(ReportTotals { personas, wins, losses })
    }

    /// Most active personas first, or the one persona asked for.
    pub fn report_rows(&self, persona_id: Option<i64>, limit: u32) -> Result<Vec<ReportRow>, String> {
        let mut conn = self.conn()?;
        let rows: Vec<mysql::Row> = conn
            .exec(
                "SELECT s.persona_id, p.user_id, p.display_name, s.wins, s.losses, s.win_streak, s.loss_streak,
                        TIMESTAMPDIFF(SECOND, '1970-01-01 00:00:00', s.updated_at), s.last_general_id, s.last_faction,
                        s.shortest_match_secs, s.best_win_secs
                 FROM persona_match_stats s
                 LEFT JOIN personas p ON p.id = s.persona_id
                 WHERE (:pid IS NULL OR s.persona_id = :pid)
                 ORDER BY (s.wins + s.losses) DESC, s.persona_id ASC
                 LIMIT :limit",
                params! { "pid" => persona_id, "limit" => limit },
            )
            .map_err(|e| format!("mysql report rows: {e}"))?;
        Ok(rows
            .into_iter()
            .map(|r| ReportRow {
                persona_id: column(&r, 0).unwrap_or_default(),
                user_id: column(&r, 1),
                display_name: column(&r, 2),
                wins: column(&r, 3).unwrap_or_default(),
                losses: column(&r, 4).unwrap_or_default(),
                win_streak: column(&r, 5).unwrap_or_default(),
                loss_streak: column(&r, 6).unwrap_or_default(),
                updated_at: column(&r, 7),
                last_general_id: column(&r, 8),
                last_faction: column(&r, 9),
                shortest_match_secs: column(&r, 10),
                best_win_secs: column(&r, 11),
            })
            .collect())
    }

    fn count(&self, sql: &str) -> Result<i64, String> {
        let mut conn = self
            .pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))?;
        conn.query_first(sql)
            .map_err(|e| format!("mysql count: {e}"))?
            .ok_or_else(|| "mysql count returned no row".to_string())
    }
}

/// Nullable column `i` of `row`; NULL and unconvertible values are `None`.
fn column<T: mysql::prelude::FromValue>(row: &mysql::Row, i: usize) -> Option<T> {
    row.get_opt::<Option<T>, _>(i).and_then(Result::ok).flatten()
}
