use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

fn default_redirector_tls() -> bool {
    true
}

/// Per-title listen ports (emulator binds these when the matching service is enabled).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServicePorts {
    pub web_http: u16,
    pub web_https: u16,
    pub web_http_alt: u16,
    pub web_https_alt: u16,
    pub blaze_gosredirector: u16,
    pub blaze_gosca: u16,
    pub blaze_main: u16,
    pub blaze_alt: u16,
    pub blaze_sec: u16,
    pub qos_coordinator: u16,
    pub qos_data: u16,
    pub qos_alt: u16,
    pub rtm: u16,
    pub lsx: u16,
}

impl Default for ServicePorts {
    fn default() -> Self {
        Self {
            web_http: 80,
            web_https: 443,
            web_http_alt: 8080,
            web_https_alt: 8443,
            blaze_gosredirector: 42230,
            blaze_gosca: 44325,
            blaze_main: 10042,
            blaze_alt: 10040,
            blaze_sec: 15271,
            qos_coordinator: 3659,
            qos_data: 4001,
            qos_alt: 10010,
            rtm: 8095,
            lsx: 3216,
        }
    }
}

/// Game information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub frostbite_build: String,
    pub blaze_build: String,
    #[serde(default = "default_redirector_tls")]
    pub redirector_tls: bool,
    pub enabled_services: Vec<String>,
    #[serde(default)]
    pub service_ports: ServicePorts,
}

impl GameInfo {
    pub fn new(
        id: String,
        name: String,
        protocol: String,
        frostbite_build: String,
        blaze_build: String,
        redirector_tls: bool,
        enabled_services: Vec<String>,
        service_ports: ServicePorts,
    ) -> Self {
        Self {
            id,
            name,
            protocol,
            frostbite_build,
            blaze_build,
            redirector_tls,
            enabled_services,
            service_ports,
        }
    }
}

/// User game preference (stored in `settings.json`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePreference {
    pub current_game: String,
}

impl Default for GamePreference {
    fn default() -> Self {
        Self {
            current_game: crate::common::game::games_registry::DEFAULT_GAME_ID.to_string(),
        }
    }
}

/// Game selection for UI (definitions from `games.json`)
#[derive(Debug, Clone)]
pub struct GameSelection {
    pub available_games: Vec<GameInfo>,
    pub current_game: String,
}

pub(crate) static GLOBAL_GAME_PREFERENCE: Mutex<Option<GamePreference>> = Mutex::new(None);

pub fn current_service_ports() -> ServicePorts {
    get_current_game()
        .map(|g| g.service_ports)
        .unwrap_or_default()
}

pub fn get_game_selection() -> GameSelection {
    use crate::common::game::games_registry;
    use crate::common::settings;

    let preference = settings::get_game_preference();

    GameSelection {
        available_games: games_registry::get_all_game_definitions(),
        current_game: preference.current_game,
    }
}

pub fn set_current_game(game_id: &str) -> Result<(), String> {
    use crate::common::game::games_registry;
    use crate::common::settings;

    if games_registry::get_game_definition(game_id).is_none() {
        return Err(format!("Game with id '{}' not found", game_id));
    }

    let mut preference = settings::get_game_preference();
    preference.current_game = game_id.to_string();
    settings::update_game_preference(preference)?;
    Ok(())
}

pub fn get_current_game() -> Option<GameInfo> {
    let selection = get_game_selection();
    selection
        .available_games
        .into_iter()
        .find(|g| g.id == selection.current_game)
}

pub fn get_current_game_id() -> String {
    get_game_selection().current_game
}

pub fn remove_registered_game(game_id: &str) -> Result<(), String> {
    use crate::common::game::games_registry;
    games_registry::remove_game(game_id)?;
    if games_registry::get_game_definition(&get_current_game_id()).is_none() {
        set_current_game(&games_registry::get_default_game_id())?;
    }
    Ok(())
}
