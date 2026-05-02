use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Session key aligned with UserAuthenticated / login2
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
