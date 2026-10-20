//! Command handling and all of the game's writing.

use super::parser::{parse, Command, Noun, ParseError, Verb};
use super::scene::is_near;
use super::state::{Flag, GameState, Item, MAX_SCORE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Death {
    pub cause: &'static str,
    pub epitaph: Vec<&'static str>,
}

/// Side effects the platform layer must perform.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    Died(Death),
    Save,
    Restore,
    Restart,
    Quit,
    /// Score changed (the platform may play a ding).
    Scored(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub messages: Vec<String>,
    pub effect: Effect,
}

impl Reply {
    fn say(s: impl Into<String>) -> Reply {
        Reply { messages: vec![s.into()], effect: Effect::None }
    }
    fn many(v: Vec<String>) -> Reply {
        Reply { messages: v, effect: Effect::None }
    }
    fn effect(effect: Effect) -> Reply {
        Reply { messages: vec![], effect }
    }
    fn with(mut self, effect: Effect) -> Reply {
        self.effect = effect;
        self
    }
}

pub const INTRO: &str = "Ocean Bluffs, 11:42 PM. The tide is out, the rent is due, and Chip Valentine has squeezed into his lucky powder-blue tuxedo for one perfect night on the town.\n\nHis first stop: The Rusty Anchor, a bar so dingy the cockroaches wipe their feet on the way OUT.";

pub const HELP: &str = "Type commands like LOOK, LOOK AT BARTENDER, TAKE MATCHBOOK, TALK TO GUS, USE QUARTER ON JUKEBOX, INVENTORY (or I), SCORE, SAVE, RESTORE, RESTART.\n\nArrow keys walk (press again to stop). Click to walk. Click a thing to use the current verb on it; right-click (or Tab) changes the verb.";

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

pub struct Game {
    pub state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn pick<'a>(lines: &[&'a str], n: u32) -> &'a str {
    lines[n as usize % lines.len()]
}

fn too_far(s: &GameState) -> Reply {
    Reply::say(pick(&TOO_FAR, s.moves))
}

fn scored(mut r: Reply, pts: u32) -> Reply {
    if pts > 0 {
        r.effect = Effect::Scored(pts);
    }
    r
}

impl Game {
    pub fn new() -> Game {
        Game { state: GameState::new() }
    }

    pub fn score_line(&self) -> String {
        format!("Score: {} of {}", self.state.score, MAX_SCORE)
    }

    /// Handle one typed line.
    pub fn input(&mut self, line: &str) -> Reply {
        if self.state.dead {
            return Reply::say("You're dead. It's a whole thing. Restore, restart, or quit.");
        }
        match parse(line) {
            Ok(cmd) => {
                self.state.moves += 1;
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
                super::scene::noun_word(n)
            )),
        }
    }

    fn run(&mut self, cmd: &Command) -> Reply {
        use Noun as N;
        use Verb as V;
        let n = cmd.noun();
        match cmd.verb {
            V::Help => Reply::say(HELP),
            V::Inventory => self.inventory(),
            V::Score => Reply::say(format!(
                "Your score is {} of {}. Rank: {}.",
                self.state.score,
                MAX_SCORE,
                self.state.rank()
            )),
            V::Save => Reply::effect(Effect::Save),
            V::Restore => Reply::effect(Effect::Restore),
            V::Restart => Reply::effect(Effect::Restart),
            V::Quit => Reply::effect(Effect::Quit),
            V::Look => self.look(n),
            V::Read => match n {
                Some(N::Matchbook) | None if self.state.has(Item::Matchbook) => self.read_matchbook(),
                Some(N::Matchbook) => Reply::say("You'd have to pick it up first. Chip's eyes are dreamy, not telescopic."),
                Some(N::Sign) => self.look(Some(N::Sign)),
                Some(N::Restroom) => self.look(Some(N::Restroom)),
                _ => Reply::say("There's nothing to read there. Chip is relieved; reading is how wrinkles start."),
            },
            V::Take => self.take(n),
            V::Drop => match n {
                Some(N::Matchbook) | Some(N::Quarter) | Some(N::Comb) | Some(N::Drink) => Reply::say(
                    "Drop it? On THIS floor? It would be fused to the planks by morning.",
                ),
                _ => Reply::say("You're not holding that."),
            },
            V::Talk => match n {
                Some(N::Bartender) | None => self.talk_gus(cmd.second() == Some(N::Drink)),
                Some(N::Drink) => self.talk_gus(true),
                Some(N::Patron) => Reply::say("You clear your throat at the regular. Without waking, he mumbles, \"Not now, Doris, the tide's coming in,\" and goes back to snoring in perfect time with the ceiling fan."),
                Some(N::Jukebox) => Reply::say("You whisper sweet nothings to the jukebox. It says nothing sweet back. Typical."),
                Some(N::Me) => Reply::say("\"Looking good, Valentine.\" You agree with yourself. As usual."),
                Some(_) => Reply::say("It doesn't answer. You've had chattier dates, but not many."),
            },
            V::Order => self.order_drink(),
            V::Use | V::Play | V::Give => self.use_thing(cmd),
            V::Open | V::Go => match n {
                Some(N::Restroom) => Reply::say("The restroom door is duct-taped shut under a sign reading OUT OF ORDER. Someone has added in marker: \"SO IS THE CUSTOMER.\" You decide you can hold it."),
                Some(N::Door) | None => self.leave(),
                Some(N::Jukebox) => Reply::say("The jukebox is padlocked. Gus trusts no one, especially not the jukebox."),
                Some(N::Matchbook) if self.state.has(Item::Matchbook) => self.read_matchbook(),
                _ => Reply::say("That doesn't open. Neither do most people's hearts in this town."),
            },
            V::Close => Reply::say("It's as closed as it's going to get."),
            V::Eat => match n {
                Some(N::Egg) => self.eat_egg(),
                Some(N::Matchbook) => Reply::say("You are a gentleman, not a goat."),
                Some(N::Comb) => Reply::say("You're hungry, Chip, but not THAT hungry."),
                _ => Reply::say("You're saving your appetite. For love. And maybe nachos."),
            },
            V::Drink => match n {
                Some(N::Drink) | None if self.state.has(Item::Drink) => self.drink_special(),
                Some(N::Floor) => Reply::say("Absolutely not. Even the cockroaches won't."),
                _ => Reply::say("You don't have anything to drink. The puddle on the floor does not count, no matter how long the night gets."),
            },
            V::Sit => match n {
                Some(N::Stool) | Some(N::Bar) | None => {
                    if !is_near(&self.state, N::Stool) {
                        return too_far(&self.state);
                    }
                    self.state.set(Flag::SatOnStool);
                    Reply::say("You perch on a stool. It wobbles. You wobble back. You both agree never to speak of this, and you stand up again.")
                }
                Some(N::Table) => Reply::say("The table is taken by a sleeping regular. He's been there so long he's technically furniture."),
                _ => Reply::say("You'd rather not sit on that. Your tux is rented. Well, \"rented.\""),
            },
            V::Dance => {
                if self.state.flag(Flag::PlayedJukebox) {
                    Reply::say("Chip unleashes his signature move, \"The Undertow.\" It involves a lot of hips and not enough rhythm. Somewhere, a disco ball sheds a single tear.")
                } else {
                    Reply::say("Dance? Without music? Chip has standards. Low ones, but standards.")
                }
            }
            V::Flirt => match n {
                Some(N::Bartender) => Reply::say("You give Gus your best smolder. Gus gives you his best \"there's a baseball bat under this counter.\" A stalemate."),
                Some(N::Patron) => Reply::say("He's asleep. Even Chip has rules."),
                Some(N::Jukebox) => Reply::say("You lean on the jukebox and wink at it. It's the longest relationship you've had all year."),
                _ => Reply::say("There's nobody here to flirt with but Gus, a snoring sailor, and a jukebox. You've had worse Saturday nights. Not many."),
            },
            V::Smell => Reply::say("The Rusty Anchor smells like low tide, warm beer, and Chip's cologne, which he applies with a garden hose. The cologne is winning."),
        }
    }

    fn inventory(&self) -> Reply {
        if self.state.inventory.is_empty() {
            return Reply::say("You are carrying nothing but charm. Mostly nothing.");
        }
        let mut s = String::from("You are carrying:\n");
        for i in &self.state.inventory {
            s.push_str("  ");
            s.push_str(i.name());
            s.push('\n');
        }
        Reply::say(s.trim_end().to_string())
    }

    fn look(&self, n: Option<Noun>) -> Reply {
        use Noun as N;
        let s = &self.state;
        let text: String = match n {
            None | Some(N::Room) => "The Rusty Anchor: teal walls, sticky floors, and a neon anchor that buzzes like an angry bee. Gus the bartender holds court behind the bar. A jukebox glows in the corner, a regular snores at a table, and the front door leads back out to the boardwalk.".into(),
            Some(N::Bartender) => "Gus has an eyepatch, a striped sailor shirt, and a mustache that has seen things. He's polishing a glass. It's been the same glass since 1979.".into(),
            Some(N::Jukebox) => {
                if s.flag(Flag::PlayedJukebox) {
                    "The jukebox is lit up like a cruise ship and blasting a smooth saxophone number. It's the most romantic thing in the building, including you.".into()
                } else {
                    "A dented old jukebox. A hand-lettered card reads: \"25 CENTS. NO REFUNDS. NO POLKA.\" The coin slot looks hungry.".into()
                }
            }
            Some(N::Matchbook) => {
                if s.has(Item::Matchbook) {
                    "A red matchbook with \"THE RUSTY ANCHOR\" in gold. Something is scribbled inside the flap. You could READ it.".into()
                } else {
                    "A red matchbook sits on the bar. Something is scribbled on the inside flap. Finders keepers, as they say in polite society.".into()
                }
            }
            Some(N::Quarter) if s.has(Item::Quarter) => "Your lucky quarter. Heads, you win. Tails, you also win, because you've glued two quarters together.".into(),
            Some(N::Comb) => "A pocket comb, its teeth worn down by years of maintaining the most aerodynamic pompadour on the coast.".into(),
            Some(N::Drink) if s.has(Item::Drink) => "The Bilge Water Special: something green, something blue, and a paper umbrella that has given up.".into(),
            Some(N::Drink) => "Gus's \"menu\" is a chalkboard reading BEER and SPECIAL. Under SPECIAL, someone wrote \"ask nicely.\"".into(),
            Some(N::Egg) => "A big jar of pickled eggs, floating in a cloudy brine. The label says \"BEST BEFORE\" and the rest of the label has dissolved.".into(),
            Some(N::Bar) => "The bar is scarred mahogany and brass. Mostly scars. There's a beer tap, a jar of pickled eggs, and Gus.".into(),
            Some(N::Stool) => "Four vinyl bar stools in a fetching shade of ketchup. Each one wobbles in its own unique way.".into(),
            Some(N::Door) => "The front door, with a brass porthole. Through it, the boardwalk's neon lights are flickering with promise. Or short-circuiting.".into(),
            Some(N::Restroom) => "The restroom door is duct-taped shut with a sign: OUT OF ORDER. Someone added in marker: \"SO IS THE CUSTOMER.\"".into(),
            Some(N::Sign) => "A pink neon anchor, buzzing. Half the letters of \"RUSTY ANCHOR\" burned out years ago, which is why the locals call it \"The R   Y ANC  R.\"".into(),
            Some(N::Bottles) => "Rows of bottles behind the bar. Several of the labels are in languages that don't exist. You see yourself in the mirror behind them and, frankly, you're thrilled.".into(),
            Some(N::Patron) => "A regular in a captain's hat, face-down on the table and snoring. His hat says \"CAPTAIN.\" His breath says \"rum.\"".into(),
            Some(N::Table) => "A wobbly round table, currently serving as a pillow for the regular.".into(),
            Some(N::Floor) => "The floor is sticky in a way that suggests a history. There's a puddle you've decided not to think about, and peanut shells crunch underfoot.".into(),
            Some(N::Me) => "Chip Valentine: powder-blue tuxedo, ruffled shirt, an aggressive spray tan and a pompadour you could land a seaplane on. A man in his prime. Which prime is up for debate.".into(),
            Some(N::Quarter) => "You don't have that anymore. The jukebox thanks you for your service.".into(),
        };
        Reply::say(text)
    }

    fn take(&mut self, n: Option<Noun>) -> Reply {
        use Noun as N;
        match n {
            None => Reply::say("Take what? A hint? A chance? Be specific."),
            Some(N::Matchbook) => {
                if self.state.has(Item::Matchbook) {
                    return Reply::say("You already have it. One matchbook is plenty; Chip is a lover, not an arsonist.");
                }
                if !is_near(&self.state, N::Matchbook) {
                    return too_far(&self.state);
                }
                self.state.add(Item::Matchbook);
                let pts = self.state.award(Flag::TookMatchbook, 5);
                scored(
                    Reply::say("You casually palm the matchbook off the bar. Gus doesn't notice. Or he does, and simply doesn't care, which is the Rusty Anchor's motto."),
                    pts,
                )
            }
            Some(N::Egg) => {
                if !is_near(&self.state, N::Egg) {
                    return too_far(&self.state);
                }
                self.state.set(Flag::EggWarned);
                Reply::say("As your hand nears the jar, Gus growls: \"Those eggs came with the building, pal. I wouldn't.\" You pull your hand back. For now.")
            }
            Some(N::Drink) if !self.state.has(Item::Drink) => {
                Reply::say("There's no drink in front of you. Gus serves people who ask.")
            }
            Some(N::Quarter) | Some(N::Comb) | Some(N::Drink) => Reply::say("You already have that."),
            Some(N::Bartender) => Reply::say("Gus is not portable. Gus is barely mobile."),
            Some(N::Patron) => Reply::say("Leave the man his dignity. And his hat. Mostly the hat."),
            Some(N::Bottles) => Reply::say("You reach for the top shelf. Gus clears his throat in a way that sounds like a boat engine. You reconsider."),
            Some(N::Stool) => Reply::say("The stools are bolted down. Gus learned that lesson in 1983."),
            Some(N::Jukebox) | Some(N::Bar) | Some(N::Table) | Some(N::Door) | Some(N::Restroom) => {
                Reply::say("That won't fit in your tux. Nothing fits in your tux. That's the point of the tux.")
            }
            Some(N::Sign) => Reply::say("It's bolted to the wall and buzzing with enough voltage to curl your hair. Your hair is already perfect."),
            Some(N::Floor) => Reply::say("You'd need a chisel. And a tetanus shot."),
            Some(N::Room) | Some(N::Me) => Reply::say("Take yourself less seriously? Never."),
        }
    }

    fn read_matchbook(&mut self) -> Reply {
        let pts = self.state.award(Flag::ReadMatchbook, 3);
        scored(
            Reply::say("Inside the flap, in smeared lipstick: \"Ask Gus for THE SPECIAL. Tell him Vonda sent you.\" Below that, a phone number with too many sevens. Chip's heart does a little cha-cha."),
            pts,
        )
    }

    fn talk_gus(&mut self, about_special: bool) -> Reply {
        if !is_near(&self.state, Noun::Bartender) {
            return Reply::say("Gus can't hear you over the ice machine's death rattle. Step up to the bar.");
        }
        if about_special || self.state.flag(Flag::ReadMatchbook) {
            return self.order_drink();
        }
        self.state.gus_talks += 1;
        let lines = [
            "Gus: \"Evening. We got beer, we got the Special, and we got eggs. I only recommend one of those, and it ain't the eggs.\"",
            "Gus: \"Nice tux. Did you lose a bet, or win one?\"",
            "Gus: \"Look, pal, I don't do small talk. I do medium talk, and only on Tuesdays.\" It is not Tuesday.",
        ];
        Reply::say(pick(&lines, self.state.gus_talks - 1))
    }

    fn order_drink(&mut self) -> Reply {
        if !is_near(&self.state, Noun::Bartender) {
            return Reply::say("You wave at Gus from across the room. Gus waves back with a different number of fingers. Get closer.");
        }
        let s = &mut self.state;
        if s.flag(Flag::GotDrink) {
            return Reply::say("Gus: \"One Special per customer. Doctor's orders. Several doctors, actually.\"");
        }
        if !s.flag(Flag::ReadMatchbook) {
            return Reply::say("Gus: \"Beer's two bucks.\" You have a lucky quarter and a comb. Gus does not accept combs.");
        }
        if !s.flag(Flag::PlayedJukebox) {
            return Reply::say("You lean in: \"Vonda sent me. I'll have the Special.\" Gus squints. \"Vonda, huh? Tell you what: this place is quieter than a tomb with the lid on. Put some music on and the first one's on the house.\"");
        }
        s.add(Item::Drink);
        let pts = s.award(Flag::GotDrink, 5);
        scored(
            Reply::say("Gus nods along to the saxophone. \"Now THAT'S music.\" He slides you a Bilge Water Special in a glass shaped like a mermaid. \"On the house. Vonda's friends drink free, once.\""),
            pts,
        )
    }

    fn use_thing(&mut self, cmd: &Command) -> Reply {
        use Noun as N;
        let (a, b) = (cmd.noun(), cmd.second());
        let target = match (a, b) {
            (Some(N::Quarter), Some(t)) => Some(t),
            (Some(N::Jukebox), _) => Some(N::Jukebox),
            (Some(t), Some(N::Quarter)) => Some(t),
            _ => None,
        };
        let mentions_quarter = a == Some(N::Quarter) || b == Some(N::Quarter);
        if target == Some(N::Jukebox) {
            if cmd.verb == Verb::Give && !mentions_quarter {
                return Reply::say("The jukebox only accepts quarters. And compliments, but those don't make it play.");
            }
            if !self.state.has(Item::Quarter) {
                return if self.state.flag(Flag::PlayedJukebox) {
                    Reply::say("The jukebox is already playing. You've spent your fortune, all twenty-five cents of it.")
                } else {
                    Reply::say("The jukebox wants a quarter. You've got nothing but good intentions.")
                };
            }
            if !is_near(&self.state, N::Jukebox) {
                return too_far(&self.state);
            }
            self.state.remove(Item::Quarter);
            let pts = self.state.award(Flag::PlayedJukebox, 5);
            return scored(
                Reply::say("You flip your lucky quarter into the slot. The jukebox shudders, lights up like a cruise ship, and oozes a slow saxophone number that sounds like it was recorded in a hot tub. Chip does finger guns at no one. Behind the bar, Gus almost smiles."),
                pts,
            );
        }
        match (cmd.verb, a, b) {
            (_, Some(N::Comb), _) => Reply::say("You run the comb through your pompadour. It reaches maximum altitude. Air traffic control is notified."),
            (Verb::Give, Some(N::Quarter), Some(N::Bartender)) | (Verb::Give, Some(N::Bartender), Some(N::Quarter)) => Reply::say("Gus looks at your quarter. \"What is this, 1962?\" He pushes it back across the bar."),
            (_, Some(N::Matchbook), _) => Reply::say("Strike a match in here? With these fumes? You'd be the first tuxedo in orbit."),
            (_, Some(N::Drink), _) if self.state.has(Item::Drink) => self.drink_special(),
            (_, Some(N::Quarter), Some(N::Egg)) => Reply::say("The eggs are not for sale. They are not, legally speaking, food."),
            (_, Some(N::Quarter), None) => Reply::say("Use the quarter on what? It's lucky, not psychic."),
            (_, Some(N::Quarter), _) => Reply::say("That's not a coin slot, no matter how hard you squint."),
            (_, Some(N::Restroom), _) => self.run(&Command { verb: Verb::Open, nouns: vec![N::Restroom] }),
            (_, Some(N::Stool), _) => self.run(&Command { verb: Verb::Sit, nouns: vec![N::Stool] }),
            (_, Some(N::Door), _) => self.leave(),
            (Verb::Play, None, _) => Reply::say("Play what? Chip plays the field, mostly. Badly."),
            _ => Reply::say("Chip considers this for a moment, then reconsiders. Wisely."),
        }
    }

    fn drink_special(&mut self) -> Reply {
        self.state.remove(Item::Drink);
        let pts = self.state.award(Flag::DrankDrink, 2);
        scored(
            Reply::say("You knock back the Bilge Water Special. It tastes like coconut, regret, and a hint of outboard motor. Your confidence swells to dangerous levels. You feel ready for anything. Or at least for the boardwalk."),
            pts,
        )
    }

    fn eat_egg(&mut self) -> Reply {
        if !is_near(&self.state, Noun::Egg) {
            return too_far(&self.state);
        }
        if !self.state.flag(Flag::EggWarned) {
            self.state.set(Flag::EggWarned);
            return Reply::say("Gus slaps the lid on the jar. \"Pal. Those eggs came with the building. The LAST guy who ate one is why we have the plaque.\" You decide to think about it.");
        }
        self.state.dead = true;
        Reply::many(vec![
            "Ignoring every instinct you have, plus a few you borrowed, you fish out a pickled egg and pop it in your mouth.".into(),
            "It's crunchy. Eggs should not be crunchy. Your eyes water, your knees buckle, and your pompadour finally, tragically, deflates.".into(),
        ])
        .with(Effect::Died(Death {
            cause: "Death by pickled egg",
            epitaph: vec![
                "HERE LIES CHIP VALENTINE",
                "He came for a night out.",
                "He got a night out cold.",
                "",
                "Survived by one pickled egg,",
                "still at large.",
            ],
        }))
    }

    fn leave(&mut self) -> Reply {
        if self.state.score >= MAX_SCORE {
            Reply::say("Chip straightens his bow tie, checks his pompadour in the porthole, and swaggers toward the boardwalk... TO BE CONTINUED in the next milestone! You've wrung every last point out of the Rusty Anchor.")
        } else {
            Reply::say("Leave already? The night is young, and so is Chip, comparatively speaking. Something tells you the Rusty Anchor isn't done with you yet. (The boardwalk opens in a future build.)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::scene::hotspot_for;

    fn go(g: &mut Game, noun: Noun) {
        g.state.hero = hotspot_for(&g.state, noun).unwrap().approach;
    }

    #[test]
    fn unknown_words_get_snark() {
        let mut g = Game::new();
        let r = g.input("frobnicate the bartender");
        assert_eq!(r.messages.len(), 1);
        assert!(r.messages[0].contains("frobnicate"));
        assert_eq!(r.effect, Effect::None);
    }

    #[test]
    fn look_works_from_anywhere() {
        let mut g = Game::new();
        let r = g.input("look at bartender");
        assert!(r.messages[0].contains("eyepatch"));
        let r = g.input("l");
        assert!(r.messages[0].contains("Rusty Anchor"));
    }

    #[test]
    fn taking_requires_proximity_and_scores_once() {
        let mut g = Game::new();
        let r = g.input("take matchbook");
        assert!(TOO_FAR.contains(&r.messages[0].as_str()));
        assert!(!g.state.has(Item::Matchbook));
        go(&mut g, Noun::Matchbook);
        let r = g.input("get the matches");
        assert_eq!(r.effect, Effect::Scored(5));
        assert!(g.state.has(Item::Matchbook));
        assert_eq!(g.state.score, 5);
        let r = g.input("take matchbook");
        assert_eq!(r.effect, Effect::None);
        assert_eq!(g.state.score, 5);
        assert!(g.input("i").messages[0].contains("matchbook"));
    }

    #[test]
    fn full_walkthrough_reaches_max_score() {
        let mut g = Game::new();
        go(&mut g, Noun::Matchbook);
        g.input("take matchbook");
        assert_eq!(g.input("read matchbook").effect, Effect::Scored(3));
        go(&mut g, Noun::Bartender);
        // Not yet: needs music.
        let r = g.input("order the special");
        assert!(r.messages[0].contains("music"));
        assert!(!g.state.has(Item::Drink));
        go(&mut g, Noun::Jukebox);
        assert_eq!(g.input("use quarter on jukebox").effect, Effect::Scored(5));
        assert!(!g.state.has(Item::Quarter));
        go(&mut g, Noun::Bartender);
        assert_eq!(g.input("ask gus about the special").effect, Effect::Scored(5));
        assert!(g.state.has(Item::Drink));
        assert_eq!(g.input("drink it").effect, Effect::Scored(2));
        assert_eq!(g.state.score, MAX_SCORE);
        assert!(g.input("score").messages[0].contains("20 of 20"));
        assert!(g.input("leave").messages[0].contains("TO BE CONTINUED"));
    }

    #[test]
    fn jukebox_needs_quarter() {
        let mut g = Game::new();
        go(&mut g, Noun::Jukebox);
        g.state.remove(Item::Quarter);
        let r = g.input("play jukebox");
        assert!(r.messages[0].contains("quarter"));
        assert!(!g.state.flag(Flag::PlayedJukebox));
    }

    #[test]
    fn egg_death_after_warning() {
        let mut g = Game::new();
        go(&mut g, Noun::Egg);
        let r = g.input("eat egg");
        assert_eq!(r.effect, Effect::None);
        assert!(!g.state.dead);
        let r = g.input("eat the pickled egg");
        assert!(matches!(r.effect, Effect::Died(_)));
        assert!(g.state.dead);
        // Dead men type no commands.
        assert!(g.input("look").messages[0].contains("dead"));
    }

    #[test]
    fn meta_commands_produce_effects() {
        let mut g = Game::new();
        assert_eq!(g.input("save").effect, Effect::Save);
        assert_eq!(g.input("restore game").effect, Effect::Restore);
        assert_eq!(g.input("restart").effect, Effect::Restart);
        assert_eq!(g.input("quit").effect, Effect::Quit);
        assert!(g.input("help").messages[0].contains("INVENTORY"));
        assert!(g.input("inventory").messages[0].contains("lucky quarter"));
    }

    #[test]
    fn noun_without_verb_and_empty() {
        let mut g = Game::new();
        assert!(g.input("jukebox").messages[0].contains("verb"));
        assert!(!g.input("").messages[0].is_empty());
    }

    #[test]
    fn talking_far_away_is_refused() {
        let mut g = Game::new();
        assert!(g.input("talk to gus").messages[0].contains("Step up"));
        go(&mut g, Noun::Bartender);
        assert!(g.input("talk to gus").messages[0].starts_with("Gus:"));
    }
}
