//! Game state: current scene, score, inventory, cash, flags, NPC
//! conversation counters, and save-game serialization (versioned).

/// Every point in the game. See WALKTHROUGH.md for where each one comes from.
pub const MAX_SCORE: u32 = 60;

/// Current save format tag.
pub const SAVE_MAGIC: &str = "NN2";
/// Milestone 1 save format (bar only). Still loads.
pub const SAVE_MAGIC_V1: &str = "NN1";

/// localStorage key for the single save slot.
pub const SAVE_KEY: &str = "neon-knights.save.v1";
/// Keys used by earlier builds (the game was briefly called "Neon Nights").
/// Saves found under these are migrated to [`SAVE_KEY`] on restore.
pub const LEGACY_SAVE_KEYS: &[&str] = &["neon-nights.save.v1"];

/// Hero start position (feet) in the Rusty Anchor.
pub const START_POS: (i32, i32) = (40, 150);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SceneId {
    Bar,
    Boardwalk,
    Alley,
    Store,
    Casino,
    Rooftop,
}

impl SceneId {
    pub const ALL: [SceneId; 6] = [
        SceneId::Bar,
        SceneId::Boardwalk,
        SceneId::Alley,
        SceneId::Store,
        SceneId::Casino,
        SceneId::Rooftop,
    ];
    pub fn code(self) -> u8 {
        self as u8
    }
    pub fn from_code(c: u8) -> Option<SceneId> {
        SceneId::ALL.get(c as usize).copied()
    }
    pub fn name(self) -> &'static str {
        match self {
            SceneId::Bar => "The Rusty Anchor",
            SceneId::Boardwalk => "The Boardwalk",
            SceneId::Alley => "Back Alley",
            SceneId::Store => "Lucky Gull 24/7",
            SceneId::Casino => "The Golden Clam",
            SceneId::Rooftop => "The Crow's Nest",
        }
    }
    pub fn visited_flag(self) -> Flag {
        match self {
            SceneId::Bar => Flag::SeenIntro,
            SceneId::Boardwalk => Flag::VisitedBoardwalk,
            SceneId::Alley => Flag::VisitedAlley,
            SceneId::Store => Flag::VisitedStore,
            SceneId::Casino => Flag::VisitedCasino,
            SceneId::Rooftop => Flag::VisitedRooftop,
        }
    }
}

/// The game's comedic deaths. Each has its own epitaph and tombstone art.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeathKind {
    Egg,
    Dumpster,
    Slushie,
    Jackpot,
    Pool,
}

impl DeathKind {
    pub const ALL: [DeathKind; 5] = [
        DeathKind::Egg,
        DeathKind::Dumpster,
        DeathKind::Slushie,
        DeathKind::Jackpot,
        DeathKind::Pool,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Quarter,
    Comb,
    Matchbook,
    Drink,
    HotDog,
    Carnation,
    Mints,
}

impl Item {
    pub const ALL: [Item; 7] = [
        Item::Quarter,
        Item::Comb,
        Item::Matchbook,
        Item::Drink,
        Item::HotDog,
        Item::Carnation,
        Item::Mints,
    ];

    pub fn code(self) -> char {
        match self {
            Item::Quarter => 'q',
            Item::Comb => 'c',
            Item::Matchbook => 'm',
            Item::Drink => 'd',
            Item::HotDog => 'h',
            Item::Carnation => 'f',
            Item::Mints => 'n',
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
            Item::HotDog => "a complimentary hot dog",
            Item::Carnation => "a red carnation",
            Item::Mints => "a tin of Arctic Blast breath mints",
        }
    }
}

/// One-shot story/score flags, stored as a 64-bit set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Flag {
    TookMatchbook = 1 << 0,
    ReadMatchbook = 1 << 1,
    PlayedJukebox = 1 << 2,
    GotDrink = 1 << 3,
    DrankDrink = 1 << 4,
    EggWarned = 1 << 5,
    SatOnStool = 1 << 6,
    SeenIntro = 1 << 7,
    // Milestone 2.
    VisitedBoardwalk = 1 << 8,
    VisitedAlley = 1 << 9,
    VisitedStore = 1 << 10,
    VisitedCasino = 1 << 11,
    VisitedRooftop = 1 << 12,
    GotHotDog = 1 << 13,
    UsedTelescope = 1 << 14,
    MimeFed = 1 << 15,
    GotFiver = 1 << 16,
    BoughtCarnation = 1 << 17,
    BoughtMints = 1 << 18,
    WearingCarnation = 1 << 19,
    TinyApproved = 1 << 20,
    AteMints = 1 << 21,
    Won = 1 << 22,
}

/// Conversation counters, one per talking NPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Npc {
    Gus = 0,
    Dot = 1,
    Mime = 2,
    Brenda = 3,
    Tiny = 4,
    Vonda = 5,
}
pub const NPC_COUNT: usize = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub scene: SceneId,
    pub score: u32,
    pub inventory: Vec<Item>,
    /// Dollars (the quarter is tracked separately as an item).
    pub cash: u32,
    pub flags: u64,
    pub hero: (i32, i32),
    pub moves: u32,
    pub talks: [u32; NPC_COUNT],
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
            scene: SceneId::Bar,
            score: 0,
            inventory: vec![Item::Quarter, Item::Comb],
            cash: 0,
            flags: 0,
            hero: START_POS,
            moves: 0,
            talks: [0; NPC_COUNT],
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
        self.flags & f as u64 != 0
    }
    pub fn set(&mut self, f: Flag) {
        self.flags |= f as u64;
    }

    /// Bump and return an NPC's conversation counter (1-based).
    pub fn talk(&mut self, npc: Npc) -> u32 {
        self.talks[npc as usize] += 1;
        self.talks[npc as usize]
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
            0..=9 => "Wallflower",
            10..=19 => "Barfly",
            20..=34 => "Smooth Operator (Trainee)",
            35..=49 => "Local Legend (Self-Proclaimed)",
            50..=59 => "Boardwalk Royalty",
            _ => "Gentleman of the Evening",
        }
    }

    /// Compact text format for localStorage (v2):
    /// `NN2|scene|score|flags|inventory|cash|x|y|moves|talks,comma,separated|dead`
    pub fn serialize(&self) -> String {
        let inv: String = self.inventory.iter().map(|i| i.code()).collect();
        let talks: Vec<String> = self.talks.iter().map(|t| t.to_string()).collect();
        format!(
            "{SAVE_MAGIC}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.scene.code(),
            self.score,
            self.flags,
            inv,
            self.cash,
            self.hero.0,
            self.hero.1,
            self.moves,
            talks.join(","),
            self.dead as u8
        )
    }

    pub fn deserialize(s: &str) -> Result<GameState, LoadError> {
        let parts: Vec<&str> = s.trim().split('|').collect();
        match parts.first() {
            Some(&SAVE_MAGIC) => Self::load_v2(&parts),
            Some(&SAVE_MAGIC_V1) => Self::load_v1(&parts),
            _ => Err(LoadError::BadMagic),
        }
    }

    fn num(parts: &[&str], i: usize, what: &'static str) -> Result<i64, LoadError> {
        parts[i]
            .parse::<i64>()
            .map_err(|_| LoadError::Malformed(what))
    }

    fn inventory(codes: &str) -> Result<Vec<Item>, LoadError> {
        codes
            .chars()
            .map(|c| Item::from_code(c).ok_or(LoadError::Malformed("inventory")))
            .collect()
    }

    fn check_score(score: i64) -> Result<u32, LoadError> {
        if (0..=MAX_SCORE as i64).contains(&score) {
            Ok(score as u32)
        } else {
            Err(LoadError::Malformed("score range"))
        }
    }

    fn load_v2(parts: &[&str]) -> Result<GameState, LoadError> {
        if parts.len() != 11 {
            return Err(LoadError::Malformed("field count"));
        }
        let scene = SceneId::from_code(Self::num(parts, 1, "scene")? as u8)
            .ok_or(LoadError::Malformed("scene"))?;
        let mut talks = [0u32; NPC_COUNT];
        let raw: Vec<&str> = parts[9].split(',').collect();
        if raw.len() != NPC_COUNT {
            return Err(LoadError::Malformed("talks"));
        }
        for (slot, r) in talks.iter_mut().zip(raw) {
            *slot = r.parse().map_err(|_| LoadError::Malformed("talks"))?;
        }
        Ok(GameState {
            scene,
            score: Self::check_score(Self::num(parts, 2, "score")?)?,
            flags: parts[3]
                .parse::<u64>()
                .map_err(|_| LoadError::Malformed("flags"))?,
            inventory: Self::inventory(parts[4])?,
            cash: Self::num(parts, 5, "cash")? as u32,
            hero: (
                Self::num(parts, 6, "x")? as i32,
                Self::num(parts, 7, "y")? as i32,
            ),
            moves: Self::num(parts, 8, "moves")? as u32,
            talks,
            dead: Self::num(parts, 10, "dead")? != 0,
        })
    }

    /// Milestone 1: `NN1|score|flags|inventory|x|y|moves|gus_talks|dead`,
    /// always in the bar.
    fn load_v1(parts: &[&str]) -> Result<GameState, LoadError> {
        if parts.len() != 9 {
            return Err(LoadError::Malformed("field count"));
        }
        let mut talks = [0u32; NPC_COUNT];
        talks[Npc::Gus as usize] = Self::num(parts, 7, "gus")? as u32;
        Ok(GameState {
            scene: SceneId::Bar,
            score: Self::check_score(Self::num(parts, 1, "score")?)?,
            // v1 only ever used the low 8 bits.
            flags: (Self::num(parts, 2, "flags")? as u64) & 0xff,
            inventory: Self::inventory(parts[3])?,
            cash: 0,
            hero: (
                Self::num(parts, 4, "x")? as i32,
                Self::num(parts, 5, "y")? as i32,
            ),
            moves: Self::num(parts, 6, "moves")? as u32,
            talks,
            dead: Self::num(parts, 8, "dead")? != 0,
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
        assert_eq!(s.scene, SceneId::Bar);
        assert!(s.has(Item::Quarter));
        assert!(s.has(Item::Comb));
        assert!(!s.has(Item::Matchbook));
        assert_eq!(s.cash, 0);
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
        assert_eq!(s.rank(), "Gentleman of the Evening");
    }

    #[test]
    fn flags_use_high_bits() {
        let mut s = GameState::new();
        s.set(Flag::Won);
        assert!(s.flag(Flag::Won));
        assert!(!s.flag(Flag::AteMints));
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
    fn save_roundtrip_v2() {
        let mut s = GameState::new();
        s.scene = SceneId::Casino;
        s.award(Flag::TookMatchbook, 5);
        s.award(Flag::BoughtCarnation, 3);
        s.add(Item::Carnation);
        s.remove(Item::Quarter);
        s.cash = 2;
        s.hero = (123, 140);
        s.moves = 17;
        s.talks = [1, 2, 3, 0, 4, 0];
        let text = s.serialize();
        assert!(text.starts_with("NN2|4|"));
        assert_eq!(GameState::deserialize(&text), Ok(s));
    }

    #[test]
    fn milestone1_saves_still_load() {
        let s = GameState::deserialize("NN1|8|3|qcm|120|118|5|2|0").unwrap();
        assert_eq!(s.scene, SceneId::Bar);
        assert_eq!(s.score, 8);
        assert!(s.flag(Flag::TookMatchbook) && s.flag(Flag::ReadMatchbook));
        assert_eq!(
            s.inventory,
            vec![Item::Quarter, Item::Comb, Item::Matchbook]
        );
        assert_eq!(s.hero, (120, 118));
        assert_eq!(s.talks[Npc::Gus as usize], 2);
        assert_eq!(s.cash, 0);
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
        assert!(GameState::deserialize("NN2|9|0|0|q|0|1|1|0|0,0,0,0,0,0|0").is_err());
        assert!(GameState::deserialize("NN2|1|0|0|q|0|1|1|0|0,0|0").is_err());
    }
}
