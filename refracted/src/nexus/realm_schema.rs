//! Realm database schema. Refracted owns it and migrates it at boot, before any listener binds.
//! Append new migrations to [`MIGRATIONS`]; never edit one that has shipped.

use mysql::prelude::Queryable;
use mysql::{params, PooledConn};

const LOCK_NAME: &str = "refracted_realm_schema";
const LOCK_WAIT_SECS: u32 = 30;

struct Migration {
    version: u32,
    name: &'static str,
    apply: fn(&mut PooledConn) -> Result<(), String>,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "baseline: personas mirror, match stats, match history",
    apply: baseline,
}];

/// Apply every pending migration. Concurrent processes on one database serialize on a lock.
pub fn migrate(conn: &mut PooledConn) -> Result<Vec<u32>, String> {
    let locked: Option<i64> = conn
        .exec_first("SELECT GET_LOCK(:name, :wait)", params! { "name" => LOCK_NAME, "wait" => LOCK_WAIT_SECS })
        .map_err(|e| format!("schema lock: {e}"))?;
    if locked != Some(1) {
        return Err(format!("schema lock {LOCK_NAME} is held by another process"));
    }
    let result = apply_pending(conn);
    let _ = conn.exec_drop("SELECT RELEASE_LOCK(:name)", params! { "name" => LOCK_NAME });
    result
}

fn apply_pending(conn: &mut PooledConn) -> Result<Vec<u32>, String> {
    exec(
        conn,
        "CREATE TABLE IF NOT EXISTS realm_schema_migrations (
            version INT NOT NULL PRIMARY KEY,
            name VARCHAR(191) NOT NULL,
            applied_at DATETIME NOT NULL
        )",
    )?;
    let done: Vec<u32> = conn
        .query("SELECT version FROM realm_schema_migrations")
        .map_err(|e| format!("read realm_schema_migrations: {e}"))?;
    let mut applied = Vec::new();
    for m in MIGRATIONS.iter().filter(|m| !done.contains(&m.version)) {
        (m.apply)(conn).map_err(|e| format!("migration {} ({}): {e}", m.version, m.name))?;
        conn.exec_drop(
            "INSERT INTO realm_schema_migrations (version, name, applied_at) VALUES (:v, :n, UTC_TIMESTAMP())",
            params! { "v" => m.version, "n" => m.name },
        )
        .map_err(|e| format!("record migration {}: {e}", m.version))?;
        applied.push(m.version);
    }
    Ok(applied)
}

/// Creates what is missing and leaves existing tables alone, so a realm whose tables were moved
/// out of the old shared Nexus database adopts them as they are.
fn baseline(conn: &mut PooledConn) -> Result<(), String> {
    exec(
        conn,
        "CREATE TABLE IF NOT EXISTS personas (
            id BIGINT NOT NULL PRIMARY KEY,
            user_id BIGINT NOT NULL,
            display_name VARCHAR(64) NOT NULL,
            name_tag VARCHAR(16) NOT NULL DEFAULT '',
            updated_at DATETIME NOT NULL,
            KEY idx_personas_user_id (user_id)
        )",
    )?;
    exec(
        conn,
        "CREATE TABLE IF NOT EXISTS persona_match_stats (
            persona_id BIGINT NOT NULL PRIMARY KEY,
            wins INT NOT NULL DEFAULT 0,
            losses INT NOT NULL DEFAULT 0,
            win_streak INT NOT NULL DEFAULT 0,
            loss_streak INT NOT NULL DEFAULT 0,
            updated_at DATETIME NOT NULL,
            last_general_id BIGINT NULL,
            last_faction VARCHAR(8) NULL,
            shortest_match_secs INT NULL,
            best_win_secs INT NULL
        )",
    )?;
    add_missing_columns(
        conn,
        "persona_match_stats",
        &[
            ("last_general_id", "BIGINT NULL"),
            ("last_faction", "VARCHAR(8) NULL"),
            ("shortest_match_secs", "INT NULL"),
            ("best_win_secs", "INT NULL"),
        ],
    )?;
    exec(
        conn,
        "CREATE TABLE IF NOT EXISTS persona_match_history (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            persona_id BIGINT NOT NULL,
            victory TINYINT NOT NULL,
            ended_at DATETIME NOT NULL,
            duration_secs INT NULL,
            general_id BIGINT NULL,
            faction VARCHAR(8) NULL,
            KEY persona_ended (persona_id, ended_at)
        )",
    )?;
    add_missing_columns(
        conn,
        "persona_match_history",
        &[
            ("duration_secs", "INT NULL"),
            ("general_id", "BIGINT NULL"),
            ("faction", "VARCHAR(8) NULL"),
        ],
    )?;
    add_missing_columns(conn, "personas", &[("name_tag", "VARCHAR(16) NOT NULL DEFAULT ''")])
}

fn add_missing_columns(conn: &mut PooledConn, table: &str, columns: &[(&str, &str)]) -> Result<(), String> {
    let have: Vec<String> = conn
        .exec_map(
            "SELECT COLUMN_NAME FROM information_schema.COLUMNS
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = :t",
            params! { "t" => table },
            |name: String| name.to_ascii_lowercase(),
        )
        .map_err(|e| format!("columns of {table}: {e}"))?;
    for (name, ddl) in columns {
        if !have.iter().any(|h| h == name) {
            exec(conn, &format!("ALTER TABLE {table} ADD COLUMN {name} {ddl}"))?;
        }
    }
    Ok(())
}

fn exec(conn: &mut PooledConn, sql: &str) -> Result<(), String> {
    conn.query_drop(sql).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_unique_and_ascending() {
        let versions: Vec<u32> = MIGRATIONS.iter().map(|m| m.version).collect();
        assert!(versions.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(versions.first(), Some(&1));
    }
}
