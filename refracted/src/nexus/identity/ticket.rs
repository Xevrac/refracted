//! Nexus ticket: `nx1.<payload>.<signature>`, base64url, Ed25519 over `nx1.<payload>`.
//! Nexus signs it at launcher sign-in; a realm verifies it with the public key alone.

use base64::Engine;

const PREFIX: &str = "nx1";

/// Identity a valid ticket carries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Ticket {
    pub jti: String,
    pub uid: i64,
    pub pid: i64,
    pub dn: String,
    #[serde(default)]
    pub tag: String,
    pub exp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketError {
    Malformed,
    BadSignature,
    Expired(Ticket),
}

impl std::fmt::Display for TicketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => f.write_str("malformed ticket (sign in again from the launcher)"),
            Self::BadSignature => f.write_str("ticket signature does not match this realm's ticket_public_key"),
            Self::Expired(_) => f.write_str("session expired"),
        }
    }
}

/// Decode a base64 (standard or url-safe) 32-byte Ed25519 public key.
pub fn parse_public_key(raw: &str) -> Result<Vec<u8>, String> {
    let raw = raw.trim();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(raw))
        .map_err(|e| format!("ticket_public_key is not base64: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("ticket_public_key must be 32 bytes, got {}", bytes.len()));
    }
    Ok(bytes)
}

pub fn verify(presented: &str, public_key: &[u8], now_unix: i64) -> Result<Ticket, TicketError> {
    let mut parts = presented.trim().split('.');
    let (Some(PREFIX), Some(payload), Some(sig), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(TicketError::Malformed);
    };
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let sig = b64.decode(sig).map_err(|_| TicketError::Malformed)?;
    let signed = format!("{PREFIX}.{payload}");
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key)
        .verify(signed.as_bytes(), &sig)
        .map_err(|_| TicketError::BadSignature)?;
    let json = b64.decode(payload).map_err(|_| TicketError::Malformed)?;
    let ticket: Ticket = serde_json::from_slice(&json).map_err(|_| TicketError::Malformed)?;
    if ticket.uid <= 0 || ticket.pid <= 0 || ticket.jti.is_empty() {
        return Err(TicketError::Malformed);
    }
    if ticket.exp <= now_unix {
        return Err(TicketError::Expired(ticket));
    }
    Ok(ticket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn sign(pair: &Ed25519KeyPair, json: &str) -> String {
        let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let signed = format!("nx1.{}", b64.encode(json));
        let sig = pair.sign(signed.as_bytes());
        format!("{signed}.{}", b64.encode(sig.as_ref()))
    }

    fn pair(seed: u8) -> Ed25519KeyPair {
        Ed25519KeyPair::from_seed_unchecked(&[seed; 32]).unwrap()
    }

    const BODY: &str = r#"{"jti":"j1","uid":10,"pid":20,"dn":"Alice","tag":"developer","iat":1,"exp":2000}"#;

    #[test]
    fn valid_ticket_carries_identity() {
        let p = pair(7);
        let t = verify(&sign(&p, BODY), p.public_key().as_ref(), 1000).unwrap();
        assert_eq!((t.uid, t.pid, t.dn.as_str(), t.tag.as_str()), (10, 20, "Alice", "developer"));
    }

    #[test]
    fn other_key_is_rejected() {
        let ticket = sign(&pair(7), BODY);
        assert_eq!(verify(&ticket, pair(8).public_key().as_ref(), 1000), Err(TicketError::BadSignature));
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let p = pair(7);
        let ticket = sign(&p, BODY);
        let forged = sign(&pair(9), &BODY.replace("\"uid\":10", "\"uid\":11"));
        let mixed = format!(
            "nx1.{}.{}",
            forged.split('.').nth(1).unwrap(),
            ticket.split('.').nth(2).unwrap()
        );
        assert_eq!(verify(&mixed, p.public_key().as_ref(), 1000), Err(TicketError::BadSignature));
    }

    #[test]
    fn expired_and_malformed() {
        let p = pair(7);
        assert!(matches!(verify(&sign(&p, BODY), p.public_key().as_ref(), 2000), Err(TicketError::Expired(_))));
        assert_eq!(verify("deadbeef", p.public_key().as_ref(), 0), Err(TicketError::Malformed));
    }

    #[test]
    fn public_key_accepts_both_alphabets() {
        let p = pair(7);
        let std = base64::engine::general_purpose::STANDARD.encode(p.public_key().as_ref());
        assert_eq!(parse_public_key(&std).unwrap(), p.public_key().as_ref());
        assert!(parse_public_key("AAAA").is_err());
    }
}
