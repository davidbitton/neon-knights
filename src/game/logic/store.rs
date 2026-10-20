//! Lucky Gull 24/7: Brenda, carnations, mints, and the slushie of doom.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{DeathKind, Flag, Item, Npc};

pub const INTRO: &str = "The Lucky Gull 24/7 greets you with an electronic chime that sounds like a seagull being stepped on. Fluorescent lights, a checkerboard floor, a bucket of carnations by the register, and a slushie machine humming ominously in the corner. Behind the counter, Brenda the clerk does not look up.";

fn at_counter(g: &Game) -> bool {
    near(&g.state, N::Brenda) || near(&g.state, N::Carnation)
}

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Store)) => say("The Lucky Gull 24/7: flickering tubes, a checkerboard floor, and aisles of things nobody needs at midnight. Brenda the clerk is behind the counter. Carnations sit in a bucket by the register, breath mints are on the counter, and the slushie machine hums in the corner. The door leads back out to the boardwalk."),
        (V::Look, Some(N::Brenda)) => say("Brenda is sixteen going on bored. She has headphones on, gum in, and a magazine open to a quiz titled \"Is He Worth It?\" She glances at you and circles \"No.\""),
        (V::Look, Some(N::Carnation)) => say(if g.state.flag(Flag::BoughtCarnation) {
            "A bucket of red carnations, a little emptier thanks to you. $3 each."
        } else {
            "A bucket of red carnations, $3 each. They're a little wilted, but so is everybody at this hour."
        }),
        (V::Look, Some(N::Mints)) => say("Arctic Blast breath mints, $2 a tin. \"So strong they're technically a weather event.\""),
        (V::Look, Some(N::Slushie)) => say("The slushie machine churns two flavors: Blue and Red. Not blueberry and cherry. Just Blue and Red. A sign says FREE - DRINK AT OWN RISK. The machine appears to be breathing."),
        (V::Look, Some(N::Register)) => say("An ancient cash register next to Brenda's magazine. The magazine is getting more attention."),
        (V::Look, Some(N::Bar)) => say("A green counter with a register, a rack of breath mints, and Brenda's elbows."),
        (V::Look, Some(N::Bottles)) => say("An aisle of snacks with names like Cheez Wavez and Krunch Nuggetz. Several have expiration dates in roman numerals."),
        (V::Look, Some(N::Door)) => say("The door back out to the boardwalk. The welcome mat says GO AWAY, which feels honest."),
        (V::Look, Some(N::Floor)) => say("A checkerboard floor, mopped sometime during the previous administration."),
        (V::Talk, Some(N::Brenda) | None) => {
            if !at_counter(g) {
                return say("Brenda can't hear you over her headphones. Get closer. Then wave.");
            }
            let t = g.state.talk(Npc::Brenda);
            say(pick(
                &[
                    "Brenda, without looking up: \"Welcome to the Lucky Gull. Sadly.\"",
                    "Brenda: \"Carnations are three bucks. Mints are two. The slushie's free, 'cause the machine's been running since 1994 and nobody's brave enough to unplug it.\"",
                    "Brenda pops her gum. \"You're, like, really committed to that tux.\"",
                ],
                t - 1,
            ))
        }
        (V::Flirt, Some(N::Brenda) | None) => say("Brenda looks at you over her magazine. \"Ew. You're, like, my grandpa's age.\" You are not. You're almost positive you are not."),
        (V::Order | V::Take, Some(N::Carnation)) => buy(g, Item::Carnation, cmd.verb == V::Take),
        (V::Order | V::Take, Some(N::Mints)) => buy(g, Item::Mints, cmd.verb == V::Take),
        (V::Order, None | Some(N::Brenda)) => say("Brenda: \"We got carnations, mints, and slushies. And regrets, but those are free with purchase.\""),
        (V::Give, _) if cmd.mentions(N::Money) || cmd.mentions(N::Quarter) => say("Brenda: \"Tell me what you want to BUY, I'm not a wishing well.\""),
        (V::Order | V::Take, Some(N::Slushie)) => say("Brenda: \"It's free and it's self-serve. If you're gonna drink it, I'm not watching. Liability.\""),
        (V::Drink | V::Eat | V::Use, Some(N::Slushie)) => {
            if !near(&g.state, N::Slushie) {
                return too_far(&g.state);
            }
            g.die(
                DeathKind::Slushie,
                vec![
                    "You pull a jumbo cup of Blue and drink it in one heroic, unbroken slurp. Brenda finally looks up.",
                    "The brain freeze arrives like a glacier with a grudge. Your eyes cross. Your pompadour frosts over. Somewhere a polar bear says, \"Whoa.\"",
                ],
            )
        }
        (V::Take, Some(N::Register)) => say("Brenda, without looking up: \"Touch that and I'm telling my mom. She's a cop. Well, a crossing guard. Same energy.\""),
        (V::Smell, _) => say("Floor cleaner, hot-dog-roller grease, and the faint ozone of a slushie machine gaining sentience."),
        _ => None,
    }
}

fn buy(g: &mut Game, item: Item, taking: bool) -> Option<Reply> {
    let (price, flag, pts, done) = match item {
        Item::Carnation => (
            3,
            Flag::BoughtCarnation,
            3,
            "One's plenty. A gentleman wears one carnation. Two is a funeral.",
        ),
        _ => (
            2,
            Flag::BoughtMints,
            3,
            "One tin of Arctic Blast is plenty. Two would be a war crime.",
        ),
    };
    if g.state.flag(flag) {
        return say(done);
    }
    if !at_counter(g) {
        return too_far(&g.state);
    }
    if g.state.cash < price {
        let pocket = if g.state.has(Item::Quarter) {
            " Your lucky quarter is not, no matter how lucky, enough."
        } else {
            ""
        };
        return say(if taking {
            format!("Brenda, without looking up: \"That's ${price}. Steal it and I'm telling my mom. She's a crossing guard.\" You pat down your tux and find lint, a comb, and false hope.{pocket}")
        } else {
            format!("Brenda: \"${price}.\" You pat down your tux and find lint, a comb, and false hope.{pocket}")
        });
    }
    g.state.cash -= price;
    g.state.add(item);
    let award = g.state.award(flag, pts);
    let text = match item {
        Item::Carnation => "Brenda takes three soggy dollars with two fingers and hands you a red carnation. \"Big date?\" she asks, then answers herself: \"Big date.\"",
        _ => "Brenda trades you a tin of Arctic Blast mints for two damp dollars, then wipes her hands on a napkin. Then another napkin.",
    };
    scored(Reply::say(text), award)
}
