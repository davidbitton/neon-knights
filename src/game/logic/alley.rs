//! The back alley: a hungry mime standing on a storm grate.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{DeathKind, Flag, Item, Npc};

pub const INTRO: &str = "You slip into the alley behind the boardwalk. Bricks, a dumpster, a fire escape, and the Rusty Anchor's back door under a flickering bulb. And, standing perfectly still on a storm grate, a mime.";

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    let fed = g.state.flag(Flag::MimeFed);
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Alley)) => say(if fed {
            "A narrow back alley: bricks, a dumpster, trash bags, a fire escape, and the Rusty Anchor's back door. The mime is on his break, leaning against the wall. The boardwalk is back to the east."
        } else {
            "A narrow back alley: bricks, a dumpster, trash bags, a fire escape, and the Rusty Anchor's back door. A mime is trapped in an invisible box on top of a storm grate. The boardwalk is back to the east."
        }),
        (V::Look, Some(N::Mime)) => say(if fed {
            "The mime leans against the bricks, miming a relaxing cigarette break. He looks content. Also full."
        } else {
            "A mime in a striped shirt and beret, pressing his white-gloved hands against the walls of an invisible box. He's standing right on a storm grate. He looks hungry, in a silent, expressive way."
        }),
        (V::Talk, Some(N::Mime) | None) => {
            if !near(&g.state, N::Mime) {
                return say("You call out to the mime. He mimes cupping his ear. You'll have to get closer.");
            }
            let t = g.state.talk(Npc::Mime);
            if fed {
                return say("The mime gives you a big thumbs-up, then points at the storm grate and mimes fishing something out of it with great delicacy. Subtle, for a mime.");
            }
            say(pick(
                &[
                    "The mime mimes being trapped in a box. Then mimes the box getting smaller. Then mimes a tiny violin, for himself.",
                    "The mime points at his stomach, then at his mouth, then gives you enormous sad eyes. Even his eyebrows look hungry.",
                    "You ask the mime his name. He mimes a long, elaborate name. You think it's \"Gerard.\" It might be \"garage.\"",
                ],
                t - 1,
            ))
        }
        (V::Give | V::Use, _) if cmd.mentions(N::Mime) || cmd.mentions(N::HotDog) => feed_mime(g, cmd),
        (V::Flirt, Some(N::Mime)) => say("You wink at the mime. He mimes being flattered, then mimes a wedding ring. Taken."),
        (V::Take, Some(N::Mime)) => say("You can't take the mime. Nobody can take a mime. That's the whole problem with mimes."),
        (V::Look, Some(N::Grate)) => say(match (fed, g.state.flag(Flag::GotFiver)) {
            (_, true) => "A rusty storm grate. Nothing left down there but a bottle cap and a strong smell.",
            (false, _) => "A rusty storm grate. Something green and papery is wedged between the bars, but the mime is standing right on top of it.",
            (true, false) => "A rusty storm grate. Wedged between the bars, just out of finger range, is something green and papery. Money?",
        }),
        (V::Use | V::Search | V::Take | V::Open, Some(N::Grate) | Some(N::Money) | Some(N::Comb)) => fish(g, cmd),
        (V::Look, Some(N::Dumpster)) => say("A big green dumpster stenciled OCEAN BLUFFS SANITATION - PICKUP AT MIDNIGHT. Your watch says 11:59."),
        (V::Open | V::Search, Some(N::Dumpster)) => say("You lift the lid. The smell lifts YOU. There's nothing in there but trash and a strong sense of foreboding. You let the lid drop."),
        (V::Climb | V::Go | V::Jump | V::Sit, Some(N::Dumpster)) => g.die(
            DeathKind::Dumpster,
            vec![
                "You climb into the dumpster to look for loose change. It's surprisingly cozy. You close the lid to keep out the draft.",
                "At precisely midnight, a garbage truck backs into the alley, beeping politely. The dumpster rises. Chip rises with it. The rest is recycling.",
            ],
        ),
        (V::Take, Some(N::Dumpster)) => say("It's a dumpster. It has wheels, but it also has standards."),
        (V::Look, Some(N::Trash)) => say("Bags of trash. One of them is moving. You decide it isn't."),
        (V::Search, Some(N::Trash)) => say("You find a single sequin and a phone bill for someone named Lance. Neither is useful. Lance owes forty dollars."),
        (V::Take, Some(N::Trash)) => say("You're a gentleman, not a raccoon."),
        (V::Look, Some(N::Door)) => say("The Rusty Anchor's back door: steel, locked, and stenciled DELIVERIES ONLY. Above it, a bare bulb flickers like it's tapping out HELP in Morse code."),
        (V::Open | V::Use, Some(N::Door)) => say("Locked. Gus doesn't trust deliveries either."),
        (V::Look, Some(N::FireEscape)) => say("A rusty fire escape zigzags up the wall. The bottom ladder dangles eight feet off the ground, which is seven feet more than Chip can jump."),
        (V::Climb | V::Use | V::Jump, Some(N::FireEscape)) => say("You leap for the ladder. Your fingertips graze it. Your dignity does not survive the landing."),
        (V::Look, Some(N::City)) => say("A thin strip of night sky, crisscrossed by laundry lines and regret."),
        (V::Look, Some(N::Floor)) => say("Cracked asphalt and a puddle with a rainbow sheen. Romantic, if you squint and don't breathe."),
        (V::Smell, _) => say("Garbage, motor oil, and a whiff of greasepaint. The greasepaint is the mime."),
        _ => None,
    }
}

fn feed_mime(g: &mut Game, cmd: &Command) -> Option<Reply> {
    if g.state.flag(Flag::MimeFed) {
        return say("The mime mimes being stuffed. He couldn't eat another invisible bite.");
    }
    if !g.state.has(Item::HotDog) {
        return say("The mime pats his empty stomach and mimes a long, sad sigh. You have nothing he'd eat. He's a mime, not a goat.");
    }
    if !cmd.mentions(N::Mime) && cmd.verb == V::Use {
        return say("Use the hot dog on what? Someone in this alley looks awfully hungry.");
    }
    if !near(&g.state, N::Mime) {
        return too_far(&g.state);
    }
    g.state.remove(Item::HotDog);
    let pts = g.state.award(Flag::MimeFed, 5);
    scored(Reply::say("The mime accepts the hot dog with a deep bow, eats it in three very real bites, and then, miraculously, climbs out of his invisible box. He mimes \"thank you,\" tips an invisible hat, and strolls over to lean against the wall. He'd been standing on a storm grate the whole time."), pts)
}

fn fish(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let s = &mut g.state;
    if s.flag(Flag::GotFiver) {
        return say("Nothing left down there but a bottle cap and a strong smell.");
    }
    if cmd.mentions(N::Comb) && !cmd.mentions(N::Grate) && !cmd.mentions(N::Money) {
        return None; // "use comb" on its own is handled elsewhere
    }
    if !s.flag(Flag::MimeFed) {
        return say("The mime is standing on the grate inside his invisible box, and he's not budging. Maybe he'd move for the right offer.");
    }
    if !near(s, N::Grate) {
        return too_far(s);
    }
    if !cmd.mentions(N::Comb) {
        return say("You try to reach between the bars. Your fingers are too thick, which nobody has ever told you before. You need something thin, with teeth. Something a gentleman always carries.");
    }
    s.cash += 5;
    let pts = s.award(Flag::GotFiver, 5);
    scored(Reply::say("You thread your trusty comb between the bars and, with the delicacy of a surgeon (or at least a very good barber), tease out a soggy five-dollar bill! You give the comb a shake. It's seen worse. It's seen your hair."), pts)
}
