//! Pure game logic. No browser types in here: everything unit-tests natively.

pub mod logic;
pub mod parser;
pub mod state;
pub mod text;
pub mod world;

/// The game's name, used on the title screen and in UI text.
pub const TITLE: &str = "Neon Knights";
pub const SUBTITLE: &str = "A Gentleman's Evening Out";

pub use logic::{Death, Effect, Game, Reply};
pub use parser::{parse, Command, Noun, ParseError, Verb};
pub use state::{DeathKind, Flag, GameState, Item, SceneId, LEGACY_SAVE_KEYS, MAX_SCORE, SAVE_KEY};
