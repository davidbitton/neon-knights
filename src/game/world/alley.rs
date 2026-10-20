//! The back alley behind the boardwalk: bricks, a dumpster, and a mime.

use super::{c, hs, npcs, stars, Exit, Hotspot, Lcg, Placed, Rect, PIC_BOTTOM, STATUS_H};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::game::parser::Noun;
use crate::game::state::{Flag, GameState, SceneId};

/// The mime stands on the storm grate until he's been fed.
pub const MIME_BOXED: (i32, i32) = (188, 147);
pub const MIME_FREE: (i32, i32) = (240, 128);

pub const EXITS: [Exit; 1] = [Exit {
    zone: Rect::new(312, 101, 320, 174),
    to: SceneId::Boardwalk,
    arrive: (16, 128),
    nouns: &[Noun::Boardwalk],
    back: (300, 140),
}];

pub fn mime_pos(state: &GameState) -> (i32, i32) {
    if state.flag(Flag::MimeFed) {
        MIME_FREE
    } else {
        MIME_BOXED
    }
}

const BY_GRATE: Rect = Rect::new(150, 128, 230, 174);

pub fn hotspots(state: &GameState) -> Vec<Hotspot> {
    let (mx, my) = mime_pos(state);
    let mime = if state.flag(Flag::MimeFed) {
        hs(
            Noun::Mime,
            Rect::new(mx - 6, my - 26, mx + 6, my + 1),
            Rect::new(210, 110, 280, 160),
            (226, 140),
        )
    } else {
        hs(
            Noun::Mime,
            Rect::new(mx - 6, my - 26, mx + 6, my + 1),
            BY_GRATE,
            (170, 158),
        )
    };
    vec![
        mime,
        hs(
            Noun::Grate,
            Rect::new(174, 144, 203, 155),
            BY_GRATE,
            (170, 158),
        ),
        hs(
            Noun::Trash,
            Rect::new(155, 100, 189, 118),
            Rect::new(140, 112, 200, 140),
            (170, 124),
        ),
        hs(
            Noun::Dumpster,
            Rect::new(80, 68, 154, 115),
            Rect::new(70, 112, 170, 140),
            (118, 122),
        ),
        hs(
            Noun::Door,
            Rect::new(20, 36, 56, 101),
            Rect::new(10, 101, 70, 126),
            (38, 110),
        ),
        hs(
            Noun::FireEscape,
            Rect::new(214, 24, 298, 82),
            Rect::new(196, 101, 316, 132),
            (250, 112),
        ),
        hs(
            Noun::City,
            Rect::new(110, STATUS_H, 210, 24),
            Rect::new(0, 0, 320, 200),
            (250, 112),
        ),
    ]
}

pub fn actors(state: &GameState, ticks: u64) -> Vec<Placed> {
    let (mx, my) = mime_pos(state);
    let boxed = !state.flag(Flag::MimeFed);
    let mut v = vec![Placed::feet(
        npcs::mime((ticks / 8 % 2) as u8, boxed),
        mx,
        my,
        priority_for_y(my),
    )];
    // The bulb over the back door flickers.
    if ticks % 37 > 2 {
        v.push(Placed::top_left(
            super::rect_sprite(3, 3, YELLOW),
            37,
            37,
            15,
        ));
    }
    v
}

pub fn draw(state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Brick walls on three sides, a sliver of sky overhead.
    p.fill_rect(0, 0, 320, 100, c(RED));
    let mut rng = Lcg(3);
    for (row, y) in (STATUS_H..100).step_by(5).enumerate() {
        p.line(0, y, 319, y, c(BROWN));
        let off = if row % 2 == 0 { 0 } else { 6 };
        for x in (off..320).step_by(12) {
            p.line(x, y, x, (y + 4).min(99), c(BROWN));
        }
        if rng.next(4) == 0 {
            let x = rng.next(300);
            p.fill_rect(x, y + 1, x + 11, y + 5, c(LIGHT_RED));
        }
    }
    p.fill_rect(110, 0, 210, 24, c(BLACK));
    stars(&mut p, 11, 14, STATUS_H, 22);
    p.line(110, STATUS_H, 110, 24, c(DARK_GRAY));
    p.line(209, STATUS_H, 209, 24, c(DARK_GRAY));

    // Back door of the Rusty Anchor, with a bare bulb and a dithered glow.
    for y in 40..100 {
        let half = (y - 40) / 3 + 2;
        for x in (38 - half)..(38 + half) {
            if (x + y) % 3 == 0 {
                p.plot(x, y, c(LIGHT_RED));
            }
        }
    }
    p.fill_rect(24, 50, 53, 100, c(DARK_GRAY));
    p.rect(23, 49, 54, 101, c(BLACK));
    p.fill_rect(28, 43, 49, 48, c(WHITE));
    p.line(30, 45, 47, 45, c(RED));
    p.fill_rect(48, 74, 51, 77, c(LIGHT_GRAY));

    // Fire escape, its bottom ladder well out of reach.
    let k = c(BLACK);
    for py in [32, 60] {
        p.fill_rect(216, py, 296, py + 2, k);
        for x in (216..297).step_by(6) {
            p.line(x, py - 8, x, py, k);
        }
        p.line(216, py - 8, 296, py - 8, k);
    }
    p.fill_rect(236, 38, 270, 56, c(DARK_GRAY));
    p.fill_rect(238, 40, 252, 54, c(BLUE));
    p.polyline(&[(226, 60), (272, 34)], k);
    p.polyline(&[(228, 61), (274, 35)], k);
    p.line(286, 62, 286, 80, k);
    p.line(292, 62, 292, 80, k);
    for y in (64..80).step_by(4) {
        p.line(286, y, 292, y, k);
    }

    // Asphalt.
    p.fill_rect(0, 100, 320, PIC_BOTTOM, c(DARK_GRAY));
    p.line(0, 100, 319, 100, c(BLACK));
    p.polyline(&[(20, 130), (40, 136), (52, 150), (90, 156)], c(BLACK));
    p.polyline(&[(250, 150), (270, 160), (300, 162)], c(BLACK));
    p.fill_ellipse(60, 160, 14, 3, c(BLUE));
    p.fill_ellipse(276, 116, 9, 2, c(BLUE));
    p.fill_rect(4, 101, 320, 174, Pen::control_only(CTRL_WALK));

    // Storm grate (a few green pixels say someone lost something).
    p.fill_rect(176, 146, 201, 153, c(BLACK));
    for x in (177..201).step_by(3) {
        p.line(x, 147, x, 152, c(LIGHT_GRAY));
    }
    if !state.flag(Flag::GotFiver) {
        p.fill_rect(184, 149, 188, 151, c(GREEN));
    }

    // Dumpster.
    let dp = priority_for_y(114);
    p.fill_polygon(
        &[(84, 80), (150, 80), (146, 112), (88, 112)],
        Pen::color(GREEN).pri(dp),
    );
    p.fill_rect(82, 73, 153, 80, Pen::color(GREEN).pri(dp));
    p.rect(82, 73, 153, 80, Pen::color(BLACK).pri(dp));
    p.fill_rect(98, 88, 136, 92, Pen::color(YELLOW).pri(dp));
    p.line(90, 96, 144, 96, Pen::color(BLACK).pri(dp));
    for wx in [92, 142] {
        p.fill_ellipse(wx, 113, 3, 2, Pen::color(BLACK).pri(dp));
    }
    p.fill_rect(84, 100, 152, 116, Pen::control_only(CTRL_BLOCK));

    // Trash bags.
    let tb = priority_for_y(117);
    p.fill_ellipse(166, 108, 9, 7, Pen::color(BLACK).pri(tb));
    p.fill_ellipse(180, 111, 7, 6, Pen::color(BLACK).pri(tb));
    p.plot(164, 104, Pen::color(LIGHT_GRAY).pri(tb));
    p.plot(179, 107, Pen::color(LIGHT_GRAY).pri(tb));
    p.fill_rect(156, 102, 188, 118, Pen::control_only(CTRL_BLOCK));

    // Wherever the mime is standing is not walkable.
    let (mx, my) = mime_pos(state);
    p.fill_rect(
        mx - 5,
        my - 3,
        mx + 6,
        my + 2,
        Pen::control_only(CTRL_BLOCK),
    );
    p
}
