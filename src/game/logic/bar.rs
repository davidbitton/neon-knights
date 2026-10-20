//! The Rusty Anchor: Gus, the jukebox, the matchbook, and the eggs.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{DeathKind, Flag, Item, Npc};

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    match cmd.verb {
        V::Look => look(g, n),
        V::Take => take(g, n),
        V::Talk => match n {
            Some(N::Bartender) | None => talk_gus(g, cmd.second() == Some(N::Drink)),
            Some(N::Drink) => talk_gus(g, true),
            Some(N::Patron) => say("You clear your throat at the regular. Without waking, he mumbles, \"Not now, Doris, the tide's coming in,\" and goes back to snoring in perfect time with the ceiling fan."),
            Some(N::Jukebox) => say("You whisper sweet nothings to the jukebox. It says nothing sweet back. Typical."),
            _ => None,
        },
        V::Order if n.is_none() || matches!(n, Some(N::Drink) | Some(N::Bartender)) => order_drink(g),
        V::Use | V::Play | V::Give => use_thing(g, cmd),
        V::Open => match n {
            Some(N::Restroom) => say("The restroom door is duct-taped shut under a sign reading OUT OF ORDER. Someone has added in marker: \"SO IS THE CUSTOMER.\" You decide you can hold it."),
            Some(N::Jukebox) => say("The jukebox is padlocked. Gus trusts no one, especially not the jukebox."),
            _ => None,
        },
        V::Eat => match n {
            Some(N::Egg) => eat_egg(g),
            Some(N::Matchbook) => say("You are a gentleman, not a goat."),
            _ => None,
        },
        V::Sit => match n {
            Some(N::Stool) | Some(N::Bar) | None => {
                if !near(&g.state, N::Stool) {
                    return too_far(&g.state);
                }
                g.state.set(Flag::SatOnStool);
                say("You perch on a stool. It wobbles. You wobble back. You both agree never to speak of this, and you stand up again.")
            }
            Some(N::Table) => say("The table is taken by a sleeping regular. He's been there so long he's technically furniture."),
            _ => None,
        },
        V::Dance if g.state.flag(Flag::PlayedJukebox) => say("Chip unleashes his signature move, \"The Undertow.\" It involves a lot of hips and not enough rhythm. Somewhere, a disco ball sheds a single tear."),
        V::Flirt => match n {
            Some(N::Bartender) => say("You give Gus your best smolder. Gus gives you his best \"there's a baseball bat under this counter.\" A stalemate."),
            Some(N::Patron) => say("He's asleep. Even Chip has rules."),
            Some(N::Jukebox) => say("You lean on the jukebox and wink at it. It's the longest relationship you've had all year."),
            None => say("There's nobody here to flirt with but Gus, a snoring sailor, and a jukebox. You've had worse Saturday nights. Not many."),
            _ => None,
        },
        V::Smell => say("The Rusty Anchor smells like low tide, warm beer, and Chip's cologne, which he applies with a garden hose. The cologne is winning."),
        _ => None,
    }
}

fn look(g: &Game, n: Option<N>) -> Option<Reply> {
    let s = &g.state;
    say(match n {
        None | Some(N::Room) => "The Rusty Anchor: teal walls, sticky floors, and a neon anchor that buzzes like an angry bee. Gus the bartender holds court behind the bar. A jukebox glows in the corner, a regular snores at a table, and the front door leads out to the boardwalk.",
        Some(N::Bartender) => "Gus has an eyepatch, a striped sailor shirt, and a mustache that has seen things. He's polishing a glass. It's been the same glass since 1979.",
        Some(N::Jukebox) if s.flag(Flag::PlayedJukebox) => "The jukebox is lit up like a cruise ship and oozing a smooth saxophone number. It's the most romantic thing in the building, including you.",
        Some(N::Jukebox) => "A dented old jukebox. A hand-lettered card reads: \"25 CENTS. NO REFUNDS. NO POLKA.\" The coin slot looks hungry.",
        Some(N::Matchbook) => "A red matchbook sits on the bar. Something is scribbled on the inside flap. Finders keepers, as they say in polite society.",
        Some(N::Drink) => "Gus's \"menu\" is a chalkboard reading BEER and SPECIAL. Under SPECIAL, someone wrote \"ask nicely.\"",
        Some(N::Egg) => "A big jar of pickled eggs, floating in a cloudy brine. The label says \"BEST BEFORE\" and the rest of the label has dissolved.",
        Some(N::Bar) => "The bar is scarred mahogany and brass. Mostly scars. There's a beer tap, a jar of pickled eggs, and Gus.",
        Some(N::Stool) => "Four vinyl bar stools in a fetching shade of ketchup. Each one wobbles in its own unique way.",
        Some(N::Door) => "The front door, with a brass porthole. Through it, the boardwalk's neon flickers with promise. Or short-circuits.",
        Some(N::Restroom) => "The restroom door is duct-taped shut with a sign: OUT OF ORDER. Someone added in marker: \"SO IS THE CUSTOMER.\"",
        Some(N::Sign) => "A pink neon anchor, buzzing. Half the letters of \"RUSTY ANCHOR\" burned out years ago, which is why the locals call it \"The R   Y ANC  R.\"",
        Some(N::Bottles) => "Rows of bottles behind the bar. Several of the labels are in languages that don't exist. You see yourself in the mirror behind them and, frankly, you're thrilled.",
        Some(N::Patron) => "A regular in a captain's hat, face-down on the table and snoring. His hat says \"CAPTAIN.\" His breath says \"rum.\"",
        Some(N::Table) => "A wobbly round table, currently serving as a pillow for the regular.",
        Some(N::Floor) => "The floor is sticky in a way that suggests a history. There's a puddle you've decided not to think about, and peanut shells crunch underfoot.",
        Some(N::Me) => "Chip Valentine: powder-blue tuxedo, ruffled shirt, an aggressive spray tan and a pompadour you could land a seaplane on. A man in his prime. Which prime is up for debate.",
        Some(N::Quarter) => "You don't have that anymore. The jukebox thanks you for your service.",
        _ => return None,
    })
}

fn take(g: &mut Game, n: Option<N>) -> Option<Reply> {
    match n {
        Some(N::Matchbook) => {
            if !near(&g.state, N::Matchbook) {
                return too_far(&g.state);
            }
            g.state.add(Item::Matchbook);
            let pts = g.state.award(Flag::TookMatchbook, 5);
            scored(Reply::say("You casually palm the matchbook off the bar. Gus doesn't notice. Or he does, and simply doesn't care, which is the Rusty Anchor's motto."), pts)
        }
        Some(N::Egg) => {
            if !near(&g.state, N::Egg) {
                return too_far(&g.state);
            }
            g.state.set(Flag::EggWarned);
            say("As your hand nears the jar, Gus growls: \"Those eggs came with the building, pal. I wouldn't.\" You pull your hand back. For now.")
        }
        Some(N::Drink) => say("There's no drink in front of you. Gus serves people who ask."),
        Some(N::Bartender) => say("Gus is not portable. Gus is barely mobile."),
        Some(N::Patron) => say("Leave the man his dignity. And his hat. Mostly the hat."),
        Some(N::Bottles) => say("You reach for the top shelf. Gus clears his throat in a way that sounds like a boat engine. You reconsider."),
        Some(N::Stool) => say("The stools are bolted down. Gus learned that lesson in 1983."),
        Some(N::Jukebox) | Some(N::Bar) | Some(N::Table) | Some(N::Door) | Some(N::Restroom) => {
            say("That won't fit in your tux. Nothing fits in your tux. That's the point of the tux.")
        }
        Some(N::Sign) => say("It's bolted to the wall and buzzing with enough voltage to curl your hair. Your hair is already perfect."),
        _ => None,
    }
}

fn talk_gus(g: &mut Game, about_special: bool) -> Option<Reply> {
    if !near(&g.state, N::Bartender) {
        return say("Gus can't hear you over the ice machine's death rattle. Step up to the bar.");
    }
    let s = &mut g.state;
    if s.flag(Flag::GotDrink) {
        let n = s.talk(Npc::Gus);
        return say(pick(
            &[
                "Gus: \"Vonda? She sings up at the Crow's Nest, on the roof of the Golden Clam. Classy joint. They make you wear a flower.\"",
                "Gus: \"If you're going up to the Crow's Nest, you'll need a red carnation. The Lucky Gull sells 'em. Bring cash. Brenda doesn't do IOUs.\"",
                "Gus: \"You're still here? Go. Be young. Or at least be younger than me.\"",
            ],
            n - 1,
        ));
    }
    if about_special || s.flag(Flag::ReadMatchbook) {
        return order_drink(g);
    }
    let n = s.talk(Npc::Gus);
    say(pick(
        &[
            "Gus: \"Evening. We got beer, we got the Special, and we got eggs. I only recommend one of those, and it ain't the eggs.\"",
            "Gus: \"Nice tux. Did you lose a bet, or win one?\"",
            "Gus: \"Look, pal, I don't do small talk. I do medium talk, and only on Tuesdays.\" It is not Tuesday.",
        ],
        n - 1,
    ))
}

fn order_drink(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Bartender) {
        return say("You wave at Gus from across the room. Gus waves back with a different number of fingers. Get closer.");
    }
    let s = &mut g.state;
    if s.flag(Flag::GotDrink) {
        return say(
            "Gus: \"One Special per customer. Doctor's orders. Several doctors, actually.\"",
        );
    }
    if !s.flag(Flag::ReadMatchbook) {
        return say("Gus: \"Beer's two bucks.\" You have a lucky quarter and a comb. Gus does not accept combs.");
    }
    if !s.flag(Flag::PlayedJukebox) {
        return say("You lean in: \"Vonda sent me. I'll have the Special.\" Gus squints. \"Vonda, huh? Tell you what: this place is quieter than a tomb with the lid on. Put some music on and the first one's on the house.\"");
    }
    s.add(Item::Drink);
    let pts = s.award(Flag::GotDrink, 5);
    scored(Reply::say("Gus nods along to the saxophone. \"Now THAT'S music.\" He slides you a Bilge Water Special in a glass shaped like a mermaid. \"On the house. Vonda's friends drink free, once.\""), pts)
}

fn use_thing(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let (a, b) = (cmd.noun(), cmd.second());
    let mentions_quarter = cmd.mentions(N::Quarter);
    if cmd.mentions(N::Jukebox) {
        if cmd.verb == V::Give && !mentions_quarter {
            return say(
                "The jukebox only accepts quarters. And compliments, but those don't make it play.",
            );
        }
        if !g.state.has(Item::Quarter) {
            return if g.state.flag(Flag::PlayedJukebox) {
                say("The jukebox is already playing. You've spent your fortune, all twenty-five cents of it.")
            } else {
                say("The jukebox wants a quarter. You've got nothing but good intentions.")
            };
        }
        if !near(&g.state, N::Jukebox) {
            return too_far(&g.state);
        }
        g.state.remove(Item::Quarter);
        let pts = g.state.award(Flag::PlayedJukebox, 5);
        return scored(Reply::say("You flip your lucky quarter into the slot. The jukebox shudders, lights up like a cruise ship, and oozes a slow saxophone number that sounds like it was recorded in a hot tub. Chip does finger guns at no one. Behind the bar, Gus almost smiles."), pts);
    }
    match (cmd.verb, a, b) {
        (V::Give, _, _) if mentions_quarter && cmd.mentions(N::Bartender) => say(
            "Gus looks at your quarter. \"What is this, 1962?\" He pushes it back across the bar.",
        ),
        (_, Some(N::Quarter), Some(N::Egg)) => {
            say("The eggs are not for sale. They are not, legally speaking, food.")
        }
        (_, Some(N::Restroom), _) => handle(g, &Command::new(V::Open, &[N::Restroom])),
        (_, Some(N::Stool), _) => handle(g, &Command::new(V::Sit, &[N::Stool])),
        _ => None,
    }
}

fn eat_egg(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Egg) {
        return too_far(&g.state);
    }
    if !g.state.flag(Flag::EggWarned) {
        g.state.set(Flag::EggWarned);
        return say("Gus slaps the lid on the jar. \"Pal. Those eggs came with the building. The LAST guy who ate one is why we have the plaque.\" You decide to think about it.");
    }
    g.die(
        DeathKind::Egg,
        vec![
            "Ignoring every instinct you have, plus a few you borrowed, you fish out a pickled egg and pop it in your mouth.",
            "It's crunchy. Eggs should not be crunchy. Your eyes water, your knees buckle, and your pompadour finally, tragically, deflates.",
        ],
    )
}
