use crate::blaze::tdf::TdfEncoder;
use crate::common::error::BlazeResult;
use crate::jwt::{parse_jwt_token, SessionInfo};
use bytes::Bytes;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn blaze_session_key(user_id: i64, persona_id: i64) -> String {
    let mut hasher = DefaultHasher::new();
    user_id.hash(&mut hasher);
    persona_id.hash(&mut hasher);
    let session_key_seed = hasher.finish();
    let session_key_prefix = format!("{:08x}", (session_key_seed & 0xFFFFFFFF) as u32);
    format!(
        "{}_hR$c7q*6eVx5u2ZgP*3kccE$*g4EQ$m5LJqOMm3sM8w=",
        session_key_prefix
    )
}

pub fn handle_auth_login(payload: &[u8]) -> BlazeResult<Bytes> {
    let jwt_token = TdfEncoder::find_string_field(payload, "TOKN").unwrap_or_else(|| {
        for i in 0..payload.len().saturating_sub(10) {
            if payload[i] == b'e' && payload[i + 1] == b'y' && payload[i + 2] == b'J' {
                let mut end = i + 3;
                while end < payload.len() && payload[end] != 0 && payload[end] >= 32 && payload[end] < 127 {
                    end += 1;
                }
                if end - i > 100 {
                    if let Ok(jwt_str) = String::from_utf8(payload[i..end].to_vec()) {
                        crate::console_println!(
                            "\x1b[38;2;255;215;0m[gRPC]\x1b[0m Found JWT token via string search (length: {})",
                            jwt_str.len()
                        );
                        return jwt_str;
                    }
                }
            }
        }
        String::new()
    });

    if jwt_token.is_empty() {
        crate::console_println!("\x1b[38;2;255;150;150m[gRPC]\x1b[0m WARN: No JWT in TOKN — using LSX session state");
    } else {
        crate::console_println!(
            "\x1b[38;2;0;200;255m[gRPC]\x1b[0m Extracted JWT from TOKN (length: {})",
            jwt_token.len()
        );
    }

    let session_info = if !jwt_token.is_empty() {
        parse_jwt_token(&jwt_token)
    } else {
        use crate::session::get_user_session;
        let session = get_user_session();
        SessionInfo {
            user_id: session.user_id as i64,
            persona_id: session.persona_id as i64,
            display_name: session.display_name.clone(),
            email: session.email.clone(),
            psid: session.psid as i32,
            ausrc: "324320".to_string(),
        }
    };

    crate::console_println!(
        "\x1b[38;2;0;200;255m[gRPC]\x1b[0m Blaze login — user_id={}, persona_id={}, display_name={}",
        session_info.user_id,
        session_info.persona_id,
        session_info.display_name
    );

    let session_key = blaze_session_key(session_info.user_id, session_info.persona_id);

    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_string("SKEY", &session_key));
    response.extend_from_slice(&TdfEncoder::encode_int("ANON", 0));

    let mut lqsz = Vec::new();
    lqsz.extend_from_slice(&TdfEncoder::encode_int("LQAR", 0));
    lqsz.extend_from_slice(&TdfEncoder::encode_int("LQPS", 0));
    lqsz.extend_from_slice(&TdfEncoder::encode_int("LQRA", 20_000_000));
    lqsz.extend_from_slice(&TdfEncoder::encode_int("LQRT", 0));
    lqsz.extend_from_slice(&TdfEncoder::encode_int("LQSZ", 0));
    lqsz.extend_from_slice(&TdfEncoder::encode_int("OQPS", 0));
    lqsz.extend_from_slice(&TdfEncoder::encode_int("OQSZ", 0));
    response.extend_from_slice(&TdfEncoder::encode_struct("LQSZ", &lqsz));
    response.extend_from_slice(&TdfEncoder::encode_int("LQTK", 0));

    let mut sess = Vec::new();
    sess.extend_from_slice(&TdfEncoder::encode_int("1CON", 0));

    let mut aids = Vec::new();
    let mut eaid = Vec::new();
    eaid.extend_from_slice(&TdfEncoder::encode_string("NAME", &session_info.display_name));
    eaid.extend_from_slice(&TdfEncoder::encode_long("NID ", session_info.user_id));
    eaid.extend_from_slice(&TdfEncoder::encode_long("PCID", session_info.persona_id));
    aids.extend_from_slice(&TdfEncoder::encode_struct("EAID", &eaid));

    let mut exid = Vec::new();
    exid.extend_from_slice(&TdfEncoder::encode_int("PSID", session_info.psid));
    exid.extend_from_slice(&TdfEncoder::encode_int("STID", 0));
    exid.extend_from_slice(&TdfEncoder::encode_string("SWID", ""));
    exid.extend_from_slice(&TdfEncoder::encode_int("XBID", 0));
    aids.extend_from_slice(&TdfEncoder::encode_struct("EXID", &exid));
    aids.extend_from_slice(&TdfEncoder::encode_int("PLAT", 4));
    sess.extend_from_slice(&TdfEncoder::encode_struct("AIDS", &aids));

    sess.extend_from_slice(&TdfEncoder::encode_long("BUID", session_info.persona_id));
    sess.extend_from_slice(&TdfEncoder::encode_int("FRST", 0));
    sess.extend_from_slice(&TdfEncoder::encode_int("GEO ", 1));
    sess.extend_from_slice(&TdfEncoder::encode_string("KEY ", &session_key));

    use std::time::{SystemTime, UNIX_EPOCH};
    let llog = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    sess.extend_from_slice(&TdfEncoder::encode_long("LLOG", llog));
    sess.extend_from_slice(&TdfEncoder::encode_int("PAAI", 0));

    let mut pdtl = Vec::new();
    pdtl.extend_from_slice(&TdfEncoder::encode_string("DSNM", &session_info.display_name));
    pdtl.extend_from_slice(&TdfEncoder::encode_int("LAST", 0));
    pdtl.extend_from_slice(&TdfEncoder::encode_long("PID ", session_info.persona_id));
    pdtl.extend_from_slice(&TdfEncoder::encode_int("STAS", 0));
    pdtl.extend_from_slice(&TdfEncoder::encode_long("XREF", session_info.user_id));
    sess.extend_from_slice(&TdfEncoder::encode_struct("PDTL", &pdtl));

    sess.extend_from_slice(&TdfEncoder::encode_long("UID ", session_info.user_id));
    response.extend_from_slice(&TdfEncoder::encode_struct("SESS", &sess));

    Ok(Bytes::from(response))
}

pub fn handle_auth_logout(_payload: &[u8]) -> BlazeResult<Bytes> {
    let mut response = Vec::new();
    response.extend_from_slice(&TdfEncoder::encode_bool("SUCC", true));
    Ok(Bytes::from(response))
}
