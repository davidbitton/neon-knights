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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub verb: Verb,
    pub nouns: Vec<Noun>,
}

impl Command {
    pub fn noun(&self) -> Option<Noun> {
        self.nouns.first().copied()
    }
    pub fn second(&self) -> Option<Noun> {
        self.nouns.get(1).copied()
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
];

pub fn verb_for(word: &str) -> Option<Verb> {
    use Verb::*;
    Some(match word {
        "look" | "l" | "examine" | "x" | "inspect" | "check" | "view" | "see" | "watch"
        | "observe" | "describe" | "ogle" => Look,
        "read" => Read,
        "take" | "get" | "grab" | "pick" | "steal" | "pocket" | "snag" | "nab" => Take,
        "drop" | "discard" | "toss" => Drop,
        "talk" | "speak" | "chat" | "ask" | "greet" | "hello" | "hi" | "hey" | "say" | "tell" => {
            Talk
        }
        "use" | "insert" | "put" | "push" | "press" => Use,
        "open" | "unlock" | "enter" => Open,
        "close" | "shut" => Close,
        "eat" | "taste" | "bite" | "swallow" | "devour" | "nibble" => Eat,
        "drink" | "sip" | "chug" | "gulp" | "quaff" => Drink,
        "order" | "buy" | "purchase" => Order,
        "give" | "hand" | "offer" | "pay" | "tip" => Give,
        "play" | "start" => Play,
        "sit" => Sit,
        "dance" | "boogie" | "groove" => Dance,
        "flirt" | "wink" | "smolder" | "woo" | "kiss" => Flirt,
        "smell" | "sniff" => Smell,
        "walk" | "go" | "leave" | "exit" | "depart" | "run" => Go,
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
        "room" | "bar's" | "place" | "joint" | "dive" | "anchor" | "rusty" => Room,
        "bartender" | "gus" | "barkeep" | "barman" | "barkeeper" | "man" | "guy" => Bartender,
        "jukebox" | "juke" | "music" | "song" | "songs" | "record" | "records" | "box" => Jukebox,
        "matchbook" | "matches" | "match" | "matchbox" | "book" | "number" | "phone" => Matchbook,
        "quarter" | "coin" | "money" | "change" | "cash" | "25" => Quarter,
        "comb" | "hair" | "pompadour" => Comb,
        "egg" | "eggs" | "jar" | "pickled" => Egg,
        "drink" | "drinks" | "beer" | "booze" | "cocktail" | "special" | "bilge" | "water"
        | "glass" | "liquor" => Drink,
        "bar" | "counter" | "bartop" => Bar,
        "stool" | "stools" | "seat" | "barstool" => Stool,
        "door" | "street" | "outside" | "boardwalk" => Door,
        "restroom" | "bathroom" | "toilet" | "loo" | "wc" | "lavatory" | "mens" | "john" => {
            Restroom
        }
        "sign" | "neon" | "light" | "lights" => Sign,
        "bottles" | "bottle" | "shelf" | "shelves" | "mirror" => Bottles,
        "patron" | "drunk" | "regular" | "sailor" | "barfly" | "captain" | "customer" | "hat" => {
            Patron
        }
        "table" => Table,
        "floor" | "puddle" | "ground" => Floor,
        "me" | "self" | "myself" | "chip" | "tux" | "tuxedo" | "suit" | "clothes" => Me,
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
