use mysql::prelude::Queryable;
use mysql::{params, Pool};

use crate::common::app_env::MysqlParams;

use super::migrate::{self, Migration};

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

    pub fn migrate(&self) -> Result<(), String> {
        migrate::apply(self)
    }

    pub(crate) fn ensure_migrations_table(&self) -> Result<(), String> {
        let mut conn = self
            .pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))?;
        conn.query_drop(
            r#"CREATE TABLE IF NOT EXISTS schema_migrations (
                version INT NOT NULL PRIMARY KEY,
                name VARCHAR(128) NOT NULL,
                applied_at DATETIME NOT NULL
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
        )
        .map_err(|e| format!("mysql schema_migrations: {e}"))
    }

    pub(crate) fn applied_versions(&self) -> Result<Vec<i64>, String> {
        let mut conn = self
            .pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))?;
        conn.query("SELECT version FROM schema_migrations")
            .map_err(|e| format!("mysql select migrations: {e}"))
    }

    pub(crate) fn apply_migration(&self, migration: &Migration) -> Result<(), String> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let mut conn = self
            .pool
            .get_conn()
            .map_err(|e| format!("mysql get_conn: {e}"))?;
        for sql in migration.mysql {
            conn.query_drop(*sql).map_err(|e| {
                format!(
                    "mysql migration {} ({}) : {e}",
                    migration.version, migration.name
                )
            })?;
        }
        conn.exec_drop(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (:version, :name, :applied_at)",
            params! {
                "version" => migration.version,
                "name" => migration.name,
                "applied_at" => now,
            },
        )
        .map_err(|e| format!("mysql record migration: {e}"))
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
