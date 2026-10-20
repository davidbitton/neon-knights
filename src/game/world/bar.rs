//! The Rusty Anchor, a dive bar on the boardwalk.

use super::{c, hs, rect_sprite, Exit, Hotspot, Lcg, Placed, Rect, PIC_BOTTOM};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::engine::sprite::{gus_frame, Sprite};
use crate::game::parser::Noun;
use crate::game::state::{Flag, GameState, SceneId};

pub const STOOLS: [i32; 4] = [62, 102, 142, 182];
pub const GUS_POS: (i32, i32) = (151, 85);

const FRONT_OF_BAR: Rect = Rect::new(30, 115, 220, 128);
const BY_TABLE: Rect = Rect::new(226, 101, 316, 174);

pub const EXITS: [Exit; 1] = [Exit {
    zone: Rect::new(262, 101, 296, 105),
    to: SceneId::Boardwalk,
    arrive: (51, 110),
    nouns: &[Noun::Door, Noun::Boardwalk],
    back: (278, 112),
}];

pub fn hotspots(state: &GameState) -> Vec<Hotspot> {
    let mut v = Vec::new();
    if !state.flag(Flag::TookMatchbook) {
        v.push(hs(
            Noun::Matchbook,
            Rect::new(111, 74, 129, 84),
            Rect::new(95, 115, 145, 128),
            (120, 118),
        ));
    }
    v.push(hs(
        Noun::Egg,
        Rect::new(185, 64, 201, 82),
        Rect::new(170, 115, 220, 128),
        (193, 118),
    ));
    v.push(hs(
        Noun::Bartender,
        Rect::new(138, 58, 164, 80),
        FRONT_OF_BAR,
        (151, 118),
    ));
    v.push(hs(
        Noun::Jukebox,
        Rect::new(219, 54, 253, 108),
        Rect::new(204, 101, 262, 128),
        (236, 112),
    ));
    v.push(hs(
        Noun::Door,
        Rect::new(259, 28, 297, 101),
        Rect::new(250, 101, 316, 122),
        (278, 103),
    ));
    v.push(hs(
        Noun::Restroom,
        Rect::new(5, 44, 35, 101),
        Rect::new(4, 101, 42, 126),
        (18, 108),
    ));
    v.push(hs(
        Noun::Patron,
        Rect::new(262, 106, 300, 124),
        BY_TABLE,
        (248, 140),
    ));
    v.push(hs(
        Noun::Table,
        Rect::new(256, 122, 304, 154),
        BY_TABLE,
        (248, 140),
    ));
    for sx in STOOLS {
        v.push(hs(
            Noun::Stool,
            Rect::new(sx - 7, 113, sx + 7, 134),
            Rect::new(30, 113, 220, 150),
            (sx, 137),
        ));
    }
    v.push(hs(
        Noun::Sign,
        Rect::new(56, 17, 196, 35),
        Rect::new(0, 0, 320, 200),
        (120, 140),
    ));
    v.push(hs(
        Noun::Bottles,
        Rect::new(42, 35, 208, 76),
        FRONT_OF_BAR,
        (100, 118),
    ));
    v.push(hs(
        Noun::Bar,
        Rect::new(32, 76, 218, 114),
        FRONT_OF_BAR,
        (100, 118),
    ));
    v
}

fn note_sprite() -> Sprite {
    Sprite::from_rows(&["..YY", "..Y.", "..Y.", "YYY.", "YYY."])
}

pub(crate) fn z_sprite() -> Sprite {
    Sprite::from_rows(&["WWW", "..W", ".W.", "WWW"])
}

pub fn actors(state: &GameState, ticks: u64) -> Vec<Placed> {
    let mut v = Vec::new();
    if state.flag(Flag::PlayedJukebox) {
        let cols = [LIGHT_CYAN, LIGHT_MAGENTA, YELLOW, LIGHT_GREEN];
        v.push(Placed::top_left(
            rect_sprite(21, 8, cols[(ticks / 4 % 4) as usize]),
            226,
            70,
            15,
        ));
        let rise = (ticks % 30) as i32;
        v.push(Placed::top_left(
            note_sprite(),
            240 + rise / 6,
            54 - rise,
            15,
        ));
    }
    let zr = (ticks % 40) as i32;
    if zr < 30 {
        v.push(Placed::top_left(z_sprite(), 292 + zr / 6, 104 - zr / 2, 15));
    }
    v.push(Placed::feet(
        gus_frame((ticks / 12 % 2) as u8),
        GUS_POS.0,
        GUS_POS.1,
        priority_for_y(GUS_POS.1),
    ));
    v
}

/// The Rusty Anchor, a bar that has seen better decades.
pub fn draw(state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Ceiling and back wall (teal plaster, water-stained planks).
    p.fill_rect(0, 0, 320, 18, c(BLACK));
    p.fill_rect(0, 18, 320, 100, c(CYAN));
    for x in (0..320).step_by(20) {
        p.line(x, 18, x, 84, c(BLUE));
    }
    p.fill_rect(0, 84, 320, 100, c(BROWN)); // wainscot
    p.line(0, 84, 319, 84, c(BLACK));
    p.fill_ellipse(214, 30, 6, 4, c(LIGHT_GRAY)); // a water stain shaped like regret
    p.fill_ellipse(40, 26, 4, 3, c(LIGHT_GRAY));

    // Floor: sticky planks in perspective; walkable.
    p.fill_rect(0, 100, 320, PIC_BOTTOM, c(DARK_GRAY));
    p.line(0, 100, 319, 100, c(BLACK));
    for i in -9..=9 {
        p.line(160 + i * 18, 100, 160 + i * 52, PIC_BOTTOM - 1, c(BLACK));
    }
    p.fill_rect(4, 101, 316, 174, Pen::control_only(CTRL_WALK));
    p.fill_ellipse(150, 160, 13, 3, c(BLUE)); // the puddle
    p.fill_ellipse(148, 159, 5, 1, c(LIGHT_BLUE));
    let mut rng = Lcg(7);
    for _ in 0..25 {
        let (x, y) = (rng.next(300) + 10, rng.next(60) + 112);
        p.plot(x, y, c(YELLOW)); // peanut shells
    }

    // Back shelf with mirror and bottles.
    p.fill_rect(44, 36, 206, 75, c(DARK_GRAY));
    p.rect(43, 35, 207, 76, c(LIGHT_GRAY));
    p.fill_rect(46, 38, 204, 50, c(LIGHT_GRAY));
    p.line(46, 50, 204, 50, c(BLACK));
    for (y0, y1) in [(51, 53), (72, 74)] {
        p.fill_rect(44, y0, 206, y1, c(BROWN));
    }
    let bottle_colors = [
        GREEN,
        YELLOW,
        LIGHT_RED,
        LIGHT_GREEN,
        MAGENTA,
        WHITE,
        BROWN,
        LIGHT_BLUE,
    ];
    let mut rng = Lcg(42);
    for (shelf_y, start) in [(51, 50), (72, 48)] {
        let mut x = start;
        let mut i = 0;
        while x < 198 {
            let h = 6 + rng.next(6);
            let col = bottle_colors[(i + rng.next(3) as usize) % bottle_colors.len()];
            p.fill_rect(x, shelf_y - h, x + 4, shelf_y, c(col));
            p.fill_rect(x + 1, shelf_y - h - 3, x + 3, shelf_y - h, c(col));
            x += 6 + rng.next(4);
            i += 1;
        }
    }

    // Neon: an anchor flanked by tubes.
    let ax = 126;
    p.rect(ax - 1, 19, ax + 2, 22, c(LIGHT_MAGENTA));
    p.line(ax, 22, ax, 33, c(LIGHT_MAGENTA));
    p.line(ax - 4, 24, ax + 4, 24, c(LIGHT_MAGENTA));
    p.polyline(
        &[
            (ax - 7, 29),
            (ax - 5, 32),
            (ax, 33),
            (ax + 5, 32),
            (ax + 7, 29),
        ],
        c(LIGHT_MAGENTA),
    );
    p.line(ax - 7, 29, ax - 8, 31, c(LIGHT_MAGENTA));
    p.line(ax + 7, 29, ax + 8, 31, c(LIGHT_MAGENTA));
    for (x0, x1) in [(60, 112), (140, 192)] {
        p.line(x0, 26, x1, 26, c(LIGHT_CYAN));
        p.line(x0, 28, x1, 28, c(LIGHT_MAGENTA));
    }

    // Hanging lamps.
    for lx in [70, 186] {
        p.line(lx, 0, lx, 12, c(DARK_GRAY));
        p.fill_polygon(
            &[(lx - 7, 18), (lx + 7, 18), (lx + 3, 12), (lx - 3, 12)],
            c(GREEN),
        );
        p.fill_rect(lx - 1, 18, lx + 2, 20, c(YELLOW));
    }

    // Restroom door, duct-taped shut.
    p.fill_rect(8, 46, 33, 100, c(LIGHT_GRAY));
    p.rect(7, 45, 34, 101, c(BLACK));
    p.fill_rect(14, 54, 27, 62, c(WHITE));
    p.rect(14, 54, 27, 62, c(BLACK));
    p.line(9, 62, 32, 82, c(YELLOW));
    p.line(9, 63, 32, 83, c(YELLOW));
    p.line(32, 62, 9, 82, c(YELLOW));
    p.line(32, 63, 9, 83, c(YELLOW));
    p.plot(29, 75, c(BLACK));

    // Front door with a porthole.
    p.fill_rect(262, 38, 295, 100, c(BROWN));
    p.rect(261, 37, 296, 101, c(BLACK));
    p.fill_ellipse(278, 56, 7, 7, c(YELLOW));
    p.fill_ellipse(278, 56, 5, 5, c(BLUE));
    p.plot(276, 54, c(WHITE));
    p.fill_rect(266, 70, 269, 73, c(YELLOW));
    p.fill_rect(270, 29, 287, 35, c(RED));
    p.line(272, 32, 285, 32, c(LIGHT_RED));

    // Jukebox (lights up once it's been fed).
    let jpri = priority_for_y(106);
    let lit = state.flag(Flag::PlayedJukebox);
    p.fill_ellipse(236, 66, 14, 9, Pen::color(YELLOW).pri(jpri));
    p.fill_ellipse(
        236,
        67,
        11,
        7,
        Pen::color(if lit { LIGHT_MAGENTA } else { MAGENTA }).pri(jpri),
    );
    p.fill_rect(222, 67, 251, 106, Pen::color(RED).pri(jpri));
    p.rect(222, 67, 251, 106, Pen::color(BLACK).pri(jpri));
    p.fill_rect(
        226,
        70,
        247,
        78,
        Pen::color(if lit { LIGHT_CYAN } else { CYAN }).pri(jpri),
    );
    p.fill_rect(226, 82, 247, 100, Pen::color(BLACK).pri(jpri));
    for x in (227..247).step_by(3) {
        p.line(
            x,
            83,
            x,
            99,
            Pen::color(if lit { YELLOW } else { BROWN }).pri(jpri),
        );
    }
    p.fill_rect(234, 79, 239, 81, Pen::color(DARK_GRAY).pri(jpri)); // coin slot
    p.fill_rect(218, 100, 255, 109, Pen::control_only(CTRL_BLOCK));

    // The bar counter. Its priority hides Gus from the waist down.
    let bpri = priority_for_y(112);
    p.fill_rect(70, 70, 72, 80, Pen::color(LIGHT_GRAY).pri(bpri)); // beer tap
    p.fill_rect(69, 66, 73, 70, Pen::color(BLACK).pri(bpri));
    p.fill_rect(34, 80, 216, 86, Pen::color(RED).pri(bpri));
    p.line(34, 80, 215, 80, Pen::color(LIGHT_RED).pri(bpri));
    p.fill_rect(36, 86, 214, 112, Pen::color(BROWN).pri(bpri));
    for x in (36..214).step_by(22) {
        p.rect(x + 3, 89, x + 19, 105, Pen::color(BLACK).pri(bpri));
    }
    p.line(36, 108, 213, 108, Pen::color(YELLOW).pri(bpri)); // brass foot rail
    p.fill_rect(36, 110, 214, 112, Pen::color(BLACK).pri(bpri));
    p.fill_rect(30, 100, 220, 115, Pen::control_only(CTRL_BLOCK));

    // On the counter: the matchbook (until taken) and the egg jar.
    if !state.flag(Flag::TookMatchbook) {
        p.fill_rect(116, 77, 125, 80, Pen::color(RED).pri(bpri));
        p.line(117, 78, 123, 78, Pen::color(WHITE).pri(bpri));
    }
    p.fill_rect(188, 66, 199, 69, Pen::color(RED).pri(bpri));
    p.fill_rect(188, 69, 199, 80, Pen::color(LIGHT_GREEN).pri(bpri));
    p.rect(187, 68, 200, 80, Pen::color(WHITE).pri(bpri));
    for (ex, ey) in [(191, 76), (196, 75), (193, 72)] {
        p.fill_ellipse(ex, ey, 2, 1, Pen::color(YELLOW).pri(bpri));
    }

    // Bar stools: the hero can walk behind (north of) them.
    let spri = priority_for_y(133);
    for sx in STOOLS {
        p.fill_rect(sx - 1, 119, sx + 2, 131, Pen::color(LIGHT_GRAY).pri(spri));
        p.fill_ellipse(sx, 117, 7, 2, Pen::color(LIGHT_RED).pri(spri));
        p.line(sx - 6, 118, sx + 6, 118, Pen::color(RED).pri(spri));
        p.fill_ellipse(sx, 131, 5, 1, Pen::color(BLACK).pri(spri));
        p.fill_rect(sx - 5, 128, sx + 6, 134, Pen::control_only(CTRL_BLOCK));
    }

    // Corner table with a dozing regular.
    let tpri = priority_for_y(153);
    p.fill_rect(266, 110, 295, 126, Pen::color(BLUE).pri(tpri));
    p.fill_ellipse(280, 118, 6, 4, Pen::color(LIGHT_RED).pri(tpri));
    p.fill_rect(271, 108, 290, 113, Pen::color(WHITE).pri(tpri));
    p.line(270, 113, 291, 113, Pen::color(BLACK).pri(tpri));
    p.fill_rect(276, 109, 285, 110, Pen::color(YELLOW).pri(tpri));
    p.fill_ellipse(280, 127, 23, 5, Pen::color(BROWN).pri(tpri));
    p.line(258, 127, 302, 127, Pen::color(RED).pri(tpri));
    p.fill_rect(296, 116, 299, 125, Pen::color(GREEN).pri(tpri));
    p.fill_rect(278, 132, 282, 150, Pen::color(BLACK).pri(tpri));
    p.fill_ellipse(280, 151, 9, 2, Pen::color(BLACK).pri(tpri));
    p.fill_rect(260, 128, 300, 155, Pen::control_only(CTRL_BLOCK));

    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::world::{hotspot_at, hotspot_for, is_near};

    #[test]
    fn counter_hides_gus_legs_and_stools_occlude_hero() {
        let pic = draw(&GameState::new());
        let gus_pri = priority_for_y(GUS_POS.1);
        assert!(pic.priority_at(GUS_POS.0, 84).unwrap() > gus_pri);
        assert!(pic.priority_at(GUS_POS.0, 66).unwrap() <= gus_pri);
        let sx = STOOLS[1];
        assert!(pic.priority_at(sx, 125).unwrap() > priority_for_y(122));
        assert!(pic.priority_at(sx, 125).unwrap() <= priority_for_y(140));
        assert!(!pic.walkable(120, 105));
    }

    #[test]
    fn matchbook_disappears_once_taken() {
        let mut s = GameState::new();
        assert!(hotspot_at(&s, 120, 79).map(|h| h.noun) == Some(Noun::Matchbook));
        s.set(Flag::TookMatchbook);
        assert_ne!(
            hotspot_at(&s, 120, 79).map(|h| h.noun),
            Some(Noun::Matchbook)
        );
        let pic = draw(&s);
        assert_ne!(pic.visual_at(120, 78), Some(RED));
    }

    #[test]
    fn proximity() {
        let mut s = GameState::new();
        assert!(!is_near(&s, Noun::Bartender));
        s.hero = hotspot_for(&s, Noun::Bartender).unwrap().approach;
        assert!(is_near(&s, Noun::Bartender));
        assert!(!is_near(&s, Noun::Jukebox));
    }
}
