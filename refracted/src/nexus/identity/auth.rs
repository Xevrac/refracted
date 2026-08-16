//! Session credentials: secret, token, JWT `jti`. Identity comes from the `auth_sessions` row.

use mysql::params;
use mysql::prelude::Queryable;
use sha2::{Digest, Sha256};

use super::store::{BoundSession, IdentityStore};

const SESSION_TTL_SECS: i64 = 259_200; // 3 days, matches issued JWT exp

/// Credentials returned once at login. Store hashes only.
#[derive(Debug, Clone)]
pub struct IssuedCredentials {
    pub user_id: i64,
    pub persona_id: i64,
    pub token: String,
    pub jwt: String,
    pub jwt_id: String,
    pub expires_at: String,
}

pub fn hash_secret(salt_hex: &str, secret: &str) -> String {
    sha256_hex(&format!("{salt_hex}:{secret}"))
}

pub fn hash_token(token: &str) -> String {
    sha256_hex(token)
}

pub fn new_salt_hex() -> String {
    random_hex(16)
}

pub fn new_token() -> String {
    random_hex(32)
}

pub fn new_jwt_id() -> String {
    random_hex(16)
}

pub fn session_expiry_utc() -> String {
    (chrono::Utc::now() + chrono::Duration::seconds(SESSION_TTL_SECS))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Constant-time compare of equal-length hex hashes.
pub fn hashes_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes()
        .iter()
        .zip(b.as_bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Reject a client that presents a valid session but claims a different user/persona.
pub fn assert_bound_identity(
    bound: &BoundSession,
    claimed_user: i64,
    claimed_persona: i64,
) -> Result<(), String> {
    if bound.user_id != claimed_user || bound.persona_id != claimed_persona {
        return Err("identity mismatch: presented session does not own that user/persona".into());
    }
    Ok(())
}

/// `jti` only
pub fn jwt_id_from_token(jwt: &str) -> Option<String> {
    let payload = jwt.split('.').nth(1)?;
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        payload,
    )
    .or_else(|_| {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, payload)
    })
    .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    json.get("jti")?.as_str().map(str::to_string)
}

impl IdentityStore {
    pub fn set_user_secret(&self, user_id: i64, secret: &str) -> Result<(), String> {
        if secret.is_empty() {
            return Err("account secret must not be empty".into());
        }
        let salt = new_salt_hex();
        let hash = hash_secret(&salt, secret);
        let mut conn = self.conn()?;
        conn.exec_drop(
            "UPDATE users SET secret_hash = :hash, secret_salt = :salt WHERE id = :id",
            params! {
                "hash" => hash,
                "salt" => salt,
                "id" => user_id,
            },
        )
        .map_err(|e| format!("mysql set secret: {e}"))
    }

    pub fn verify_user_secret(&self, user_id: i64, secret: &str) -> Result<bool, String> {
        let mut conn = self.conn()?;
        let row: Option<(String, String)> = conn
            .exec_first(
                "SELECT secret_hash, secret_salt FROM users WHERE id = :id",
                params! { "id" => user_id },
            )
            .map_err(|e| format!("mysql load secret: {e}"))?;
        let Some((hash, salt)) = row else {
            return Ok(false);
        };
        if hash.is_empty() || salt.is_empty() {
            return Ok(false);
        }
        Ok(hashes_eq(&hash, &hash_secret(&salt, secret)))
    }

    /// Issue token + JWT bound to a persona the user actually owns.
    pub fn issue_session(
        &self,
        user_id: i64,
        persona_id: i64,
        display_name: &str,
    ) -> Result<IssuedCredentials, String> {
        let owned = self
            .personas_for_user(user_id)?
            .into_iter()
            .any(|p| p.id == persona_id);
        if !owned {
            return Err("persona is not owned by that user".into());
        }
        let token = new_token();
        let jwt_id = new_jwt_id();
        let token_hash = hash_token(&token);
        let expires_at = session_expiry_utc();
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let jwt = crate::jwt::generate_ea_jwt_token(
            &jwt_id,
            &(persona_id as u64),
            display_name,
            crate::jwt::NEXUS_GATEWAY_CLIENT_ID,
            &(user_id as u64),
        );
        let mut conn = self.conn()?;
        conn.exec_drop(
            "INSERT INTO auth_sessions (user_id, persona_id, token_hash, jwt_id, expires_at, created_at, last_seen_at)
             VALUES (:user_id, :persona_id, :token_hash, :jwt_id, :expires_at, :now, :now)",
            params! {
                "user_id" => user_id,
                "persona_id" => persona_id,
                "token_hash" => token_hash,
                "jwt_id" => jwt_id.clone(),
                "expires_at" => expires_at.clone(),
                "now" => now,
            },
        )
        .map_err(|e| format!("mysql issue session: {e}"))?;
        Ok(IssuedCredentials {
            user_id,
            persona_id,
            token,
            jwt,
            jwt_id,
            expires_at,
        })
    }

    /// Resolve token or JWT to the **database** identity. Claims inside a JWT are ignored.
    pub fn resolve_presented(&self, presented: &str) -> Result<BoundSession, String> {
        let presented = presented.trim();
        if presented.is_empty() {
            return Err("missing session token".into());
        }
        let bound = if presented.matches('.').count() >= 2 {
            let jti = jwt_id_from_token(presented).ok_or("invalid jwt")?;
            self.load_session_by_jwt_id(&jti)?
        } else {
            self.load_session_by_token_hash(&hash_token(presented))?
        };
        let Some(bound) = bound else {
            return Err("unknown or revoked session".into());
        };
        if bound.expired {
            return Err("session expired".into());
        }
        Ok(bound)
    }

    /// Game client join: presented credential must own the claimed user + persona.
    pub fn bind_client(
        &self,
        presented: &str,
        claimed_user: i64,
        claimed_persona: i64,
    ) -> Result<BoundSession, String> {
        let bound = self.resolve_presented(presented)?;
        assert_bound_identity(&bound, claimed_user, claimed_persona)?;
        Ok(bound)
    }

    pub fn revoke_session_by_token(&self, token: &str) -> Result<(), String> {
        let hash = hash_token(token);
        let now = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let mut conn = self.conn()?;
        conn.exec_drop(
            "UPDATE auth_sessions SET revoked_at = :now WHERE token_hash = :hash AND revoked_at IS NULL",
            params! { "now" => now, "hash" => hash },
        )
        .map_err(|e| format!("mysql revoke session: {e}"))
    }

    fn load_session_by_token_hash(&self, token_hash: &str) -> Result<Option<BoundSession>, String> {
        self.load_session(
            "s.token_hash = :key",
            token_hash,
        )
    }

    fn load_session_by_jwt_id(&self, jwt_id: &str) -> Result<Option<BoundSession>, String> {
        self.load_session("s.jwt_id = :key", jwt_id)
    }

    fn load_session(&self, where_clause: &str, key: &str) -> Result<Option<BoundSession>, String> {
        let sql = format!(
            "SELECT s.user_id, s.persona_id, u.email, p.display_name, s.expires_at, s.revoked_at
             FROM auth_sessions s
             JOIN users u ON u.id = s.user_id
             JOIN personas p ON p.id = s.persona_id
             WHERE {where_clause} AND s.revoked_at IS NULL"
        );
        let mut conn = self.conn()?;
        let row: Option<(i64, i64, String, String, String, Option<String>)> = conn
            .exec_first(sql, params! { "key" => key })
            .map_err(|e| format!("mysql load session: {e}"))?;
        let Some((user_id, persona_id, email, display_name, expires_at, revoked_at)) = row else {
            return Ok(None);
        };
        if revoked_at.is_some() {
            return Ok(None);
        }
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let expired = expires_at < now;
        Ok(Some(BoundSession {
            user_id,
            persona_id,
            email,
            display_name,
            expired,
        }))
    }
}

fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

fn random_hex(nbytes: usize) -> String {
    use rand::RngCore;
    let mut buf = vec![0u8; nbytes];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nexus::identity::store::BoundSession;

    fn sample_bound() -> BoundSession {
        BoundSession {
            user_id: 10,
            persona_id: 20,
            email: "a@b.c".into(),
            display_name: "Alice".into(),
            expired: false,
        }
    }

    #[test]
    fn secret_hash_needs_salt() {
        let a = hash_secret("aa", "password");
        let b = hash_secret("bb", "password");
        assert_ne!(a, b);
        assert!(hashes_eq(&a, &hash_secret("aa", "password")));
        assert!(!hashes_eq(&a, &hash_secret("aa", "other")));
    }

    #[test]
    fn masquerade_is_rejected() {
        let bound = sample_bound();
        assert!(assert_bound_identity(&bound, 10, 20).is_ok());
        assert!(assert_bound_identity(&bound, 10, 99).is_err());
        assert!(assert_bound_identity(&bound, 11, 20).is_err());
    }

    #[test]
    fn jwt_id_reads_jti_only() {
        let payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            br#"{"jti":"abc123","pid":"999"}"#,
        );
        let jwt = format!("eyJhbGciOiJub25lIn0.{payload}.sig");
        assert_eq!(jwt_id_from_token(&jwt).as_deref(), Some("abc123"));
    }
}
