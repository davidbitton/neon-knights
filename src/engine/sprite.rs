//! Code-defined pixel-art sprites ("views" in AGI terms) and priority blitting.

use super::gfx::{Frame, Picture};
use super::palette::*;

/// An indexed sprite; `None` pixels are transparent.
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Option<u8>>,
}

/// Character -> palette mapping used by all the ASCII-art sprites.
pub fn color_key(ch: char) -> Option<u8> {
    match ch {
        'K' => Some(BLACK),
        'W' => Some(WHITE),
        'B' => Some(LIGHT_CYAN), // powder-blue tux
        'b' => Some(CYAN),
        'S' => Some(BROWN), // a spray tan of questionable provenance
        'R' => Some(RED),
        'r' => Some(LIGHT_RED),
        'L' => Some(LIGHT_GRAY),
        'D' => Some(DARK_GRAY),
        'N' => Some(BLUE),
        'Y' => Some(YELLOW),
        'G' => Some(GREEN),
        'M' => Some(LIGHT_MAGENTA),
        _ => None,
    }
}

impl Sprite {
    pub fn from_rows(rows: &[&str]) -> Sprite {
        let h = rows.len() as i32;
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
        let mut px = vec![None; (w * h) as usize];
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                px[y * w as usize + x] = color_key(ch);
            }
        }
        Sprite { w, h, px }
    }

    pub fn blank(w: i32, h: i32) -> Sprite {
        Sprite {
            w,
            h,
            px: vec![None; (w * h) as usize],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<u8> {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            None
        } else {
            self.px[(y * self.w + x) as usize]
        }
    }

    pub fn set(&mut self, x: i32, y: i32, c: Option<u8>) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    pub fn mirrored(&self) -> Sprite {
        let mut out = Sprite::blank(self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                out.set(self.w - 1 - x, y, self.get(x, y));
            }
        }
        out
    }

    /// Draw with its bottom-center ("feet") at (fx, fy), honoring the picture's
    /// priority plane: a pixel is drawn only where `pri >= picture priority`.
    pub fn blit_priority(&self, frame: &mut Frame, pic: &Picture, fx: i32, fy: i32, pri: u8) {
        let x0 = fx - self.w / 2;
        let y0 = fy - self.h + 1;
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(c) = self.get(x, y) {
                    let (sx, sy) = (x0 + x, y0 + y);
                    if let Some(p) = pic.priority_at(sx, sy) {
                        if pri >= p {
                            frame.set(sx, sy, c);
                        }
                    }
                }
            }
        }
    }

    /// Plain blit (top-left anchored), no priority test.
    pub fn blit(&self, frame: &mut Frame, x0: i32, y0: i32) {
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(c) = self.get(x, y) {
                    frame.set(x0 + x, y0 + y, c);
                }
            }
        }
    }
}

/// Facing directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

const HERO_FRONT: [&str; 16] = [
    "..KKKKK..",
    ".KKKKKKK.",
    ".KSSSSSK.",
    "..SKSKS..",
    "..SSSSS..",
    "..SSWSS..",
    "...SSS...",
    "..BWRWB..",
    ".BBWRWBB.",
    "BBBWWWBBB",
    "BBBWKWBBB",
    "BbBWWWBbB",
    "BbBWKWBbB",
    "BbBBWBBbB",
    "SbBBBBBbS",
    ".BBBBBBB.",
];

const HERO_BACK: [&str; 16] = [
    "..KKKKK..",
    ".KKKKKKK.",
    ".KKKKKKK.",
    "..KKKKK..",
    "..KKKKK..",
    "..SKKKS..",
    "...SSS...",
    "..BBBBB..",
    ".BBBBBBB.",
    "BBBBBBBBB",
    "BBBBBBBBB",
    "BbBBBBBbB",
    "BbBBBBBbB",
    "BbBBBBBbB",
    "SbBBBBBbS",
    ".BBBBBBB.",
];

const HERO_SIDE: [&str; 16] = [
    "..KKKK...",
    ".KKKKKK..",
    ".KKSSSS..",
    ".KSSSKS..",
    "..SSSSSS.",
    "..SSSWS..",
    "...SSS...",
    "...BWR...",
    "..BBWWB..",
    "..BBWWB..",
    "..BBBWB..",
    "..BBBBB..",
    "..BbBBB..",
    "..BbBBB..",
    "..BSBBB..",
    "..BBBBB..",
];

pub const HERO_W: i32 = 9;
pub const HERO_H: i32 = 26;

/// Build one hero animation frame. `frame` is 0..4; 0 and 2 are "standing".
pub fn hero_frame(dir: Dir, frame: u8) -> Sprite {
    let top: &[&str] = match dir {
        Dir::Down => &HERO_FRONT,
        Dir::Up => &HERO_BACK,
        Dir::Left | Dir::Right => &HERO_SIDE,
    };
    let mut s = Sprite::blank(HERO_W, HERO_H);
    let upper = Sprite::from_rows(top);
    for y in 0..upper.h {
        for x in 0..upper.w {
            s.set(x, y, upper.get(x, y));
        }
    }
    let tux = Some(LIGHT_CYAN);
    let shoe = Some(BLACK);
    match dir {
        Dir::Down | Dir::Up => {
            // Two straight legs; on frames 1 and 3 one foot is lifted a row.
            let lift_left = frame % 4 == 1;
            let lift_right = frame % 4 == 3;
            for (x0, lifted) in [(1, lift_left), (5, lift_right)] {
                let end = if lifted { 23 } else { 24 };
                for y in 16..end {
                    for dx in 0..3 {
                        s.set(x0 + dx, y, tux);
                    }
                }
                for dx in 0..3 {
                    s.set(x0 + dx, end, shoe);
                    s.set(x0 + dx, end + 1, shoe);
                }
            }
        }
        Dir::Left | Dir::Right => {
            let stride = [0, 1, 2, 1][(frame % 4) as usize];
            for t in 0..8 {
                let off = (stride * t + 3) / 7;
                let y = 16 + t;
                s.set(3 - off, y, tux);
                s.set(4 - off, y, tux);
                s.set(4 + off, y, tux);
                s.set(5 + off, y, tux);
            }
            for y in 24..26 {
                for x in (3 - stride)..(6 - stride) {
                    s.set(x, y, shoe);
                }
                for x in (4 + stride)..(7 + stride) {
                    s.set(x, y, shoe);
                }
            }
            if dir == Dir::Left {
                return s.mirrored();
            }
        }
    }
    s
}

const GUS: [&str; 22] = [
    "....rrrrr......",
    "...rrrrrrr.....",
    "..KKKKrrrrK....",
    "...KKrrrKrr....",
    "...KKrrrrrr....",
    "...rrrrrrrr....",
    "...rLLLLLrr....",
    "....rrrrrr.....",
    "..WWWWWWWWWW...",
    ".WNNNNNNNNNNW..",
    ".rWWWWWWWWWWr..",
    ".rNNNNNNNNNNr..",
    ".rWWWWWWWWWWr..",
    ".rNNNNNNNNNNr..",
    "..WWWWWWWWWW...",
    "..NNNNNNNNNN...",
    "..WWWWWWWWWW...",
    "..NNNNNNNNNN...",
    "..WWWWWWWWWW...",
    "..NNNNNNNNNN...",
    "..WWWWWWWWWW...",
    "..NNNNNNNNNN...",
];

/// Gus the bartender, polishing the same glass he's polished since 1979.
pub fn gus_frame(frame: u8) -> Sprite {
    let mut s = Sprite::from_rows(&GUS);
    // Hands + glass move between two positions.
    let gx = if frame.is_multiple_of(2) { 4 } else { 8 };
    for y in 13..16 {
        s.set(gx, y, Some(LIGHT_GRAY));
        s.set(gx + 1, y, Some(WHITE));
        s.set(gx + 2, y, Some(LIGHT_GRAY));
    }
    s.set(gx - 1, 15, Some(LIGHT_RED));
    s.set(gx + 3, 15, Some(LIGHT_RED));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::gfx::{Pen, Picture};

    #[test]
    fn hero_frames_have_expected_size_and_feet() {
        for dir in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
            for f in 0..4 {
                let s = hero_frame(dir, f);
                assert_eq!((s.w, s.h), (HERO_W, HERO_H));
                // There's a shoe somewhere in the last two rows.
                let feet = (0..s.w).any(|x| {
                    s.get(x, HERO_H - 1) == Some(BLACK) || s.get(x, HERO_H - 2) == Some(BLACK)
                });
                assert!(feet, "{dir:?} frame {f} has no feet");
            }
        }
    }

    #[test]
    fn walk_frames_differ() {
        assert_ne!(hero_frame(Dir::Right, 0), hero_frame(Dir::Right, 1));
        assert_ne!(hero_frame(Dir::Down, 1), hero_frame(Dir::Down, 3));
    }

    #[test]
    fn priority_hides_sprite_behind_foreground() {
        let mut pic = Picture::new();
        // Foreground object with high priority covering the left half.
        pic.fill_rect(0, 0, 160, 200, Pen::priority_only(14));
        let mut frame = Frame::new();
        let s = Sprite::from_rows(&["WWWW"]);
        s.blit_priority(&mut frame, &pic, 160, 50, 5);
        assert_eq!(frame.get(158, 50), 0, "hidden behind foreground");
        assert_eq!(frame.get(160, 50), WHITE, "visible in open floor");
        s.blit_priority(&mut frame, &pic, 160, 60, 14);
        assert_eq!(frame.get(158, 60), WHITE, "equal priority draws in front");
    }
}
