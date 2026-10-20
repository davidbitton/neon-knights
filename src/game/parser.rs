//! A small verb-noun text parser in the spirit of the late-'80s parser
//! adventures: lowercase, strip punctuation, drop filler words, map synonyms.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Verb {
    Look,
    Read,
    Take,
    Drop,
    Talk,
    Use,
    Open,
    Close,
    Eat,
    Drink,
    Order,
    Give,
    Play,
    Sit,
    Dance,
    Flirt,
    Smell,
    Go,
    Climb,
    Jump,
    Wear,
    Sing,
    Search,
    Inventory,
    Score,
    Help,
    Save,
    Restore,
    Restart,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Noun {
    Room,
    Bartender,
    Jukebox,
    Matchbook,
    Quarter,
    Comb,
    Egg,
    Drink,
    Bar,
    Stool,
    Door,
    Restroom,
    Sign,
    Bottles,
    Patron,
    Table,
    Floor,
    Me,
    // Generic words resolved per scene by the game logic.
    Man,
    Woman,
    Machine,
    // Places (used for typed travel: "go to the casino").
    Boardwalk,
    Alley,
    Store,
    Casino,
    Rooftop,
    // Boardwalk.
    Dot,
    Cart,
    HotDog,
    Telescope,
    Ocean,
    Railing,
    // Alley.
    Mime,
    Dumpster,
    Grate,
    FireEscape,
    Trash,
    // Store.
    Brenda,
    Carnation,
    Mints,
    Slushie,
    Register,
    // Casino.
    Tiny,
    Slots,
    Elevator,
    Chandelier,
    Plant,
    // Rooftop.
    Vonda,
    Piano,
    Pool,
    City,
    // Inventory.
    Money,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub verb: Verb,
    pub nouns: Vec<Noun>,
}

impl Command {
    pub fn new(verb: Verb, nouns: &[Noun]) -> Command {
        Command {
            verb,
            nouns: nouns.to_vec(),
        }
    }
    pub fn noun(&self) -> Option<Noun> {
        self.nouns.first().copied()
    }
    pub fn second(&self) -> Option<Noun> {
        self.nouns.get(1).copied()
    }
    pub fn mentions(&self, n: Noun) -> bool {
        self.nouns.contains(&n)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    UnknownWord(String),
    NoVerb(Noun),
}

/// Words that are silently skipped.
pub const FILLER: &[&str] = &[
    "the",
    "a",
    "an",
    "at",
    "to",
    "with",
    "on",
    "in",
    "into",
    "inside",
    "up",
    "down",
    "my",
    "your",
    "some",
    "please",
    "around",
    "of",
    "for",
    "from",
    "that",
    "this",
    "it",
    "him",
    "her",
    "about",
    "and",
    "then",
    "out",
    "over",
    "under",
    "behind",
    "can",
    "i'd",
    "id",
    "like",
    "would",
    "lets",
    "let's",
    "game",
    "quickly",
    "slowly",
    "carefully",
    "again",
    "one",
    "another",
    "here",
    "there",
    "now",
    "just",
    "all",
    "is",
    "am",
    "are",
    "be",
    "using",
    "by",
    "off",
    "through",
    "thru",
    "onto",
    "toward",
    "towards",
    "back",
    "nice",
    "something",
    "red",
    "big",
    "little",
    "old",
    "invisible",
    "lapel",
    "coin-op",
    "self",
    "very",
    "really",
    "storm",
];

pub fn verb_for(word: &str) -> Option<Verb> {
    use Verb::*;
    Some(match word {
        "look" | "l" | "examine" | "x" | "inspect" | "check" | "view" | "see" | "watch"
        | "observe" | "describe" | "ogle" | "peer" => Look,
        "read" => Read,
        "take" | "get" | "grab" | "pick" | "steal" | "pocket" | "snag" | "nab" => Take,
        "drop" | "discard" | "toss" => Drop,
        "talk" | "speak" | "chat" | "ask" | "greet" | "hello" | "hi" | "hey" | "say" | "tell" => {
            Talk
        }
        "use" | "insert" | "put" | "push" | "press" | "pull" | "yank" => Use,
        "open" | "unlock" | "enter" => Open,
        "close" | "shut" => Close,
        "eat" | "taste" | "bite" | "swallow" | "devour" | "nibble" | "chew" => Eat,
        "drink" | "sip" | "chug" | "gulp" | "quaff" | "slurp" => Drink,
        "order" | "buy" | "purchase" => Order,
        "give" | "hand" | "offer" | "pay" | "tip" | "feed" => Give,
        "play" | "start" | "gamble" => Play,
        "sit" => Sit,
        "dance" | "boogie" | "groove" => Dance,
        "flirt" | "wink" | "smolder" | "woo" | "kiss" | "compliment" | "praise" | "charm"
        | "flatter" | "sweettalk" | "sweet-talk" => Flirt,
        "smell" | "sniff" => Smell,
        "walk" | "go" | "leave" | "exit" | "depart" | "run" | "ride" => Go,
        "climb" | "scale" => Climb,
        "jump" | "dive" | "leap" | "swim" | "plunge" => Jump,
        "wear" | "don" | "pin" | "attach" => Wear,
        "sing" | "croon" | "serenade" | "perform" | "karaoke" => Sing,
        "search" | "fish" | "dig" | "rummage" | "reach" => Search,
        "inventory" | "inv" | "i" => Inventory,
        "score" | "points" => Score,
        "help" | "?" | "hint" | "hints" | "commands" => Help,
        "save" => Save,
        "restore" | "load" => Restore,
        "restart" => Restart,
        "quit" | "q" => Quit,
        _ => return None,
    })
}

pub fn noun_for(word: &str) -> Option<Noun> {
    use Noun::*;
    Some(match word {
        "room" | "place" | "joint" | "dive" | "around" => Room,
        "bartender" | "gus" | "barkeep" | "barman" | "barkeeper" => Bartender,
        "man" | "guy" | "dude" | "fellow" | "gentleman" => Man,
        "woman" | "lady" | "girl" | "gal" | "dame" => Woman,
        "machine" | "machines" => Machine,
        "jukebox" | "juke" | "music" | "song" | "songs" | "record" | "records" | "box" => Jukebox,
        "matchbook" | "matches" | "match" | "matchbox" | "book" | "number" | "phone" | "note" => {
            Matchbook
        }
        "quarter" | "coin" | "change" => Quarter,
        "money" | "cash" | "bill" | "fiver" | "dollars" | "dollar" | "bucks" | "buck" | "five" => {
            Money
        }
        "comb" | "hair" | "pompadour" => Comb,
        "egg" | "eggs" | "jar" | "pickled" => Egg,
        "drink" | "drinks" | "beer" | "booze" | "cocktail" | "special" | "bilge" | "glass"
        | "liquor" => Drink,
        "bar" | "counter" | "bartop" | "anchor" | "rusty" => Bar,
        "stool" | "stools" | "seat" | "barstool" => Stool,
        "door" | "doors" | "exit" | "entrance" => Door,
        "restroom" | "bathroom" | "toilet" | "loo" | "wc" | "lavatory" | "mens" | "john" => {
            Restroom
        }
        "sign" | "neon" | "light" | "lights" | "marquee" => Sign,
        "bottles" | "bottle" | "shelf" | "shelves" | "mirror" | "aisle" | "products" => Bottles,
        "patron" | "drunk" | "regular" | "sailor" | "captain" | "customer" | "hat" => Patron,
        "table" => Table,
        "floor" | "puddle" | "ground" | "carpet" | "planks" | "deck" => Floor,
        "me" | "myself" | "chip" | "tux" | "tuxedo" | "suit" | "clothes" => Me,
        "boardwalk" | "street" | "outside" | "promenade" => Boardwalk,
        "alley" | "alleyway" => Alley,
        "store" | "shop" | "mart" | "market" | "gull" | "lucky" => Store,
        "casino" | "clam" | "golden" | "lobby" => Casino,
        "roof" | "rooftop" | "lounge" | "nest" | "crow's" | "crows" | "upstairs" => Rooftop,
        "dot" | "vendor" | "skates" => Dot,
        "cart" | "stand" | "umbrella" => Cart,
        "hotdog" | "hot" | "dog" | "frank" | "wiener" | "sausage" => HotDog,
        "telescope" | "scope" | "binoculars" | "viewer" | "eyepiece" => Telescope,
        "ocean" | "sea" | "beach" | "waves" | "surf" | "pier" => Ocean,
        "railing" | "rail" | "edge" | "ledge" => Railing,
        "mime" | "gerard" | "performer" => Mime,
        "dumpster" | "bin" => Dumpster,
        "grate" | "drain" | "sewer" | "grating" | "bars" => Grate,
        "fire" | "escape" | "ladder" | "stairs" => FireEscape,
        "trash" | "garbage" | "bags" | "bag" | "cans" => Trash,
        "brenda" | "clerk" | "cashier" | "teen" | "teenager" => Brenda,
        "carnation" | "carnations" | "flower" | "flowers" | "rose" | "roses" | "bucket" => {
            Carnation
        }
        "mints" | "mint" | "breath" | "tin" => Mints,
        "slushie" | "slushy" | "slush" | "slushies" => Slushie,
        "register" | "till" | "magazine" => Register,
        "tiny" | "doorman" | "bouncer" | "giant" => Tiny,
        "slot" | "slots" | "lever" | "jackpot" | "bandit" => Slots,
        "elevator" | "lift" => Elevator,
        "chandelier" | "crystal" => Chandelier,
        "plant" | "palm" | "fern" | "tree" => Plant,
        "vonda" | "singer" | "diva" | "sweetheart" => Vonda,
        "piano" | "keys" | "pianist" => Piano,
        "pool" | "swimming" => Pool,
        "city" | "skyline" | "view" | "stars" | "sky" | "moon" => City,
        _ => return None,
    })
}

/// Normalize a word: lowercase, keep letters/digits/apostrophes/'?'.
fn normalize(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_alphanumeric() || *c == '\'' || *c == '?')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Parse a typed line into a [`Command`].
///
/// The first verb word wins; subsequent words that could be either verb or
/// noun (e.g. "drink") are treated as nouns. Duplicate adjacent nouns
/// ("pickled egg") collapse into one.
pub fn parse(input: &str) -> Result<Command, ParseError> {
    let mut verb: Option<Verb> = None;
    let mut nouns: Vec<Noun> = Vec::new();
    let mut any = false;
    for raw in input.split(|c: char| c.is_whitespace() || c == ',' || c == '.' || c == '!') {
        let w = normalize(raw);
        if w.is_empty() {
            continue;
        }
        any = true;
        if FILLER.contains(&w.as_str()) {
            continue;
        }
        if verb.is_none() {
            if let Some(v) = verb_for(&w) {
                verb = Some(v);
                continue;
            }
        }
        if let Some(n) = noun_for(&w) {
            if nouns.last() != Some(&n) {
                nouns.push(n);
            }
            continue;
        }
        if verb_for(&w).is_some() {
            // A second verb ("look and take"): ignore it rather than fail.
            continue;
        }
        return Err(ParseError::UnknownWord(w));
    }
    if !any {
        return Err(ParseError::Empty);
    }
    match verb {
        Some(v) => Ok(Command { verb: v, nouns }),
        None => match nouns.first() {
            Some(n) => Err(ParseError::NoVerb(*n)),
            // Only filler words ("the the the").
            None => Err(ParseError::Empty),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(v: Verb, n: &[Noun]) -> Result<Command, ParseError> {
        Ok(Command {
            verb: v,
            nouns: n.to_vec(),
        })
    }

    #[test]
    fn basic_verb_noun() {
        assert_eq!(parse("look bartender"), cmd(Verb::Look, &[Noun::Bartender]));
        assert_eq!(parse("take matchbook"), cmd(Verb::Take, &[Noun::Matchbook]));
    }

    #[test]
    fn synonyms() {
        assert_eq!(parse("examine gus"), parse("l barkeep"));
        assert_eq!(parse("x jukebox"), cmd(Verb::Look, &[Noun::Jukebox]));
        assert_eq!(parse("grab matches"), cmd(Verb::Take, &[Noun::Matchbook]));
        assert_eq!(parse("get coin"), cmd(Verb::Take, &[Noun::Quarter]));
        assert_eq!(parse("get the money"), cmd(Verb::Take, &[Noun::Money]));
        assert_eq!(parse("i"), cmd(Verb::Inventory, &[]));
        assert_eq!(parse("inv"), cmd(Verb::Inventory, &[]));
        assert_eq!(parse("load"), cmd(Verb::Restore, &[]));
    }

    #[test]
    fn filler_words_are_ignored() {
        assert_eq!(
            parse("look at the bartender"),
            cmd(Verb::Look, &[Noun::Bartender])
        );
        assert_eq!(
            parse("please pick up the matchbook"),
            cmd(Verb::Take, &[Noun::Matchbook])
        );
        assert_eq!(parse("talk to gus"), cmd(Verb::Talk, &[Noun::Bartender]));
        assert_eq!(parse("look around"), cmd(Verb::Look, &[]));
    }

    #[test]
    fn case_and_punctuation() {
        assert_eq!(
            parse("  LOOK   At THE Bartender!!"),
            cmd(Verb::Look, &[Noun::Bartender])
        );
        assert_eq!(parse("look, gus."), cmd(Verb::Look, &[Noun::Bartender]));
    }

    #[test]
    fn two_nouns() {
        assert_eq!(
            parse("use the quarter on the jukebox"),
            cmd(Verb::Use, &[Noun::Quarter, Noun::Jukebox])
        );
        assert_eq!(
            parse("put coin in juke box"),
            cmd(Verb::Use, &[Noun::Quarter, Noun::Jukebox])
        );
        assert_eq!(
            parse("ask gus about the special"),
            cmd(Verb::Talk, &[Noun::Bartender, Noun::Drink])
        );
    }

    #[test]
    fn verb_noun_ambiguity() {
        assert_eq!(parse("drink"), cmd(Verb::Drink, &[]));
        assert_eq!(parse("drink the drink"), cmd(Verb::Drink, &[Noun::Drink]));
        assert_eq!(parse("order a drink"), cmd(Verb::Order, &[Noun::Drink]));
        assert_eq!(parse("eat pickled egg"), cmd(Verb::Eat, &[Noun::Egg]));
    }

    #[test]
    fn milestone2_vocabulary() {
        assert_eq!(
            parse("give the hot dog to the mime"),
            cmd(Verb::Give, &[Noun::HotDog, Noun::Mime])
        );
        assert_eq!(parse("feed mime"), cmd(Verb::Give, &[Noun::Mime]));
        assert_eq!(
            parse("use comb on the storm drain"),
            cmd(Verb::Use, &[Noun::Comb, Noun::Grate])
        );
        assert_eq!(
            parse("use comb on drain"),
            cmd(Verb::Use, &[Noun::Comb, Noun::Grate])
        );
        assert_eq!(
            parse("buy a red carnation"),
            cmd(Verb::Order, &[Noun::Carnation])
        );
        assert_eq!(
            parse("wear the carnation"),
            cmd(Verb::Wear, &[Noun::Carnation])
        );
        assert_eq!(
            parse("pin flower on lapel"),
            cmd(Verb::Wear, &[Noun::Carnation])
        );
        assert_eq!(
            parse("look through the telescope"),
            cmd(Verb::Look, &[Noun::Telescope])
        );
        assert_eq!(parse("dive into the pool"), cmd(Verb::Jump, &[Noun::Pool]));
        assert_eq!(
            parse("climb into dumpster"),
            cmd(Verb::Climb, &[Noun::Dumpster])
        );
        assert_eq!(parse("go to the casino"), cmd(Verb::Go, &[Noun::Casino]));
        assert_eq!(parse("enter store"), cmd(Verb::Open, &[Noun::Store]));
        assert_eq!(parse("compliment dot"), cmd(Verb::Flirt, &[Noun::Dot]));
        assert_eq!(parse("sing with vonda"), cmd(Verb::Sing, &[Noun::Vonda]));
        assert_eq!(
            parse("play slot machine"),
            cmd(Verb::Play, &[Noun::Slots, Noun::Machine])
        );
        assert_eq!(parse("talk to the woman"), cmd(Verb::Talk, &[Noun::Woman]));
        assert_eq!(parse("ride the elevator"), cmd(Verb::Go, &[Noun::Elevator]));
    }

    #[test]
    fn errors() {
        assert_eq!(parse(""), Err(ParseError::Empty));
        assert_eq!(parse("   "), Err(ParseError::Empty));
        assert_eq!(parse("the the"), Err(ParseError::Empty));
        assert_eq!(parse("xyzzy"), Err(ParseError::UnknownWord("xyzzy".into())));
        assert_eq!(
            parse("look at xyzzy"),
            Err(ParseError::UnknownWord("xyzzy".into()))
        );
        assert_eq!(parse("bartender"), Err(ParseError::NoVerb(Noun::Bartender)));
    }
}
