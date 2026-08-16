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
}
