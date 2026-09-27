//! Consume launcher DPAPI handoff and bind mysql identity before LSX/Blaze.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use parking_lot::Mutex;

use super::{bind_mysql_client, current_bound_session, BoundSession};

const HANDOFF_ENTROPY: &[u8] = b"Refracted.Nexus.Handoff.v1";
const HANDOFF_FILE: &str = "nexus-bound-session.bin";

#[derive(Debug, serde::Deserialize)]
struct HandoffPayload {
    token: String,
    #[serde(default)]
    jwt: String,
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "personaId")]
    persona_id: String,
    #[serde(default, rename = "displayName")]
    display_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HandoffStamp {
    modified_secs: u64,
    len: u64,
}

static LAST_BOUND_STAMP: Mutex<Option<HandoffStamp>> = Mutex::new(None);

/// Well-known launcher handoff path (`PRISM_NEXUS_HANDOFF` or `%LocalAppData%/Refracted/...`).
pub fn handoff_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("PRISM_NEXUS_HANDOFF") {
        let t = p.trim();
        if !t.is_empty() {
            return Some(PathBuf::from(t));
        }
    }
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return Some(PathBuf::from(local).join("Refracted").join(HANDOFF_FILE));
        }
    }
    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("Refracted")
                    .join(HANDOFF_FILE),
            );
        }
    }
    None
}

pub fn handoff_file_present() -> bool {
    handoff_path().map(|p| p.is_file()).unwrap_or(false)
}

/// Boot / mysql-ready: bind once from the launcher handoff if present.
pub fn try_bind_launcher_handoff() -> Option<BoundSession> {
    ensure_bound_from_handoff()
}

/// Idempotent: bind (or refresh) from DPAPI handoff. Safe to call on every GetProfile.
pub fn ensure_bound_from_handoff() -> Option<BoundSession> {
    if !super::client_join_requires_login() {
        return current_bound_session();
    }
    if super::current_identity_store().is_none() {
        return None;
    }
    let path = handoff_path()?;
    if !path.is_file() {
        *LAST_BOUND_STAMP.lock() = None;
        // Sign-out deleted the handoff 
        if current_bound_session().is_some() {
            super::clear_bound_session();
            // Drop the mirrored session so GetProfile cannot echo a stale persona.
            crate::session::set_user_session(crate::session::UserSession {
                user_id: 0,
                persona_id: 0,
                display_name: String::new(),
                email: String::new(),
                psid: 0,
                jwt_token: None,
                update_network_info_count: 0,
                hwfg: 0,
                network_exip_ip: None,
                network_inip_ip: None,
                network_exip_port: None,
                network_inip_port: None,
                network_bps: None,
                next_message_id: 1160000,
            });
            crate::nexus::log_nexus_to_blaze("handoff cleared — unbound Nexus session");
        }
        return None;
    }
    let stamp = match file_stamp(&path) {
        Ok(s) => s,
        Err(e) => {
            crate::nexus::log_nexus_to_blaze(format!("handoff stamp failed: {e}"));
            return current_bound_session();
        }
    };
    if *LAST_BOUND_STAMP.lock() == Some(stamp) {
        if let Some(bound) = current_bound_session() {
            return Some(bound);
        }
    }
    match bind_from_path(&path) {
        Ok((bound, jwt)) => {
            *LAST_BOUND_STAMP.lock() = Some(stamp);
            crate::nexus::log_nexus_to_blaze(format!(
                "launcher handoff bound user_id={} persona_id={} display={}",
                bound.user_id, bound.persona_id, bound.display_name
            ));
            sync_bound_to_session(&bound, jwt);
            Some(bound)
        }
        Err(e) => {
            crate::nexus::log_nexus_to_blaze(format!("launcher handoff bind failed: {e}"));
            current_bound_session()
        }
    }
}

fn file_stamp(path: &Path) -> Result<HandoffStamp, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("stat handoff: {e}"))?;
    let modified_secs = meta
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(HandoffStamp {
        modified_secs,
        len: meta.len(),
    })
}

fn bind_from_path(path: &Path) -> Result<(BoundSession, Option<String>), String> {
    let protected = std::fs::read(path).map_err(|e| format!("read handoff: {e}"))?;
    if protected.is_empty() {
        return Err("handoff empty".into());
    }
    let plain = unprotect_current_user(&protected)?;
    // Zeroize-ish: plain dropped at end of scope.
    let payload: HandoffPayload =
        serde_json::from_slice(&plain).map_err(|e| format!("handoff json: {e}"))?;
    let user_id: i64 = payload
        .user_id
        .parse()
        .map_err(|_| "handoff userId invalid".to_string())?;
    let persona_id: i64 = payload
        .persona_id
        .parse()
        .map_err(|_| "handoff personaId invalid".to_string())?;
    if user_id <= 0 || persona_id <= 0 {
        return Err("handoff ids must be positive".into());
    }
    let presented = if !payload.token.is_empty() {
        payload.token.as_str()
    } else if !payload.jwt.is_empty() {
        payload.jwt.as_str()
    } else {
        return Err("handoff missing token".into());
    };
    // Identity comes from DB lookup of the token — claimed ids must match the row.
    let bound = bind_mysql_client(presented, user_id, persona_id)?;
    let jwt = if payload.jwt.is_empty() {
        None
    } else {
        Some(payload.jwt)
    };
    let _ = payload.display_name; // display comes from DB, not the handoff claim
    Ok((bound, jwt))
}

fn sync_bound_to_session(bound: &BoundSession, jwt: Option<String>) {
    use crate::session::{set_user_session, UserSession};
    set_user_session(UserSession {
        user_id: bound.user_id as u64,
        persona_id: bound.persona_id as u64,
        display_name: bound.display_name.clone(),
        email: bound.email.clone(),
        psid: (bound.persona_id % 1_000_000_000) as u32,
        jwt_token: jwt,
        update_network_info_count: 0,
        hwfg: 0,
        network_exip_ip: None,
        network_inip_ip: None,
        network_exip_port: None,
        network_inip_port: None,
        network_bps: None,
        next_message_id: 1160000,
    });
}

#[cfg(windows)]
fn unprotect_current_user(protected: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr;
    use winapi::um::dpapi::CryptUnprotectData;
    use winapi::um::winbase::LocalFree;
    use winapi::um::wincrypt::CRYPTOAPI_BLOB;

    unsafe {
        let mut data_in = CRYPTOAPI_BLOB {
            cbData: protected.len() as u32,
            pbData: protected.as_ptr() as *mut u8,
        };
        let mut entropy = CRYPTOAPI_BLOB {
            cbData: HANDOFF_ENTROPY.len() as u32,
            pbData: HANDOFF_ENTROPY.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };
        let ok = CryptUnprotectData(
            &mut data_in,
            ptr::null_mut(),
            &mut entropy,
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            &mut data_out,
        );
        if ok == 0 || data_out.pbData.is_null() || data_out.cbData == 0 {
            return Err("CryptUnprotectData failed (wrong user or corrupt handoff)".into());
        }
        let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
        let out = slice.to_vec();
        LocalFree(data_out.pbData as *mut _);
        Ok(out)
    }
}

#[cfg(not(windows))]
fn unprotect_current_user(_protected: &[u8]) -> Result<Vec<u8>, String> {
    Err("launcher handoff DPAPI is Windows-only".into())
}
