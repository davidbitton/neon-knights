//! Lucky Gull 24/7, a convenience store lit like an interrogation room.

use super::{c, hs, npcs, Exit, Hotspot, Lcg, Placed, Rect, PIC_BOTTOM, STATUS_H};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;
use crate::engine::sprite::Sprite;
use crate::game::parser::Noun;
use crate::game::state::{Flag, GameState, SceneId};

pub const BRENDA_POS: (i32, i32) = (60, 86);

pub const EXITS: [Exit; 1] = [Exit {
    zone: Rect::new(110, 168, 210, 176),
    to: SceneId::Boardwalk,
    arrive: (152, 110),
    nouns: &[Noun::Door, Noun::Boardwalk],
    back: (160, 160),
}];

const AT_COUNTER: Rect = Rect::new(8, 115, 140, 136);

pub fn hotspots(_state: &GameState) -> Vec<Hotspot> {
    vec![
        hs(
            Noun::Brenda,
            Rect::new(48, 60, 74, 79),
            AT_COUNTER,
            (60, 119),
        ),
        hs(
            Noun::Mints,
            Rect::new(74, 66, 95, 79),
            AT_COUNTER,
            (84, 119),
        ),
        hs(
            Noun::Register,
            Rect::new(22, 62, 48, 79),
            AT_COUNTER,
            (34, 119),
        ),
        hs(
            Noun::Carnation,
            Rect::new(110, 84, 136, 122),
            Rect::new(100, 112, 150, 140),
            (124, 127),
        ),
        hs(
            Noun::Slushie,
            Rect::new(258, 44, 308, 106),
            Rect::new(236, 104, 316, 132),
            (282, 113),
        ),
        hs(Noun::Bar, Rect::new(8, 76, 116, 114), AT_COUNTER, (60, 119)),
        hs(
            Noun::Bottles,
            Rect::new(128, 26, 254, 126),
            Rect::new(124, 100, 256, 150),
            (190, 134),
        ),
        hs(
            Noun::Door,
            Rect::new(110, 160, 210, PIC_BOTTOM),
            Rect::new(100, 150, 220, 176),
            (160, 171),
        ),
    ]
}

pub fn actors(_state: &GameState, ticks: u64) -> Vec<Placed> {
    let mut v = vec![Placed::feet(
        npcs::brenda((ticks / 10 % 4) as u8),
        BRENDA_POS.0,
        BRENDA_POS.1,
        priority_for_y(BRENDA_POS.1),
    )];
    // Slushie swirl.
    let t = (ticks % 12) as i32;
    let swirl = Sprite::from_rows(&["W.", ".W"]);
    v.push(Placed::top_left(
        swirl.clone(),
        268 + t,
        60 + (t * 7) % 16,
        15,
    ));
    v.push(Placed::top_left(swirl, 299 - t, 60 + (t * 5) % 16, 15));
    v
}

pub fn draw(state: &GameState) -> Picture {
    let mut p = Picture::new();
    p.priority_bands();

    // Walls with buzzing fluorescent tubes.
    p.fill_rect(0, 0, 320, 100, c(WHITE));
    p.fill_rect(0, STATUS_H, 320, 15, c(LIGHT_GRAY));
    for x in [40, 140, 240] {
        p.fill_rect(x, 10, x + 60, 13, c(LIGHT_CYAN));
    }
    p.fill_rect(0, 92, 320, 100, c(GREEN));
    p.line(0, 92, 319, 92, c(DARK_GRAY));

    // Wall shelving behind the aisle.
    p.fill_rect(130, 28, 252, 90, c(BROWN));
    let mut rng = Lcg(24);
    let goods = [
        RED,
        YELLOW,
        LIGHT_BLUE,
        LIGHT_GREEN,
        MAGENTA,
        WHITE,
        LIGHT_RED,
    ];
    for sy in [40, 54, 68, 82] {
        p.line(130, sy, 251, sy, c(BLACK));
        let mut x = 132;
        while x < 248 {
            let w = 3 + rng.next(4);
            let h = 5 + rng.next(6);
            p.fill_rect(
                x,
                sy - h,
                x + w,
                sy,
                c(goods[rng.next(goods.len() as u32) as usize]),
            );
            x += w + 1;
        }
    }

    // Checkerboard floor.
    for y in 100..PIC_BOTTOM {
        for x in 0..320 {
            let col = if ((x / 10) + (y - 100) / 6) % 2 == 0 {
                LIGHT_GRAY
            } else {
                WHITE
            };
            p.plot(x, y, c(col));
        }
    }
    p.line(0, 100, 319, 100, c(DARK_GRAY));
    p.fill_rect(130, 168, 190, PIC_BOTTOM, c(RED)); // door mat
    p.fill_rect(4, 101, 316, PIC_BOTTOM, Pen::control_only(CTRL_WALK));

    // Counter, register, mints. Its priority hides Brenda's lower half.
    let cp = priority_for_y(112);
    p.fill_rect(10, 78, 114, 84, Pen::color(DARK_GRAY).pri(cp));
    p.fill_rect(12, 84, 112, 112, Pen::color(GREEN).pri(cp));
    p.line(12, 98, 111, 98, Pen::color(YELLOW).pri(cp));
    p.fill_rect(24, 66, 47, 78, Pen::color(DARK_GRAY).pri(cp));
    p.fill_rect(28, 68, 43, 72, Pen::color(LIGHT_GREEN).pri(cp));
    p.fill_rect(76, 70, 93, 78, Pen::color(LIGHT_CYAN).pri(cp));
    for x in (78..92).step_by(3) {
        p.fill_rect(x, 72, x + 2, 74, Pen::color(WHITE).pri(cp));
    }
    p.fill_rect(8, 100, 116, 116, Pen::control_only(CTRL_BLOCK));

    // Bucket of carnations.
    let bp = priority_for_y(121);
    let flowers: &[i32] = if state.flag(Flag::BoughtCarnation) {
        &[118, 124, 130]
    } else {
        &[116, 121, 126, 131]
    };
    for &fx in flowers {
        p.line(fx, 92, 123, 106, Pen::color(GREEN).pri(bp));
        p.fill_ellipse(fx, 91, 2, 2, Pen::color(RED).pri(bp));
    }
    p.fill_polygon(
        &[(116, 104), (131, 104), (129, 120), (118, 120)],
        Pen::color(LIGHT_GRAY).pri(bp),
    );
    p.line(116, 104, 131, 104, Pen::color(DARK_GRAY).pri(bp));
    p.fill_rect(114, 110, 134, 122, Pen::control_only(CTRL_BLOCK));

    // Freestanding aisle: Chip can lurk behind it.
    let ap = priority_for_y(126);
    p.fill_rect(150, 82, 232, 124, Pen::color(BROWN).pri(ap));
    for sy in [94, 106, 118] {
        p.line(150, sy, 231, sy, Pen::color(BLACK).pri(ap));
        let mut x = 152;
        while x < 228 {
            let w = 3 + rng.next(4);
            let h = 5 + rng.next(5);
            p.fill_rect(
                x,
                sy - h,
                x + w,
                sy,
                Pen::color(goods[rng.next(goods.len() as u32) as usize]).pri(ap),
            );
            x += w + 1;
        }
    }
    p.fill_rect(148, 112, 234, 127, Pen::control_only(CTRL_BLOCK));

    // The slushie machine, running since before Brenda was born.
    let sp = priority_for_y(106);
    p.fill_rect(262, 48, 305, 104, Pen::color(LIGHT_GRAY).pri(sp));
    p.rect(262, 48, 305, 104, Pen::color(DARK_GRAY).pri(sp));
    p.fill_rect(266, 54, 282, 80, Pen::color(LIGHT_BLUE).pri(sp));
    p.fill_rect(286, 54, 302, 80, Pen::color(LIGHT_RED).pri(sp));
    p.line(267, 55, 267, 78, Pen::color(WHITE).pri(sp));
    p.line(287, 55, 287, 78, Pen::color(WHITE).pri(sp));
    for nx in [273, 293] {
        p.fill_rect(nx, 82, nx + 4, 87, Pen::color(BLACK).pri(sp));
    }
    p.fill_rect(266, 90, 302, 94, Pen::color(DARK_GRAY).pri(sp));
    p.fill_rect(258, 100, 308, 108, Pen::control_only(CTRL_BLOCK));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_hides_brenda_and_aisle_hides_chip() {
        let p = draw(&GameState::new());
        assert!(p.priority_at(BRENDA_POS.0, 84).unwrap() > priority_for_y(BRENDA_POS.1));
        assert!(p.priority_at(190, 100).unwrap() > priority_for_y(108));
        assert!(!p.walkable(190, 120));
    }
}
