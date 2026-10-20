//! The Golden Clam: Tiny the doorman and a very generous slot machine.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{DeathKind, Flag, Item, Npc};

pub const INTRO: &str = "The Golden Clam's lobby hits you like a perfume sample: red velvet, gold trim, a crystal chandelier, and a carpet so loud it should be arrested. Slot machines jingle to the left. To the right, a brass elevator marked CROW'S NEST, guarded by a mountain in a suit.";

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    let ok = g.state.flag(Flag::TinyApproved);
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Casino)) => say(if ok {
            "The Golden Clam's lobby: velvet, gold, a chandelier, slot machines, and a carpet that could induce a migraine. The elevator to the Crow's Nest stands open, and Tiny has stepped aside. The exit to the boardwalk is behind you."
        } else {
            "The Golden Clam's lobby: velvet, gold, a chandelier, slot machines, and a carpet that could induce a migraine. A brass elevator marked CROW'S NEST is guarded by Tiny, the doorman. The exit to the boardwalk is behind you."
        }),
        (V::Look, Some(N::Tiny)) => say("Tiny is roughly the size of a vending machine and twice as hard to argue with. He wears sunglasses indoors, at midnight, which is how you know he's serious. He has a small notebook in his breast pocket."),
        (V::Talk, Some(N::Tiny) | None) => talk_tiny(g),
        (V::Flirt, Some(N::Tiny)) => say("You flash Tiny your best smile. Tiny flashes his best \"I bench-press motorcycles\" smile right back."),
        (V::Give, _) if cmd.mentions(N::Tiny) => say(if cmd.mentions(N::Carnation) {
            "Tiny: \"I don't want it, friend. I want to SEE it. On you.\""
        } else if cmd.mentions(N::Money) || cmd.mentions(N::Quarter) {
            "Tiny: \"I can't be bought, friend. I can barely be rented.\""
        } else {
            "Tiny politely declines. He's on duty."
        }),
        (V::Take | V::Climb, Some(N::Tiny)) => say("You consider it. Tiny considers you considering it. You stop considering it."),
        (V::Look, Some(N::Elevator)) => say(if ok {
            "The brass elevator doors stand open, a velvet bench inside. Next stop: the Crow's Nest."
        } else {
            "A brass elevator with a plaque: CROW'S NEST ROOFTOP LOUNGE - DRESS CODE ENFORCED. Underneath, smaller: \"RED CARNATION REQUIRED. NO EXCEPTIONS. THIS MEANS YOU, LANCE.\""
        }),
        (V::Look, Some(N::Slots)) => say("Three slot machines, jingling with the desperate charm of a lounge act. The sign on top says MEGA CLAM PROGRESSIVE JACKPOT: 11,000 QUARTERS. It doesn't say where they come out."),
        (V::Play | V::Use, _) if cmd.mentions(N::Slots) => slots(g),
        (V::Give, _) if cmd.mentions(N::Slots) => slots(g),
        (V::Look, Some(N::Chandelier)) => say("A crystal chandelier the size of a hot tub. It sways slightly every time the slots pay out, which is never."),
        (V::Take | V::Climb | V::Jump, Some(N::Chandelier)) => say("Chip has seen enough movies to know how that ends."),
        (V::Look, Some(N::Plant)) => say("A potted palm. Plastic, with one real cigarette burn. It's the most honest thing in the casino."),
        (V::Take, Some(N::Plant)) => say("It's bolted down. Even the plants here don't trust you."),
        (V::Look, Some(N::Door)) => say("The way back out to the boardwalk, through a curtain of gold tinsel."),
        (V::Look, Some(N::Floor)) => say("The carpet is magenta with gold dots, designed by scientists to keep you awake and gambling. It's working on the awake part."),
        (V::Smell, _) => say("Cigar smoke, carpet shampoo, and desperation, in that order."),
        _ => None,
    }
}

fn talk_tiny(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Tiny) {
        return say("Tiny doesn't respond to shouting. Tiny doesn't respond to much. Walk over.");
    }
    let s = &mut g.state;
    let t = s.talk(Npc::Tiny);
    if s.flag(Flag::TinyApproved) {
        return say(pick(
            &[
                "Tiny: \"I write haiku on my breaks. Big man at the door. / Small flower lets you go by. / Lunch is at three a.m.\"",
                "Tiny: \"They call me Tiny because of my feet. Size nineteen. It's ironic. No, wait. It isn't.\"",
                "Tiny: \"Go on up, friend. Vonda doesn't wait forever. Only most of forever.\"",
            ],
            t,
        ));
    }
    if s.flag(Flag::WearingCarnation) {
        let pts = s.award(Flag::TinyApproved, 5);
        return scored(Reply::say("Tiny lowers his sunglasses, studies your carnation, and nods slowly. \"That flower is doing a lot of work, friend. Go on up.\" He steps aside and the elevator doors slide open with a ding."), pts);
    }
    if s.has(Item::Carnation) {
        return say("Tiny: \"A carnation in your pocket isn't a carnation on your lapel, friend. Rules are rules.\"");
    }
    say(pick(
        &[
            "Tiny: \"Crow's Nest has a dress code, friend. Red carnation on the lapel. I don't make the rules. I just enforce 'em. Gently, mostly.\"",
            "Tiny: \"No carnation, no elevator. The Lucky Gull sells 'em, if you've got the cash.\"",
        ],
        t - 1,
    ))
}

fn slots(g: &mut Game) -> Option<Reply> {
    let s = &g.state;
    if s.cash == 0 && !s.has(Item::Quarter) {
        return say("You pull the lever. Nothing happens. The slot machine eyes your empty pockets with open contempt.");
    }
    if !near(s, N::Slots) {
        return too_far(s);
    }
    g.die(
        DeathKind::Jackpot,
        vec![
            "You feed the machine and give the lever a gentlemanly pull. Cherry. Cherry. Clam.",
            "Sirens! Lights! MEGA CLAM PROGRESSIVE JACKPOT! Eleven thousand quarters come thundering out of the machine, which, it turns out, was designed by the same people who designed the sinking ship ride at the pier.",
        ],
    )
}
