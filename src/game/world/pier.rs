//! Gull Point Pier: a bait shack, a lighthouse, and Percival the pelican.

use super::{c, hs, npcs, rect_sprite, stars, text, Exit, Hotspot, Lcg, Placed, Rect, PIC_BOTTOM};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::game::parser::Noun;
use crate::game::state::{Flag, GameState, SceneId};

pub const EXITS: [Exit; 2] = [
    Exit {
        zone: Rect::new(0, 101, 8, 150),
        to: SceneId::Boardwalk,
        arrive: (300, 128),
        nouns: &[Noun::Boardwalk],
        back: (16, 128),
    },
    Exit {
        zone: Rect::new(312, 101, 320, 150),
        to: SceneId::Motel,
        arrive: (16, 128),
        nouns: &[Noun::Motel],
        back: (300, 128),
    },
];

/// Percival patrols the pier between these x positions, one pixel per
/// `PELICAN_PACE` ticks.
pub const PELICAN_X0: i32 = 122;
pub const PELICAN_SPAN: i32 = 110;
pub const PELICAN_PACE: u64 = 6;
pub const PELICAN_Y: i32 = 128;

/// Where the pelican is at `clock`: (x, y, facing east).
pub fn pelican_pos(clock: u64) -> (i32, i32, bool) {
    let span = PELICAN_SPAN as u64;
    let t = (clock / PELICAN_PACE) % (2 * span);
    if t < span {
        (PELICAN_X0 + t as i32, PELICAN_Y, true)
    } else {
        (PELICAN_X0 + (2 * span - t) as i32, PELICAN_Y, false)
    }
}

pub fn hotspots(state: &GameState) -> Vec<Hotspot> {
    let (px, py, _) = pelican_pos(state.clock);
    let mut v = vec![hs(
        Noun::Pelican,
        Rect::new(px - 10, py - 17, px + 10, py + 1),
        Rect::new(px - 36, 112, px + 36, 150),
        (px, 134),
    )];
    if !state.flag(Flag::GotRod) {
        v.push(hs(
            Noun::Rod,
            Rect::new(88, 56, 102, 101),
            Rect::new(60, 100, 124, 128),
            (96, 110),
        ));
    }
    v.extend([
        hs(
            Noun::Shack,
            Rect::new(10, 40, 98, 101),
            Rect::new(10, 100, 124, 128),
            (60, 110),
        ),
        hs(
            Noun::Ocean,
            Rect::new(0, 151, 320, PIC_BOTTOM),
            Rect::new(0, 134, 320, 151),
            (216, 142),
        ),
        hs(
            Noun::City,
            Rect::new(0, 9, 320, 84),
            Rect::new(0, 0, 320, 200),
            (216, 142),
        ),
    ]);
    v
}

pub fn actors(state: &GameState, ticks: u64) -> Vec<Placed> {
    let (px, py, right) = pelican_pos(state.clock);
    let waddle = (ticks / 5 % 2) as u8;
    let mut v = vec![Placed::feet(
        npcs::pelican(
            waddle + (ticks / 10 % 2) as u8 * 2,
            right,
            !state.flag(Flag::GotKey),
        ),
        px,
        py,
        priority_for_y(py),
    )];
    // The lighthouse beam sweeps on and off.
    if ticks / 40 % 3 != 2 {
        let wide = (ticks / 40 % 3) as i32;
        v.push(Placed::top_left(
            rect_sprite(10 + wide * 6, 2, YELLOW),
            270 - wide * 6,
            35,
            15,
        ));
    }
    // Moon-glitter on the water.
    let phase = (ticks / 8 % 3) as i32;
    for i in 0..5 {
        let x = 236 + (i * 7 + phase * 3) % 26;
        v.push(Placed::top_left(
            rect_sprite(4, 1, YELLOW),
            x,
            86 + i * 3,
            15,
        ));
    }
    v
}

pub fn draw(state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Night sky, a moon, and the sea out to the horizon.
    p.fill_rect(0, 0, 320, 84, c(BLACK));
    for y in 56..84 {
        for x in 0..320 {
            if y >= 72 || (x + y) % 2 == 0 {
                p.plot(x, y, c(BLUE));
            }
        }
    }
    stars(&mut p, 57, 70, 9, 56);
    p.fill_ellipse(236, 26, 9, 9, c(YELLOW));
    p.fill_ellipse(240, 24, 8, 8, c(BLACK));
    p.fill_rect(0, 84, 320, 101, c(BLUE));
    let mut rng = Lcg(21);
    for _ in 0..40 {
        let (x, y) = (rng.next(320), 85 + rng.next(14));
        p.line(x, y, x + 3 + rng.next(6), y, c(LIGHT_BLUE));
    }
    // A fishing boat riding at anchor far out.
    p.fill_polygon(&[(150, 80), (186, 80), (180, 88), (156, 88)], c(BROWN));
    p.fill_rect(160, 72, 174, 80, c(WHITE));
    p.fill_rect(163, 74, 167, 77, c(YELLOW));
    p.line(180, 80, 180, 62, c(DARK_GRAY));
    p.line(180, 62, 170, 80, c(DARK_GRAY));

    // The lighthouse out on its rock.
    p.fill_ellipse(288, 88, 16, 4, c(DARK_GRAY));
    p.fill_polygon(&[(280, 86), (296, 86), (293, 44), (283, 44)], c(WHITE));
    for (y0, y1) in [(50, 58), (66, 74)] {
        p.fill_polygon(&[(283, y0), (293, y0), (294, y1), (282, y1)], c(RED));
    }
    p.fill_rect(281, 38, 295, 44, c(YELLOW));
    p.fill_polygon(&[(280, 38), (296, 38), (288, 30)], c(RED));
    p.rect(281, 38, 295, 44, c(BLACK));

    // The bait shack, shuttered for the night.
    p.fill_rect(12, 54, 92, 100, c(CYAN));
    for x in (16..92).step_by(8) {
        p.line(x, 54, x, 99, c(LIGHT_CYAN));
    }
    p.fill_polygon(&[(6, 54), (98, 54), (88, 40), (16, 40)], c(BLUE));
    p.line(6, 54, 98, 54, c(BLACK));
    p.fill_rect(28, 62, 72, 84, c(BLACK));
    p.fill_rect(30, 64, 70, 82, c(DARK_GRAY));
    p.fill_rect(30, 64, 70, 70, c(BROWN));
    p.fill_rect(30, 72, 70, 82, c(DARK_GRAY));
    for x in (28..72).step_by(8) {
        p.fill_rect(x, 84, x + 4, 88, c(RED));
        p.fill_rect(x + 4, 84, x + 8, 88, c(WHITE));
    }
    p.fill_rect(40, 72, 60, 80, c(WHITE));
    p.fill_rect(42, 74, 58, 79, c(RED));
    text(&mut p, 24, 44, "BAIT", WHITE, 1);
    text(&mut p, 50, 44, "&", YELLOW, 1);
    text(&mut p, 58, 44, "TACKLE", WHITE, 1);
    p.fill_rect(12, 88, 24, 100, c(BROWN));
    p.rect(12, 88, 24, 100, c(BLACK));
    // The honor-system sign on the right-hand wall.
    p.fill_rect(76, 70, 90, 86, c(YELLOW));
    p.rect(76, 70, 90, 86, c(BLACK));
    for y in [74, 78, 82] {
        p.line(78, y, 88, y, c(BLACK));
    }
    // The loaner rod, leaning on the wall.
    if !state.flag(Flag::GotRod) {
        p.line(97, 100, 92, 58, c(BROWN));
        p.line(98, 100, 93, 58, c(BROWN));
        p.line(92, 58, 90, 70, c(WHITE));
        p.fill_rect(95, 90, 100, 96, c(DARK_GRAY));
    }

    // Lamp posts with their dithered glow.
    for lx in [112, 236] {
        for y in 56..100 {
            let half = (y - 56) / 5 + 1;
            for x in (lx - half)..(lx + half + 1) {
                if (x + y) % 3 == 0 {
                    p.plot(x, y, c(BROWN));
                }
            }
        }
        p.line(lx, 56, lx, 100, c(DARK_GRAY));
        p.fill_rect(lx - 3, 50, lx + 4, 57, c(YELLOW));
        p.rect(lx - 3, 50, lx + 4, 57, c(BLACK));
    }

    // Back railing along the far edge of the deck.
    for x in (0..320).step_by(24) {
        p.fill_rect(x, 90, x + 3, 101, c(LIGHT_GRAY));
    }
    p.fill_rect(0, 90, 320, 92, c(WHITE));
    p.fill_rect(0, 96, 320, 97, c(LIGHT_GRAY));

    // The deck.
    p.fill_rect(0, 101, 320, 151, c(BROWN));
    p.line(0, 100, 319, 100, c(BLACK));
    let mut y = 104;
    let mut gap = 4;
    while y < 151 {
        p.line(0, y, 319, y, c(BLACK));
        let mut x = rng.next(30);
        while x < 320 {
            p.line(x, y - gap + 1, x, y - 1, c(BLACK));
            x += 40 + rng.next(40);
        }
        gap += 1;
        y += gap;
    }
    p.fill_rect(0, 101, 320, 150, Pen::control_only(CTRL_WALK));
    // Rope coil by the shack, and a bollard at the far end.
    p.fill_ellipse(30, 116, 10, 3, c(LIGHT_GRAY));
    p.fill_ellipse(30, 115, 6, 1, c(BROWN));
    p.fill_rect(288, 108, 296, 118, c(DARK_GRAY));
    p.fill_ellipse(292, 108, 6, 2, c(BLACK));

    // A stack of lobster crates.
    let cp = priority_for_y(112);
    for (x0, y0) in [(118, 94), (136, 94), (127, 84)] {
        p.fill_rect(x0, y0, x0 + 17, y0 + 18, Pen::color(BROWN).pri(cp));
        p.rect(x0, y0, x0 + 17, y0 + 18, Pen::color(BLACK).pri(cp));
        for sx in (x0 + 4..x0 + 17).step_by(4) {
            p.line(sx, y0, sx, y0 + 18, Pen::color(BLACK).pri(cp));
        }
    }
    p.fill_rect(116, 100, 156, 113, Pen::control_only(CTRL_BLOCK));

    // Water under the pier, with pilings, and a railing in front.
    p.fill_rect(0, 151, 320, PIC_BOTTOM, c(BLUE));
    for _ in 0..30 {
        let (x, y) = (rng.next(320), 153 + rng.next(22));
        p.line(x, y, x + 2 + rng.next(5), y, c(LIGHT_BLUE));
    }
    let rp = priority_for_y(158);
    for x in (10..320).step_by(46) {
        p.fill_rect(x, 151, x + 6, PIC_BOTTOM, Pen::color(BROWN).pri(rp));
        p.line(x + 2, 151, x + 2, PIC_BOTTOM, Pen::color(BLACK).pri(rp));
    }
    p.fill_rect(0, 140, 320, 142, Pen::color(WHITE).pri(rp));
    p.fill_rect(0, 147, 320, 149, Pen::color(LIGHT_GRAY).pri(rp));
    for x in (4..320).step_by(20) {
        p.fill_rect(x, 140, x + 3, 158, Pen::color(LIGHT_GRAY).pri(rp));
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pelican_patrols_back_and_forth() {
        let (x0, _, r0) = pelican_pos(0);
        assert_eq!((x0, r0), (PELICAN_X0, true));
        let (x1, _, r1) = pelican_pos(PELICAN_PACE * PELICAN_SPAN as u64);
        assert_eq!((x1, r1), (PELICAN_X0 + PELICAN_SPAN, false));
        let full = PELICAN_PACE * 2 * PELICAN_SPAN as u64;
        assert_eq!(pelican_pos(full), pelican_pos(0));
        for t in (0..full).step_by(7) {
            let (x, _, _) = pelican_pos(t);
            assert!((PELICAN_X0..=PELICAN_X0 + PELICAN_SPAN).contains(&x));
        }
    }

    #[test]
    fn rod_vanishes_when_taken_and_front_rail_hides_legs() {
        let mut s = GameState::new();
        s.scene = SceneId::Pier;
        assert!(hotspots(&s).iter().any(|h| h.noun == Noun::Rod));
        let before = draw(&s);
        s.set(Flag::GotRod);
        assert!(!hotspots(&s).iter().any(|h| h.noun == Noun::Rod));
        assert_ne!(before.visual, draw(&s).visual);
        assert!(before.priority_at(24, 141).unwrap() > priority_for_y(149));
    }
}
