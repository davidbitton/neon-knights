//! The boardwalk: Dot's hot dogs and a coin-op telescope.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{Flag, Item, Npc};

pub const INTRO: &str = "You step out onto the Ocean Bluffs boardwalk. Salt air, flickering neon, and planks that have survived a million flip-flops. The Lucky Gull 24/7 hums next door, the Golden Clam Casino glitters at the far end, and an alley yawns off to the west.";

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Boardwalk)) => say("The Ocean Bluffs boardwalk. To the west, the Rusty Anchor; in the middle, the Lucky Gull 24/7 mart; to the east, the Golden Clam Casino, glittering like a disco ball that won the lottery. Dot's hot dog cart sits by the railing, there's a coin-op telescope at the end, and an alley runs off to the west."),
        (V::Look, Some(N::Dot)) => say("Dot is somewhere between sixty and immortal. She's on roller skates, in a paper hat, with the forearms of a woman who has flipped forty years of hot dogs."),
        (V::Look, Some(N::Cart)) => say("A hot dog cart with a striped umbrella and a painted wiener on the side that's smiling way too hard. The sign says HOT DOGS $2."),
        (V::Look | V::Use, Some(N::Telescope)) => telescope(g),
        (V::Look, Some(N::Bar)) => say("The Rusty Anchor, from outside. It looks worse from out here, which you didn't think was possible."),
        (V::Look, Some(N::Store)) => say("The Lucky Gull 24/7. Its fluorescent lights hum a single, endless note of despair. A sign in the window: FLOWERS - MINTS - SLUSHIES."),
        (V::Look, Some(N::Casino)) => say("The Golden Clam Casino: a giant gold clam on the roof, a red carpet, and a sign promising LOOSE SLOTS. Up top, the lights of the Crow's Nest lounge twinkle."),
        (V::Look, Some(N::Ocean)) => say("The Atlantic, sloshing against the pilings. Somewhere out there, a seagull is plotting against you."),
        (V::Look, Some(N::City)) => say("The moon hangs over the casino like it's waiting to be let in, too."),
        (V::Look, Some(N::Alley)) => say("An alley opens off the west end of the boardwalk. It smells like adventure, or possibly fish."),
        (V::Look, Some(N::Floor)) => say("Weathered planks, a few loose nails, and a fossilized wad of gum shaped like Florida."),
        (V::Talk, Some(N::Dot) | None) => talk_dot(g),
        (V::Flirt, Some(N::Dot) | None) => flirt_dot(g),
        (V::Order, None | Some(N::HotDog) | Some(N::Cart) | Some(N::Dot)) => {
            if g.state.flag(Flag::GotHotDog) {
                say("Dot: \"One freebie per customer, Casanova. I'm sweet, not stupid.\"")
            } else {
                say("Dot: \"Two bucks. Or free, for anybody who says something nice to me. Go on. I'll wait. I've been waiting since the Bicentennial.\"")
            }
        }
        (V::Take, Some(N::HotDog) | Some(N::Cart)) => say("Dot swats your hand with a spatula. \"Ask nicely, sugar. Nicer than that.\""),
        (V::Give, _) if cmd.mentions(N::Dot) => say("Dot: \"Keep it, hon. Just say something sweet.\""),
        (V::Eat, Some(N::HotDog)) => say("You'd have to get one first. Dot's the hot dog lady."),
        (V::Jump | V::Climb, Some(N::Ocean) | Some(N::Railing)) => say("The water under the boardwalk is two feet deep and full of shopping carts. Chip values his tux."),
        (V::Open, Some(N::Door)) => say("Which door? The Rusty Anchor, the Lucky Gull, or the Golden Clam? Try ENTER BAR, ENTER STORE, or ENTER CASINO. Or just walk in."),
        (V::Smell, _) => say("Fried onions, sea salt, and sunscreen from 1979."),
        (V::Sit, _) => say("You sit on the railing for a moment, letting the sea breeze ruin your hair, then get up and fix it."),
        (V::Dance, _) => say("You moonwalk a few planks. A seagull follows you, judging."),
        _ => None,
    }
}

fn telescope(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Telescope) {
        return say("A coin-op telescope stands at the end of the railing. You'd have to walk over to look through it.");
    }
    if g.state.flag(Flag::UsedTelescope) {
        return say("You peek again. The lady in sequins is still singing on the casino roof. You can't hear her, but your heart can.");
    }
    let pts = g.state.award(Flag::UsedTelescope, 2);
    scored(Reply::say("You put your eye to the coin-op telescope. It's stuck in the \"free\" position, the only free thing in town. You swing it up to the casino roof: under a string of party lights, a woman in pink sequins is singing at a grand piano. Vonda? Your heart does a little cha-cha."), pts)
}

fn talk_dot(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Dot) {
        return say("Dot can't hear you over the surf. Skate on over. Or walk, if you must.");
    }
    let s = &mut g.state;
    let n = s.talk(Npc::Dot);
    if s.flag(Flag::GotHotDog) {
        return say(pick(
            &[
                "Dot: \"Go on, Casanova. Bring that dog to somebody who needs it more than you.\"",
                "Dot: \"I've been skating this boardwalk since before the casino. Before the bar. Before the ocean, some say.\"",
            ],
            n,
        ));
    }
    say(pick(
        &[
            "Dot: \"Hot dogs! Two bucks, or free for anybody who says something nice to me. Nobody's managed it since the Bicentennial.\"",
            "Dot: \"That mime down the alley's been stuck in his invisible box since Tuesday. Poor thing looks starved.\"",
            "Dot: \"The Golden Clam's got a fancy lounge on the roof. The doorman's named Tiny. Tiny is not tiny.\"",
        ],
        n - 1,
    ))
}

fn flirt_dot(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Dot) {
        return too_far(&g.state);
    }
    if g.state.flag(Flag::GotHotDog) {
        return say("Dot: \"Flattery'll get you one free dog, sugar. Not two.\"");
    }
    g.state.add(Item::HotDog);
    let pts = g.state.award(Flag::GotHotDog, 3);
    scored(Reply::say("You tell Dot her roller skates bring out her eyes. She goes pink, does a slow, proud loop around the cart, and hands you a hot dog with everything. \"On the house, Casanova. Nobody's said that to me since disco.\""), pts)
}
