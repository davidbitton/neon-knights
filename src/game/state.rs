//! Game state: score, inventory, flags, and save-game serialization.

pub const MAX_SCORE: u32 = 20;
pub const SAVE_MAGIC: &str = "NN1";

/// localStorage key for the single save slot.
pub const SAVE_KEY: &str = "neon-knights.save.v1";
/// Keys used by earlier builds (the game was briefly called "Neon Nights").
/// Saves found under these are migrated to [`SAVE_KEY`] on restore. The save
/// format itself (`NN1|...`) is unchanged.
pub const LEGACY_SAVE_KEYS: &[&str] = &["neon-nights.save.v1"];

/// Hero start position (feet) in the Rusty Anchor.
pub const START_POS: (i32, i32) = (40, 150);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Quarter,
    Comb,
    Matchbook,
    Drink,
}

impl Item {
    pub const ALL: [Item; 4] = [Item::Quarter, Item::Comb, Item::Matchbook, Item::Drink];

    pub fn code(self) -> char {
        match self {
            Item::Quarter => 'q',
            Item::Comb => 'c',
            Item::Matchbook => 'm',
            Item::Drink => 'd',
        }
    }
    pub fn from_code(c: char) -> Option<Item> {
        Item::ALL.into_iter().find(|i| i.code() == c)
    }
    pub fn name(self) -> &'static str {
        match self {
            Item::Quarter => "a lucky quarter",
            Item::Comb => "a pocket comb",
            Item::Matchbook => "a Rusty Anchor matchbook",
            Item::Drink => "a Bilge Water Special",
        }
    }
}

/// One-shot story/score flags, stored as a bitset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Flag {
    TookMatchbook = 1 << 0,
    ReadMatchbook = 1 << 1,
    PlayedJukebox = 1 << 2,
    GotDrink = 1 << 3,
    DrankDrink = 1 << 4,
    EggWarned = 1 << 5,
    SatOnStool = 1 << 6,
    SeenIntro = 1 << 7,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub score: u32,
    pub inventory: Vec<Item>,
    pub flags: u32,
    pub hero: (i32, i32),
    pub moves: u32,
    pub gus_talks: u32,
    pub dead: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    BadMagic,
    Malformed(&'static str),
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            score: 0,
            inventory: vec![Item::Quarter, Item::Comb],
            flags: 0,
            hero: START_POS,
            moves: 0,
            gus_talks: 0,
            dead: false,
        }
    }

    pub fn has(&self, item: Item) -> bool {
        self.inventory.contains(&item)
    }
    pub fn add(&mut self, item: Item) {
        if !self.has(item) {
            self.inventory.push(item);
        }
    }
    pub fn remove(&mut self, item: Item) -> bool {
        let before = self.inventory.len();
        self.inventory.retain(|i| *i != item);
        before != self.inventory.len()
    }

    pub fn flag(&self, f: Flag) -> bool {
        self.flags & f as u32 != 0
    }
    pub fn set(&mut self, f: Flag) {
        self.flags |= f as u32;
    }

    /// Set a flag and award points the first time only. Returns the points
    /// actually awarded.
    pub fn award(&mut self, f: Flag, points: u32) -> u32 {
        if self.flag(f) {
            return 0;
        }
        self.set(f);
        self.score = (self.score + points).min(MAX_SCORE);
        points
    }

    pub fn rank(&self) -> &'static str {
        match self.score {
            0..=4 => "Wallflower",
            5..=9 => "Barfly",
            10..=14 => "Smooth Operator (Trainee)",
            15..=19 => "Local Legend (Self-Proclaimed)",
            _ => "Gentleman of the Evening",
        }
    }

    /// Compact text format for localStorage:
    /// `NN1|score|flags|inventory-codes|x|y|moves|gus_talks|dead`
    pub fn serialize(&self) -> String {
        let inv: String = self.inventory.iter().map(|i| i.code()).collect();
        format!(
            "{SAVE_MAGIC}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.score,
            self.flags,
            inv,
            self.hero.0,
            self.hero.1,
            self.moves,
            self.gus_talks,
            self.dead as u8
        )
    }

    pub fn deserialize(s: &str) -> Result<GameState, LoadError> {
        let parts: Vec<&str> = s.trim().split('|').collect();
        if parts.first() != Some(&SAVE_MAGIC) {
            return Err(LoadError::BadMagic);
        }
        if parts.len() != 9 {
            return Err(LoadError::Malformed("field count"));
        }
        let num = |i: usize, what: &'static str| -> Result<i64, LoadError> {
            parts[i]
                .parse::<i64>()
                .map_err(|_| LoadError::Malformed(what))
        };
        let mut inventory = Vec::new();
        for c in parts[3].chars() {
            inventory.push(Item::from_code(c).ok_or(LoadError::Malformed("inventory"))?);
        }
        let score = num(1, "score")?;
        if !(0..=MAX_SCORE as i64).contains(&score) {
            return Err(LoadError::Malformed("score range"));
        }
        Ok(GameState {
            score: score as u32,
            flags: num(2, "flags")? as u32,
            inventory,
            hero: (num(4, "x")? as i32, num(5, "y")? as i32),
            moves: num(6, "moves")? as u32,
            gus_talks: num(7, "gus")? as u32,
            dead: num(8, "dead")? != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_defaults() {
        let s = GameState::new();
        assert_eq!(s.score, 0);
        assert!(s.has(Item::Quarter));
        assert!(s.has(Item::Comb));
        assert!(!s.has(Item::Matchbook));
        assert_eq!(s.rank(), "Wallflower");
    }

    #[test]
    fn award_only_once_and_capped() {
        let mut s = GameState::new();
        assert_eq!(s.award(Flag::TookMatchbook, 5), 5);
        assert_eq!(s.award(Flag::TookMatchbook, 5), 0);
        assert_eq!(s.score, 5);
        s.award(Flag::GotDrink, 500);
        assert_eq!(s.score, MAX_SCORE);
    }

    #[test]
    fn inventory_add_remove() {
        let mut s = GameState::new();
        s.add(Item::Matchbook);
        s.add(Item::Matchbook);
        assert_eq!(
            s.inventory
                .iter()
                .filter(|i| **i == Item::Matchbook)
                .count(),
            1
        );
        assert!(s.remove(Item::Quarter));
        assert!(!s.remove(Item::Quarter));
    }

    #[test]
    fn save_roundtrip() {
        let mut s = GameState::new();
        s.award(Flag::TookMatchbook, 5);
        s.add(Item::Matchbook);
        s.remove(Item::Quarter);
        s.hero = (123, 140);
        s.moves = 17;
        s.gus_talks = 2;
        let text = s.serialize();
        assert!(text.starts_with("NN1|"));
        assert_eq!(GameState::deserialize(&text), Ok(s));
    }

    #[test]
    fn save_keys_use_current_name() {
        assert!(SAVE_KEY.starts_with("neon-knights."));
        assert!(!LEGACY_SAVE_KEYS.contains(&SAVE_KEY));
    }

    #[test]
    fn rejects_garbage_saves() {
        assert_eq!(GameState::deserialize("hello"), Err(LoadError::BadMagic));
        assert!(GameState::deserialize("NN1|1|2").is_err());
        assert!(GameState::deserialize("NN1|5|0|qz|1|1|0|0|0").is_err());
        assert!(GameState::deserialize("NN1|999|0|q|1|1|0|0|0").is_err());
    }
}
