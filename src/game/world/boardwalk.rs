//! The boardwalk: the hub connecting the bar, the alley, the store and the casino.

use super::{
    c, hs, npcs, rect_sprite, stars, Exit, Hotspot, Lcg, Placed, Rect, PIC_BOTTOM, STATUS_H,
};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::game::parser::Noun;
use crate::game::state::{GameState, SceneId};

pub const DOT_POS: (i32, i32) = (168, 127);

pub const EXITS: [Exit; 4] = [
    Exit {
        zone: Rect::new(38, 101, 64, 105),
        to: SceneId::Bar,
        arrive: (278, 112),
        nouns: &[Noun::Bar],
        back: (51, 112),
    },
    Exit {
        zone: Rect::new(140, 101, 164, 105),
        to: SceneId::Store,
        arrive: (160, 160),
        nouns: &[Noun::Store],
        back: (152, 112),
    },
    Exit {
        zone: Rect::new(244, 101, 278, 105),
        to: SceneId::Casino,
        arrive: (160, 160),
        nouns: &[Noun::Casino],
        back: (261, 112),
    },
    Exit {
        zone: Rect::new(0, 101, 8, 150),
        to: SceneId::Alley,
        arrive: (300, 140),
        nouns: &[Noun::Alley],
        back: (16, 128),
    },
];

const BY_CART: Rect = Rect::new(130, 100, 216, 150);

pub fn hotspots(_state: &GameState) -> Vec<Hotspot> {
    vec![
        hs(
            Noun::Dot,
            Rect::new(159, 100, 178, 118),
            BY_CART,
            (202, 130),
        ),
        hs(
            Noun::Cart,
            Rect::new(146, 80, 208, 142),
            BY_CART,
            (202, 130),
        ),
        hs(
            Noun::Telescope,
            Rect::new(287, 114, 305, 147),
            Rect::new(266, 118, 316, 150),
            (282, 140),
        ),
        hs(
            Noun::Bar,
            Rect::new(2, 32, 98, 101),
            Rect::new(20, 101, 80, 120),
            (51, 103),
        ),
        hs(
            Noun::Store,
            Rect::new(102, 46, 200, 101),
            Rect::new(124, 101, 180, 120),
            (152, 103),
        ),
        hs(
            Noun::Casino,
            Rect::new(202, 6, 318, 101),
            Rect::new(230, 101, 292, 120),
            (261, 103),
        ),
        hs(
            Noun::Ocean,
            Rect::new(0, 151, 320, PIC_BOTTOM),
            Rect::new(0, 0, 320, 200),
            (100, 146),
        ),
        hs(
            Noun::City,
            Rect::new(0, STATUS_H, 320, 30),
            Rect::new(0, 0, 320, 200),
            (100, 146),
        ),
    ]
}

pub fn actors(_state: &GameState, ticks: u64) -> Vec<Placed> {
    let mut v = vec![Placed::feet(
        npcs::dot((ticks / 10 % 2) as u8),
        DOT_POS.0,
        DOT_POS.1,
        priority_for_y(DOT_POS.1),
    )];
    // Chasing marquee bulbs.
    let phase = (ticks / 3 % 4) as i32;
    for i in 0..28 {
        if i % 4 == phase {
            let x = 206 + i * 4;
            v.push(Placed::top_left(rect_sprite(2, 2, WHITE), x, 28, 15));
            v.push(Placed::top_left(
                rect_sprite(2, 2, WHITE),
                316 - (x - 206),
                35,
                15,
            ));
        }
    }
    v
}

pub fn draw(_state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Night sky.
    p.fill_rect(0, 0, 320, 100, c(BLACK));
    for y in 60..100 {
        for x in 0..320 {
            if y >= 80 || (x + y) % 2 == 0 {
                p.plot(x, y, c(BLUE));
            }
        }
    }
    stars(&mut p, 31, 60, STATUS_H, 60);
    p.fill_ellipse(178, 22, 8, 8, c(YELLOW));
    p.fill_ellipse(181, 20, 7, 7, c(BLACK));

    // The Rusty Anchor's weathered facade.
    p.fill_rect(6, 46, 94, 100, c(BROWN));
    for y in (50..100).step_by(6) {
        p.line(6, y, 93, y, c(BLACK));
    }
    p.fill_polygon(&[(2, 46), (98, 46), (84, 34), (16, 34)], c(RED));
    p.line(2, 46, 97, 46, c(BLACK));
    for (px, py) in [(22, 72), (80, 72)] {
        p.fill_ellipse(px, py, 6, 6, c(LIGHT_GRAY));
        p.fill_ellipse(px, py, 4, 4, c(YELLOW));
    }
    let ax = 51;
    p.rect(ax - 1, 50, ax + 2, 53, c(LIGHT_MAGENTA));
    p.line(ax, 53, ax, 62, c(LIGHT_MAGENTA));
    p.line(ax - 3, 55, ax + 3, 55, c(LIGHT_MAGENTA));
    p.polyline(
        &[(ax - 5, 59), (ax - 3, 62), (ax + 3, 62), (ax + 5, 59)],
        c(LIGHT_MAGENTA),
    );
    p.fill_rect(40, 66, 62, 100, c(DARK_GRAY));
    p.rect(39, 65, 63, 101, c(BLACK));
    p.fill_rect(57, 82, 60, 85, c(YELLOW));

    // Lucky Gull 24/7 mart, fluorescent and unloved.
    p.fill_rect(104, 50, 198, 100, c(LIGHT_GRAY));
    p.fill_rect(104, 50, 198, 62, c(GREEN));
    p.polyline(
        &[(112, 54), (116, 58), (120, 55), (124, 58), (128, 54)],
        c(WHITE),
    );
    for x in (136..194).step_by(8) {
        p.fill_rect(x, 54, x + 5, 59, c(LIGHT_GREEN));
    }
    for (x0, x1) in [(110, 138), (166, 192)] {
        p.fill_rect(x0, 68, x1, 94, c(LIGHT_CYAN));
        p.rect(x0 - 1, 67, x1 + 1, 95, c(DARK_GRAY));
        for y in [76, 85] {
            p.line(x0, y, x1 - 1, y, c(BLUE));
        }
    }
    p.fill_rect(114, 70, 124, 75, c(RED)); // "HOT DOGS 2 FOR $5" poster, probably
    p.fill_rect(142, 66, 163, 100, c(CYAN));
    p.rect(141, 65, 164, 101, c(BLACK));
    p.line(146, 82, 146, 88, c(WHITE));

    // The Golden Clam Casino.
    p.fill_ellipse(261, 20, 24, 13, c(YELLOW));
    for dx in [-18, -9, 0, 9, 18] {
        p.line(261, 30, 261 + dx, 9, c(BROWN));
    }
    p.fill_ellipse(261, 24, 3, 3, c(WHITE));
    p.fill_rect(204, 26, 318, 100, c(MAGENTA));
    p.fill_rect(204, 26, 318, 38, c(RED));
    for x in (206..318).step_by(4) {
        p.plot(x, 28, c(YELLOW));
        p.plot(x, 35, c(YELLOW));
    }
    for x in [208, 236, 282, 308] {
        p.fill_rect(x, 40, x + 6, 100, c(LIGHT_MAGENTA));
    }
    for x in (216..306).step_by(10) {
        if !(240..282).contains(&x) {
            p.fill_rect(x, 44, x + 5, 54, c(YELLOW));
        }
    }
    p.fill_rect(244, 62, 279, 100, c(YELLOW));
    p.fill_rect(247, 66, 276, 100, c(BROWN));
    p.fill_rect(249, 68, 259, 98, c(LIGHT_CYAN));
    p.fill_rect(264, 68, 274, 98, c(LIGHT_CYAN));

    // Boardwalk planks.
    p.fill_rect(0, 100, 320, 151, c(BROWN));
    p.line(0, 100, 319, 100, c(BLACK));
    let mut y = 104;
    let mut gap = 4;
    let mut rng = Lcg(9);
    while y < 151 {
        p.line(0, y, 319, y, c(BLACK));
        let mut x = rng.next(30);
        while x < 320 {
            p.line(x, y - gap + 1, x, y - 1, c(BLACK));
            x += 30 + rng.next(30);
        }
        gap += 1;
        y += gap;
    }
    p.fill_polygon(&[(246, 101), (276, 101), (288, 124), (234, 124)], c(RED)); // red carpet
    p.fill_rect(0, 101, 320, 150, Pen::control_only(CTRL_WALK));

    // Ocean beyond the railing.
    p.fill_rect(0, 151, 320, PIC_BOTTOM, c(BLUE));
    for _ in 0..30 {
        let (x, y) = (rng.next(320), 153 + rng.next(22));
        p.line(x, y, x + 2 + rng.next(5), y, c(LIGHT_BLUE));
    }

    // Railing in the foreground: hides Chip's legs when he leans on it.
    let rp = priority_for_y(158);
    p.fill_rect(0, 140, 320, 142, Pen::color(WHITE).pri(rp));
    p.fill_rect(0, 147, 320, 149, Pen::color(LIGHT_GRAY).pri(rp));
    for x in (4..320).step_by(20) {
        p.fill_rect(x, 140, x + 3, 158, Pen::color(LIGHT_GRAY).pri(rp));
    }

    // Coin-op telescope at the railing.
    let tp = priority_for_y(146);
    p.fill_rect(295, 124, 298, 146, Pen::color(DARK_GRAY).pri(tp));
    p.fill_rect(289, 116, 304, 123, Pen::color(LIGHT_GRAY).pri(tp));
    p.fill_rect(286, 117, 289, 122, Pen::color(BLACK).pri(tp));
    p.fill_ellipse(296, 146, 5, 1, Pen::color(BLACK).pri(tp));
    p.fill_rect(290, 140, 304, 148, Pen::control_only(CTRL_BLOCK));

    // Dot's hot dog cart. Its priority hides Dot from the waist down.
    let cp = priority_for_y(142);
    p.line(188, 84, 188, 113, Pen::color(LIGHT_GRAY).pri(cp));
    p.fill_polygon(
        &[(168, 90), (208, 90), (199, 81), (177, 81)],
        Pen::color(RED).pri(cp),
    );
    for x in [176, 186, 196] {
        p.fill_polygon(
            &[(x, 90), (x + 5, 90), (x + 3, 81), (x + 1, 81)],
            Pen::color(YELLOW).pri(cp),
        );
    }
    p.fill_rect(148, 113, 193, 116, Pen::color(LIGHT_GRAY).pri(cp));
    p.fill_rect(150, 116, 191, 138, Pen::color(WHITE).pri(cp));
    p.fill_rect(150, 122, 191, 125, Pen::color(RED).pri(cp));
    p.fill_rect(158, 128, 182, 134, Pen::color(YELLOW).pri(cp)); // a painted hot dog, smiling
    p.fill_ellipse(170, 131, 10, 2, Pen::color(LIGHT_RED).pri(cp));
    for wx in [156, 184] {
        p.fill_ellipse(wx, 139, 3, 3, Pen::color(BLACK).pri(cp));
    }
    p.fill_rect(146, 128, 195, 143, Pen::control_only(CTRL_BLOCK));
    p.fill_rect(158, 120, 180, 129, Pen::control_only(CTRL_BLOCK));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cart_hides_dots_legs_and_railing_hides_chips() {
        let p = draw(&GameState::new());
        assert!(p.priority_at(DOT_POS.0, 120).unwrap() > priority_for_y(DOT_POS.1));
        assert!(p.priority_at(DOT_POS.0, 105).unwrap() <= priority_for_y(DOT_POS.1));
        assert!(p.priority_at(24, 141).unwrap() > priority_for_y(149));
        assert!(!p.walkable(170, 135));
    }
}
