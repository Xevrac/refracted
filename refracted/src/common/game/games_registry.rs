//! User-editable game list persisted in `{app_data}/games.json`.
//! The first run seeds from `resources/default_games.json` (embedded at compile time).

use super::game_module::GameInfo;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;

pub const DEFAULT_GAME_ID: &str = "bf-labs";

static GLOBAL_GAMES: Mutex<Option<GamesDocument>> = Mutex::new(None);

const EMBEDDED_DEFAULT_GAMES: &str = include_str!("../../../resources/default_games.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamesDocument {
    #[serde(default = "schema_current")]
    pub schema_version: u32,
    pub games: Vec<GameInfo>,
}

fn schema_current() -> u32 {
    1
}

impl Default for GamesDocument {
    fn default() -> Self {
        serde_json::from_str(EMBEDDED_DEFAULT_GAMES).unwrap_or_else(|_| Self {
            schema_version: schema_current(),
            games: vec![],
        })
    }
}

fn games_json_path() -> std::path::PathBuf {
    crate::common::paths::games_json_path()
}

fn load_embedded_default() -> GamesDocument {
    serde_json::from_str(EMBEDDED_DEFAULT_GAMES).unwrap_or_else(|_| GamesDocument {
        schema_version: schema_current(),
        games: vec![],
    })
}

fn write_document(path: &std::path::Path, doc: &GamesDocument) -> Result<(), String> {
    let json = serde_json::to_string_pretty(doc)
        .map_err(|e| format!("Failed to serialize games.json: {}", e))?;
    crate::common::paths::ensure_parent_dir(path)
        .map_err(|e| format!("Failed to create games.json parent dir: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Failed to write games.json: {}", e))
}

/// Load or create `games.json` and store in memory. Call during app startup (before reading settings).
pub fn init_games_registry() -> Result<(), String> {
    let path = games_json_path();
    let mut doc = if path.exists() {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read games.json: {}", e))?;
        match serde_json::from_str::<GamesDocument>(&content) {
            Ok(d) => d,
            Err(e) => {
                let backup = path.with_extension("json.bak");
                let _ = fs::copy(&path, &backup);
                let fresh = load_embedded_default();
                write_document(&path, &fresh)?;
                eprintln!(
                    "games.json parse error ({}), restored default from embedded seed; backup: {}",
                    e,
                    backup.display()
                );
                fresh
            }
        }
    } else {
        let fresh = load_embedded_default();
        write_document(&path, &fresh)?;
        fresh
    };

    repair_document(&mut doc);
    write_document(&path, &doc)?;
    *GLOBAL_GAMES.lock() = Some(doc);
    Ok(())
}

fn repair_document(doc: &mut GamesDocument) {
    doc.schema_version = schema_current();

    if doc.games.is_empty() {
        *doc = load_embedded_default();
        return;
    }
    // CNC + Prism: TLS to gosredirector :42127; old seeds used plain 42230 and never saw traffic.
    for g in &mut doc.games {
        if g.id == "cnc" {
            g.redirector_tls = true;
            if g.service_ports.blaze_gosredirector == 42230 {
                g.service_ports.blaze_gosredirector = 42127;
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    doc.games.retain(|g| {
        if g.id.is_empty() || seen.contains(&g.id) {
            return false;
        }
        seen.insert(g.id.clone());
        true
    });
    if doc.games.is_empty() {
        *doc = load_embedded_default();
    }
}

fn with_document_mut(f: impl FnOnce(&mut GamesDocument) -> Result<(), String>) -> Result<(), String> {
    let mut guard = GLOBAL_GAMES.lock();
    let doc = guard
        .as_mut()
        .ok_or("Games registry not initialized (init_games_registry not called)")?;
    f(doc)?;
    write_document(&games_json_path(), doc)?;
    Ok(())
}

/// Insert or replace a game by `id` and persist.
pub fn upsert_game(game: GameInfo) -> Result<(), String> {
    if game.id.is_empty() {
        return Err("Game id cannot be empty".to_string());
    }
    with_document_mut(|doc| {
        if let Some(i) = doc.games.iter().position(|g| g.id == game.id) {
            doc.games[i] = game;
        } else {
            doc.games.push(game);
        }
        Ok(())
    })
}

pub fn remove_game(game_id: &str) -> Result<(), String> {
    with_document_mut(|doc| {
        doc.games.retain(|g| g.id != game_id);
        if doc.games.is_empty() {
            *doc = load_embedded_default();
        }
        Ok(())
    })
}

fn snapshot() -> GamesDocument {
    GLOBAL_GAMES
        .lock()
        .clone()
        .unwrap_or_else(load_embedded_default)
}

pub fn get_all_game_definitions() -> Vec<GameInfo> {
    snapshot().games
}

pub fn get_game_definition(game_id: &str) -> Option<GameInfo> {
    snapshot()
        .games
        .into_iter()
        .find(|g| g.id == game_id)
}

pub fn get_default_game_id() -> String {
    let doc = snapshot();
    doc
        .games
        .first()
        .map(|g| g.id.clone())
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| DEFAULT_GAME_ID.to_string())
}
