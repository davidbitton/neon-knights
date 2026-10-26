//! Suds & Duds: Dee, a lint bin, and Dryer 3.

use super::{near, pick, say, scored, too_far, Game, Reply};
use crate::game::parser::{Command, Noun as N, Verb as V};
use crate::game::state::{DeathKind, Flag, Item, Npc};

pub const INTRO: &str = "Beyond the alley's west end, a buzzing neon sign: SUDS & DUDS, OPEN 24 HRS. Inside, a wall of washers, a wall of dryers, a folding table, and a woman in curlers folding a towel into a smaller and smaller square. The air smells like warm cotton and unresolved issues.";

pub(super) fn handle(g: &mut Game, cmd: &Command) -> Option<Reply> {
    let n = cmd.noun();
    let open = g.state.flag(Flag::DryerOpen);
    match (cmd.verb, n) {
        (V::Look, None | Some(N::Room) | Some(N::Laundromat)) => say("Suds & Duds: washers along the left wall, four columns of dryers on the right, and a folding table in the middle where Dee guards the quarter-change like a dragon. One dryer, number 3, has an OUT OF ORDER sign. The alley is back to the east."),
        (V::Look, Some(N::Dee)) => say("Dee: curlers the size of soup cans, a smock with a stain-shaped map of the western hemisphere, and the folding technique of a surgeon who hates her patients. She hasn't looked up at you yet. She will, when she's ready."),
        (V::Talk | V::Flirt, Some(N::Dee)) => dee(g, false),
        (V::Give | V::Use | V::Look, _) if cmd.mentions(N::Ticket) && g.state.has(Item::Ticket) => dee(g, true),
        (V::Give, _) if cmd.mentions(N::Dee) => say("Dee eyes your offering, shakes her head, and goes back to her towel. \"Only thing I take is quarters and facts.\""),
        (V::Look, Some(N::Sign)) => say(if open { "The OUT OF ORDER sign now sits on a perfectly working dryer, which makes it the most honest sign in town." } else { "OUT OF ORDER, in red marker, taped crooked, and underlined twice. Behind it, through the round window, something sequined winks at you in the dark." }),
        (V::Look, Some(N::Dryer)) => say(if open { "Dryer 3 hangs open, empty and warm, with a faint smell of toasted sequin. The other dryers hum along, tumbling other people's secrets." } else { "A wall of dryers, humming. Dryer 3 is the odd one out: sealed with a little brass lock and an OUT OF ORDER sign. Through its round window, something gold and sequined glitters like a tiny disco." }),
        (V::Open | V::Use | V::Take, Some(N::Dryer)) | (V::Use, Some(N::Key)) if g.state.has(Item::Key) && !open => unlock(g),
        (V::Open | V::Use | V::Search, Some(N::Dryer | N::Sign)) if !open => say("Dryer 3 is sealed with a little brass lock. You tug the door. It tugs back, with the confidence of a thing that has never lost an argument. You'd need the key."),
        (V::Open | V::Use | V::Search, Some(N::Dryer)) => say("Dryer 3 is open and empty. The others are busy with someone else's socks."),
        (V::Climb | V::Go | V::Jump | V::Sit, Some(N::Dryer)) => dryer_death(g),
        (V::Look, Some(N::Washer)) => say("A row of front-loaders, with a window full of suds that's been going around since you walked in. One has a note: \"EATS SOCKS. DON'T ASK.\""),
        (V::Use | V::Open | V::Search, Some(N::Washer)) => say("You have no laundry. Chip's tux is dry clean only, and so is his conscience. Dee glances at you, then at the washer, then at you. You step back."),
        (V::Look, Some(N::Lint)) => say("A rolling bin overflowing with lint: gray, soft, and shaped, in places, like small unsolved mysteries."),
        (V::Search | V::Take | V::Use | V::Open, Some(N::Lint)) => say("You dig through the lint. You find a striped sock, a button, and an old, apologetic receipt. Nothing you need. You wipe your hands, which only makes it worse."),
        (V::Look, Some(N::Floor)) => say("A black-and-white linoleum checkerboard that has been tragically, repeatedly mopped. It has never been as clean as it is pretending to be."),
        (V::Look, Some(N::Alley)) => say("East, through the door, the alley and its single flickering bulb. You can see the mime's silhouette. Maybe."),
        (V::Look, Some(N::City)) => say("The ceiling is a stained acoustic-tile sky with one working fluorescent star."),
        (V::Smell, _) => say("Fabric softener, warm lint, and the baked-in scent of ten thousand wash cycles. Not bad, really. For a place where people take off their pants for strangers."),
        (V::Sit, _) => say("You sit in an orange plastic chair. Somewhere in the walls, a dryer thumps in sympathy. You get up before it asks for your number."),
        (V::Use | V::Play, Some(N::Quarter)) if cmd.mentions(N::Dryer) || cmd.mentions(N::Washer) => say("Your lucky quarter has never done laundry in its life. You decide that streak is worth preserving."),
        _ => None,
    }
}

fn dee(g: &mut Game, offering: bool) -> Option<Reply> {
    if !near(&g.state, N::Dee) {
        return say("Dee keeps folding. \"If you want to talk to me, honey, come over to the table. I don't shout across a laundromat. I have a reputation.\"");
    }
    let t = g.state.talk(Npc::Dee);
    let s = &mut g.state;
    if s.flag(Flag::DeeBriefed) {
        let has_key = s.has(Item::Key);
        return say(if has_key {
            "Dee: \"Well, look at that. A brass key, and not a scratch on it. Go ahead and use it on Dryer 3. I didn't see anything. I never do.\""
        } else {
            "Dee: \"The key's in Percival's pouch. The only thing that bird respects is flounder, and the only way to get a flounder is to have something to catch it with and something it'd bite. Try the pier. And the motel. They give out shrimp.\""
        });
    }
    if s.has(Item::Ticket) {
        let pts = s.award(Flag::DeeBriefed, 2);
        return scored(Reply::many(vec![
            "You slide the claim ticket across the folding table. Dee looks at it for a long time, then at you, then at the ticket again.",
            "\"Vonda's gown. Gold lamé. Nine thousand sequins, give or take. It's in Dryer 3, and Dryer 3 is out of order, because Percival the pelican swiped the key off my hook two weeks ago. He thinks it's a fish. He keeps it in his pouch. The only thing he'll trade it for is a flounder.\" She sighs. \"Get the key, bring it here, and I'll look the other way while you open it. I'm not allowed to open it for you. Union rules.\"",
        ]), pts);
    }
    if offering {
        return None;
    }
    say(pick(
        &[
            "Dee doesn't look up from her towel. \"Wash, dry, or fold, honey. Pick one. Or pick a different place to be.\" She folds the towel into a square so tight it might collapse into a black hole.",
            "\"You look like a man who's never done his own laundry,\" says Dee. \"And you smell like a man who's never done anyone else's.\" A pause. \"Dryer 3's out of order. Don't ask. If you've got a claim ticket, I'll talk. If you don't, I won't.\"",
            "Dee glances at your tux, then your face, then your tux. \"Dry clean only. I can tell. I can ALWAYS tell. Come back with a claim ticket if you've got business with me.\"",
        ],
        t - 1,
    ))
}

fn unlock(g: &mut Game) -> Option<Reply> {
    if !near(&g.state, N::Dryer) {
        return too_far(&g.state);
    }
    if !g.state.flag(Flag::DeeBriefed) {
        return say("You jam the brass key toward the lock. \"HEY!\" Dee, from across the room, doesn't even look up. \"That's somebody's laundry. Ticket. Then key. Order matters here.\"");
    }
    g.state.add(Item::Gown);
    g.state.set(Flag::GotGown);
    let pts = g.state.award(Flag::DryerOpen, 3);
    scored(Reply::many(vec![
        "The brass key slides into the lock like it was made for it. Because it was. The door swings open on a rush of warm air, and out tumbles an avalanche of gold lamé and approximately nine thousand sequins. You catch it in both arms. It's still toasty.",
        "Dee, across the room, doesn't look up. \"I didn't see a thing,\" she says. \"Tell Vonda the dry cleaning's on the house. And that she owes me forty bucks.\"",
    ]), pts)
}

fn dryer_death(g: &mut Game) -> Option<Reply> {
    if !g.state.flag(Flag::DryerOpen) {
        return say("Dryer 3 is locked tight, and the others are full of someone else's underwear. Even Chip has limits. Probably.");
    }
    if !near(&g.state, N::Dryer) {
        return too_far(&g.state);
    }
    g.die(
        DeathKind::Dryer,
        vec![
            "The open dryer is warm, soft, and smells like a hug. You climb in to see what Vonda's gown felt, curl up in the drum, and pull the door shut behind you. Just for a second.",
            "Across the laundromat, Dee notices a customer with a quarter, a free dryer, and a mean streak. She sets the dial to HIGH, the cycle to 90 minutes, and feeds the machine. Chip tumbles, round and round, his tux shrinking by the minute. When it's done, he's a very small, very warm, very wrinkled man. Permanent press, indeed.",
        ],
    )
}
