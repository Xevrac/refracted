use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use rand::Rng;

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub user_id: u64,
    pub persona_id: u64,
    pub display_name: String,
    #[serde(default)]
    pub email: String,
    pub psid: u32,
    pub ausrc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steam_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            username: "Xevrac".to_string(),
            user_id: 1012711274866,
            persona_id: 1201618778,
            display_name: "Xevrac".to_string(),
            email: "xevrac@ea.com".to_string(),
            psid: 0,
            ausrc: "324320".to_string(),
            steam_id: None,
            session_id: None,
            auth_token: None,
        }
    }
}

/// User profiles storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfiles {
    pub profiles: HashMap<String, UserProfile>,
    pub current_profile: String,
}

impl Default for UserProfiles {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        let default_profile = UserProfile::default();
        profiles.insert("Xevrac".to_string(), default_profile);
        
        Self {
            profiles,
            current_profile: "Xevrac".to_string(),
        }
    }
}

pub(crate) static GLOBAL_PROFILES: Mutex<Option<UserProfiles>> = Mutex::new(None);
static PROFILES_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Initialize user profiles from JSON file
pub fn init_profiles(file_path: PathBuf) -> Result<(), String> {
    let mut file_path_guard = PROFILES_FILE.lock();
    *file_path_guard = Some(file_path.clone());
    drop(file_path_guard);
    
    let profiles = if file_path.exists() {
        // Load from file
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read profiles file: {}", e))?;
        serde_json::from_str::<UserProfiles>(&content)
            .map_err(|e| format!("Failed to parse profiles file: {}", e))?
    } else {
        // Create default
        let default = UserProfiles::default();
        save_profiles(&default)?;
        default
    };
    
    let mut global = GLOBAL_PROFILES.lock();
    *global = Some(profiles);
    Ok(())
}

/// Save profiles to JSON file
pub fn save_profiles(profiles: &UserProfiles) -> Result<(), String> {
    let file_path_guard = PROFILES_FILE.lock();
    let file_path = file_path_guard.as_ref()
        .ok_or("Profiles file path not set")?;
    
    let json = serde_json::to_string_pretty(profiles)
        .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
    
    fs::write(file_path, json)
        .map_err(|e| format!("Failed to write profiles file: {}", e))?;
    
    Ok(())
}

/// Get current profiles
pub fn get_profiles() -> UserProfiles {
    use crate::common::settings;
    settings::get_user_profiles()
}

/// Update profiles
pub fn update_profiles(profiles: UserProfiles) -> Result<(), String> {
    use crate::common::settings;
    settings::update_user_profiles(profiles)
}

/// Get current active profile
pub fn get_current_profile() -> UserProfile {
    let profiles = get_profiles();
    profiles.profiles
        .get(&profiles.current_profile)
        .cloned()
        .unwrap_or_default()
}

/// Set current active profile
pub fn set_current_profile(profile_name: &str) -> Result<(), String> {
    let mut profiles = get_profiles();
    if !profiles.profiles.contains_key(profile_name) {
        return Err(format!("Profile '{}' not found", profile_name));
    }
    profiles.current_profile = profile_name.to_string();
    update_profiles(profiles)
}

/// Add or update a profile
/// Username and display_name are automatically synced with the profile name
pub fn save_profile(name: &str, mut profile: UserProfile) -> Result<(), String> {
    // Sync username and display_name with profile name
    profile.username = name.to_string();
    profile.display_name = name.to_string();
    
    let mut profiles = get_profiles();
    profiles.profiles.insert(name.to_string(), profile);
    update_profiles(profiles)
}

/// Delete a profile
pub fn delete_profile(name: &str) -> Result<(), String> {
    let mut profiles = get_profiles();
    if profiles.profiles.len() <= 1 {
        return Err("Cannot delete the last profile".to_string());
    }
    if profiles.current_profile == name {
        // Switch to first available profile
        let first_name = profiles.profiles.keys()
            .find(|k| *k != name)
            .ok_or("No other profiles available")?
            .clone();
        profiles.current_profile = first_name;
    }
    profiles.profiles.remove(name);
    update_profiles(profiles)
}

/// Create a new profile with randomized UUIDs and incremented PSID
/// username and display_name will be set from the profile name when saved
pub fn create_new_profile() -> UserProfile {
    let profiles = get_profiles();
    
    // Find the maximum PSID from existing profiles
    let max_psid = profiles.profiles.values()
        .map(|p| p.psid)
        .max()
        .unwrap_or(0);
    
    // Generate random user_id and persona_id
    // EA user IDs are typically 13 digits, starting with 10
    let mut rng = rand::thread_rng();
    let user_id = 10_000_000_000_000 + rng.gen_range(0..999_999_999_999);
    let persona_id = 10_000_000_000_000 + rng.gen_range(0..999_999_999_999);
    
    UserProfile {
        username: "NewUser".to_string(), // Will be updated from profile name
        user_id,
        persona_id,
        display_name: "NewUser".to_string(), // Will be updated from profile name
        email: "newuser@ea.com".to_string(),
        psid: max_psid + 1, // Increment PSID
        ausrc: "324320".to_string(),
        steam_id: None,
        session_id: None,
        auth_token: None,
    }
}

/// Update session state from current profile
pub fn sync_profile_to_session() {
    use crate::session::{set_user_session, UserSession};
    let profile = get_current_profile();
    crate::nucleus::log_nucleus_to_blaze(format!(
        "session fields from profile `{}` (persona_id={}, user_id={})",
        profile.display_name, profile.persona_id, profile.user_id
    ));
    set_user_session(UserSession {
        jwt_token: None,
        user_id: profile.user_id,
        persona_id: profile.persona_id,
        display_name: profile.display_name.clone(),
        email: profile.email.clone(),
        psid: profile.psid,
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

