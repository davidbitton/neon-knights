//! The Crow's Nest: Vonda, a piano, and a pool with no water in it.

use super::{near, say, Game, Reply};
use crate::game::logic::Effect;
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{DeathKind, Flag, Npc};

pub const INTRO: &str = "The elevator doors open onto the Crow's Nest: a rooftop lounge strung with party lights, the whole town glittering below. A grand piano. A microphone. A drained swimming pool. And there, by the piano, in a gown of approximately nine thousand sequins: Vonda.";

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Rooftop)) => say("The Crow's Nest: party lights, a grand piano, a microphone, one cocktail table, and a swimming pool that has been drained for repairs. Vonda stands by the piano, glittering. The elevator goes back down to the casino."),
        (V::Look, Some(N::Vonda)) => say("Vonda: sequins from neck to ankle, a voice like warm honey poured over gravel, and hair so big it has its own weather. She's humming a ballad and watching the elevator."),
        (V::Talk | V::Sing | V::Flirt | V::Dance, Some(N::Vonda) | None) => vonda(g),
        (V::Give, _) if cmd.mentions(N::Vonda) => vonda(g),
        (V::Look, Some(N::Piano)) => say("A black grand piano with a cigarette burn on middle C. It has played more weddings than a priest."),
        (V::Play | V::Use, Some(N::Piano)) => say("You plunk out \"Chopsticks.\" Vonda winces, in perfect pitch."),
        (V::Look, Some(N::Pool)) => say("A swimming pool, drained for repairs. A sign says CLOSED. The bottom is twelve feet down and decorated with leaves, a lost flip-flop, and a sad rubber duck."),
        (V::Jump | V::Go | V::Climb, Some(N::Pool)) => {
            if !near(&g.state, N::Pool) {
                return say("You'd have to walk over to the pool first. Which, honestly, is advice you should ignore.");
            }
            g.die(
                DeathKind::Pool,
                vec![
                    "Overcome by romance, Bilge Water, and the party lights, you take a running start and perform a flawless swan dive into the pool.",
                    "Your form is magnificent. Your judges would give you a ten. Your judges would also point out that the pool is empty.",
                ],
            )
        }
        (V::Look, Some(N::Railing) | Some(N::City)) => say("The whole of Ocean Bluffs glitters below: the boardwalk, the pier, the Rusty Anchor's sad little neon anchor. From up here, even the bad parts look like jewelry."),
        (V::Jump | V::Climb, Some(N::Railing) | Some(N::City)) => say("You lean over the railing, get dizzy, and lean right back. You came up here for romance, not gravity."),
        (V::Look, Some(N::Table)) => say("A cocktail table holding a pink drink with a tiny umbrella. Not yours."),
        (V::Take | V::Drink, Some(N::Table) | Some(N::Drink)) => say("That pink drink belongs to Vonda. Stealing a lady's drink is not how gentlemen begin an evening."),
        (V::Look, Some(N::Elevator)) => say("The elevator back down to the casino lobby. It plays a muzak version of a song you can't place."),
        (V::Look, Some(N::Floor)) => say("Teak decking, polished to a high shine and dotted with confetti from somebody's better night."),
        (V::Smell, _) => say("Jasmine, chlorine (from memory), and Vonda's perfume, which smells like a dare."),
        _ => None,
    }
}

fn vonda(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Vonda) {
        return say("Vonda is mid-ballad; you'd have to walk up to the piano to get a word in.");
    }
    let s = &mut g.state;
    s.talk(Npc::Vonda);
    if !s.flag(Flag::DrankDrink) {
        return say("Vonda spots your carnation and stops mid-note. \"So YOU'RE my matchbook man. Well, don't just stand there, sugar. Sing me something.\" You open your mouth. What comes out is a squeak only dolphins can hear. You need some liquid courage.");
    }
    if !s.flag(Flag::AteMints) {
        return say("Vonda spots your carnation and leans in close... then leans way, way back. \"Honey, your breath smells like bilge water.\" Which, to be fair, it is.");
    }
    s.award(Flag::Won, 10);
    Some(
        Reply::many(vec![
            "Vonda spots your carnation and smiles a smile that could jump-start a stalled car. \"So YOU'RE my matchbook man.\" You smile back, minty fresh and brimming with Bilge Water courage.",
            "She hands you the microphone. The piano swells. And for three glorious minutes, Chip Valentine sings his heart out under the party lights, hitting notes that haven't been hit on this coast since 1977.",
            "The lounge erupts. Someone throws a carnation. Vonda takes your hand, looks deep into your eyes, and says the words you've waited all night to hear:",
            "\"You're hired.\"",
        ])
        .with(Effect::Won),
    )
}
