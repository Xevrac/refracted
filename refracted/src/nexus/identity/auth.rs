//! Game-client sign-in: verify the Nexus ticket offline, then apply the revocation feed.

use super::store::{BoundSession, IdentityStore};
use super::ticket::{self, Ticket, TicketError};
use super::ClientLoginRefusal;
use crate::nexus::revocations;

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

/// Bans come first: banning also revokes every session, and the ban is what must be reported.
fn ban_refusal(t: &Ticket) -> Option<ClientLoginRefusal> {
    revocations::active_ban(t.uid).map(|until_unix| ClientLoginRefusal::Banned {
        until_unix,
        user_id: t.uid,
        persona_id: t.pid,
        discord_id: revocations::ban_discord_id(t.uid),
    })
}

impl IdentityStore {
    /// Identity from the ticket signature alone; claims outside the signed payload are ignored.
    pub fn resolve_presented(
        &self,
        presented: &str,
        public_key: &[u8],
    ) -> Result<BoundSession, ClientLoginRefusal> {
        let now = chrono::Utc::now().timestamp();
        let t = match ticket::verify(presented, public_key, now) {
            Ok(t) => t,
            Err(TicketError::Expired(t)) => {
                return Err(ban_refusal(&t).unwrap_or_else(|| {
                    ClientLoginRefusal::Unauthorized(TicketError::Expired(t).to_string())
                }))
            }
            Err(e) => return Err(ClientLoginRefusal::Unauthorized(e.to_string())),
        };
        if let Some(refusal) = ban_refusal(&t) {
            return Err(refusal);
        }
        if revocations::is_revoked(&t.jti) {
            return Err(ClientLoginRefusal::Unauthorized(
                "session revoked (signed out, or revoked on Nexus)".into(),
            ));
        }
        // The mirror feeds leaderboards and chat tags; it must not refuse a valid login.
        if let Err(e) = self.upsert_persona(t.pid, t.uid, &t.dn, &t.tag) {
            tracing::warn!("persona mirror update failed persona={}: {e}", t.pid);
        }
        Ok(BoundSession {
            user_id: t.uid,
            persona_id: t.pid,
            email: String::new(),
            display_name: t.dn,
            expired: false,
        })
    }

    /// Launcher handoff: the ticket must own the claimed user + persona.
    pub fn bind_client(
        &self,
        presented: &str,
        public_key: &[u8],
        claimed_user: i64,
        claimed_persona: i64,
    ) -> Result<BoundSession, String> {
        let bound = self
            .resolve_presented(presented, public_key)
            .map_err(|refusal| match refusal {
                ClientLoginRefusal::Banned { .. } => "account banned".to_string(),
                ClientLoginRefusal::Unauthorized(reason) => reason,
            })?;
        assert_bound_identity(&bound, claimed_user, claimed_persona)?;
        Ok(bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masquerade_is_rejected() {
        let bound = BoundSession {
            user_id: 10,
            persona_id: 20,
            email: String::new(),
            display_name: "Alice".into(),
            expired: false,
        };
        assert!(assert_bound_identity(&bound, 10, 20).is_ok());
        assert!(assert_bound_identity(&bound, 10, 99).is_err());
        assert!(assert_bound_identity(&bound, 11, 20).is_err());
    }
}
