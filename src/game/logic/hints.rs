//! The HINT command: progressive nudges, one objective at a time.
//!
//! Each objective belongs to a scene, has a "done" test, a saved counter
//! slot, and three hints that go from a nudge to a near-solution. Asking
//! again about the same objective moves to the next hint; the counters are
//! part of the save file. If everything in the current area is done, HINT
//! points to the next area that still has something to do.

use super::Reply;
use crate::game::state::{Flag, GameState, SceneId};

struct Objective {
    scene: SceneId,
    /// Counter slot in `GameState::hints` (must be unique and < HINT_SLOTS).
    slot: usize,
    flag: Flag,
    hints: [&'static str; 3],
}

use SceneId as S;

/// In rough play order, so "what next?" can find the first unfinished one.
fn objectives() -> Vec<Objective> {
    let o = |scene, slot, flag, hints| Objective {
        scene,
        slot,
        flag,
        hints,
    };
    vec![
        o(S::Bar, 0, Flag::TookMatchbook, [
            "Something small and red is sitting on the bar. Sloppy bartenders leave all kinds of things lying around.",
            "Look at the bar, then at the matchbook. Gentlemen collect matchbooks.",
            "Walk up to the bar and TAKE MATCHBOOK.",
        ]),
        o(S::Bar, 1, Flag::ReadMatchbook, [
            "A matchbook is only half a story. The other half is inside the flap.",
            "Have you tried reading what's written inside?",
            "READ MATCHBOOK.",
        ]),
        o(S::Bar, 2, Flag::PlayedJukebox, [
            "Gus won't mix anything fancy in a silent room. The jukebox looks hungry.",
            "A jukebox runs on coins. Chip has exactly one lucky coin.",
            "USE QUARTER ON JUKEBOX.",
        ]),
        o(S::Bar, 3, Flag::GotDrink, [
            "Vonda's matchbook mentioned a special drink. Gus mixes it, but only for a gentleman who asks.",
            "Talk to Gus, and ask about the Special. Be polite, and make sure there's music.",
            "Stand at the bar and ASK GUS ABOUT THE SPECIAL.",
        ]),
        o(S::Bar, 4, Flag::DrankDrink, [
            "Courage doesn't do any good in a glass on the counter.",
            "Whatever Gus handed you, the roof is no place to be sober.",
            "DRINK SPECIAL, before you go up to the Crow's Nest.",
        ]),
        o(S::Boardwalk, 5, Flag::UsedTelescope, [
            "The coin-op telescope at the end of the railing has a view. A nice one.",
            "You can look through the telescope. You might spot a certain sequined someone on the casino roof.",
            "Walk to the telescope at the east end of the railing, and LOOK THROUGH TELESCOPE.",
        ]),
        o(S::Boardwalk, 6, Flag::GotHotDog, [
            "Dot's hot dogs are two dollars, which you don't have. But Dot is a woman who values the finer things, like sweet talk.",
            "Try telling Dot something nice. She lights up for compliments.",
            "Stand at Dot's cart and COMPLIMENT DOT. Or FLIRT WITH DOT.",
        ]),
        o(S::Alley, 7, Flag::MimeFed, [
            "That mime looks awfully hungry, and he's rooted to one spot. Maybe he's standing on something.",
            "Dot is selling something the mime would love. A gentleman shares.",
            "Bring a hot dog to the mime: GIVE HOT DOG TO MIME.",
        ]),
        o(S::Alley, 8, Flag::GotFiver, [
            "Something green and papery is wedged in the storm grate. Fingers won't fit.",
            "A thin, toothy tool in Chip's pocket might. It has kept his pompadour in check for years.",
            "USE COMB ON GRATE.",
        ]),
        o(S::Store, 9, Flag::BoughtCarnation, [
            "Chip needs a flower for his lapel, if he's going to get past the doorman. Brenda sells them.",
            "Carnations cost $3. You'll need to have fished that money out of somewhere first.",
            "Stand at the counter and BUY CARNATION.",
        ]),
        o(S::Store, 10, Flag::BoughtMints, [
            "Bilge Water on your breath is going to need a chaser. The store sells just the thing.",
            "Mints cost $2, and the Arctic Blast tin is by the counter.",
            "BUY MINTS.",
        ]),
        o(S::Casino, 11, Flag::WearingCarnation, [
            "That carnation is just sitting in your pocket, being wasted.",
            "Where do gentlemen wear flowers? Think lapel.",
            "WEAR CARNATION.",
        ]),
        o(S::Casino, 12, Flag::TinyApproved, [
            "The doorman guards the elevator. He's big, but he has a soft spot.",
            "Tiny hates people who don't dress the part. You do.",
            "TALK TO TINY while wearing the carnation.",
        ]),
        o(S::Rooftop, 13, Flag::AteMints, [
            "Chip's breath could strip paint. The roof is a place where people get close.",
            "You bought something that fixes bad breath.",
            "EAT MINTS, then talk to Vonda.",
        ]),
        o(S::Rooftop, 14, Flag::Sang, [
            "Vonda is waiting. A gentleman needs courage and fresh breath first.",
            "You need to have drunk the Special and eaten the mints. Then talk to her.",
            "SING TO VONDA after DRINK SPECIAL and EAT MINTS.",
        ]),
        o(S::Laundromat, 15, Flag::DeeBriefed, [
            "Dee, the laundromat attendant, only talks business with customers who have a ticket. Vonda may have given you something.",
            "Talk to Dee at the folding table, with the claim ticket in your pocket.",
            "TALK TO DEE, or GIVE TICKET TO DEE.",
        ]),
        o(S::Motel, 16, Flag::FerrisAway, [
            "Ferris, the night manager, has a problem with her hair and a problem with her breakfast tray. They may be related.",
            "Talk to Ferris a couple of times. A comb in the right hands solves a lot of things.",
            "USE COMB ON FERRIS. She'll leave her desk to admire her handiwork.",
        ]),
        o(S::Motel, 17, Flag::GotShrimp, [
            "The continental breakfast is for guests only. Guests, and people who ask nicely while the manager is away.",
            "With Ferris out of the way, the shrimp tray is yours for the taking. Don't eat it.",
            "TAKE SHRIMP, once Ferris has left the desk.",
        ]),
        o(S::Pier, 18, Flag::GotRod, [
            "The bait shack's closed, but it left something for borrowers.",
            "There's a rod leaning on the shack's wall. Mort says to bring it back. Mort's in Florida.",
            "Walk to the shack and TAKE ROD.",
        ]),
        o(S::Pier, 19, Flag::CaughtFlounder, [
            "A rod and the sea. Fish are picky, though. They want something that smells.",
            "Stand at the front railing with the rod and the shrimp from the motel. Cast away.",
            "Walk to the railing and USE ROD (you must be holding the shrimp). It's bait, not lunch.",
        ]),
        o(S::Pier, 20, Flag::GotKey, [
            "Percival the pelican has something in his pouch. He won't give it up for conversation.",
            "A pelican trades fish for treasure. The fish needs to be a flounder.",
            "GIVE FLOUNDER TO PELICAN. He roams the pier, so walk up to him.",
        ]),
        o(S::Laundromat, 21, Flag::DryerOpen, [
            "Dryer 3 has a lock, and a gown behind it. Dee won't open it for you, but she won't stop you either.",
            "A little brass key from a bird would open it. You can't unlock it without Dee's briefing.",
            "Stand at Dryer 3 and USE KEY ON DRYER. (Do not climb in afterwards.)",
        ]),
        o(S::Rooftop, 22, Flag::Won, [
            "Vonda's midnight set is about to start. Something is missing from her act.",
            "You've got something sequined, gold, and warm from a dryer. She's been waiting.",
            "Take the gown to the Crow's Nest and TALK TO VONDA, or GIVE GOWN TO VONDA.",
        ]),
    ]
}

/// The next unfinished objective that belongs to `scene`.
fn first_undone(s: &GameState, scene: Option<SceneId>) -> Option<Objective> {
    objectives()
        .into_iter()
        .find(|o| scene.is_none_or(|sc| o.scene == sc) && !s.flag(o.flag))
}

pub(super) fn hint(g: &mut super::Game) -> Reply {
    let s = &mut g.state;
    if let Some(o) = first_undone(s, Some(s.scene)) {
        let level = s.hints[o.slot].min(2);
        s.hints[o.slot] = level + 1;
        return Reply::say(format!(
            "Hint ({}/3): {}",
            level + 1,
            o.hints[level as usize]
        ));
    }
    match first_undone(s, None) {
        Some(o) => Reply::say(format!(
            "There's nothing more to do here for now. Try {}. (Ask again there for a nudge.)",
            o.scene.name()
        )),
        None => Reply::say("You've done absolutely everything. There's nothing left to hint at, and nothing left to learn. Go enjoy your evening, Chip."),
    }
}

#[cfg(test)]
pub(super) fn objective_slots() -> Vec<usize> {
    objectives().iter().map(|o| o.slot).collect()
}
