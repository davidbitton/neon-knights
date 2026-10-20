//! Pure game logic. No browser types in here: everything unit-tests natively.

pub mod logic;
pub mod parser;
pub mod scene;
pub mod state;
pub mod text;

pub use logic::{Death, Effect, Game, Reply};
pub use parser::{parse, Command, Noun, ParseError, Verb};
pub use state::{Flag, GameState, Item, MAX_SCORE};
