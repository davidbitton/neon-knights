use super::*;
use crate::game::state::DeathKind;
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

    // The Crow's Nest.
    scored(&g.input("eat mints"), 2);
    go(&mut g, Noun::Vonda);
    let r = g.input("sing to vonda");
    assert_eq!(r.effect, Effect::Won);
    assert!(r.messages.last().unwrap().contains("hired"));
    assert_eq!(g.state.score, MAX_SCORE);
    assert!(g.state.flag(Flag::Won));
    let end = g.ending_lines().join("\n");
    assert!(end.contains(&format!("Final score: {MAX_SCORE} of {MAX_SCORE}")));
    assert!(end.contains("Gentleman of the Evening"));
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
