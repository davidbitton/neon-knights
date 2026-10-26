//! The Sandpiper Motor Court: a front office, a guest book, and a buffet
//! that has seen better decades.

use super::{c, hs, npcs, text, Exit, Hotspot, Lcg, Placed, Rect};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::game::parser::Noun;
use crate::game::state::{Flag, GameState, SceneId};

pub const FERRIS_POS: (i32, i32) = (200, 106);

pub const EXITS: [Exit; 1] = [Exit {
    zone: Rect::new(0, 101, 8, 174),
    to: SceneId::Pier,
    arrive: (300, 128),
    nouns: &[Noun::Pier],
    back: (16, 128),
}];

const AT_DESK: Rect = Rect::new(146, 118, 256, 152);

pub fn hotspots(state: &GameState) -> Vec<Hotspot> {
    let mut v = vec![];
    if !state.flag(Flag::FerrisAway) {
        v.push(hs(
            Noun::Ferris,
            Rect::new(190, 68, 212, 108),
            AT_DESK,
            (200, 126),
        ));
    }
    v.extend([
        hs(
            Noun::Bell,
            Rect::new(160, 98, 174, 108),
            AT_DESK,
            (166, 126),
        ),
        hs(
            Noun::Desk,
            Rect::new(146, 106, 256, 120),
            AT_DESK,
            (200, 126),
        ),
    ]);
    if !state.flag(Flag::GotShrimp) {
        v.push(hs(
            Noun::Shrimp,
            Rect::new(46, 80, 100, 100),
            Rect::new(34, 112, 116, 146),
            (74, 120),
        ));
    }
    v.push(hs(
        Noun::Tv,
        Rect::new(262, 40, 306, 76),
        Rect::new(258, 100, 316, 140),
        (290, 112),
    ));
    v
}

pub fn actors(state: &GameState, ticks: u64) -> Vec<Placed> {
    let mut v = vec![];
    if !state.flag(Flag::FerrisAway) {
        v.push(Placed::feet(
            npcs::ferris((ticks / 16 % 4) as u8),
            FERRIS_POS.0,
            FERRIS_POS.1,
            priority_for_y(FERRIS_POS.1),
        ));
    }
    // The soap opera on the television flickers between two scenes.
    let col = [LIGHT_MAGENTA, LIGHT_CYAN, YELLOW][(ticks / 7 % 3) as usize];
    v.push(Placed::top_left(
        super::rect_sprite(16, 12, col),
        272,
        48,
        priority_for_y(100),
    ));
    // VACANCY, in a neon that doesn't know when to quit.
    if !(ticks / 9).is_multiple_of(5) {
        v.push(Placed::top_left(
            super::rect_sprite(40, 1, LIGHT_GREEN),
            33,
            62,
            priority_for_y(100),
        ));
    }
    v
}

pub fn draw(state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Knotty-pine paneling and a hopeful sign in the window.
    p.fill_rect(0, 0, 320, 100, c(BROWN));
    for x in (0..320).step_by(10) {
        p.line(x, 9, x, 99, c(BLACK));
    }
    for (x, y) in [(33, 30), (118, 60), (215, 22), (291, 84), (66, 82)] {
        p.fill_ellipse(x, y, 2, 3, c(BLACK));
    }
    p.fill_rect(0, 100, 320, 176, c(RED));
    p.line(0, 100, 319, 100, c(BLACK));
    for y in (104..176).step_by(8) {
        for x in (0..320).step_by(16) {
            let off = if ((y - 104) / 8) % 2 == 0 { 0 } else { 8 };
            p.plot(x + off, y, c(YELLOW));
            p.plot(x + off + 1, y + 1, c(YELLOW));
        }
    }
    p.fill_rect(0, 101, 320, 174, Pen::control_only(CTRL_WALK));

    // The front door: a wedge of night, with a bell on a spring.
    p.fill_rect(0, 36, 24, 100, c(BLACK));
    p.rect(0, 36, 24, 100, c(LIGHT_GRAY));
    p.fill_rect(4, 42, 20, 96, c(BLUE));
    stars(&mut p, 3);
    p.fill_rect(0, 18, 24, 30, c(WHITE));
    text(&mut p, 0, 21, "EXIT", RED, 1);

    // The VACANCY window sign.
    p.fill_rect(30, 52, 80, 70, c(BLACK));
    p.rect(30, 52, 80, 70, c(DARK_GRAY));
    text(&mut p, 33, 54, "VACANCY", LIGHT_GREEN, 1);
    p.line(33, 62, 72, 62, c(DARK_GRAY));
    text(&mut p, 33, 63, "NO", RED, 1);

    // A restroom door; the light over it says who's in.
    p.fill_rect(106, 30, 136, 100, c(DARK_GRAY));
    p.rect(106, 30, 136, 100, c(BLACK));
    p.fill_rect(120, 62, 123, 66, c(YELLOW));
    p.fill_rect(110, 22, 132, 29, c(BLACK));
    if state.flag(Flag::FerrisAway) {
        text(&mut p, 112, 22, "BUSY", LIGHT_RED, 1);
    } else {
        text(&mut p, 112, 22, "FREE", LIGHT_GREEN, 1);
    }

    // Key rack behind the desk, thirty hooks of mostly missing keys.
    p.fill_rect(150, 30, 250, 66, c(BLACK));
    p.rect(150, 30, 250, 66, c(YELLOW));
    let mut rng = Lcg(8);
    for r in 0..3 {
        for col in 0..10 {
            let (x, y) = (156 + col * 9, 36 + r * 10);
            p.plot(x, y, c(LIGHT_GRAY));
            if rng.next(3) != 0 {
                p.fill_rect(x - 1, y + 1, x + 2, y + 5, c(YELLOW));
            }
        }
    }

    // The television, on a shelf in the corner, in a wood-grain case.
    p.fill_rect(258, 38, 310, 80, c(BLACK));
    p.rect(258, 38, 310, 80, c(DARK_GRAY));
    p.fill_rect(266, 44, 302, 72, c(DARK_GRAY));
    p.fill_rect(268, 46, 300, 70, c(CYAN));
    p.fill_rect(262, 80, 306, 84, c(LIGHT_GRAY));
    p.line(284, 38, 276, 28, c(LIGHT_GRAY));
    p.line(284, 38, 294, 28, c(LIGHT_GRAY));
    p.fill_rect(302, 60, 306, 64, c(RED));

    // The "continental breakfast": a rolling cart, a tray, and a heat lamp
    // the cart has been using as a sunlamp since Carter.
    let cp = priority_for_y(112);
    p.fill_rect(44, 92, 102, 112, Pen::color(LIGHT_GRAY).pri(cp));
    p.rect(44, 92, 102, 112, Pen::color(DARK_GRAY).pri(cp));
    p.fill_rect(46, 94, 100, 100, Pen::color(WHITE).pri(cp));
    if !state.flag(Flag::GotShrimp) {
        p.fill_ellipse(72, 91, 20, 4, Pen::color(LIGHT_CYAN).pri(cp));
        for sx in [58, 66, 74, 82, 88] {
            p.fill_ellipse(sx, 89, 3, 2, Pen::color(LIGHT_RED).pri(cp));
            p.plot(sx + 1, 87, Pen::color(RED).pri(cp));
        }
    }
    p.fill_rect(48, 102, 98, 107, Pen::color(YELLOW).pri(cp));
    for wx in [52, 94] {
        p.fill_ellipse(wx, 113, 3, 2, Pen::color(BLACK).pri(cp));
    }
    p.line(60, 80, 84, 80, Pen::color(DARK_GRAY).pri(cp));
    p.line(72, 80, 72, 88, Pen::color(DARK_GRAY).pri(cp));
    p.fill_rect(42, 90, 104, 114, Pen::control_only(CTRL_BLOCK));

    // The front desk; it hides Ferris from the waist down.
    let dp = priority_for_y(118);
    p.fill_rect(146, 108, 256, 118, Pen::color(LIGHT_RED).pri(dp));
    p.fill_rect(146, 108, 256, 111, Pen::color(WHITE).pri(dp));
    p.rect(146, 108, 256, 118, Pen::color(BLACK).pri(dp));
    for x in (154..250).step_by(24) {
        p.fill_rect(x, 113, x + 16, 117, Pen::color(RED).pri(dp));
    }
    // Bell, register, and the open guest book.
    p.fill_ellipse(167, 107, 5, 2, Pen::color(YELLOW).pri(dp));
    p.fill_rect(166, 103, 169, 105, Pen::color(YELLOW).pri(dp));
    p.fill_rect(222, 100, 244, 108, Pen::color(WHITE).pri(dp));
    p.line(233, 100, 233, 108, Pen::color(BLACK).pri(dp));
    p.fill_rect(146, 100, 256, 118, Pen::control_only(CTRL_BLOCK));

    // A sad potted fern.
    p.fill_rect(294, 130, 306, 146, c(RED));
    p.rect(294, 130, 306, 146, c(BLACK));
    for (x, y) in [(292, 118), (296, 112), (300, 108), (304, 112), (308, 118)] {
        p.line(300, 130, x, y, c(GREEN));
    }
    p
}

fn stars(p: &mut Picture, seed: u32) {
    let mut rng = Lcg(seed);
    for _ in 0..10 {
        p.plot(5 + rng.next(14), 44 + rng.next(50), c(WHITE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desk_hides_ferris_legs_and_ferris_can_leave() {
        let mut s = GameState::new();
        s.scene = SceneId::Motel;
        let p = draw(&s);
        assert!(p.priority_at(FERRIS_POS.0, 112).unwrap() > priority_for_y(FERRIS_POS.1));
        assert!(!p.walkable(FERRIS_POS.0, 112));
        assert!(hotspots(&s).iter().any(|h| h.noun == Noun::Ferris));
        assert_eq!(actors(&s, 9).len(), 3);
        s.set(Flag::FerrisAway);
        assert!(!hotspots(&s).iter().any(|h| h.noun == Noun::Ferris));
        assert_eq!(actors(&s, 9).len(), 2);
        assert_ne!(p.visual, draw(&s).visual);
    }

    #[test]
    fn shrimp_tray_empties_when_taken() {
        let mut s = GameState::new();
        s.scene = SceneId::Motel;
        let full = draw(&s);
        s.set(Flag::GotShrimp);
        assert_ne!(full.visual, draw(&s).visual);
        assert!(!hotspots(&s).iter().any(|h| h.noun == Noun::Shrimp));
    }
}
