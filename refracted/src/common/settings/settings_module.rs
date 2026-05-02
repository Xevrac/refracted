use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::common::game::GamePreference;
use crate::common::user_profile::UserProfiles;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub debug_logging: bool,
    pub theme: String, // "light" or "dark"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            debug_logging: false,
            theme: "dark".to_string(),
        }
    }
}

/// Proxy listen/target ports for research mode (client ↔ upstream forwarding).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub http_listen_port: u16,
    pub https_listen_port: u16,
    pub grpc_listen_port: u16,
    pub blaze_listen_port: u16,
    pub lsx_listen_port: u16,
    pub target_host: String,
    pub target_http_port: u16,
    pub target_https_port: u16,
    pub target_grpc_port: u16,
    pub target_blaze_port: u16,
    pub target_lsx_port: u16,
    #[serde(default = "default_true")]
    pub enable_http: bool,
    #[serde(default = "default_true")]
    pub enable_https: bool,
    #[serde(default = "default_true")]
    pub enable_grpc: bool,
    #[serde(default = "default_true")]
    pub enable_blaze: bool,
    #[serde(default = "default_true")]
    pub enable_lsx: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            http_listen_port: 80,      // Match emulator HTTP port
            https_listen_port: 443,    // Match emulator HTTPS port
            grpc_listen_port: 443,     // Match emulator gRPC port
            blaze_listen_port: 10042,  // Match emulator Blaze TLS port
            lsx_listen_port: 3216,     // Match emulator LSX port
            target_host: "localhost".to_string(),
            target_http_port: 80,
            target_https_port: 443,
            target_grpc_port: 443,
            target_blaze_port: 10042,
            target_lsx_port: 3216,
            enable_http: true,
            enable_https: true,
            enable_grpc: true,
            enable_blaze: true,
            enable_lsx: true,
        }
    }
}

/// Unified settings file containing all user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSettings {
    pub game_preference: GamePreference,
    pub user_profiles: UserProfiles,
    pub app_settings: AppSettings,
    #[serde(default)]
    pub proxy_settings: ProxySettings,
}

impl Default for UnifiedSettings {
    fn default() -> Self {
        Self {
            game_preference: GamePreference::default(),
            user_profiles: UserProfiles::default(),
            app_settings: AppSettings::default(),
            proxy_settings: ProxySettings::default(),
        }
    }
}

static GLOBAL_SETTINGS: Mutex<Option<UnifiedSettings>> = Mutex::new(None);
static SETTINGS_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

fn parse_unified_settings_json(content: &str) -> Result<UnifiedSettings, String> {
    let mut value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("JSON parse error: {}", e))?;
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("rfr_tools");
        map.remove("wv_tools");
    }
    serde_json::from_value(value).map_err(|e| format!("Settings deserialize: {}", e))
}

/// Initialize unified settings from JSON file
pub fn init_settings(file_path: PathBuf) -> Result<(), String> {
    let mut file_path_guard = SETTINGS_FILE.lock();
    *file_path_guard = Some(file_path.clone());
    drop(file_path_guard);

    crate::common::paths::ensure_parent_dir(&file_path)
        .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    crate::common::paths::ensure_app_data_dir()
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    crate::common::game::init_games_registry()
        .map_err(|e| format!("Failed to initialize games registry: {}", e))?;

    let settings = if file_path.exists() {
        // Load from file
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
        match parse_unified_settings_json(&content) {
            Ok(loaded) => validate_and_repair_settings(loaded),
            Err(_) => {
                let default = UnifiedSettings::default();
                save_settings(&default)?;
                default
            }
        }
    } else {
        // Create default
        let default = UnifiedSettings::default();
        save_settings(&default)?;
        default
    };
    
    // Store and sync
    sync_settings_to_globals(&settings);
    
    Ok(())
}

/// Sync unified settings to legacy module globals for backward compatibility
fn sync_settings_to_globals(settings: &UnifiedSettings) {
    let mut global = GLOBAL_SETTINGS.lock();
    *global = Some(settings.clone());
    
    // Sync to legacy module globals
    {
        use crate::common::game;
        let mut game_global = game::GLOBAL_GAME_PREFERENCE.lock();
        *game_global = Some(settings.game_preference.clone());
    }
    {
        use crate::common::user_profile;
        let mut profile_global = user_profile::GLOBAL_PROFILES.lock();
        *profile_global = Some(settings.user_profiles.clone());
    }
}

/// Validate and repair settings
fn validate_and_repair_settings(mut settings: UnifiedSettings) -> UnifiedSettings {
    use crate::common::game::games_registry;

    let mut needs_repair = false;

    if games_registry::get_game_definition(&settings.game_preference.current_game).is_none() {
        settings.game_preference.current_game = games_registry::get_default_game_id();
        needs_repair = true;
    }
    
    // Validate theme
    if settings.app_settings.theme != "light" && settings.app_settings.theme != "dark" {
        settings.app_settings.theme = "dark".to_string();
        needs_repair = true;
    }
    
    if needs_repair {
        if let Err(e) = save_settings(&settings) {
            eprintln!("Warning: Failed to save repaired settings: {}", e);
        }
    }
    
    settings
}

/// Save unified settings to JSON file
pub fn save_settings(settings: &UnifiedSettings) -> Result<(), String> {
    let file_path_guard = SETTINGS_FILE.lock();
    let file_path = file_path_guard.as_ref()
        .ok_or("Settings file path not set")?;
    
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(file_path, json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    Ok(())
}

/// Get unified settings
pub fn get_settings() -> UnifiedSettings {
    let global = GLOBAL_SETTINGS.lock();
    global.clone().unwrap_or_else(|| UnifiedSettings::default())
}

/// Update unified settings
pub fn update_settings(settings: UnifiedSettings) -> Result<(), String> {
    save_settings(&settings)?;
    sync_settings_to_globals(&settings);
    Ok(())
}

/// Get app settings
pub fn get_app_settings() -> AppSettings {
    get_settings().app_settings
}

/// Update app settings
pub fn update_app_settings(app_settings: AppSettings) -> Result<(), String> {
    let mut settings = get_settings();
    settings.app_settings = app_settings;
    update_settings(settings)
}

/// Get game preference
pub fn get_game_preference() -> GamePreference {
    get_settings().game_preference
}

/// Update game preference
pub fn update_game_preference(preference: GamePreference) -> Result<(), String> {
    let mut settings = get_settings();
    settings.game_preference = preference.clone();
    update_settings(settings)?;
    
    // Sync to legacy global
    {
        use crate::common::game;
        let mut game_global = game::GLOBAL_GAME_PREFERENCE.lock();
        *game_global = Some(preference);
    }
    
    Ok(())
}

/// Get user profiles
pub fn get_user_profiles() -> UserProfiles {
    get_settings().user_profiles
}

/// Update user profiles
pub fn update_user_profiles(profiles: UserProfiles) -> Result<(), String> {
    let mut settings = get_settings();
    settings.user_profiles = profiles.clone();
    update_settings(settings)?;
    
    // Sync to legacy global
    {
        use crate::common::user_profile;
        let mut profile_global = user_profile::GLOBAL_PROFILES.lock();
        *profile_global = Some(profiles);
    }
    
    Ok(())
}

/// Get proxy settings
pub fn get_proxy_settings() -> ProxySettings {
    get_settings().proxy_settings
}

/// Update proxy settings
pub fn update_proxy_settings(proxy_settings: ProxySettings) -> Result<(), String> {
    let mut settings = get_settings();
    settings.proxy_settings = proxy_settings;
    update_settings(settings)
}

