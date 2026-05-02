pub mod game_module;
pub mod games_registry;

pub use game_module::*;
pub use games_registry::{
    get_all_game_definitions, get_default_game_id, get_game_definition, init_games_registry,
    upsert_game, GamesDocument, DEFAULT_GAME_ID,
};
