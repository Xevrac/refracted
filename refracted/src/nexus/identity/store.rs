use mysql::prelude::Queryable;
use mysql::{params, Pool};

use crate::common::app_env::MysqlParams;

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct PersonaRecord {
    pub id: i64,
    pub user_id: i64,
    pub display_name: String,
}

/// Session row after token/JWT lookup.
#[derive(Debug, Clone, Copy)]
pub struct MatchStats {
    pub persona_id: i64,
    pub wins: i32,
    pub losses: i32,
    pub win_streak: i32,
    pub loss_streak: i32,
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

    /// Schema is owned by the frontend (`php artisan nexus:migrate`); refuse to boot without it.
    pub fn verify_schema(&self) -> Result<(), String> {
        const REQUIRED: &[&str] = &[
            "users.id",
            "users.secret_hash",
            "users.secret_salt",
            "users.discord_id",
            "users.web_user_id",
            "personas.id",
            "personas.user_id",
            "auth_sessions.token_hash",
            "auth_sessions.jwt_id",
            "auth_sessions.client_ip",
            "auth_sessions.game_id",
            "settings.key",
            "signup_whitelist.discord_id",
            "bans.discord_id",
        ];
        let mut conn = self.conn()?;
        let present: Vec<String> = conn
            .query(
                "SELECT CONCAT(TABLE_NAME, '.', COLUMN_NAME) FROM information_schema.COLUMNS \
                 WHERE TABLE_SCHEMA = DATABASE()",
            )
            .map_err(|e| format!("mysql schema check: {e}"))?;
        let missing: Vec<&str> = REQUIRED
            .iter()
            .copied()
            .filter(|col| !present.iter().any(|p| p.eq_ignore_ascii_case(col)))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        Err(format!(
            "nexus schema incomplete (missing {}); run `php artisan nexus:migrate --force` on the frontend",
            missing.join(", ")
        ))
    }

    pub fn user_count(&self) -> Result<i64, String> {
        self.count("SELECT COUNT(*) FROM users")
    }

    pub fn persona_count(&self) -> Result<i64, String> {
        self.count("SELECT COUNT(*) FROM personas")
    }

    pub fn insert_user(&self, id: i64, username: &str, email: &str) -> Result<(), String> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let mut conn = self
            .pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))?;
        conn.exec_drop(
            "INSERT INTO users (id, username, email, created_at) VALUES (:id, :username, :email, :created_at)",
            params! {
                "id" => id,
                "username" => username,
                "email" => email,
                "created_at" => now,
            },
        )
        .map_err(|e| format!("mysql insert user: {e}"))
    }

    pub fn insert_persona(
        &self,
        id: i64,
        user_id: i64,
        display_name: &str,
    ) -> Result<(), String> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let mut conn = self
            .pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))?;
        conn.exec_drop(
            "INSERT INTO personas (id, user_id, display_name, created_at) VALUES (:id, :user_id, :display_name, :created_at)",
            params! {
                "id" => id,
                "user_id" => user_id,
                "display_name" => display_name,
                "created_at" => now,
            },
        )
        .map_err(|e| format!("mysql insert persona: {e}"))
    }

    pub fn personas_for_user(&self, user_id: i64) -> Result<Vec<PersonaRecord>, String> {
        let mut conn = self
            .pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))?;
        conn.exec_map(
            "SELECT id, user_id, display_name FROM personas WHERE user_id = :user_id ORDER BY id",
            params! { "user_id" => user_id },
            |(id, user_id, display_name)| PersonaRecord {
                id,
                user_id,
                display_name,
            },
        )
        .map_err(|e| format!("mysql personas: {e}"))
    }

    /// Active account ban (permanent or not yet expired).
    pub fn is_user_banned(&self, user_id: i64) -> Result<bool, String> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let mut conn = self.conn()?;
        let row: Option<(i64,)> = conn
            .exec_first(
                "SELECT id FROM bans
                 WHERE (user_id = :user_id
                        OR discord_id = (SELECT u.discord_id FROM users u WHERE u.id = :user_id))
                   AND lifted_at IS NULL
                   AND (banned_until IS NULL OR banned_until > :now)
                 LIMIT 1",
                params! { "user_id" => user_id, "now" => now },
            )
            .map_err(|e| {
                // Table may not exist until Nexus admin migration runs.
                format!("mysql ban lookup: {e}")
            })?;
        Ok(row.is_some())
    }

    pub fn user_discord_id(&self, user_id: i64) -> Result<Option<String>, String> {
        let mut conn = self.conn()?;
        let row: Option<(Option<String>,)> = conn
            .exec_first(
                "SELECT discord_id FROM users WHERE id = :id",
                params! { "id" => user_id },
            )
            .map_err(|e| format!("mysql discord lookup: {e}"))?;
        Ok(row.and_then(|(d,)| d))
    }

    /// Active ban: `Some(None)` permanent, `Some(Some(unix))` until that time, `None` not banned.
    pub fn active_ban_until(&self, user_id: i64) -> Result<Option<Option<i64>>, String> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let mut conn = self.conn()?;
        let row: Option<(Option<String>,)> = conn
            .exec_first(
                "SELECT CAST(banned_until AS CHAR) FROM bans
                 WHERE (user_id = :user_id
                        OR discord_id = (SELECT u.discord_id FROM users u WHERE u.id = :user_id))
                   AND lifted_at IS NULL
                   AND (banned_until IS NULL OR banned_until > :now)
                 ORDER BY banned_until IS NULL DESC, banned_until DESC
                 LIMIT 1",
                params! { "user_id" => user_id, "now" => now },
            )
            .map_err(|e| format!("mysql ban lookup: {e}"))?;
        Ok(row.map(|(until,)| {
            until.and_then(|t| {
                chrono::NaiveDateTime::parse_from_str(&t, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|dt| dt.and_utc().timestamp())
            })
        }))
    }

    pub fn match_stats(&self, persona_id: i64) -> Result<MatchStats, String> {
        self.ensure_match_stats()?;
        let mut conn = self.conn()?;
        let row: Option<(i32, i32, i32, i32)> = conn
            .exec_first(
                "SELECT wins, losses, win_streak, loss_streak
                 FROM persona_match_stats WHERE persona_id = :id",
                params! { "id" => persona_id },
            )
            .map_err(|e| format!("mysql match stats: {e}"))?;
        Ok(match row {
            Some((wins, losses, win_streak, loss_streak)) => MatchStats {
                persona_id,
                wins,
                losses,
                win_streak,
                loss_streak,
            },
            None => MatchStats {
                persona_id,
                wins: 0,
                losses: 0,
                win_streak: 0,
                loss_streak: 0,
            },
        })
    }

    pub fn record_match_outcome(&self, persona_id: i64, victory: bool) -> Result<MatchStats, String> {
        if persona_id <= 0 {
            return Err("persona id is required".into());
        }
        self.ensure_match_stats()?;
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let win_inc: i32 = if victory { 1 } else { 0 };
        let loss_inc: i32 = if victory { 0 } else { 1 };
        let mut conn = self.conn()?;
        conn.exec_drop(
            "INSERT INTO persona_match_stats
                (persona_id, wins, losses, win_streak, loss_streak, updated_at)
             VALUES (:id, :wins, :losses, :win_streak, :loss_streak, :updated_at)
             ON DUPLICATE KEY UPDATE
                wins = wins + :wins,
                losses = losses + :losses,
                win_streak = IF(:victory = 1, win_streak + 1, 0),
                loss_streak = IF(:victory = 1, 0, loss_streak + 1),
                updated_at = :updated_at",
            params! {
                "id" => persona_id,
                "wins" => win_inc,
                "losses" => loss_inc,
                "win_streak" => win_inc,
                "loss_streak" => loss_inc,
                "victory" => win_inc,
                "updated_at" => now,
            },
        )
        .map_err(|e| format!("mysql match record: {e}"))?;
        self.match_stats(persona_id)
    }

    fn ensure_match_stats(&self) -> Result<(), String> {
        let mut conn = self.conn()?;
        conn.query_drop(
            "CREATE TABLE IF NOT EXISTS persona_match_stats (
                persona_id BIGINT NOT NULL PRIMARY KEY,
                wins INT NOT NULL DEFAULT 0,
                losses INT NOT NULL DEFAULT 0,
                win_streak INT NOT NULL DEFAULT 0,
                loss_streak INT NOT NULL DEFAULT 0,
                updated_at DATETIME NOT NULL
            )",
        )
        .map_err(|e| format!("mysql persona_match_stats: {e}"))
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
