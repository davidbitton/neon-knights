//! The Sandpiper Motor Court: Ferris, a comb-able bouffant, and shrimp.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{Flag, Item, Npc};

pub const INTRO: &str = "The Sandpiper Motor Court's front office smells like pine cleaner, coffee that gave up, and cigarettes from a more optimistic era. Wood paneling. A desk with a bell. A television murmuring a soap opera. A cart of \"complimentary continental breakfast.\" And behind the desk, a woman whose bouffant has its own zip code.";

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    let away = g.state.flag(Flag::FerrisAway);
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Motel)) => say(if away {
            "The Sandpiper's front office: wood paneling, a key rack, a TV playing a soap opera, and a cart of continental breakfast. The front desk is empty. The restroom light says BUSY, and from behind the door comes the sound of someone admiring herself. The pier is back to the west."
        } else {
            "The Sandpiper's front office: wood paneling, a key rack, a TV playing a soap opera, a cart of continental breakfast, and Ferris at the front desk, nodding off over a paperback. The pier is back to the west."
        }),
        (V::Look, Some(N::Ferris)) => say("Ferris, night manager, by the badge. Her bouffant is a ruin: it has slumped to one side like a tent in a storm. She's pretending to read a paperback called \"Passion on the Pier\" and watching her soaps with one eye. The other eye is taking inventory of you."),
        (V::Talk | V::Flirt, Some(N::Ferris)) => talk_ferris(g),
        (V::Use | V::Give | V::Look, _) if cmd.mentions(N::Comb) && (cmd.mentions(N::Ferris) || cmd.verb == V::Give) => comb(g, cmd),
        (V::Look, Some(N::Desk)) => say("A laminated front desk with a guest book, a call bell, and a sign: NO REFUNDS, NO RETURNS, NO QUESTIONS. The guest book's last entry: \"Mr. & Mrs. Smith. Hourly.\" The one before that: \"Lance. Alone. Please don't tell Lance.\""),
        (V::Read, None | Some(N::Desk)) => say("The guest book, in a dozen different handwritings: \"Mr. & Mrs. Smith (hourly).\" \"Mr. & Mrs. Jones (also hourly).\" \"Lance (alone, please, no questions).\" The Sandpiper is, clearly, a place where people come to not be found."),
        (V::Look, Some(N::Bell)) => say("A brass call bell, polished by a thousand impatient palms."),
        (V::Use | V::Play, Some(N::Bell)) => {
            if !near(&g.state, N::Bell) {
                return too_far(&g.state);
            }
            say(if away {
                "Ding! From behind the restroom door: \"I HEARD THAT, HONEY. TEN MINUTES!\" She sounds like a woman having the best ten minutes of her year."
            } else {
                "Ding! Ferris, without taking her eyes off the TV: \"I SEE you, honey. I'm RIGHT here. Don't ring that again, it echoes in my fillings.\""
            })
        }
        (V::Look, Some(N::Tv)) => say("A fuzzy television playing \"As the Tide Turns.\" A man in a lab coat is accusing a woman in a nightgown of something that happened in a previous season. Everyone's very upset. Everyone's wearing shoulder pads."),
        (V::Use | V::Play, Some(N::Tv)) => say(if away { "You flip the channel. A man in a swimsuit is selling a mop. From the restroom: \"CHANGE IT BACK! I KNOW WHO DID IT IN THE LAB!\"" } else { "You reach for the TV knob. Ferris, not looking up: \"Touch that dial and I'll make you a sofa.\" You back away." }),
        (V::Look, Some(N::Shrimp)) => say(if g.state.flag(Flag::GotShrimp) { "An empty tray, a puddle of pink sauce, and a heat lamp that has seen things." } else { "A cart with a sign: CONTINENTAL BREAKFAST (GUESTS ONLY). On a bed of wilted lettuce sits a tray of cocktail shrimp: pink, curled, glistening, and ever so slightly warm. The sign says they're \"complimentary.\" The shrimp say nothing, but they smell like a threat." }),
        (V::Take, Some(N::Shrimp)) => take_shrimp(g),
        (V::Eat, Some(N::Shrimp)) if !g.state.has(Item::Shrimp) => eat_tray(g),
        (V::Look, Some(N::Door | N::Restroom)) => say(if away { "The restroom door has a little BUSY light on above it. From inside: humming, and the scritch of a comb on a very big bouffant. Chip is, as always, a gentleman. He does not knock." } else { "A plain restroom door. The light above it says FREE. Ferris has been watching it for twenty minutes, waiting for her chance, but not leaving her post." }),
        (V::Open | V::Go, Some(N::Door | N::Restroom)) => say("You reach for the restroom door. Something about this feels like a bad idea. You've been wrong about that before, but not tonight."),
        (V::Look, Some(N::Pier)) => say("West, through the glass door, the pier and its pelican. Nothing wrong with that view."),
        (V::Look, Some(N::City)) => say("The ceiling's water stains have formed what looks like a map of Florida. Or possibly a fish."),
        (V::Look, Some(N::Floor)) => say("Red carpet with a pattern of yellow diamonds that were chosen by someone who hated carpet. It's held up well, if \"well\" means \"unsettlingly.\""),
        (V::Smell, _) => say("Pine cleaner, lukewarm coffee, and something warm and pink from the breakfast cart."),
        (V::Sit, _) => say("The lobby has no chairs. Presumably, they were stolen. Or occupied by regretful guests. You stand."),
        (V::Take, Some(N::Desk | N::Bell)) => say("The bell, the book, and the desk are all bolted down. The Sandpiper has learned things."),
        _ => None,
    }
}

fn talk_ferris(g: &mut Game) -> Option<Reply> {
    if g.state.flag(Flag::FerrisAway) {
        return say("You lean toward the restroom door. \"Ferris?\" \"Not NOW, honey!\" She's busy. You're surprisingly fine with that.");
    }
    if !near(&g.state, N::Ferris) {
        return say("Ferris, from behind the desk, doesn't look away from her soap opera. \"If you want a room, sugar, come to the desk. Rooms are by the hour, by the night, or by the 'I'd rather not say.'\"");
    }
    let t = g.state.talk(Npc::Ferris);
    say(pick(
        &[
            "\"Rooms are forty a night,\" says Ferris, \"or twelve an hour, no questions, no refunds, no rebooking.\" She doesn't blink. \"The shrimp is for guests only. And it's complimentary, so don't make it weird.\"",
            "Ferris touches her bouffant, which slumps another inch. \"Humidity,\" she says darkly. \"It's been killing my 'do since 1983. I'd give anything for somebody who knows how to tease a beehive properly.\" She stares at your pompadour with new interest.",
            "\"Gorgeous hair,\" Ferris says to your pompadour, with the longing of a woman who's lost her own to the sea air. \"I bet you have a COMB. A good one. I'd trade a lot for ten minutes with a good comb and a mirror. Dignity, even. I've got some.\" She doesn't.",
        ],
        t - 1,
    ))
}

fn comb(g: &mut Game, cmd: &Command) -> Option<Reply> {
    if cmd.verb == V::Look {
        return None;
    }
    if !g.state.has(Item::Comb) {
        return say(
            "You don't have a comb to offer. Even Chip's pompadour can't do this one on its own.",
        );
    }
    if g.state.flag(Flag::FerrisAway) {
        return say("Ferris is in the restroom, admiring her work. You can hear the humming through the door. Let her have her moment.");
    }
    if !near(&g.state, N::Ferris) {
        return too_far(&g.state);
    }
    g.state.set(Flag::FerrisAway);
    say("You step up to the desk and offer to help with the 'do. Ferris, startled, lets you tease. Four deft strokes, a puff of Chip's pocket hairspray, and the bouffant rises like the dawn: tall, proud, and structurally sound. Ferris touches it, gasps, and bolts for the restroom mirror, calling over her shoulder: \"Don't touch anything! ESPECIALLY the shrimp!\" The restroom door clicks shut. You get your comb back. She'll be a while.")
}

fn take_shrimp(g: &mut Game) -> Option<Reply> {
    if g.state.has(Item::Shrimp) {
        return say("You already have a shrimp. One complimentary shrimp per customer. It's not a rule, it's a survival strategy.");
    }
    if !near(&g.state, N::Shrimp) {
        return too_far(&g.state);
    }
    if !g.state.flag(Flag::FerrisAway) {
        return say("Your fingers close on a shrimp. \"HEY!\" Ferris's paperback slams the desk. \"Guests only, sugar. Registered guests. Get your hand out of my breakfast.\" You let go. The shrimp shivers, either from relief or from its age.");
    }
    g.state.add(Item::Shrimp);
    let pts = g.state.award(Flag::GotShrimp, 2);
    scored(Reply::say("With Ferris busy admiring herself, you lift the tray lid, pluck the plumpest shrimp, and drop it in your pocket. It's cold, slick, and smells like the sea giving up. Whatever you do with this, you should NOT eat it."), pts)
}

fn eat_tray(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Shrimp) {
        return too_far(&g.state);
    }
    if !g.state.flag(Flag::FerrisAway) {
        return say("Your hand drifts to the shrimp tray. \"Guests ONLY.\" Ferris doesn't even look up. How does she DO that?");
    }
    super::shrimp_death(g)
}
