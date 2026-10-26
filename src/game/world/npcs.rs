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

const DEE: [&str; 25] = [
    "..MMMMMMM..",
    ".MmMmMmMmM.",
    ".MMrrrrrMM.",
    "..rKrrrKr..",
    "..rrrrrrr..",
    "..rrRRRrr..",
    "...rrrrr...",
    "..CCCCCCC..",
    ".CCCCCCCCC.",
    "rCCCCCCCCCr",
    "rCCWWWWWCCr",
    ".CCWWWWWCC.",
    "..CCWWWCC..",
    "..CCCCCCC..",
    "..CCCCCCC..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    "..NNN.NNN..",
    ".WWWW.WWWW.",
    ".WWWW.WWWW.",
    ".KKKK.KKKK.",
];

/// Dee, the laundromat attendant, folding a towel with grim efficiency.
pub fn dee(frame: u8) -> Sprite {
    let mut s = Sprite::from_rows(&DEE);
    if frame % 2 == 1 {
        // The towel goes up a row; so do her hands.
        for x in 3..8 {
            s.set(x, 9, Some(WHITE));
            s.set(x, 12, None);
        }
        s.set(0, 8, Some(LIGHT_RED));
        s.set(10, 8, Some(LIGHT_RED));
        s.set(0, 9, Some(LIGHT_RED));
        s.set(10, 9, Some(LIGHT_RED));
    }
    s
}

const FERRIS: [&str; 25] = [
    "....MMM....",
    "...MMmMM...",
    "..MMMMMMM..",
    "..MmMMMmM..",
    "..MMMMMMM..",
    "..MMrrrMM..",
    "..KWKrKWK..",
    "..rrrrrrr..",
    "..rrRRRrr..",
    "...rrrrr...",
    "..mmmmmmm..",
    ".mmmYmmmmm.",
    ".mmmmmmmmm.",
    "rmmmmmmmmmr",
    "rmmmmmmmmmr",
    ".mmmmmmmmm.",
    "..mmmmmmm..",
    "..KKKKKKK..",
    "..KKKKKKK..",
    "..KKKKKKK..",
    "..KKK.KKK..",
    "..rrr.rrr..",
    "..rrr.rrr..",
    ".WWWW.WWWW.",
    ".KKKK.KKKK.",
];

/// Ferris, the motel's night manager, nodding off in front of her soaps.
pub fn ferris(frame: u8) -> Sprite {
    let mut s = Sprite::from_rows(&FERRIS);
    if frame % 4 >= 2 {
        // Eyes shut behind the glasses; a Z floats up.
        for x in [2, 3, 4, 6, 7, 8] {
            s.set(x, 6, Some(BLACK));
        }
        s.set(9, 3, Some(WHITE));
        s.set(10, 2, Some(WHITE));
    }
    s
}

const PELICAN: [&str; 16] = [
    "........WWWW........",
    ".......WWWWWW.......",
    ".......WWKWWW.......",
    ".......WWWWWWYYYYYYY",
    ".......WWWWWWYYYYYYY",
    "......WWWWWWWSSSSSSY",
    ".....WWWWWWWWWSSSSS.",
    "....WWWWWWWWWWWSSS..",
    "..LLWWWWWWWWWWWW....",
    ".LLLWWWWWWWWWWWW....",
    ".LLLWWWWWWWWWWWL....",
    "..LLLWWWWWWWWLL.....",
    "....LLLLLLLLL.......",
    ".....Y....Y.........",
    ".....Y....Y.........",
    "....YYY..YYY........",
];

/// Percival the pelican. `right`: facing east. `glint`: a brass key twinkles
/// in his pouch. `frame` waddles his feet and bobs his head.
pub fn pelican(frame: u8, right: bool, glint: bool) -> Sprite {
    let mut s = Sprite::from_rows(&PELICAN);
    if frame % 2 == 1 {
        // The other foot forward, head bobbing one pixel.
        for y in 13..16 {
            for x in 0..s.w {
                s.set(x, y, None);
            }
        }
        for (x, y) in [
            (7, 13),
            (7, 14),
            (6, 15),
            (7, 15),
            (8, 15),
            (11, 13),
            (11, 14),
            (10, 15),
            (11, 15),
            (12, 15),
        ] {
            s.set(x, y, Some(YELLOW));
        }
    }
    if glint {
        let on = frame % 4 < 2;
        s.set(16, 6, Some(if on { WHITE } else { BROWN }));
        s.set(17, 6, Some(if on { YELLOW } else { BROWN }));
    }
    if right {
        s
    } else {
        s.mirrored()
    }
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
        assert_ne!(dee(0), dee(1));
        assert_ne!(ferris(0), ferris(2));
        assert_ne!(pelican(0, true, true), pelican(1, true, true));
        assert_ne!(pelican(0, true, false), pelican(0, true, true));
        assert_ne!(pelican(0, true, false), pelican(0, false, false));
    }
}
