//! Suds & Duds: a 24-hour laundromat, Dee, and a locked dryer.

use super::{c, hs, npcs, text, Exit, Hotspot, Lcg, Placed, Rect};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::game::parser::Noun;
use crate::game::state::{Flag, GameState, SceneId};

pub const DEE_POS: (i32, i32) = (136, 116);

pub const EXITS: [Exit; 1] = [Exit {
    zone: Rect::new(312, 101, 320, 174),
    to: SceneId::Alley,
    arrive: (16, 140),
    nouns: &[Noun::Alley],
    back: (300, 140),
}];

/// Left edge of dryer column `i` (0-based; "Dryer 3" is column 2).
fn dryer_x(i: i32) -> i32 {
    184 + i * 32
}

pub fn hotspots(_state: &GameState) -> Vec<Hotspot> {
    let dryers = Rect::new(176, 100, 316, 136);
    vec![
        hs(
            Noun::Dee,
            Rect::new(128, 92, 146, 118),
            Rect::new(100, 132, 176, 164),
            (136, 140),
        ),
        hs(
            Noun::Sign,
            Rect::new(dryer_x(2) + 4, 76, dryer_x(2) + 26, 90),
            dryers,
            (dryer_x(2) + 15, 108),
        ),
        hs(
            Noun::Lint,
            Rect::new(20, 136, 46, 156),
            Rect::new(8, 128, 70, 172),
            (54, 150),
        ),
        hs(
            Noun::Dryer,
            Rect::new(dryer_x(0), 32, dryer_x(3) + 30, 100),
            dryers,
            (dryer_x(2) + 15, 108),
        ),
        hs(
            Noun::Washer,
            Rect::new(10, 50, 112, 100),
            Rect::new(8, 100, 120, 126),
            (60, 108),
        ),
    ]
}

pub fn actors(_state: &GameState, ticks: u64) -> Vec<Placed> {
    let mut v = vec![Placed::feet(
        npcs::dee((ticks / 12 % 2) as u8),
        DEE_POS.0,
        DEE_POS.1,
        priority_for_y(DEE_POS.1),
    )];
    // Suds creep up the window of the middle washer.
    let h = (ticks / 6 % 5) as i32;
    v.push(Placed::top_left(
        super::rect_sprite(9, 1 + h, WHITE),
        57,
        78 - h,
        priority_for_y(100),
    ));
    v
}

pub fn draw(state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Pale green walls, a tiled floor, a neon sign.
    p.fill_rect(0, 0, 320, 100, c(LIGHT_CYAN));
    for y in (STATUS_BAND..100).step_by(10) {
        p.line(0, y, 319, y, c(CYAN));
    }
    p.fill_rect(40, 12, 180, 40, c(BLACK));
    p.rect(39, 11, 181, 41, c(MAGENTA));
    text(&mut p, 46, 15, "SUDS & DUDS", LIGHT_MAGENTA, 2);
    text(&mut p, 77, 31, "OPEN 24 HRS", YELLOW, 1);
    // A flickering fluorescent tube.
    p.fill_rect(4, 20, 34, 23, c(WHITE));
    p.fill_rect(264, 20, 300, 23, c(WHITE));

    p.fill_rect(0, 100, 320, 176, c(WHITE));
    p.line(0, 100, 319, 100, c(BLACK));
    for y in (100..176).step_by(10) {
        for x in (0..320).step_by(20) {
            let off = if ((y - 100) / 10) % 2 == 0 { 0 } else { 10 };
            p.fill_rect(x + off, y, x + off + 10, y + 10, c(LIGHT_GRAY));
        }
    }
    p.fill_rect(4, 101, 320, 174, Pen::control_only(CTRL_WALK));

    // Front-loading washers along the left wall.
    for i in 0..3 {
        let x0 = 12 + i * 34;
        p.fill_rect(x0, 56, x0 + 32, 100, c(WHITE));
        p.rect(x0, 56, x0 + 32, 100, c(DARK_GRAY));
        p.fill_rect(x0 + 2, 58, x0 + 30, 66, c(LIGHT_GRAY));
        p.fill_rect(x0 + 4, 60, x0 + 8, 64, c(RED));
        p.fill_rect(x0 + 22, 60, x0 + 28, 64, c(BLUE));
        p.fill_ellipse(x0 + 16, 83, 12, 12, c(DARK_GRAY));
        p.fill_ellipse(x0 + 16, 83, 9, 9, c(LIGHT_CYAN));
        p.fill_ellipse(x0 + 16, 86, 9, 5, c(LIGHT_BLUE));
        if i != 1 {
            p.plot(x0 + 12, 80, c(WHITE));
            p.plot(x0 + 19, 84, c(WHITE));
        }
    }

    // Dryers: two tiers, four columns. Dryer 3 is the broken one.
    for i in 0..4 {
        let x0 = dryer_x(i);
        for (y0, y1) in [(32, 64), (66, 100)] {
            p.fill_rect(x0, y0, x0 + 30, y1, c(LIGHT_GRAY));
            p.rect(x0, y0, x0 + 30, y1, c(DARK_GRAY));
            p.fill_ellipse(x0 + 15, (y0 + y1) / 2 + 3, 11, 11, c(DARK_GRAY));
            p.fill_ellipse(x0 + 15, (y0 + y1) / 2 + 3, 8, 8, c(BLACK));
            p.fill_rect(x0 + 3, y0 + 2, x0 + 8, y0 + 5, c(RED));
        }
    }
    let x0 = dryer_x(2);
    if state.flag(Flag::DryerOpen) {
        // The door hangs open; the drum is empty.
        p.fill_polygon(
            &[
                (x0 + 6, 75),
                (x0 + 26, 75),
                (x0 + 36, 82),
                (x0 + 36, 94),
                (x0 + 26, 97),
                (x0 + 6, 97),
            ],
            c(DARK_GRAY),
        );
        p.fill_ellipse(x0 + 15, 86, 9, 9, c(BLACK));
        p.fill_rect(x0 + 2, 70, x0 + 26, 72, c(YELLOW));
    } else {
        // Sequins glitter inside the locked drum.
        let mut rng = Lcg(5);
        for _ in 0..16 {
            let (x, y) = (x0 + 9 + rng.next(13), 79 + rng.next(14));
            p.plot(x, y, c(if rng.next(2) == 0 { YELLOW } else { WHITE }));
        }
        p.fill_rect(x0 + 24, 82, x0 + 29, 88, c(BROWN)); // lock plate
        p.plot(x0 + 26, 85, c(BLACK));
        p.line(x0 + 26, 85, x0 + 26, 87, c(BLACK));
        // OUT OF ORDER sign, taped crooked.
        p.fill_rect(x0 + 1, 73, x0 + 31, 96, c(YELLOW));
        p.rect(x0 + 1, 73, x0 + 31, 96, c(BLACK));
        text(&mut p, x0 + 2, 75, "OUT", RED, 1);
        text(&mut p, x0 + 2, 82, "OF", RED, 1);
        text(&mut p, x0 + 2, 89, "ORDER", RED, 1);
    }
    text(&mut p, x0 + 12, 67, "3", WHITE, 1);

    // Dee's folding table; its front hides her legs.
    let tp = priority_for_y(134);
    p.fill_rect(98, 116, 176, 124, Pen::color(BROWN).pri(tp));
    p.fill_rect(98, 124, 176, 134, Pen::color(LIGHT_RED).pri(tp));
    p.rect(98, 116, 176, 134, Pen::color(BLACK).pri(tp));
    p.fill_rect(104, 112, 120, 116, Pen::color(WHITE).pri(tp));
    p.fill_rect(150, 108, 168, 116, Pen::color(LIGHT_CYAN).pri(tp));
    p.fill_rect(152, 112, 166, 116, Pen::color(CYAN).pri(tp));
    p.fill_rect(98, 112, 176, 134, Pen::control_only(CTRL_BLOCK));

    // A rolling lint bin, overflowing.
    let lp = priority_for_y(158);
    p.fill_rect(20, 138, 46, 156, Pen::color(DARK_GRAY).pri(lp));
    p.rect(20, 138, 46, 156, Pen::color(BLACK).pri(lp));
    p.fill_ellipse(33, 138, 14, 4, Pen::color(LIGHT_GRAY).pri(lp));
    p.fill_ellipse(30, 135, 5, 3, Pen::color(WHITE).pri(lp));
    p.fill_ellipse(37, 136, 4, 3, Pen::color(LIGHT_GRAY).pri(lp));
    p.fill_rect(18, 132, 48, 158, Pen::control_only(CTRL_BLOCK));
    // An orange plastic chair.
    p.fill_rect(268, 130, 288, 134, c(LIGHT_RED));
    p.fill_rect(268, 120, 272, 134, c(LIGHT_RED));
    p.fill_rect(270, 134, 273, 148, c(DARK_GRAY));
    p.fill_rect(284, 134, 287, 148, c(DARK_GRAY));
    p.fill_rect(266, 128, 290, 150, Pen::control_only(CTRL_BLOCK));
    p
}

const STATUS_BAND: i32 = 9;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dryer_door_swings_open() {
        let mut s = GameState::new();
        s.scene = SceneId::Laundromat;
        let shut = draw(&s);
        s.set(Flag::DryerOpen);
        assert_ne!(shut.visual, draw(&s).visual);
    }

    #[test]
    fn table_hides_dees_legs() {
        let p = draw(&GameState::new());
        assert!(p.priority_at(DEE_POS.0, 125).unwrap() > priority_for_y(DEE_POS.1));
        assert!(!p.walkable(DEE_POS.0, 125));
    }
}
