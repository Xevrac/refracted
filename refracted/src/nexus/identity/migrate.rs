//! Versioned SQL applied at headless boot when `datasource=mysql`.

use super::store::IdentityStore;

pub(crate) struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub mysql: &'static [&'static str],
}

pub(crate) const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "identity_users_personas",
        mysql: &[
            r#"CREATE TABLE IF NOT EXISTS users (
            id BIGINT NOT NULL PRIMARY KEY,
            username VARCHAR(64) NOT NULL,
            email VARCHAR(255) NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL,
            UNIQUE KEY uq_users_username (username)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
            r#"CREATE TABLE IF NOT EXISTS personas (
            id BIGINT NOT NULL PRIMARY KEY,
            user_id BIGINT NOT NULL,
            display_name VARCHAR(64) NOT NULL,
            created_at DATETIME NOT NULL,
            KEY idx_personas_user_id (user_id),
            CONSTRAINT fk_personas_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
        ],
    },
    Migration {
        version: 2,
        name: "identity_secrets_sessions",
        mysql: &[
            r#"ALTER TABLE users
            ADD COLUMN secret_hash CHAR(64) NOT NULL DEFAULT '',
            ADD COLUMN secret_salt CHAR(32) NOT NULL DEFAULT ''"#,
            r#"CREATE TABLE IF NOT EXISTS auth_sessions (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            user_id BIGINT NOT NULL,
            persona_id BIGINT NOT NULL,
            token_hash CHAR(64) NOT NULL,
            jwt_id VARCHAR(64) NOT NULL,
            expires_at DATETIME NOT NULL,
            revoked_at DATETIME NULL,
            created_at DATETIME NOT NULL,
            last_seen_at DATETIME NOT NULL,
            UNIQUE KEY uq_auth_sessions_token_hash (token_hash),
            UNIQUE KEY uq_auth_sessions_jwt_id (jwt_id),
            KEY idx_auth_sessions_user (user_id),
            CONSTRAINT fk_auth_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            CONSTRAINT fk_auth_sessions_persona FOREIGN KEY (persona_id) REFERENCES personas(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
        ],
    },
    Migration {
        version: 3,
        name: "identity_discord_web_user",
        mysql: &[
            r#"ALTER TABLE users
            ADD COLUMN discord_id VARCHAR(32) NULL,
            ADD COLUMN web_user_id BIGINT UNSIGNED NULL"#,
            r#"ALTER TABLE users
            ADD UNIQUE KEY uq_users_discord_id (discord_id),
            ADD KEY idx_users_web_user_id (web_user_id)"#,
        ],
    },
    Migration {
        version: 4,
        name: "identity_access_control",
        mysql: &[
            r#"CREATE TABLE IF NOT EXISTS settings (
            `key` VARCHAR(64) NOT NULL PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME NOT NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
            r#"CREATE TABLE IF NOT EXISTS signup_whitelist (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            discord_id VARCHAR(32) NOT NULL,
            note VARCHAR(255) NOT NULL DEFAULT '',
            created_by_web_user_id BIGINT UNSIGNED NULL,
            created_at DATETIME NOT NULL,
            UNIQUE KEY uq_signup_whitelist_discord (discord_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
            r#"CREATE TABLE IF NOT EXISTS bans (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            user_id BIGINT NULL,
            discord_id VARCHAR(32) NULL,
            reason VARCHAR(512) NOT NULL DEFAULT '',
            banned_until DATETIME NULL,
            created_by_web_user_id BIGINT UNSIGNED NULL,
            created_at DATETIME NOT NULL,
            lifted_by_web_user_id BIGINT UNSIGNED NULL,
            lifted_at DATETIME NULL,
            KEY idx_bans_user_id (user_id),
            KEY idx_bans_discord_id (discord_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"#,
        ],
    },
    Migration {
        version: 5,
        name: "identity_session_client_telemetry",
        mysql: &[
            r#"ALTER TABLE auth_sessions
            ADD COLUMN client_ip VARCHAR(45) NULL,
            ADD COLUMN country_code CHAR(2) NULL,
            ADD COLUMN game_id VARCHAR(64) NULL"#,
            r#"ALTER TABLE auth_sessions
            ADD KEY idx_auth_sessions_game (game_id)"#,
        ],
    },
];

pub(crate) fn apply(store: &IdentityStore) -> Result<(), String> {
    store.ensure_migrations_table()?;
    let applied = store.applied_versions()?;
    for migration in MIGRATIONS {
        if applied.contains(&migration.version) {
            continue;
        }
        store.apply_migration(migration)?;
        tracing::info!(
            "identity migration {} ({}) applied",
            migration.version,
            migration.name
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_migration_defines_users_and_personas() {
        let m = &MIGRATIONS[0];
        assert_eq!(m.version, 1);
        let joined = m.mysql.join(";");
        assert!(joined.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(joined.contains("CREATE TABLE IF NOT EXISTS personas"));
        assert!(joined.contains("FOREIGN KEY (user_id) REFERENCES users(id)"));
    }

    #[test]
    fn second_migration_adds_secret_and_auth_sessions() {
        let m = &MIGRATIONS[1];
        assert_eq!(m.version, 2);
        let joined = m.mysql.join(";");
        assert!(joined.contains("secret_hash"));
        assert!(joined.contains("CREATE TABLE IF NOT EXISTS auth_sessions"));
        assert!(joined.contains("token_hash"));
        assert!(joined.contains("jwt_id"));
    }

    #[test]
    fn third_migration_adds_discord_and_web_user_join() {
        let m = &MIGRATIONS[2];
        assert_eq!(m.version, 3);
        let joined = m.mysql.join(";");
        assert!(joined.contains("discord_id"));
        assert!(joined.contains("web_user_id"));
        assert!(joined.contains("uq_users_discord_id"));
    }
}
