use super::*;
use crate::game::state::{DeathKind, Npc};
use crate::game::world::hotspot_for;

/// Stand at a hotspot's approach point, as click-to-walk would.
fn go(g: &mut Game, noun: Noun) {
    g.state.hero = hotspot_for(&g.state, noun)
        .unwrap_or_else(|| panic!("no {noun:?} in {:?}", g.state.scene))
        .approach;
}

/// Walk into the exit leading to `to` (by its zone) and let the engine fire it.
fn walk_to_scene(g: &mut Game, to: SceneId) -> Reply {
    let exit = world::exits(g.state.scene)
        .iter()
        .find(|e| e.to == to)
        .expect("exit exists");
    let z = exit.zone;
    g.state.hero = ((z.x0 + z.x1) / 2, (z.y0 + z.y1) / 2);
    let r = g.check_exits().expect("exit fires");
    assert_eq!(r.effect, Effect::Moved);
    r
}

fn scored(r: &Reply, pts: u32) {
    assert_eq!(r.effect, Effect::Scored(pts), "{:?}", r.messages);
}

fn died(r: &Reply, kind: DeathKind) {
    match &r.effect {
        Effect::Died(d) => {
            assert_eq!(d.kind, kind);
            assert_eq!(d.epitaph[0], "HERE LIES CHIP VALENTINE");
            assert!(!d.cause.is_empty());
        }
        e => panic!("expected death {kind:?}, got {e:?} {:?}", r.messages),
    }
}

// ---------------------------------------------------------------- M1 bar tests

#[test]
fn unknown_words_get_snark() {
    let mut g = Game::new();
    let r = g.input("frobnicate the bartender");
    assert_eq!(r.messages.len(), 1);
    assert!(r.messages[0].contains("frobnicate"));
    assert_eq!(r.effect, Effect::Confused);
    assert_eq!(g.input("jukebox").effect, Effect::Confused);
    assert_eq!(g.input("look").effect, Effect::None);
}

#[test]
fn look_works_from_anywhere() {
    let mut g = Game::new();
    assert!(g.input("look at bartender").messages[0].contains("eyepatch"));
    assert!(g.input("l").messages[0].contains("Rusty Anchor"));
}

#[test]
fn taking_requires_proximity_and_scores_once() {
    let mut g = Game::new();
    let r = g.input("take matchbook");
    assert!(TOO_FAR.contains(&r.messages[0].as_str()));
    go(&mut g, Noun::Matchbook);
    scored(&g.input("get the matches"), 5);
    assert_eq!(g.input("take matchbook").effect, Effect::None);
    assert_eq!(g.state.score, 5);
    assert!(g.input("i").messages[0].contains("matchbook"));
}

#[test]
fn jukebox_needs_quarter() {
    let mut g = Game::new();
    go(&mut g, Noun::Jukebox);
    g.state.remove(Item::Quarter);
    assert!(g.input("play jukebox").messages[0].contains("quarter"));
    assert!(!g.state.flag(Flag::PlayedJukebox));
}

#[test]
fn meta_commands_produce_effects() {
    let mut g = Game::new();
    assert_eq!(g.input("save").effect, Effect::Save);
    assert_eq!(g.input("restore game").effect, Effect::Restore);
    assert_eq!(g.input("restart").effect, Effect::Restart);
    assert_eq!(g.input("quit").effect, Effect::Quit);
    assert!(g.input("help").messages[0].contains("INVENTORY"));
    assert!(g.input("inventory").messages[0].contains("lucky quarter"));
}

#[test]
fn noun_without_verb_and_empty() {
    let mut g = Game::new();
    assert!(g.input("jukebox").messages[0].contains("verb"));
    assert!(!g.input("").messages[0].is_empty());
}

#[test]
fn talking_far_away_is_refused() {
    let mut g = Game::new();
    assert!(g.input("talk to gus").messages[0].contains("Step up"));
    go(&mut g, Noun::Bartender);
    assert!(g.input("talk to gus").messages[0].starts_with("Gus:"));
}

// ---------------------------------------------------------------- walkthrough

/// The complete solution, following WALKTHROUGH.md, mixing walking through
/// exits and typed travel. Every point is earned exactly once.
#[test]
fn full_walkthrough_reaches_max_score_and_ending() {
    let mut g = Game::new();
    // The Rusty Anchor.
    go(&mut g, Noun::Matchbook);
    scored(&g.input("take matchbook"), 5);
    scored(&g.input("read matchbook"), 3);
    go(&mut g, Noun::Bartender);
    assert!(g.input("order the special").messages[0].contains("music"));
    go(&mut g, Noun::Jukebox);
    scored(&g.input("use quarter on jukebox"), 5);
    go(&mut g, Noun::Bartender);
    scored(&g.input("ask gus about the special"), 5);
    scored(&g.input("drink it"), 2);
    assert_eq!(g.state.score, 20);

    // Out the front door to the boardwalk.
    let r = walk_to_scene(&mut g, SceneId::Boardwalk);
    assert_eq!(g.state.scene, SceneId::Boardwalk);
    assert!(r.messages[0].contains("boardwalk"));
    go(&mut g, Noun::Telescope);
    scored(&g.input("look through telescope"), 2);
    go(&mut g, Noun::Dot);
    g.input("talk to dot");
    scored(&g.input("compliment dot"), 3);
    assert!(g.state.has(Item::HotDog));

    // The alley: feed the mime, fish out the fiver.
    walk_to_scene(&mut g, SceneId::Alley);
    go(&mut g, Noun::Grate);
    assert!(g.input("use comb on grate").messages[0].contains("mime"));
    go(&mut g, Noun::Mime);
    scored(&g.input("give hot dog to mime"), 5);
    assert!(!g.state.has(Item::HotDog));
    go(&mut g, Noun::Grate);
    scored(&g.input("use comb on grate"), 5);
    assert_eq!(g.state.cash, 5);

    // Shopping, via typed travel.
    assert_eq!(g.input("go to boardwalk").effect, Effect::Moved);
    assert_eq!(g.input("enter store").effect, Effect::Moved);
    assert_eq!(g.state.scene, SceneId::Store);
    go(&mut g, Noun::Brenda);
    scored(&g.input("buy carnation"), 3);
    scored(&g.input("buy mints"), 3);
    assert_eq!(g.state.cash, 0);

    // The casino.
    walk_to_scene(&mut g, SceneId::Boardwalk);
    walk_to_scene(&mut g, SceneId::Casino);
    assert_eq!(g.state.scene, SceneId::Casino);
    scored(&g.input("wear carnation"), 2);
    go(&mut g, Noun::Tiny);
    scored(&g.input("talk to tiny"), 5);
    walk_to_scene(&mut g, SceneId::Rooftop);
    assert_eq!(g.state.scene, SceneId::Rooftop);

    // The Crow's Nest: Vonda sets the quest.
    scored(&g.input("eat mints"), 2);
    go(&mut g, Noun::Vonda);
    scored(&g.input("sing to vonda"), 12);
    assert!(g.state.has(Item::Ticket));
    assert_eq!(g.state.score, 62);

    // Suds & Duds: Dee briefs us on the pelican.
    walk_to_scene(&mut g, SceneId::Casino);
    walk_to_scene(&mut g, SceneId::Boardwalk);
    walk_to_scene(&mut g, SceneId::Alley);
    walk_to_scene(&mut g, SceneId::Laundromat);
    assert_eq!(g.state.scene, SceneId::Laundromat);
    go(&mut g, Noun::Dee);
    scored(&g.input("give ticket to dee"), 2);

    // Motel: comb Ferris, pocket the shrimp.
    walk_to_scene(&mut g, SceneId::Alley);
    walk_to_scene(&mut g, SceneId::Boardwalk);
    walk_to_scene(&mut g, SceneId::Pier);
    go(&mut g, Noun::Rod);
    scored(&g.input("take rod"), 2);
    walk_to_scene(&mut g, SceneId::Motel);
    go(&mut g, Noun::Ferris);
    assert_eq!(g.input("use comb on ferris").effect, Effect::None);
    assert!(g.state.flag(Flag::FerrisAway));
    go(&mut g, Noun::Shrimp);
    scored(&g.input("take shrimp"), 2);

    // Pier: flounder, then the key.
    walk_to_scene(&mut g, SceneId::Pier);
    go(&mut g, Noun::Ocean);
    scored(&g.input("use rod"), 3);
    assert!(g.state.has(Item::Flounder));
    go(&mut g, Noun::Pelican);
    scored(&g.input("give flounder to pelican"), 4);
    assert!(g.state.has(Item::Key));

    // Back to Dryer 3 and the gown.
    walk_to_scene(&mut g, SceneId::Boardwalk);
    walk_to_scene(&mut g, SceneId::Alley);
    walk_to_scene(&mut g, SceneId::Laundromat);
    go(&mut g, Noun::Dryer);
    scored(&g.input("use key on dryer"), 3);
    assert!(g.state.has(Item::Gown));
    assert_eq!(g.state.score, 78);

    // Home to the Crow's Nest.
    walk_to_scene(&mut g, SceneId::Alley);
    walk_to_scene(&mut g, SceneId::Boardwalk);
    walk_to_scene(&mut g, SceneId::Casino);
    walk_to_scene(&mut g, SceneId::Rooftop);
    go(&mut g, Noun::Vonda);
    let r = g.input("give gown to vonda");
    assert_eq!(r.effect, Effect::Won);
    assert!(r.messages.last().unwrap().contains("hired"));
    assert_eq!(g.state.score, MAX_SCORE);
    assert!(g.state.flag(Flag::Won));
    let end = g.ending_lines().join("\n");
    assert!(end.contains(&format!("Final score: {MAX_SCORE} of {MAX_SCORE}")));
    assert!(end.contains("Legend of the Second Night"));
    // Nothing more to do; the game refuses further input.
    assert!(g.input("look").messages[0].contains("Restart"));
}

#[test]
fn points_are_awarded_only_once() {
    let mut g = Game::new();
    g.state.scene = SceneId::Boardwalk;
    go(&mut g, Noun::Telescope);
    scored(&g.input("use telescope"), 2);
    assert_eq!(g.input("use telescope").effect, Effect::None);
    go(&mut g, Noun::Dot);
    scored(&g.input("flirt with dot"), 3);
    assert_eq!(g.input("flirt with dot").effect, Effect::None);
    assert_eq!(g.state.score, 5);
    // Wearing twice doesn't pay twice.
    g.state.add(Item::Carnation);
    scored(&g.input("wear carnation"), 2);
    assert_eq!(g.input("wear carnation").effect, Effect::None);
}

#[test]
fn vonda_needs_courage_and_fresh_breath() {
    let mut g = Game::new();
    g.state.scene = SceneId::Rooftop;
    go(&mut g, Noun::Vonda);
    assert!(g.input("talk to vonda").messages[0].contains("liquid courage"));
    g.state.set(Flag::DrankDrink);
    assert!(g.input("talk to vonda").messages[0].contains("bilge water"));
    g.state.add(Item::Mints);
    g.input("eat mints");
    // The first song earns the claim ticket and sends Chip off to do laundry.
    scored(&g.input("talk to vonda"), 12);
    assert!(g.state.has(Item::Ticket));
    let r = g.input("talk to vonda");
    assert_eq!(r.effect, Effect::None);
    assert!(r.messages[0].contains("Dryer 3"));
    assert!(!g.state.flag(Flag::Won));
    // Only the gown wins her over.
    g.state.add(Item::Gown);
    assert_eq!(g.input("talk to vonda").effect, Effect::Won);
}

#[test]
fn store_needs_cash_and_proximity() {
    let mut g = Game::new();
    g.state.scene = SceneId::Store;
    go(&mut g, Noun::Brenda);
    assert!(g.input("buy carnation").messages[0].contains("$3"));
    assert!(!g.state.has(Item::Carnation));
    g.state.cash = 5;
    g.state.hero = (160, 160);
    assert!(TOO_FAR.contains(&g.input("buy mints").messages[0].as_str()));
    go(&mut g, Noun::Mints);
    scored(&g.input("buy mints"), 3);
    assert_eq!(g.state.cash, 3);
}

#[test]
fn grate_needs_the_comb() {
    let mut g = Game::new();
    g.state.scene = SceneId::Alley;
    g.state.set(Flag::MimeFed);
    g.state.remove(Item::Comb);
    go(&mut g, Noun::Grate);
    assert!(g.input("search grate").messages[0].contains("teeth"));
    assert_eq!(g.state.cash, 0);
}

// ---------------------------------------------------------------- deaths

#[test]
fn egg_death_after_warning() {
    let mut g = Game::new();
    go(&mut g, Noun::Egg);
    assert_eq!(g.input("eat egg").effect, Effect::None);
    assert!(!g.state.dead);
    died(&g.input("eat the pickled egg"), DeathKind::Egg);
    assert!(g.state.dead);
    assert!(g.input("look").messages[0].contains("dead"));
}

#[test]
fn every_death_is_reachable() {
    let mut seen = vec![];
    // Egg.
    let mut g = Game::new();
    go(&mut g, Noun::Egg);
    g.input("eat egg");
    let r = g.input("eat egg");
    died(&r, DeathKind::Egg);
    seen.push(DeathKind::Egg);

    // Dumpster: walk from the bar to the alley.
    let mut g = Game::new();
    walk_to_scene(&mut g, SceneId::Boardwalk);
    walk_to_scene(&mut g, SceneId::Alley);
    go(&mut g, Noun::Dumpster);
    assert_eq!(g.input("open dumpster").effect, Effect::None);
    died(&g.input("climb into dumpster"), DeathKind::Dumpster);
    seen.push(DeathKind::Dumpster);

    // Slushie.
    let mut g = Game::new();
    g.input("leave");
    g.input("enter store");
    go(&mut g, Noun::Slushie);
    died(&g.input("drink slushie"), DeathKind::Slushie);
    seen.push(DeathKind::Slushie);

    // Jackpot: the lucky quarter is enough.
    let mut g = Game::new();
    g.input("leave");
    g.input("go to casino");
    assert_eq!(g.state.scene, SceneId::Casino);
    go(&mut g, Noun::Slots);
    died(&g.input("play slots"), DeathKind::Jackpot);
    seen.push(DeathKind::Jackpot);
    // ...but with empty pockets the machine just sneers.
    let mut g = Game::new();
    g.state.scene = SceneId::Casino;
    g.state.remove(Item::Quarter);
    go(&mut g, Noun::Slots);
    assert_eq!(g.input("play slots").effect, Effect::None);

    // Pool.
    let mut g = Game::new();
    g.state.scene = SceneId::Rooftop;
    go(&mut g, Noun::Pool);
    died(&g.input("dive into pool"), DeathKind::Pool);
    seen.push(DeathKind::Pool);

    // Dryer: climb into Dryer 3 once it's open.
    let mut g = Game::new();
    g.state.scene = SceneId::Laundromat;
    go(&mut g, Noun::Dryer);
    assert!(g.input("climb into dryer").messages[0].contains("locked"));
    g.state.set(Flag::DryerOpen);
    died(&g.input("climb into dryer"), DeathKind::Dryer);
    seen.push(DeathKind::Dryer);

    // Shrimp: eat the pocket shrimp anywhere...
    let mut g = Game::new();
    g.state.add(Item::Shrimp);
    died(&g.input("eat shrimp"), DeathKind::Shrimp);
    // ...or the tray, once Ferris isn't watching.
    let mut g = Game::new();
    g.state.scene = SceneId::Motel;
    go(&mut g, Noun::Shrimp);
    assert_eq!(g.input("eat shrimp").effect, Effect::None);
    g.state.set(Flag::FerrisAway);
    died(&g.input("eat the shrimp"), DeathKind::Shrimp);
    seen.push(DeathKind::Shrimp);

    assert_eq!(seen, DeathKind::ALL.to_vec());
    // Each death has a distinct epitaph.
    let mut eps: Vec<_> = DeathKind::ALL.iter().map(|k| death(*k).epitaph).collect();
    eps.dedup();
    assert_eq!(eps.len(), DeathKind::ALL.len());
}

// ---------------------------------------------------------------- transitions

#[test]
fn scene_transitions_round_trip() {
    let mut g = Game::new();
    let r = g.input("leave");
    assert_eq!(r.effect, Effect::Moved);
    assert_eq!(g.state.scene, SceneId::Boardwalk);
    assert!(!r.messages.is_empty(), "first visit prints an intro");
    // Second visit: no intro.
    walk_to_scene(&mut g, SceneId::Bar);
    assert_eq!(g.state.scene, SceneId::Bar);
    let r = walk_to_scene(&mut g, SceneId::Boardwalk);
    assert!(r.messages.is_empty());
    for (to, back) in [
        (SceneId::Alley, "boardwalk"),
        (SceneId::Store, "boardwalk"),
        (SceneId::Casino, "boardwalk"),
    ] {
        walk_to_scene(&mut g, to);
        assert_eq!(g.state.scene, to);
        assert!(g.input("look").messages[0].to_lowercase().contains(back));
        walk_to_scene(&mut g, SceneId::Boardwalk);
    }
    for s in [
        SceneId::Bar,
        SceneId::Boardwalk,
        SceneId::Alley,
        SceneId::Store,
        SceneId::Casino,
    ] {
        assert!(g.state.flag(s.visited_flag()));
    }
    assert!(g.input("go to rooftop").messages[0].contains("can't get there"));
    assert!(g.input("go to boardwalk").messages[0].contains("already here"));
}

#[test]
fn tiny_guards_the_elevator() {
    let mut g = Game::new();
    g.state.scene = SceneId::Casino;
    let r = walk_to_scene(&mut g, SceneId::Rooftop);
    assert_eq!(g.state.scene, SceneId::Casino);
    assert!(r.messages[0].contains("dress code"));
    // Pushed back out of the exit zone so he doesn't loop.
    assert!(g.check_exits().is_none());
    // A carnation in the pocket doesn't count.
    g.state.add(Item::Carnation);
    go(&mut g, Noun::Tiny);
    assert!(g.input("talk to tiny").messages[0].contains("lapel"));
    g.input("wear carnation");
    walk_to_scene(&mut g, SceneId::Rooftop);
    assert_eq!(g.state.scene, SceneId::Casino, "must ask first");
    go(&mut g, Noun::Tiny);
    scored(&g.input("talk to tiny"), 5);
    assert_eq!(g.input("take elevator").effect, Effect::Moved);
    assert_eq!(g.state.scene, SceneId::Rooftop);
    assert_eq!(g.input("take elevator").effect, Effect::Moved);
    assert_eq!(g.state.scene, SceneId::Casino);
}

// ---------------------------------------------------------------- saves

#[test]
fn mid_game_save_round_trip_in_the_alley() {
    let mut g = Game::new();
    go(&mut g, Noun::Matchbook);
    g.input("take matchbook");
    g.input("leave");
    go(&mut g, Noun::Dot);
    g.input("compliment dot");
    g.input("go to alley");
    go(&mut g, Noun::Mime);
    g.input("feed hot dog to mime");
    go(&mut g, Noun::Grate);
    g.input("use comb on grate");
    assert_eq!(g.state.scene, SceneId::Alley);
    let saved = g.state.serialize();

    let mut h = Game::new();
    h.state = GameState::deserialize(&saved).expect("loads");
    assert_eq!(h.state, g.state);
    assert_eq!(h.state.scene, SceneId::Alley);
    assert_eq!(h.state.cash, 5);
    assert!(h.state.flag(Flag::MimeFed));
    // The restored game plays on: the grate is empty, and the store sells.
    assert_eq!(h.input("use comb on grate").effect, Effect::None);
    h.input("go to boardwalk");
    h.input("enter store");
    go(&mut h, Noun::Brenda);
    scored(&h.input("buy carnation"), 3);
    assert_eq!(h.state.score, g.state.score + 3);
}

#[test]
fn sound_commands_work_anytime() {
    let mut g = Game::new();
    assert_eq!(g.input("sound off").effect, Effect::Sound(Some(false)));
    assert_eq!(
        g.input("Turn the sound ON").effect,
        Effect::Sound(Some(true))
    );
    assert_eq!(g.input("mute").effect, Effect::Sound(Some(false)));
    assert_eq!(g.input("unmute").effect, Effect::Sound(Some(true)));
    assert_eq!(g.input("sound").effect, Effect::Sound(None));
    let moves = g.state.moves;
    g.state.dead = true;
    assert_eq!(g.input("sound on").effect, Effect::Sound(Some(true)));
    assert_eq!(g.state.moves, moves, "sound commands don't take a turn");
    assert!(sound_command("sound the alarm").is_none());
    assert!(sound_command("look").is_none());
}

// ---------------------------------------------------------------- milestone 4

/// Start in `scene` with everything needed to be anywhere in the quest.
fn at(scene: SceneId) -> Game {
    let mut g = Game::new();
    g.state.scene = scene;
    g
}

#[test]
fn new_scenes_connect_to_the_map() {
    let mut g = Game::new();
    g.input("leave");
    walk_to_scene(&mut g, SceneId::Pier);
    assert!(g.input("look").messages[0].contains("Percival"));
    walk_to_scene(&mut g, SceneId::Motel);
    walk_to_scene(&mut g, SceneId::Pier);
    walk_to_scene(&mut g, SceneId::Boardwalk);
    walk_to_scene(&mut g, SceneId::Alley);
    let r = walk_to_scene(&mut g, SceneId::Laundromat);
    assert!(r.messages[0].contains("SUDS & DUDS"));
    walk_to_scene(&mut g, SceneId::Alley);
    assert_eq!(g.state.scene, SceneId::Alley);
    // Typed travel works between neighbours, and not between strangers.
    assert_eq!(g.input("enter laundromat").effect, Effect::Moved);
    assert!(g.input("go to pier").messages[0].contains("can't get there"));
    g.input("leave");
    g.input("go to boardwalk");
    assert_eq!(g.input("go to pier").effect, Effect::Moved);
    assert_eq!(g.input("go to motel").effect, Effect::Moved);
    for s in [SceneId::Pier, SceneId::Laundromat, SceneId::Motel] {
        assert!(g.state.flag(s.visited_flag()));
    }
}

#[test]
fn pier_rod_fishing_and_pelican() {
    let mut g = at(SceneId::Pier);
    // No rod, no fish.
    go(&mut g, Noun::Ocean);
    assert!(g.input("fish").messages[0].contains("rod"));
    go(&mut g, Noun::Rod);
    scored(&g.input("take rod"), 2);
    assert!(g.input("take rod").messages[0].contains("already"));
    // From the wrong spot you can't cast.
    g.state.hero = (60, 112);
    assert!(g.input("use rod").messages[0].contains("front railing"));
    go(&mut g, Noun::Ocean);
    // No bait.
    assert!(g.input("use rod").messages[0].contains("bait"));
    g.state.add(Item::Shrimp);
    scored(&g.input("fish"), 3);
    assert!(!g.state.has(Item::Shrimp));
    assert!(g.input("fish").messages[0].contains("second"));
    // The pelican refuses everything but flounder.
    go(&mut g, Noun::Pelican);
    g.state.add(Item::Shrimp);
    assert!(g.input("give shrimp to pelican").messages[0].contains("standards"));
    assert!(g.state.has(Item::Shrimp));
    assert!(g.input("talk to pelican").messages[0].contains("Percival"));
    scored(&g.input("feed flounder to pelican"), 4);
    assert!(g.state.has(Item::Key) && !g.state.has(Item::Flounder));
    assert!(g.input("talk to pelican").messages[0].contains("Percival"));
}

#[test]
fn motel_ferris_the_comb_and_the_shrimp() {
    let mut g = at(SceneId::Motel);
    go(&mut g, Noun::Shrimp);
    // Ferris is watching.
    assert!(g.input("take shrimp").messages[0].contains("Guests only"));
    assert!(!g.state.has(Item::Shrimp));
    go(&mut g, Noun::Ferris);
    for _ in 0..3 {
        g.input("talk to ferris");
    }
    assert_eq!(g.state.talks[Npc::Ferris as usize], 3);
    // Without a comb there is nothing to offer.
    g.state.remove(Item::Comb);
    assert!(g.input("use comb on ferris").messages[0].contains("don't have a comb"));
    g.state.add(Item::Comb);
    g.input("use comb on ferris");
    assert!(g.state.flag(Flag::FerrisAway));
    assert!(g.state.has(Item::Comb), "the comb is never consumed");
    go(&mut g, Noun::Shrimp);
    scored(&g.input("take shrimp"), 2);
    assert!(g.input("take shrimp").messages[0].contains("already"));
}

#[test]
fn laundromat_needs_ticket_then_key() {
    let mut g = at(SceneId::Laundromat);
    go(&mut g, Noun::Dee);
    // Dee won't brief a stranger.
    for _ in 0..4 {
        assert_eq!(g.input("talk to dee").effect, Effect::None);
    }
    assert!(!g.state.flag(Flag::DeeBriefed));
    g.state.add(Item::Ticket);
    scored(&g.input("talk to dee"), 2);
    assert!(g.state.has(Item::Ticket), "the ticket stays in the pocket");
    // The key alone opens nothing until Dee has briefed us...
    let mut h = at(SceneId::Laundromat);
    h.state.add(Item::Key);
    go(&mut h, Noun::Dryer);
    assert!(h.input("use key on dryer").messages[0].contains("Ticket"));
    assert!(!h.state.has(Item::Gown));
    // ...and without the key the dryer won't budge.
    go(&mut g, Noun::Dryer);
    assert!(g.input("open dryer").messages[0].contains("key"));
    g.state.add(Item::Key);
    scored(&g.input("use key on dryer"), 3);
    assert!(g.state.has(Item::Gown) && g.state.flag(Flag::GotGown));
    assert_eq!(g.input("use key on dryer").effect, Effect::None);
}

#[test]
fn vonda_needs_the_gown_not_just_a_song() {
    let mut g = at(SceneId::Rooftop);
    g.state.set(Flag::DrankDrink);
    g.state.set(Flag::AteMints);
    go(&mut g, Noun::Vonda);
    scored(&g.input("sing"), 12);
    assert_eq!(g.state.score, 12);
    // Singing again changes nothing.
    assert_eq!(g.input("sing to vonda").effect, Effect::None);
    assert!(!g.state.flag(Flag::Won));
    g.state.add(Item::Gown);
    assert_eq!(g.input("give gown to vonda").effect, Effect::Won);
    assert_eq!(g.state.score, 14);
}

#[test]
fn hint_command_progresses_and_saturates() {
    let mut g = Game::new();
    let h1 = g.input("hint").messages[0].clone();
    let h2 = g.input("hint").messages[0].clone();
    let h3 = g.input("hint").messages[0].clone();
    let h4 = g.input("hint").messages[0].clone();
    assert!(h1.starts_with("Hint (1/3)"), "{h1}");
    assert!(h2.starts_with("Hint (2/3)"), "{h2}");
    assert!(h3.starts_with("Hint (3/3)"), "{h3}");
    assert_eq!(h3, h4, "the last hint repeats");
    assert!(h3.contains("TAKE MATCHBOOK"));
    // Hints take no turn, cost no points, and move on once the goal is done.
    assert_eq!(g.state.score, 0);
    go(&mut g, Noun::Matchbook);
    g.input("take matchbook");
    assert!(g.input("hint").messages[0].contains("Hint (1/3)"));
    assert!(g.state.hints.contains(&3));
    // Each area talks about its own business.
    let mut p = at(SceneId::Pier);
    assert!(p.input("clue").messages[0].contains("Hint (1/3)"));
    p.input("clue");
    assert!(p.input("clue").messages[0].contains("TAKE ROD"));
    let mut l = at(SceneId::Laundromat);
    assert!(l.input("stuck").messages[0].contains("Hint (1/3)"));
    let mut m = at(SceneId::Motel);
    assert!(m.input("hint").messages[0].starts_with("Hint (1/3)"));
    // Finished area: points elsewhere.
    let mut b = at(SceneId::Pier);
    for f in [Flag::GotRod, Flag::CaughtFlounder, Flag::GotKey] {
        b.state.set(f);
    }
    assert!(b.input("hint").messages[0].contains("nothing more to do here"));
}

#[test]
fn hint_slots_are_unique_and_saved() {
    let mut slots = hints::objective_slots();
    let n = slots.len();
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(slots.len(), n);
    assert!(slots.iter().all(|&s| s < crate::game::state::HINT_SLOTS));
    let mut g = Game::new();
    g.input("hint");
    g.input("hint");
    let loaded = GameState::deserialize(&g.state.serialize()).unwrap();
    assert_eq!(loaded.hints, g.state.hints);
    assert_eq!(loaded.hints[0], 2);
}

#[test]
fn verbs_command_lists_the_words() {
    let mut g = Game::new();
    let r = g.input("verbs");
    assert!(r.messages[0].contains("LOOK") && r.messages[0].contains("HINT"));
    assert!(g.input("help").messages[0].contains("HINT"));
    // Neither costs points.
    g.input("hint");
    g.input("verbs");
    assert_eq!(g.state.score, 0);
}

#[test]
fn gown_and_items_have_descriptions() {
    let mut g = Game::new();
    for (it, word) in [
        (Item::Ticket, "ticket"),
        (Item::Rod, "rod"),
        (Item::Shrimp, "shrimp"),
        (Item::Flounder, "flounder"),
        (Item::Key, "key"),
        (Item::Gown, "gown"),
    ] {
        g.state.add(it);
        let r = g.input(&format!("look at {word}"));
        assert_ne!(r.effect, Effect::Confused, "{word}");
        assert!(
            !r.messages[0].contains("don't see"),
            "{word}: {:?}",
            r.messages
        );
        assert!(
            g.input("i").messages[0].to_lowercase().contains(word),
            "{word}"
        );
    }
    assert!(g.input("eat flounder").messages[0].contains("Raw"));
}

#[test]
fn every_tombstone_fits_on_screen() {
    // The epitaph box grows with its longest line; it must stay on the
    // 320x200 screen (six pixels a character).
    for kind in DeathKind::ALL {
        let d = death(kind);
        assert!(
            d.cause.len() <= 28,
            "{kind:?} title is too long: {}",
            d.cause
        );
        for line in &d.epitaph {
            assert!(line.len() <= 30, "{kind:?} epitaph line too long: {line}");
        }
    }
}
