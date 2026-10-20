//! Command handling. Scene-specific rules and writing live in the per-scene
//! modules; this file has the shared machinery: parsing, meta commands,
//! travel between scenes, inventory items usable anywhere, deaths, and
//! the generic fallbacks.

mod alley;
mod bar;
mod boardwalk;
mod casino;
mod rooftop;
mod store;
#[cfg(test)]
mod tests;

use super::parser::{parse, Command, Noun, ParseError, Verb};
use super::state::{DeathKind, Flag, GameState, Item, SceneId, MAX_SCORE};
use super::world::{self, Exit};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Death {
    pub kind: DeathKind,
    pub cause: &'static str,
    pub epitaph: Vec<&'static str>,
}

/// Side effects the platform layer must perform.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    /// Score changed.
    Scored(u32),
    Died(Death),
    Save,
    Restore,
    Restart,
    Quit,
    /// The hero was relocated (scene change or pushed back from an exit):
    /// the platform must re-sync its actor from `state.hero`.
    Moved,
    /// The game has been won; show the ending screen.
    Won,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub messages: Vec<String>,
    pub effect: Effect,
}

impl Reply {
    pub(crate) fn say(s: impl Into<String>) -> Reply {
        Reply {
            messages: vec![s.into()],
            effect: Effect::None,
        }
    }
    pub(crate) fn many(v: Vec<&str>) -> Reply {
        Reply {
            messages: v.into_iter().map(String::from).collect(),
            effect: Effect::None,
        }
    }
    fn effect(effect: Effect) -> Reply {
        Reply {
            messages: vec![],
            effect,
        }
    }
    pub(crate) fn with(mut self, effect: Effect) -> Reply {
        self.effect = effect;
        self
    }
}

pub(crate) fn say(s: impl Into<String>) -> Option<Reply> {
    Some(Reply::say(s))
}

pub(crate) fn scored(r: Reply, pts: u32) -> Option<Reply> {
    Some(if pts > 0 {
        r.with(Effect::Scored(pts))
    } else {
        r
    })
}

pub(crate) fn pick<'a>(lines: &[&'a str], n: u32) -> &'a str {
    lines[n as usize % lines.len()]
}

pub(crate) fn too_far(s: &GameState) -> Option<Reply> {
    say(pick(&TOO_FAR, s.moves))
}

pub(crate) fn near(s: &GameState, n: Noun) -> bool {
    world::is_near(s, n)
}

pub const INTRO: &str = "Ocean Bluffs, 11:42 PM. The tide is out, the rent is due, and Chip Valentine has squeezed into his lucky powder-blue tuxedo for one perfect night on the town.\n\nHis first stop: The Rusty Anchor, a bar so dingy the cockroaches wipe their feet on the way OUT.";

pub const HELP: &str = "Type commands like LOOK, LOOK AT BARTENDER, TAKE MATCHBOOK, TALK TO GUS, USE QUARTER ON JUKEBOX, GIVE HOT DOG TO MIME, BUY CARNATION, INVENTORY (or I), SCORE, SAVE, RESTORE, RESTART.\n\nArrow keys walk (press again to stop). Click to walk. Walk off a screen edge or into a doorway to go places, or type GO TO CASINO, ENTER STORE, LEAVE. Click a thing to use the current verb on it; right-click (or Tab) changes the verb.";

pub const TOO_FAR: [&str; 3] = [
    "You'd have to get closer. Chip's arms are long, but not THAT long.",
    "From way over here? Chip is smooth, not telekinetic.",
    "Walk on over first. Gentlemen don't shout across the room.",
];

pub const UNKNOWN_WORD: [&str; 4] = [
    "\"{}\"? Chip has heard a lot of sweet nothings in his day, but never that one.",
    "Is \"{}\" a dance? A cocktail? A rash? Whatever it is, Chip doesn't know it.",
    "Somewhere a dictionary weeps. This game doesn't know the word \"{}\".",
    "Chip mouths \"{}\" silently, then pretends he meant to do that.",
];

pub const EMPTY_INPUT: [&str; 2] = [
    "Chip stares into the middle distance. Very brooding. Very handsome. Very unproductive.",
    "Silence is golden, but it doesn't score points.",
];

pub fn death(kind: DeathKind) -> Death {
    let (cause, epitaph): (&str, &[&str]) = match kind {
        DeathKind::Egg => (
            "Death by pickled egg",
            &[
                "HERE LIES CHIP VALENTINE",
                "He came for a night out.",
                "He got a night out cold.",
                "",
                "Survived by one pickled egg,",
                "still at large.",
            ],
        ),
        DeathKind::Dumpster => (
            "Collected with the trash",
            &[
                "HERE LIES CHIP VALENTINE",
                "He always said someone would",
                "pick him up at midnight.",
                "",
                "It was a garbage truck.",
                "It was right on time.",
            ],
        ),
        DeathKind::Slushie => (
            "Brain freeze, extra large",
            &[
                "HERE LIES CHIP VALENTINE",
                "He finally kept his cool.",
                "Permanently.",
                "",
                "Flavor: Blue.",
                "(Not a fruit. Just blue.)",
            ],
        ),
        DeathKind::Jackpot => (
            "Buried by a jackpot",
            &[
                "HERE LIES CHIP VALENTINE",
                "He hit the jackpot.",
                "The jackpot hit back.",
                "",
                "Eleven thousand quarters,",
                "zero survivors.",
            ],
        ),
        DeathKind::Pool => (
            "A swan dive into nothing",
            &[
                "HERE LIES CHIP VALENTINE",
                "He took the plunge.",
                "The pool did not.",
                "",
                "Drained for repairs.",
                "So, in the end, was Chip.",
            ],
        ),
    };
    Death {
        kind,
        cause,
        epitaph: epitaph.to_vec(),
    }
}

pub struct Game {
    pub state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a noun to the inventory item it names, if any.
fn item_for(n: Noun) -> Option<Item> {
    Some(match n {
        Noun::Quarter => Item::Quarter,
        Noun::Comb => Item::Comb,
        Noun::Matchbook => Item::Matchbook,
        Noun::Drink => Item::Drink,
        Noun::HotDog => Item::HotDog,
        Noun::Carnation => Item::Carnation,
        Noun::Mints => Item::Mints,
        _ => return None,
    })
}

fn place_noun(scene: SceneId) -> Noun {
    match scene {
        SceneId::Bar => Noun::Bar,
        SceneId::Boardwalk => Noun::Boardwalk,
        SceneId::Alley => Noun::Alley,
        SceneId::Store => Noun::Store,
        SceneId::Casino => Noun::Casino,
        SceneId::Rooftop => Noun::Rooftop,
    }
}

const PLACES: [Noun; 6] = [
    Noun::Bar,
    Noun::Boardwalk,
    Noun::Alley,
    Noun::Store,
    Noun::Casino,
    Noun::Rooftop,
];

impl Game {
    pub fn new() -> Game {
        Game {
            state: GameState::new(),
        }
    }

    pub fn score_line(&self) -> String {
        format!("Score: {} of {}", self.state.score, MAX_SCORE)
    }

    /// Lines for the ending screen.
    pub fn ending_lines(&self) -> Vec<String> {
        vec![
            "THE END".into(),
            String::new(),
            "Vonda hired Chip on the spot:".into(),
            "backup vocals at the Crow's Nest,".into(),
            "Thursdays and alternate Saturdays.".into(),
            String::new(),
            "He came looking for romance and found".into(),
            "something rarer in a beach town:".into(),
            "steady work with dental.".into(),
            String::new(),
            format!("Final score: {} of {}", self.state.score, MAX_SCORE),
            format!("Rank: {}", self.state.rank()),
        ]
    }

    /// Handle one typed line.
    pub fn input(&mut self, line: &str) -> Reply {
        if self.state.dead {
            return Reply::say("You're dead. It's a whole thing. Restore, restart, or quit.");
        }
        if self.state.flag(Flag::Won) {
            return Reply::say(
                "The night is over, and it was a triumph. Restart to do it all again.",
            );
        }
        match parse(line) {
            Ok(cmd) => {
                self.state.moves += 1;
                let cmd = self.resolve(cmd);
                self.run(&cmd)
            }
            Err(ParseError::Empty) => Reply::say(pick(&EMPTY_INPUT, self.state.moves)),
            Err(ParseError::UnknownWord(w)) => {
                let tmpl = pick(&UNKNOWN_WORD, self.state.moves + w.len() as u32);
                self.state.moves += 1;
                Reply::say(tmpl.replace("{}", &w))
            }
            Err(ParseError::NoVerb(n)) => Reply::say(format!(
                "What about the {}? Start with a verb, like a gentleman: LOOK, TAKE, TALK, USE...",
                world::noun_word(n)
            )),
        }
    }

    /// Resolve generic nouns ("man", "machine", "door", "money") for the
    /// current scene.
    fn resolve(&self, mut cmd: Command) -> Command {
        use Noun as N;
        use SceneId as S;
        let s = &self.state;
        let verb = cmd.verb;
        for n in cmd.nouns.iter_mut() {
            *n = match (*n, s.scene) {
                (N::Man, S::Bar) => N::Bartender,
                (N::Man, S::Alley) => N::Mime,
                (N::Man, S::Casino) => N::Tiny,
                (N::Woman, S::Boardwalk) => N::Dot,
                (N::Woman, S::Store) => N::Brenda,
                (N::Woman, S::Rooftop) => N::Vonda,
                (N::Machine, S::Bar) => N::Jukebox,
                (N::Machine, S::Store) => N::Slushie,
                (N::Machine, S::Casino) => N::Slots,
                (N::Door, S::Rooftop) => N::Elevator,
                (N::Railing, S::Boardwalk) => N::Ocean,
                (N::Money, _)
                    if s.cash == 0
                        && s.has(Item::Quarter)
                        && matches!(verb, Verb::Use | Verb::Give | Verb::Play | Verb::Order) =>
                {
                    N::Quarter
                }
                (n, _) => n,
            };
        }
        cmd.nouns.dedup();
        cmd
    }

    /// Is this noun something Chip can refer to right now?
    fn present(&self, n: Noun) -> bool {
        let s = &self.state;
        match n {
            Noun::Room | Noun::Floor | Noun::Me => true,
            Noun::Money => s.cash > 0 || s.has(Item::Quarter),
            _ if item_for(n).is_some_and(|i| s.has(i)) => true,
            _ if PLACES.contains(&n) => true,
            _ => world::hotspots(s).iter().any(|h| h.noun == n),
        }
    }

    fn run(&mut self, cmd: &Command) -> Reply {
        match cmd.verb {
            Verb::Help => return Reply::say(HELP),
            Verb::Inventory => return self.inventory(),
            Verb::Score => {
                return Reply::say(format!(
                    "Your score is {} of {}. Rank: {}.",
                    self.state.score,
                    MAX_SCORE,
                    self.state.rank()
                ))
            }
            Verb::Save => return Reply::effect(Effect::Save),
            Verb::Restore => return Reply::effect(Effect::Restore),
            Verb::Restart => return Reply::effect(Effect::Restart),
            Verb::Quit => return Reply::effect(Effect::Quit),
            _ => {}
        }
        if let Some(r) = self.travel(cmd) {
            return r;
        }
        if let Some(r) = self.items(cmd) {
            return r;
        }
        let r = match self.state.scene {
            SceneId::Bar => bar::handle(self, cmd),
            SceneId::Boardwalk => boardwalk::handle(self, cmd),
            SceneId::Alley => alley::handle(self, cmd),
            SceneId::Store => store::handle(self, cmd),
            SceneId::Casino => casino::handle(self, cmd),
            SceneId::Rooftop => rooftop::handle(self, cmd),
        };
        if let Some(r) = r {
            return r;
        }
        if let Some(n) = cmd.nouns.iter().find(|n| !self.present(**n)) {
            return Reply::say(format!("You don't see any {} here.", world::noun_word(*n)));
        }
        self.fallback(cmd)
    }

    // ------------------------------------------------------------ travel

    /// The hero's feet are checked against the scene's exits after every
    /// step; returns a reply if an exit fired (or refused).
    pub fn check_exits(&mut self) -> Option<Reply> {
        let (x, y) = self.state.hero;
        let exit = world::exit_at(self.state.scene, x, y)?;
        Some(self.go_through(exit))
    }

    fn go_through(&mut self, exit: &Exit) -> Reply {
        let s = &mut self.state;
        if s.scene == SceneId::Casino && exit.to == SceneId::Rooftop && !s.flag(Flag::TinyApproved)
        {
            s.hero = exit.back;
            let msg = if s.flag(Flag::WearingCarnation) {
                "Tiny steps in front of the elevator, but he's eyeing your carnation with interest. \"Talk to me first, friend. I like to be asked.\""
            } else {
                "Tiny steps in front of the elevator. It's like a solar eclipse wearing a tuxedo. \"Crow's Nest has a dress code, friend.\""
            };
            return Reply::say(msg).with(Effect::Moved);
        }
        s.scene = exit.to;
        s.hero = exit.arrive;
        let mut messages = vec![];
        let visited = exit.to.visited_flag();
        if !s.flag(visited) {
            s.set(visited);
            messages.push(scene_intro(exit.to).to_string());
        }
        Reply {
            messages,
            effect: Effect::Moved,
        }
    }

    fn travel(&mut self, cmd: &Command) -> Option<Reply> {
        let ride = matches!(cmd.verb, Verb::Take | Verb::Use)
            && cmd.noun() == Some(Noun::Elevator)
            && cmd.second().is_none();
        if !matches!(cmd.verb, Verb::Go | Verb::Open) && !ride {
            return None;
        }
        let exits = world::exits(self.state.scene);
        match cmd.noun() {
            None if cmd.verb == Verb::Go => {
                if exits.len() == 1 {
                    Some(self.go_through(&exits[0]))
                } else {
                    Some(Reply::say("Go where? Walk off the edge of the screen or through a door, or name a place: GO TO CASINO, ENTER STORE..."))
                }
            }
            Some(n) => {
                if let Some(e) = exits.iter().find(|e| e.nouns.contains(&n)) {
                    return Some(self.go_through(e));
                }
                if n == place_noun(self.state.scene) && n != Noun::Bar {
                    return Some(Reply::say("You're already here. Chip's sense of direction is famously bad, but not THAT bad."));
                }
                if PLACES.contains(&n) && !(n == Noun::Bar && self.state.scene == SceneId::Bar) {
                    return Some(Reply::say(
                        "You can't get there from here. Not directly, anyway.",
                    ));
                }
                None
            }
            None => None,
        }
    }

    // ------------------------------------------------------------ items

    fn inventory(&self) -> Reply {
        let s = &self.state;
        if s.inventory.is_empty() && s.cash == 0 {
            return Reply::say("You are carrying nothing but charm. Mostly nothing.");
        }
        let mut out = String::from("You are carrying:\n");
        for i in &s.inventory {
            out.push_str("  ");
            out.push_str(i.name());
            if *i == Item::Carnation && s.flag(Flag::WearingCarnation) {
                out.push_str(" (worn)");
            }
            out.push('\n');
        }
        if s.cash > 0 {
            out.push_str(&format!("  ${} in damp bills\n", s.cash));
        }
        Reply::say(out.trim_end().to_string())
    }

    /// Things you can do with inventory anywhere.
    fn items(&mut self, cmd: &Command) -> Option<Reply> {
        use Noun as N;
        use Verb as V;
        let s = &mut self.state;
        fn owned_by(s: &GameState, n: Noun) -> bool {
            item_for(n).is_some_and(|i| s.has(i))
        }
        let n = cmd.noun();
        match (cmd.verb, n) {
            (V::Look, Some(x)) if owned_by(s, x) || (x == N::Money && s.cash > 0) => say(match x {
                N::Quarter => "Your lucky quarter. Heads, you win. Tails, you also win, because you've glued two quarters together.",
                N::Comb => "A pocket comb, its teeth worn down by years of maintaining the most aerodynamic pompadour on the coast. Thin, strong, and surprisingly versatile.",
                N::Matchbook => "A red matchbook with \"THE RUSTY ANCHOR\" in gold. Something is scribbled inside the flap. You could READ it.",
                N::Drink => "The Bilge Water Special: something green, something blue, and a paper umbrella that has given up.",
                N::HotDog => "A hot dog with everything, including some things you can't identify. It's still warm. It's always going to be warm. That's the scary part.",
                N::Carnation if s.flag(Flag::WearingCarnation) => "A red carnation, pinned to your powder-blue lapel. Bold. Daring. Slightly wilted.",
                N::Carnation => "A single red carnation, slightly wilted, like a lot of things at this hour.",
                N::Mints => "Arctic Blast breath mints: \"So strong they're technically a weather event.\"",
                _ => "Five damp dollars, recently rescued from a storm drain. They smell like freedom. And drain.",
            }),
            (V::Read, Some(N::Matchbook)) | (V::Read, None) | (V::Open, Some(N::Matchbook)) if s.has(Item::Matchbook) => {
                let pts = s.award(Flag::ReadMatchbook, 3);
                scored(Reply::say("Inside the flap, in smeared lipstick: \"Ask Gus for THE SPECIAL. Tell him Vonda sent you. Then come find me at the Crow's Nest, on the roof of the Golden Clam. I'll be the one in sequins.\" Chip's heart does a little cha-cha."), pts)
            }
            (V::Read, Some(N::Matchbook)) => say("You'd have to pick it up first. Chip's eyes are dreamy, not telescopic."),
            (V::Drink, Some(N::Drink)) | (V::Drink, None) | (V::Use, Some(N::Drink)) if s.has(Item::Drink) => {
                s.remove(Item::Drink);
                let pts = s.award(Flag::DrankDrink, 2);
                scored(Reply::say("You knock back the Bilge Water Special. It tastes like coconut, regret, and a hint of outboard motor. Your confidence swells to dangerous levels. So, unfortunately, does your breath."), pts)
            }
            (V::Eat, Some(N::Mints)) | (V::Use, Some(N::Mints)) if s.has(Item::Mints) => {
                s.remove(Item::Mints);
                let pts = s.award(Flag::AteMints, 2);
                scored(Reply::say("You pop an Arctic Blast mint. Your sinuses clear, your eyes water, and somewhere a polar bear shivers. Your breath is now legally a cold front."), pts)
            }
            (V::Wear, Some(N::Carnation)) | (V::Use, Some(N::Carnation)) if s.has(Item::Carnation) && cmd.second().is_none_or(|x| x == N::Me) => {
                if s.flag(Flag::WearingCarnation) {
                    return say("It's already on your lapel, blooming with confidence. Or at least not wilting any faster.");
                }
                let pts = s.award(Flag::WearingCarnation, 2);
                scored(Reply::say("You pin the red carnation to your lapel. Red on powder blue: a bold choice. A Chip Valentine choice."), pts)
            }
            (V::Wear, _) => say("That's not exactly evening wear."),
            (V::Eat, Some(N::HotDog)) if s.has(Item::HotDog) => say("Tempting, but this hot dog feels destined for someone hungrier than you. Someone who hasn't eaten since Tuesday, say."),
            (V::Use, Some(N::Comb)) if cmd.second().is_none_or(|x| x == N::Me) => say("You run the comb through your pompadour. It reaches maximum altitude. Air traffic control is notified."),
            (V::Drop, Some(x)) if owned_by(s, x) || x == N::Money => say("Drop it? Here? Somebody would have it pawned before it hit the ground."),
            (V::Take, Some(x)) if owned_by(s, x) => say("You already have that."),
            (V::Use, Some(N::Matchbook)) if s.has(Item::Matchbook) => say("Strike a match? With your hairspray? You'd be the first tuxedo in orbit."),
            _ => None,
        }
    }

    pub(crate) fn die(&mut self, kind: DeathKind, before: Vec<&str>) -> Option<Reply> {
        self.state.dead = true;
        Some(Reply::many(before).with(Effect::Died(death(kind))))
    }

    fn fallback(&self, cmd: &Command) -> Reply {
        let has_noun = cmd.noun().is_some();
        Reply::say(match cmd.verb {
            Verb::Look => "You see nothing special. Chip's eye is trained for romance, not detail.",
            Verb::Read => "There's nothing to read there. Chip is relieved; reading is how wrinkles start.",
            Verb::Take if has_noun => "You can't take that. Not with the whole town watching, anyway.",
            Verb::Take => "Take what? A hint? A chance? Be specific.",
            Verb::Drop => "You're not holding that.",
            Verb::Talk if has_noun => "It doesn't answer. You've had chattier dates, but not many.",
            Verb::Talk => "Talking to yourself again? At least you're a good listener.",
            Verb::Use | Verb::Give | Verb::Play => "Chip considers this for a moment, then reconsiders. Wisely.",
            Verb::Open | Verb::Close => "That doesn't open. Neither do most hearts in this town.",
            Verb::Eat => "You're saving your appetite. For love. And maybe nachos.",
            Verb::Drink => "You don't have anything to drink, and the puddles don't count, no matter how long the night gets.",
            Verb::Order => "Nobody here is selling anything. At least, nothing legal.",
            Verb::Sit => "You'd rather not sit on that. Your tux is rented. Well, \"rented.\"",
            Verb::Dance => "Dance? Without music? Chip has standards. Low ones, but standards.",
            Verb::Flirt => "There's nobody here worth the smolder. You save it for later.",
            Verb::Smell => "It smells like the ocean, fried dough, and broken dreams. Ocean Bluffs' signature scent.",
            Verb::Go => "Go where? Walk off the edge of the screen or through a door.",
            Verb::Climb => "Chip's tux doesn't stretch that way. Neither does Chip.",
            Verb::Jump => "You hop in place. Nobody is impressed. Not even you.",
            Verb::Wear => "That's not exactly evening wear.",
            Verb::Sing => "You croon a few bars. Somewhere a dog howls along. In harmony, at least.",
            Verb::Search => "You find nothing but lint and regret.",
            Verb::Inventory | Verb::Score | Verb::Help | Verb::Save | Verb::Restore | Verb::Restart | Verb::Quit => "",
        })
    }
}

/// First-visit description for a scene.
pub fn scene_intro(scene: SceneId) -> &'static str {
    match scene {
        SceneId::Bar => INTRO,
        SceneId::Boardwalk => boardwalk::INTRO,
        SceneId::Alley => alley::INTRO,
        SceneId::Store => store::INTRO,
        SceneId::Casino => casino::INTRO,
        SceneId::Rooftop => rooftop::INTRO,
    }
}
