//! Code-defined NPC sprites (ASCII art -> EGA colors; see `sprite::color_key`).

use crate::engine::palette::*;
use crate::engine::sprite::Sprite;

const DOT: [&str; 25] = [
    "...WWWWW...",
    "..WRRRRRW..",
    "..LLLLLLL..",
    ".LLrrrrrLL.",
    ".LrKrrrKrL.",
    "..rrrrrrr..",
    "..rrRRRrr..",
    "...rrrrr...",
    "..WWWWWWW..",
    ".RWRWRWRWR.",
    "rRWRWRWRWRr",
    "rRWRWRWRWRr",
    "r.WRWRWRW.r",
    "..RRRRRRR..",
    "..WWWWWWW..",
    "..WWWWWWW..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..rrr.rrr..",
    "..rrr.rrr..",
    ".WWWW.WWWW.",
    ".WWWW.WWWW.",
    ".K.K...K.K.",
];

/// Dot, the roller-skating hot dog vendor, waving a mustard bottle.
pub fn dot(frame: u8) -> Sprite {
    let mut s = Sprite::from_rows(&DOT);
    let (x, y) = if frame.is_multiple_of(2) {
        (0, 8)
    } else {
        (0, 6)
    };
    for dy in 0..3 {
        s.set(x, y + dy, Some(YELLOW));
    }
    s.set(x, y + 3, Some(LIGHT_RED));
    s
}

const MIME: [&str; 26] = [
    "..KKKKK..",
    ".KKKKKKK.",
    "..WWWWW..",
    "..WKWKW..",
    "..WWWWW..",
    "..WWRWW..",
    "...WWW...",
    "..KWKWK..",
    ".WKWKWKW.",
    ".KWKWKWK.",
    ".WKWKWKW.",
    ".KWKWKWK.",
    "..KWKWK..",
    "..WKWKW..",
    "..KKKKK..",
    ".KKKKKKK.",
    ".KKK.KKK.",
    ".KKK.KKK.",
    ".KKK.KKK.",
    ".KKK.KKK.",
    ".KKK.KKK.",
    ".KKK.KKK.",
    ".KKK.KKK.",
    ".KKK.KKK.",
    "WWWW.WWWW",
    "WWWW.WWWW",
];

/// The mime. `boxed`: pressing on the walls of his invisible box (two
/// frames). Otherwise he leans and "pulls an invisible rope".
pub fn mime(frame: u8, boxed: bool) -> Sprite {
    let mut s = Sprite::from_rows(&MIME);
    let hand_y = if boxed {
        if frame.is_multiple_of(2) {
            3
        } else {
            5
        }
    } else if frame.is_multiple_of(2) {
        8
    } else {
        10
    };
    for x in [0, 8] {
        for y in hand_y..9 {
            s.set(x, y, Some(BLACK));
        }
        s.set(x, hand_y, Some(WHITE));
        s.set(x, hand_y - 1, Some(WHITE));
    }
    s
}

const BRENDA: [&str; 24] = [
    "...SSSSS...",
    "..SSSSSSS..",
    ".SSSSSSSSS.",
    ".SKrrrrrKS.",
    ".SKrKrKrKS.",
    ".SSrrrrrSS.",
    "..SrrrrrS..",
    "...rrrrr...",
    "..GGGWGGG..",
    ".GGGGWGGGG.",
    ".GGYGWGGGG.",
    "rGGGGWGGGGr",
    "rGGGGWGGGGr",
    "rGGGGWGGGGr",
    "..GGGGGGG..",
    "..NNNNNNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..KKK.KKK..",
    "..KKK.KKK..",
];

/// Brenda, the night clerk, blowing a bubble every so often.
pub fn brenda(frame: u8) -> Sprite {
    let mut s = Sprite::from_rows(&BRENDA);
    if frame % 4 == 3 {
        for (x, y) in [(5, 6), (4, 6), (6, 6), (5, 5), (5, 7)] {
            s.set(x, y, Some(LIGHT_MAGENTA));
        }
    }
    s
}

const TINY: [&str; 34] = [
    ".....SSSSS.....",
    "....SSSSSSS....",
    "....SSSSSSS....",
    "....KKKSKKK....",
    "....SSSSSSS....",
    "....SSSKSSS....",
    ".....SSSSS.....",
    "......SSS......",
    "...DDKWRWKDD...",
    "..DDDDKWKDDDD..",
    ".DDDDDKWKDDDDD.",
    "DDDDDDKWKDDDDDD",
    "DDDDDDDWDDDDDDD",
    "DDDDDDDWDDDDDDD",
    "DDDDDDDKDDDDDDD",
    "DDDDDDDWDDDDDDD",
    "SDDDDDDDDDDDDDS",
    "SDDDDDDDDDDDDDS",
    ".DDDDDDDDDDDDD.",
    "..DDDDDDDDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    "..DDDDD.DDDDD..",
    ".KKKKKK.KKKKKK.",
    ".KKKKKK.KKKKKK.",
];

/// Tiny, the doorman. Not tiny. Occasionally adjusts his sunglasses.
pub fn tiny(frame: u8) -> Sprite {
    let mut s = Sprite::from_rows(&TINY);
    if frame % 6 == 5 {
        s.set(0, 15, Some(BROWN));
        s.set(1, 14, Some(BROWN));
        s.set(3, 4, Some(BROWN));
    }
    s
}

const VONDA: [&str; 29] = [
    "..YYYYYYY..",
    ".YYYYYYYYY.",
    "YYYYYYYYYYY",
    "YYYrrrrrYYY",
    "YYrKrrrKrYY",
    "YYrrrrrrrYY",
    "YYrrRRRrrYY",
    ".YYrrrrrYY.",
    "..YYrrrYY..",
    "...rrrrr...",
    "..MMMMMMM..",
    ".WMMMMMMMW.",
    ".WMMMMMMMW.",
    ".WMMMMMMMW.",
    "..MMMMMMM..",
    "..MMMMMMM..",
    "...MMMMM...",
    "...MMMMM...",
    "..MMMMMMM..",
    "..MMMMMMM..",
    ".MMMMMMMMM.",
    ".MMMMMMMMM.",
    ".MMMMMMMMM.",
    "MMMMMMMMMMM",
    "MMMMMMMMMMM",
    "MMMMMMMMMMM",
    "MMMMMMMMMMM",
    "MMMMMMMMMMM",
    ".KK.....KK.",
];

/// Vonda, the Crow's Nest's lounge singer. Her sequins twinkle.
pub fn vonda(frame: u8) -> Sprite {
    let mut s = Sprite::from_rows(&VONDA);
    let sparkles: [&[(i32, i32)]; 3] = [
        &[(3, 12), (6, 18), (2, 24), (8, 26)],
        &[(7, 13), (4, 20), (8, 22), (5, 26)],
        &[(5, 11), (3, 19), (6, 23), (1, 27)],
    ];
    for &(x, y) in sparkles[(frame % 3) as usize] {
        s.set(x, y, Some(WHITE));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_sprites_animate() {
        assert_ne!(dot(0), dot(1));
        assert_ne!(mime(0, true), mime(1, true));
        assert_ne!(mime(0, true), mime(0, false));
        assert_ne!(brenda(0), brenda(3));
        assert_ne!(tiny(0), tiny(5));
        assert_ne!(vonda(0), vonda(1));
    }
}
