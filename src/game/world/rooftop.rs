//! The Crow's Nest: a rooftop lounge atop the Golden Clam.

use super::screens::skyline;
use super::{c, hs, npcs, rect_sprite, stars, Exit, Hotspot, Placed, Rect, PIC_BOTTOM, STATUS_H};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::game::parser::Noun;
use crate::game::state::{GameState, SceneId};

pub const VONDA_POS: (i32, i32) = (106, 127);

pub const EXITS: [Exit; 1] = [Exit {
    zone: Rect::new(140, 101, 168, 105),
    to: SceneId::Casino,
    arrive: (250, 112),
    nouns: &[Noun::Elevator, Noun::Door, Noun::Casino],
    back: (154, 112),
}];

pub fn hotspots(_state: &GameState) -> Vec<Hotspot> {
    vec![
        hs(
            Noun::Vonda,
            Rect::new(99, 96, 114, 128),
            Rect::new(96, 112, 160, 150),
            (124, 134),
        ),
        hs(
            Noun::Piano,
            Rect::new(14, 66, 98, 136),
            Rect::new(14, 136, 130, 160),
            (60, 144),
        ),
        hs(
            Noun::Pool,
            Rect::new(186, 106, 310, 168),
            Rect::new(160, 106, 316, 174),
            (178, 142),
        ),
        hs(
            Noun::Table,
            Rect::new(146, 140, 174, 162),
            Rect::new(120, 130, 200, 174),
            (140, 166),
        ),
        hs(
            Noun::Elevator,
            Rect::new(128, 42, 180, 101),
            Rect::new(120, 101, 190, 126),
            (154, 103),
        ),
        hs(
            Noun::Railing,
            Rect::new(0, 84, 320, 101),
            Rect::new(0, 0, 320, 200),
            (154, 112),
        ),
        hs(
            Noun::City,
            Rect::new(0, STATUS_H, 320, 84),
            Rect::new(0, 0, 320, 200),
            (154, 112),
        ),
    ]
}

pub fn actors(_state: &GameState, ticks: u64) -> Vec<Placed> {
    let mut v = vec![Placed::feet(
        npcs::vonda((ticks / 6 % 3) as u8),
        VONDA_POS.0,
        VONDA_POS.1,
        priority_for_y(VONDA_POS.1),
    )];
    // Party lights strung across the top, blinking in sequence.
    let cols = [LIGHT_RED, YELLOW, LIGHT_GREEN, LIGHT_CYAN, LIGHT_MAGENTA];
    for i in 0..32i32 {
        let x = 5 + i * 10;
        let sag = ((x % 106) - 53).abs();
        let y = 26 - sag / 8;
        let on = !(i as u64 + ticks / 6).is_multiple_of(3);
        let col = if on {
            cols[(i as usize) % cols.len()]
        } else {
            DARK_GRAY
        };
        v.push(Placed::top_left(rect_sprite(2, 2, col), x, y, 15));
    }
    v
}

pub fn draw(_state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    p.fill_rect(0, 0, 320, 100, c(BLACK));
    stars(&mut p, 99, 60, STATUS_H, 50);
    p.fill_ellipse(40, 32, 7, 7, c(WHITE));
    p.fill_ellipse(38, 30, 2, 2, c(LIGHT_GRAY));
    skyline(&mut p, 86);
    // Light string wire.
    for i in 0..32i32 {
        let x = 5 + i * 10;
        let sag = ((x % 106) - 53).abs();
        p.line(
            x,
            26 - sag / 8,
            x + 10,
            26 - (((x + 10) % 106) - 53).abs() / 8,
            c(DARK_GRAY),
        );
    }

    // Railing along the back of the roof.
    p.fill_rect(0, 84, 320, 86, c(LIGHT_GRAY));
    p.fill_rect(0, 94, 320, 95, c(LIGHT_GRAY));
    for x in (0..320).step_by(16) {
        p.line(x, 84, x, 100, c(LIGHT_GRAY));
    }

    // Deck.
    p.fill_rect(0, 100, 320, PIC_BOTTOM, c(BROWN));
    for y in (104..PIC_BOTTOM).step_by(7) {
        p.line(0, y, 319, y, c(BLACK));
    }
    p.line(0, 100, 319, 100, c(BLACK));
    p.fill_rect(4, 101, 316, 174, Pen::control_only(CTRL_WALK));

    // Elevator housing.
    p.fill_rect(132, 48, 177, 100, c(DARK_GRAY));
    p.fill_rect(128, 44, 181, 48, c(LIGHT_GRAY));
    p.fill_rect(140, 58, 169, 100, c(YELLOW));
    p.line(154, 58, 154, 99, c(BROWN));
    p.fill_rect(136, 50, 173, 55, c(LIGHT_CYAN));
    p.polyline(
        &[(148, 53), (152, 51), (154, 53), (156, 51), (160, 53)],
        c(BLACK),
    ); // a crow, allegedly

    // Grand piano.
    let pp = priority_for_y(136);
    p.fill_polygon(&[(22, 96), (80, 96), (40, 68)], Pen::color(BLACK).pri(pp));
    p.line(22, 96, 40, 68, Pen::color(LIGHT_GRAY).pri(pp));
    p.fill_polygon(
        &[(18, 96), (86, 96), (98, 112), (72, 124), (18, 124)],
        Pen::color(BLACK).pri(pp),
    );
    p.fill_rect(24, 112, 72, 117, Pen::color(WHITE).pri(pp));
    for x in (25..72).step_by(3) {
        p.line(x, 112, x, 114, Pen::color(BLACK).pri(pp));
    }
    for lx in [24, 70, 92] {
        p.line(lx, 124, lx, 134, Pen::color(BLACK).pri(pp));
    }
    p.fill_rect(16, 104, 100, 137, Pen::control_only(CTRL_BLOCK));
    // Vonda's microphone.
    p.line(
        117,
        104,
        117,
        127,
        Pen::color(LIGHT_GRAY).pri(priority_for_y(128)),
    );
    p.fill_ellipse(
        117,
        103,
        1,
        2,
        Pen::color(DARK_GRAY).pri(priority_for_y(128)),
    );
    p.fill_rect(98, 120, 120, 129, Pen::control_only(CTRL_BLOCK));

    // The pool. Drained. Very drained.
    p.fill_rect(188, 122, 308, 166, c(LIGHT_CYAN));
    p.fill_rect(194, 126, 302, 162, c(BLUE));
    p.fill_rect(194, 134, 302, 162, c(LIGHT_GRAY));
    for x in (198..302).step_by(10) {
        p.line(x, 134, x, 161, c(WHITE));
    }
    p.fill_ellipse(240, 152, 10, 2, c(GREEN));
    p.plot(266, 146, c(BROWN));
    p.plot(212, 158, c(BROWN));
    for x in [290, 296] {
        p.line(x, 124, x, 140, c(WHITE));
    }
    p.line(290, 130, 296, 130, c(WHITE));
    p.line(290, 136, 296, 136, c(WHITE));
    let sp = priority_for_y(122);
    p.line(306, 108, 306, 122, Pen::color(LIGHT_GRAY).pri(sp));
    p.fill_rect(288, 106, 314, 114, Pen::color(WHITE).pri(sp));
    p.line(290, 110, 311, 110, Pen::color(RED).pri(sp)); // CLOSED FOR REPAIRS
    p.fill_rect(186, 120, 310, 168, Pen::control_only(CTRL_BLOCK));

    // Cocktail table.
    let tp = priority_for_y(162);
    p.fill_ellipse(160, 146, 12, 3, Pen::color(WHITE).pri(tp));
    p.line(160, 149, 160, 161, Pen::color(LIGHT_GRAY).pri(tp));
    p.fill_ellipse(160, 161, 5, 1, Pen::color(LIGHT_GRAY).pri(tp));
    p.fill_rect(164, 138, 166, 144, Pen::color(LIGHT_MAGENTA).pri(tp));
    p.fill_rect(152, 157, 169, 163, Pen::control_only(CTRL_BLOCK));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_is_not_walkable_and_piano_hides_chip() {
        let p = draw(&GameState::new());
        assert!(!p.walkable(240, 145));
        assert!(p.priority_at(50, 110).unwrap() > priority_for_y(120));
    }
}
