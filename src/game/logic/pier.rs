//! Gull Point Pier: a closed bait shack, a loaner rod, and Percival.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{Flag, Item, Npc};

pub const INTRO: &str = "The boardwalk runs out of ideas and turns into Gull Point Pier: weathered planks, a lighthouse blinking on a rock, a bait shack that closed in the Reagan administration, and a pelican patrolling the deck like he's been hired to. Far at the east end, a neon sign buzzes: THE SANDPIPER MOTOR COURT.";

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Pier)) => say("Gull Point Pier: a plank deck over black water, a lighthouse out on its rock, and a bait shack with a loaner fishing rod leaning against it. A pelican named Percival patrols the deck. The boardwalk is back to the west, and a motel's neon glows at the east end."),
        (V::Look, Some(N::Pelican)) => say(if g.state.flag(Flag::GotKey) {
            "Percival the pelican: six feet of attitude with a beak to match. His pouch is empty now, which seems to have improved his mood."
        } else {
            "Percival the pelican: six feet of attitude with a beak to match. Something small and brassy glints inside his pouch every time he yawns. He yawns a lot, mostly at you."
        }),
        (V::Talk | V::Flirt, Some(N::Pelican)) => talk_pelican(g),
        (V::Take, Some(N::Pelican)) => say("Percival opens his beak wide enough to fit your whole head. You decide, together, that this is not a good idea."),
        (V::Take | V::Search | V::Use | V::Open, Some(N::Key)) if !g.state.has(Item::Key) && !g.state.flag(Flag::GotKey) => say("The key is in the pelican's pouch. You'd have to trade him for it. Percival has strong views on being pickpocketed."),
        (V::Give | V::Use, _) if cmd.mentions(N::Flounder) => give_flounder(g, cmd),
        (V::Give, _) if cmd.mentions(N::Pelican) => give_pelican(cmd),
        (V::Look, Some(N::Shack)) => say("BAIT & TACKLE, says the sign, and under it CLOSED. A hand-lettered card in the window: \"Gone to Florida. Borrow a rod if you must. Return it. I KNOW. -Mort\""),
        (V::Open | V::Go, Some(N::Shack)) => say("The shack is padlocked and shuttered. Whatever's in there, Mort took it to Florida."),
        (V::Look, Some(N::Rod)) if !g.state.has(Item::Rod) => say("A sturdy fishing rod leaning on the shack wall, wearing a tag: \"LOANER. YES, THAT MEANS YOU.\" The hook is shiny and hopeful. It has never caught anything."),
        (V::Take, Some(N::Rod)) if !g.state.has(Item::Rod) => take_rod(g),
        (V::Use | V::Take | V::Search | V::Go, _) if cmd.mentions(N::Rod) || cmd.mentions(N::Flounder) => fish(g, cmd),
        (V::Search, None | Some(N::Ocean)) => fish(g, cmd),
        (V::Look, Some(N::Ocean)) => say("The Atlantic, black as a bar tab. Something silvery flickers under the pier lights and vanishes. Something with opinions, probably."),
        (V::Jump | V::Climb | V::Go, Some(N::Ocean)) => say("The water is cold, dark, and has a pelican on lifeguard duty. You decide your tux would rather stay dry."),
        (V::Look, Some(N::City)) => say("Out on its rock, the lighthouse sweeps the black water with a beam that goes on, off, and on again. Behind you the boardwalk glitters, and to the east a motel sign buzzes VACANCY like it's pleading."),
        (V::Look, Some(N::Floor)) => say("Salt-bleached planks, spaced just wide enough to lose a quarter through. Mercifully, yours is in your pocket."),
        (V::Look, Some(N::Motel)) => say("The Sandpiper Motor Court, at the end of the pier: one story, one neon sign, and one light on in the front office. VACANCY, it says, hopefully."),
        (V::Look, Some(N::Boardwalk)) => say("Back west along the planks: the boardwalk, its neon, and its hot dog smell."),
        (V::Smell, _) => say("Brine, creosote, and the unmistakable perfume of a pelican who has been eating well."),
        (V::Sit, _) => say("You dangle your legs over the edge. A wave soaks one shoe. You sit up straight again, trying to look like that was the plan."),
        (V::Sing | V::Dance, _) => say("You serenade the sea. The sea doesn't clap. Percival does, sort of, with his beak. It's an ugly sound."),
        _ => None,
    }
}

fn talk_pelican(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Pelican) {
        return say("You call to the pelican. He looks at you, looks away, and keeps patrolling. You'll have to catch him first. Slowly.");
    }
    let t = g.state.talk(Npc::Pelican);
    if g.state.flag(Flag::GotKey) {
        return say(pick(
            &[
                "Percival gives you a long, dignified nod. Between gentlemen, the deal is done.",
                "Percival burps a faint puff of flounder and looks away, embarrassed for both of you.",
            ],
            t,
        ));
    }
    if g.state.has(Item::Flounder) {
        return say("Percival eyes the flounder in your hands the way a bank eyes a deposit. Maybe you should GIVE it to him.");
    }
    say(pick(
        &[
            "\"Squawk,\" says Percival, which loosely translates to: \"I run this pier.\" Then he lifts his beak, and a little brass key glints in his pouch.",
            "Percival clacks his beak and stares at the water. Whatever he wants, it's in there. And it's fish. It's always fish.",
            "Percival will not be bribed with small talk. He wants FLOUNDER. The look on his face says, specifically: not a sardine, not a hot dog, and not a hint of dignity.",
        ],
        t - 1,
    ))
}

fn give_pelican(cmd: &Command) -> Option<Reply> {
    if cmd.mentions(N::Shrimp) {
        return say("Percival sniffs the shrimp, recoils, and turns his head. Even a pelican has standards, and this shrimp does not meet them. He's holding out for flounder.");
    }
    if cmd.mentions(N::Rod) || cmd.mentions(N::Ticket) || cmd.mentions(N::Comb) {
        return say("Percival gives it a polite peck, then looks at you like you've insulted his mother. He wants a FLOUNDER.");
    }
    if cmd.nouns.len() == 1 {
        return say("Give the pelican what? He's been very clear: flounder.");
    }
    None
}

fn give_flounder(g: &mut Game, cmd: &Command) -> Option<Reply> {
    if !g.state.has(Item::Flounder) {
        return say("You don't have a flounder. They don't just hand those out.");
    }
    if cmd.mentions(N::Ocean) {
        return say("Throw a perfectly good flounder back? Percival would never forgive you. You'd never forgive you.");
    }
    if cmd.verb == V::Use && !cmd.mentions(N::Pelican) {
        return say("Use the flounder on what? There's a hungry pelican nearby with an opinion on the matter.");
    }
    if !near(&g.state, N::Pelican) {
        return too_far(&g.state);
    }
    g.state.remove(Item::Flounder);
    g.state.add(Item::Key);
    let pts = g.state.award(Flag::GotKey, 4);
    scored(Reply::say("You hold out the flounder. Percival studies it, studies you, then swallows it whole in one smooth, horrifying gulp. A burp later, he coughs a little brass key onto the planks, still warm. \"Dryer 3,\" it says on the tag. The pelican gives you the tiniest nod: a professional courtesy."), pts)
}

fn take_rod(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Rod) {
        return too_far(&g.state);
    }
    g.state.add(Item::Rod);
    let pts = g.state.award(Flag::GotRod, 2);
    scored(Reply::say("You borrow the rod, tipping an imaginary hat at Mort's card. Borrow, not steal. Mort said so. The rod hums with the power of a man who has never been out-fished. By Mort, anyway."), pts)
}

fn fish(g: &mut Game, cmd: &Command) -> Option<Reply> {
    if g.state.flag(Flag::CaughtFlounder) {
        return say("You've caught your flounder. Trying for a second would only tempt the sea, and the sea has been very patient with you so far.");
    }
    if !g.state.has(Item::Rod) {
        return say(if cmd.mentions(N::Flounder) {
            "Catch a flounder? With what, charm? Even Chip's charm has a limit, and it's about four inches of water."
        } else {
            "Fishing without a rod is just standing at the edge of a pier looking sad. There's a loaner by the bait shack."
        });
    }
    if !near(&g.state, N::Ocean) {
        return say("You can't fish from back here. Walk to the front railing, where the water is within casting distance.");
    }
    if !g.state.has(Item::Shrimp) {
        return say("You cast the empty hook into the dark water. It sinks. Nothing bites. A fish surfaces briefly, looks at the bare hook, and laughs. You'll need bait: something fishy, free, and slightly past its prime.");
    }
    g.state.remove(Item::Shrimp);
    g.state.add(Item::Flounder);
    let pts = g.state.award(Flag::CaughtFlounder, 3);
    scored(Reply::say("You thread the complimentary shrimp onto the hook, cast out over the railing, and wait. And wait. Then, with a tug that nearly takes your toupee (you don't wear one; it's the thought), you reel in a flounder the size of a hubcap. It regards you with an expression of enormous judgment. Both its eyes are on the same side of its face, and both of them are disappointed."), pts)
}
