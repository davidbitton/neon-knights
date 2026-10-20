//! The Golden Clam Casino lobby: slots, velvet, and a doorman named Tiny.

use super::{c, hs, npcs, rect_sprite, Exit, Hotspot, Placed, Rect, PIC_BOTTOM, STATUS_H};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::game::parser::Noun;
use crate::game::state::{Flag, GameState, SceneId};

/// Tiny guards the elevator until he approves of Chip's lapel.
pub const TINY_GUARDING: (i32, i32) = (238, 114);
pub const TINY_ASIDE: (i32, i32) = (200, 116);
pub const SLOTS: [i32; 3] = [20, 56, 92];

pub const EXITS: [Exit; 2] = [
    Exit {
        zone: Rect::new(110, 168, 210, 176),
        to: SceneId::Boardwalk,
        arrive: (261, 110),
        nouns: &[Noun::Door, Noun::Boardwalk],
        back: (160, 160),
    },
    Exit {
        zone: Rect::new(222, 101, 254, 105),
        to: SceneId::Rooftop,
        arrive: (154, 112),
        nouns: &[Noun::Elevator, Noun::Rooftop],
        back: (262, 124),
    },
];

pub fn tiny_pos(state: &GameState) -> (i32, i32) {
    if state.flag(Flag::TinyApproved) {
        TINY_ASIDE
    } else {
        TINY_GUARDING
    }
}

const BY_ELEVATOR: Rect = Rect::new(180, 101, 290, 140);

pub fn hotspots(state: &GameState) -> Vec<Hotspot> {
    let (tx, ty) = tiny_pos(state);
    vec![
        hs(
            Noun::Tiny,
            Rect::new(tx - 8, ty - 34, tx + 8, ty + 1),
            BY_ELEVATOR,
            (262, 124),
        ),
        hs(
            Noun::Elevator,
            Rect::new(212, 18, 264, 101),
            BY_ELEVATOR,
            (250, 103),
        ),
        hs(
            Noun::Slots,
            Rect::new(16, 54, 124, 112),
            Rect::new(8, 112, 140, 140),
            (70, 122),
        ),
        hs(
            Noun::Plant,
            Rect::new(278, 100, 314, 172),
            Rect::new(250, 130, 316, 176),
            (268, 160),
        ),
        hs(
            Noun::Chandelier,
            Rect::new(136, STATUS_H, 184, 42),
            Rect::new(0, 0, 320, 200),
            (160, 140),
        ),
        hs(
            Noun::Door,
            Rect::new(110, 164, 210, PIC_BOTTOM),
            Rect::new(100, 150, 220, 176),
            (160, 171),
        ),
    ]
}

pub fn actors(state: &GameState, ticks: u64) -> Vec<Placed> {
    let (tx, ty) = tiny_pos(state);
    let mut v = vec![Placed::feet(
        npcs::tiny((ticks / 8 % 6) as u8),
        tx,
        ty,
        priority_for_y(ty),
    )];
    // Spinning reels.
    let reel = [RED, YELLOW, LIGHT_GREEN, LIGHT_MAGENTA];
    for (i, sx) in SLOTS.iter().enumerate() {
        for r in 0..3 {
            let col = reel[((ticks / 2) as usize + i * 3 + r * 5) % reel.len()];
            v.push(Placed::top_left(
                rect_sprite(4, 6, col),
                sx + 6 + r as i32 * 6,
                73,
                15,
            ));
        }
    }
    // Chandelier glints.
    let g = (ticks / 5 % 5) as i32;
    v.push(Placed::top_left(
        rect_sprite(1, 1, WHITE),
        146 + g * 6,
        36,
        15,
    ));
    v
}

pub fn draw(state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Red velvet walls with gold trim.
    p.fill_rect(0, 0, 320, 100, c(RED));
    p.fill_rect(0, STATUS_H, 320, 16, c(BROWN));
    p.line(0, 16, 319, 16, c(YELLOW));
    for x in (10..320).step_by(40) {
        p.line(x, 20, x, 99, c(YELLOW));
    }
    for y in (26..96).step_by(14) {
        for x in (30..320).step_by(40) {
            p.polyline(
                &[(x - 4, y), (x, y - 4), (x + 4, y), (x, y + 4), (x - 4, y)],
                c(MAGENTA),
            );
        }
    }
    p.line(0, 99, 319, 99, c(YELLOW));

    // Chandelier.
    p.line(160, STATUS_H, 160, 20, c(YELLOW));
    p.fill_polygon(&[(140, 20), (180, 20), (170, 32), (150, 32)], c(YELLOW));
    for x in (144..178).step_by(4) {
        p.line(x, 32, x, 34 + (x % 3) * 2, c(LIGHT_CYAN));
    }

    // Carpet: loud, as casino carpets are legally required to be.
    p.fill_rect(0, 100, 320, PIC_BOTTOM, c(MAGENTA));
    for y in (104..PIC_BOTTOM).step_by(8) {
        let off = if (y / 8) % 2 == 0 { 0 } else { 6 };
        for x in (off..320).step_by(12) {
            p.plot(x, y, c(YELLOW));
            p.plot(x + 1, y, c(YELLOW));
        }
    }
    p.fill_rect(4, 101, 316, PIC_BOTTOM, Pen::control_only(CTRL_WALK));

    // Elevator to the Crow's Nest (doors open once Tiny approves).
    p.fill_rect(216, 40, 261, 100, c(YELLOW));
    if state.flag(Flag::TinyApproved) {
        p.fill_rect(220, 46, 257, 100, c(BLACK));
        p.fill_rect(222, 48, 255, 52, c(YELLOW));
    } else {
        p.fill_rect(220, 46, 257, 100, c(LIGHT_GRAY));
        p.line(238, 46, 238, 99, c(DARK_GRAY));
    }
    p.fill_rect(222, 22, 255, 30, c(BLACK));
    p.fill_rect(224, 24, 253, 28, c(LIGHT_CYAN)); // CROW'S NEST
    p.fill_polygon(&[(238, 32), (242, 37), (234, 37)], c(YELLOW));

    // Slot machines.
    let sp = priority_for_y(112);
    for sx in SLOTS {
        p.fill_ellipse(sx + 14, 62, 14, 6, Pen::color(RED).pri(sp));
        p.fill_rect(sx, 62, sx + 28, 110, Pen::color(LIGHT_GRAY).pri(sp));
        p.rect(sx, 62, sx + 28, 110, Pen::color(DARK_GRAY).pri(sp));
        p.fill_rect(sx + 4, 70, sx + 24, 82, Pen::color(WHITE).pri(sp));
        p.fill_rect(sx + 6, 96, sx + 22, 102, Pen::color(DARK_GRAY).pri(sp));
        p.fill_rect(sx + 3, 86, sx + 25, 90, Pen::color(YELLOW).pri(sp));
        p.line(sx + 29, 66, sx + 29, 84, Pen::color(LIGHT_GRAY).pri(sp));
        p.fill_ellipse(sx + 29, 64, 2, 2, Pen::color(RED).pri(sp));
    }
    p.fill_rect(16, 100, 124, 114, Pen::control_only(CTRL_BLOCK));

    // A potted palm in the foreground: Chip walks behind it.
    let pp = priority_for_y(171);
    for (dx, dy) in [
        (-18, -30),
        (-10, -42),
        (4, -46),
        (16, -36),
        (20, -22),
        (-20, -14),
    ] {
        let (bx, by) = (296, 150);
        p.fill_polygon(
            &[(bx, by), (bx + dx, by + dy), (bx + dx / 2 + 3, by + dy / 2)],
            Pen::color(GREEN).pri(pp),
        );
        p.line(bx, by, bx + dx, by + dy, Pen::color(LIGHT_GREEN).pri(pp));
    }
    p.fill_rect(294, 150, 298, 156, Pen::color(BROWN).pri(pp));
    p.fill_polygon(
        &[(284, 154), (308, 154), (304, 172), (288, 172)],
        Pen::color(BROWN).pri(pp),
    );
    p.line(284, 156, 307, 156, Pen::color(YELLOW).pri(pp));
    p.fill_rect(284, 162, 309, 174, Pen::control_only(CTRL_BLOCK));

    // Wherever Tiny stands is not walkable.
    let (tx, ty) = tiny_pos(state);
    p.fill_rect(
        tx - 8,
        ty - 4,
        tx + 9,
        ty + 2,
        Pen::control_only(CTRL_BLOCK),
    );
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palm_hides_chip_and_tiny_blocks() {
        let s = GameState::new();
        let p = draw(&s);
        assert!(p.priority_at(296, 153).unwrap() > priority_for_y(150));
        assert!(!p.walkable(TINY_GUARDING.0, TINY_GUARDING.1));
    }
}
