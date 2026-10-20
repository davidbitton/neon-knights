//! The world: every scene's AGI-style vector picture (all drawn in code, no
//! image files), its hotspots, exits, and the NPC/animation overlays.
//!
//! Layout follows the old interpreters: status bar on rows 0..9, the
//! picture in 9..176, the command line below it.

pub mod alley;
pub mod bar;
pub mod boardwalk;
pub mod casino;
pub mod npcs;
pub mod rooftop;
pub mod screens;
pub mod store;

use super::parser::Noun;
use super::state::{GameState, SceneId};
use crate::engine::gfx::{Pen, Picture};
use crate::engine::sprite::Sprite;

pub const STATUS_H: i32 = 9;
pub const PIC_BOTTOM: i32 = 176;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Rect {
    pub const fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
        Rect { x0, y0, x1, y1 }
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x0 && x < self.x1 && y >= self.y0 && y < self.y1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hotspot {
    pub noun: Noun,
    /// Clickable area on screen.
    pub click: Rect,
    /// Where the hero's feet must be to interact (take/use/talk).
    pub near: Rect,
    /// A walkable spot inside `near` to walk to.
    pub approach: (i32, i32),
}

pub const fn hs(noun: Noun, click: Rect, near: Rect, approach: (i32, i32)) -> Hotspot {
    Hotspot {
        noun,
        click,
        near,
        approach,
    }
}

/// A way out of a scene: walk your feet into `zone` (a doorway or a strip
/// along a screen edge) or type a matching travel command.
#[derive(Clone, Copy, Debug)]
pub struct Exit {
    pub zone: Rect,
    pub to: SceneId,
    /// Where the hero appears in the destination scene.
    pub arrive: (i32, i32),
    /// Nouns that name this exit in typed commands ("enter store").
    pub nouns: &'static [Noun],
    /// Where to put the hero if the exit refuses (e.g. a doorman says no).
    pub back: (i32, i32),
}

/// A sprite to draw this frame, anchored at its feet (bottom-center).
pub struct Placed {
    pub sprite: Sprite,
    pub x: i32,
    pub y: i32,
    pub pri: u8,
}

impl Placed {
    pub fn feet(sprite: Sprite, x: i32, y: i32, pri: u8) -> Placed {
        Placed { sprite, x, y, pri }
    }
    /// Anchor by top-left corner instead.
    pub fn top_left(sprite: Sprite, x0: i32, y0: i32, pri: u8) -> Placed {
        let (x, y) = (x0 + sprite.w / 2, y0 + sprite.h - 1);
        Placed { sprite, x, y, pri }
    }
}

/// Solid rectangle sprite (for blinking lights and the like).
pub fn rect_sprite(w: i32, h: i32, c: u8) -> Sprite {
    Sprite {
        w,
        h,
        px: vec![Some(c); (w * h) as usize],
    }
}

pub fn draw(state: &GameState) -> Picture {
    match state.scene {
        SceneId::Bar => bar::draw(state),
        SceneId::Boardwalk => boardwalk::draw(state),
        SceneId::Alley => alley::draw(state),
        SceneId::Store => store::draw(state),
        SceneId::Casino => casino::draw(state),
        SceneId::Rooftop => rooftop::draw(state),
    }
}

/// Hotspots in hit-test priority order (small things first).
pub fn hotspots(state: &GameState) -> Vec<Hotspot> {
    match state.scene {
        SceneId::Bar => bar::hotspots(state),
        SceneId::Boardwalk => boardwalk::hotspots(state),
        SceneId::Alley => alley::hotspots(state),
        SceneId::Store => store::hotspots(state),
        SceneId::Casino => casino::hotspots(state),
        SceneId::Rooftop => rooftop::hotspots(state),
    }
}

pub fn exits(scene: SceneId) -> &'static [Exit] {
    match scene {
        SceneId::Bar => &bar::EXITS,
        SceneId::Boardwalk => &boardwalk::EXITS,
        SceneId::Alley => &alley::EXITS,
        SceneId::Store => &store::EXITS,
        SceneId::Casino => &casino::EXITS,
        SceneId::Rooftop => &rooftop::EXITS,
    }
}

/// NPCs and animated bits for this frame (the hero is drawn by the caller).
pub fn actors(state: &GameState, ticks: u64) -> Vec<Placed> {
    match state.scene {
        SceneId::Bar => bar::actors(state, ticks),
        SceneId::Boardwalk => boardwalk::actors(state, ticks),
        SceneId::Alley => alley::actors(state, ticks),
        SceneId::Store => store::actors(state, ticks),
        SceneId::Casino => casino::actors(state, ticks),
        SceneId::Rooftop => rooftop::actors(state, ticks),
    }
}

/// Plan a click-to-walk route to `to`, steering clear of every exit except
/// one the destination itself lies in.
pub fn plan_walk(
    scene: SceneId,
    pic: &Picture,
    from: (i32, i32),
    to: (i32, i32),
) -> Vec<(i32, i32)> {
    let zones: Vec<Rect> = exits(scene)
        .iter()
        .map(|e| e.zone)
        .filter(|z| !z.contains(to.0, to.1))
        .collect();
    crate::engine::path::find_path(pic, from, to, |x, y| zones.iter().any(|z| z.contains(x, y)))
}

pub fn exit_at(scene: SceneId, x: i32, y: i32) -> Option<&'static Exit> {
    exits(scene).iter().find(|e| e.zone.contains(x, y))
}

pub fn hotspot_at(state: &GameState, x: i32, y: i32) -> Option<Hotspot> {
    hotspots(state).into_iter().find(|h| h.click.contains(x, y))
}

pub fn hotspot_for(state: &GameState, noun: Noun) -> Option<Hotspot> {
    hotspots(state).into_iter().find(|h| h.noun == noun)
}

/// Is the hero standing close enough to `noun` to touch it?
pub fn is_near(state: &GameState, noun: Noun) -> bool {
    let (x, y) = state.hero;
    hotspots(state)
        .iter()
        .any(|h| h.noun == noun && h.near.contains(x, y))
}

/// The word used when a click turns into a typed command.
pub fn noun_word(n: Noun) -> &'static str {
    use Noun::*;
    match n {
        Room => "room",
        Bartender => "bartender",
        Jukebox => "jukebox",
        Matchbook => "matchbook",
        Quarter => "quarter",
        Comb => "comb",
        Egg => "egg",
        Drink => "drink",
        Bar => "bar",
        Stool => "stool",
        Door => "door",
        Restroom => "restroom",
        Sign => "sign",
        Bottles => "shelves",
        Patron => "regular",
        Table => "table",
        Floor => "floor",
        Me => "me",
        Man => "man",
        Woman => "woman",
        Machine => "machine",
        Boardwalk => "boardwalk",
        Alley => "alley",
        Store => "store",
        Casino => "casino",
        Rooftop => "rooftop",
        Dot => "dot",
        Cart => "cart",
        HotDog => "hot dog",
        Telescope => "telescope",
        Ocean => "ocean",
        Railing => "railing",
        Mime => "mime",
        Dumpster => "dumpster",
        Grate => "grate",
        FireEscape => "fire escape",
        Trash => "trash",
        Brenda => "clerk",
        Carnation => "carnation",
        Mints => "mints",
        Slushie => "slushie",
        Register => "register",
        Tiny => "tiny",
        Slots => "slot machine",
        Elevator => "elevator",
        Chandelier => "chandelier",
        Plant => "palm",
        Vonda => "vonda",
        Piano => "piano",
        Pool => "pool",
        City => "skyline",
        Money => "money",
    }
}

/// Tiny deterministic PRNG for scenery (stars, bricks, products).
pub(crate) struct Lcg(pub u32);
impl Lcg {
    pub fn next(&mut self, n: u32) -> i32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((self.0 >> 16) % n) as i32
    }
}

pub(crate) fn c(col: u8) -> Pen {
    Pen::color(col)
}

/// Starfield helper.
pub(crate) fn stars(p: &mut Picture, seed: u32, n: usize, y0: i32, y1: i32) {
    use crate::engine::palette::{LIGHT_GRAY, WHITE};
    let mut rng = Lcg(seed);
    for _ in 0..n {
        let (x, y) = (rng.next(320), y0 + rng.next((y1 - y0) as u32));
        p.plot(x, y, c(if rng.next(3) == 0 { WHITE } else { LIGHT_GRAY }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::actor::can_stand;
    use crate::game::state::Flag;

    fn state_in(scene: SceneId) -> GameState {
        let mut s = GameState::new();
        s.scene = scene;
        s
    }

    #[test]
    fn every_scene_draws_and_has_hotspots_and_exits() {
        for scene in SceneId::ALL {
            let s = state_in(scene);
            let pic = draw(&s);
            assert!(
                pic.control.contains(&crate::engine::gfx::CTRL_WALK),
                "{scene:?}"
            );
            assert!(!hotspots(&s).is_empty(), "{scene:?}");
            assert!(!exits(scene).is_empty(), "{scene:?}");
        }
    }

    #[test]
    fn approach_points_are_walkable_and_near() {
        for scene in SceneId::ALL {
            let s = state_in(scene);
            let pic = draw(&s);
            for h in hotspots(&s) {
                if h.near.x1 - h.near.x0 >= 320 {
                    continue; // look-only scenery
                }
                let (x, y) = h.approach;
                assert!(
                    can_stand(&pic, x, y),
                    "{scene:?} {:?} approach {x},{y}",
                    h.noun
                );
                assert!(
                    h.near.contains(x, y),
                    "{scene:?} {:?} approach outside near",
                    h.noun
                );
            }
        }
    }

    #[test]
    fn exits_lead_to_safe_walkable_arrivals_and_are_reachable() {
        for scene in SceneId::ALL {
            let s = state_in(scene);
            let pic = draw(&s);
            for e in exits(scene) {
                // The zone itself must contain a standable spot.
                let reachable = (e.zone.y0..e.zone.y1)
                    .any(|y| (e.zone.x0..e.zone.x1).any(|x| can_stand(&pic, x, y)));
                assert!(reachable, "{scene:?} exit to {:?} unreachable", e.to);
                let dest = state_in(e.to);
                let dpic = draw(&dest);
                let (ax, ay) = e.arrive;
                assert!(
                    can_stand(&dpic, ax, ay),
                    "arrival in {:?} at {ax},{ay}",
                    e.to
                );
                assert!(
                    exit_at(e.to, ax, ay).is_none(),
                    "arrival in {:?} sits in an exit",
                    e.to
                );
                assert!(can_stand(&pic, e.back.0, e.back.1), "{scene:?} pushback");
                assert!(
                    exit_at(scene, e.back.0, e.back.1).is_none(),
                    "{scene:?} pushback in exit"
                );
                // There is a way back.
                assert!(
                    exits(e.to).iter().any(|b| b.to == scene),
                    "{:?} has no way back to {scene:?}",
                    e.to
                );
            }
        }
    }

    /// Check every hotspot can actually be reached by click-to-walk from
    /// every arrival point, without stumbling through an exit on the way.
    #[test]
    fn hotspots_reachable_by_click_to_walk_from_every_arrival() {
        use crate::engine::actor::Actor;
        let mut bad = vec![];
        for sc in SceneId::ALL {
            let mut starts: Vec<(i32, i32)> = SceneId::ALL
                .iter()
                .flat_map(|o| exits(*o).iter().filter(|e| e.to == sc).map(|e| e.arrive))
                .collect();
            if sc == SceneId::Bar {
                starts.push(crate::game::state::START_POS);
            }
            let mut st = state_in(sc);
            for f in [Flag::MimeFed, Flag::TinyApproved] {
                st.set(f);
            }
            let pic = draw(&st);
            for h in hotspots(&st) {
                if h.near.x1 - h.near.x0 >= 320 {
                    continue;
                }
                for &(x, y) in &starts {
                    let mut a = Actor::new(x, y);
                    a.follow(plan_walk(sc, &pic, (x, y), h.approach));
                    for _ in 0..2000 {
                        if !a.tick(&pic) {
                            break;
                        }
                        let in_exit = exit_at(sc, a.x, a.y);
                        if let Some(e) = in_exit {
                            if !e.zone.contains(h.approach.0, h.approach.1) {
                                bad.push(format!(
                                    "{sc:?}: from {:?} to {:?} trips the exit to {:?} at {:?}",
                                    (x, y),
                                    h.noun,
                                    e.to,
                                    (a.x, a.y)
                                ));
                                break;
                            }
                        }
                    }
                    if !h.near.contains(a.x, a.y) {
                        bad.push(format!(
                            "{sc:?}: from {:?} to {:?} stalls at {:?}",
                            (x, y),
                            h.noun,
                            (a.x, a.y)
                        ));
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }

    #[test]
    fn start_position_is_walkable() {
        let s = GameState::new();
        assert!(can_stand(&draw(&s), s.hero.0, s.hero.1));
    }

    #[test]
    fn every_scene_has_an_npc() {
        for scene in SceneId::ALL {
            assert!(!actors(&state_in(scene), 0).is_empty(), "{scene:?}");
        }
    }
}
