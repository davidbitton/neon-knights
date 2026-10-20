//! Scene content: AGI-style vector pictures (all defined in code, no image
//! files) plus hotspot geometry for The Rusty Anchor.
//!
//! Layout follows the old interpreters: status bar on row band 0..9, the
//! picture in 9..176, the command line below it.

use super::parser::Noun;
use super::state::{Flag, GameState};
use crate::engine::gfx::{priority_for_y, Pen, Picture, CTRL_BLOCK, CTRL_WALK};
use crate::engine::palette::*;

pub const STATUS_H: i32 = 9;
pub const PIC_BOTTOM: i32 = 176;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Rect {
    pub const fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
        Rect { x0, y0, x1, y1 }
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x0 && x < self.x1 && y >= self.y0 && y < self.y1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hotspot {
    pub noun: Noun,
    /// Clickable area on screen.
    pub click: Rect,
    /// Where the hero's feet must be to interact (take/use/talk).
    pub near: Rect,
    /// A walkable spot inside `near` to walk to.
    pub approach: (i32, i32),
}

pub const STOOLS: [i32; 4] = [62, 102, 142, 182];
pub const GUS_POS: (i32, i32) = (151, 85);

const fn hs(noun: Noun, click: Rect, near: Rect, approach: (i32, i32)) -> Hotspot {
    Hotspot {
        noun,
        click,
        near,
        approach,
    }
}

const FRONT_OF_BAR: Rect = Rect::new(30, 115, 220, 128);
const BY_TABLE: Rect = Rect::new(226, 101, 316, 174);

/// Hotspots in hit-test priority order (small things first).
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
        (278, 106),
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

pub fn hotspot_at(state: &GameState, x: i32, y: i32) -> Option<Hotspot> {
    hotspots(state).into_iter().find(|h| h.click.contains(x, y))
}

pub fn hotspot_for(state: &GameState, noun: Noun) -> Option<Hotspot> {
    hotspots(state).into_iter().find(|h| h.noun == noun)
}

/// Is the hero standing close enough to `noun` to touch it?
pub fn is_near(state: &GameState, noun: Noun) -> bool {
    let (x, y) = state.hero;
    hotspots(state)
        .iter()
        .any(|h| h.noun == noun && h.near.contains(x, y))
}

/// The word used when a click turns into a typed command.
pub fn noun_word(n: Noun) -> &'static str {
    match n {
        Noun::Room => "room",
        Noun::Bartender => "bartender",
        Noun::Jukebox => "jukebox",
        Noun::Matchbook => "matchbook",
        Noun::Quarter => "quarter",
        Noun::Comb => "comb",
        Noun::Egg => "egg",
        Noun::Drink => "drink",
        Noun::Bar => "bar",
        Noun::Stool => "stool",
        Noun::Door => "door",
        Noun::Restroom => "restroom",
        Noun::Sign => "sign",
        Noun::Bottles => "bottles",
        Noun::Patron => "regular",
        Noun::Table => "table",
        Noun::Floor => "floor",
        Noun::Me => "me",
    }
}

/// Tiny deterministic PRNG for scenery (stars, wave glints).
struct Lcg(u32);
impl Lcg {
    fn next(&mut self, n: u32) -> i32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((self.0 >> 16) % n) as i32
    }
}

fn c(col: u8) -> Pen {
    Pen::color(col)
}

/// The Rusty Anchor, a bar that has seen better decades.
pub fn draw_bar(state: &GameState) -> Picture {
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

/// Title screen backdrop: a moonlit boardwalk.
pub fn draw_title() -> Picture {
    let mut p = Picture::new();
    p.fill_rect(0, 0, 320, 200, c(BLACK));
    for y in 50..86 {
        for x in 0..320 {
            if y >= 70 || (x + y) % 2 == 0 && y >= 58 || (x % 4 == 0 && y % 2 == 0) {
                p.plot(x, y, c(BLUE));
            }
        }
    }
    for y in 86..120 {
        for x in 0..320 {
            let col = if y >= 108 || (y >= 98 && (x + y) % 2 == 0) {
                MAGENTA
            } else {
                BLUE
            };
            p.plot(x, y, c(col));
        }
    }
    p.fill_rect(0, 116, 320, 120, c(LIGHT_MAGENTA));
    let mut rng = Lcg(1987);
    for _ in 0..80 {
        let (x, y) = (rng.next(320), rng.next(70));
        p.plot(x, y, c(if rng.next(3) == 0 { WHITE } else { LIGHT_GRAY }));
    }
    p.fill_ellipse(262, 34, 13, 13, c(YELLOW));
    p.fill_ellipse(258, 31, 2, 2, c(BROWN));
    p.fill_ellipse(266, 39, 3, 2, c(BROWN));
    // Ocean.
    p.fill_rect(0, 120, 320, 200, c(BLUE));
    for _ in 0..70 {
        let (x, y) = (rng.next(320), 122 + rng.next(78));
        p.line(x, y, x + 3 + rng.next(6), y, c(LIGHT_BLUE));
    }
    for i in 0..12 {
        let y = 122 + i * 6;
        let w = 4 + i;
        p.line(262 - w, y, 262 + w, y, c(YELLOW));
    }
    // Pier and the bar.
    p.fill_rect(0, 126, 140, 130, c(BROWN));
    for x in (4..140).step_by(16) {
        p.line(x, 130, x, 150, c(BLACK));
    }
    p.fill_rect(20, 100, 76, 126, c(DARK_GRAY));
    p.fill_polygon(&[(16, 100), (80, 100), (48, 86)], c(RED));
    p.fill_rect(30, 108, 44, 118, c(LIGHT_MAGENTA));
    p.fill_rect(54, 110, 64, 126, c(BROWN));
    // Palm trees.
    for (bx, lean) in [(180, 18), (300, -14)] {
        let top = (bx + lean, 70);
        p.fill_polygon(
            &[
                (bx - 3, 160),
                (bx + 3, 160),
                (top.0 + 2, top.1),
                (top.0 - 1, top.1),
            ],
            c(BLACK),
        );
        for (dx, dy) in [
            (-28, 10),
            (-20, -6),
            (0, -12),
            (22, -6),
            (28, 10),
            (12, 16),
            (-14, 16),
        ] {
            let (ex, ey) = (top.0 + dx, top.1 + dy);
            let mid = (top.0 + dx / 2, top.1 + dy / 2 - 5);
            p.fill_polygon(&[top, mid, (ex, ey), (mid.0, mid.1 + 3)], c(BLACK));
        }
    }
    p.fill_polygon(
        &[(150, 200), (150, 165), (220, 158), (320, 162), (320, 200)],
        c(BROWN),
    );
    p
}

/// Death screen backdrop: a lonely headstone (left half; text goes right).
pub fn draw_tomb() -> Picture {
    let mut p = Picture::new();
    p.fill_rect(0, 0, 320, 200, c(BLACK));
    let mut rng = Lcg(13);
    for _ in 0..50 {
        p.plot(rng.next(320), STATUS_H + rng.next(100), c(LIGHT_GRAY));
    }
    p.fill_ellipse(30, 30, 9, 9, c(LIGHT_GRAY));
    p.fill_ellipse(33, 28, 8, 8, c(BLACK));
    p.fill_rect(0, 140, 320, PIC_BOTTOM, c(GREEN));
    for x in (0..320).step_by(2) {
        p.plot(x, 139, c(GREEN));
        p.plot(x + 1, 138 + (x % 3), c(DARK_GRAY));
    }
    // Headstone.
    p.fill_ellipse(70, 76, 30, 16, c(LIGHT_GRAY));
    p.fill_rect(40, 76, 101, 146, c(LIGHT_GRAY));
    p.line(40, 76, 40, 145, c(DARK_GRAY));
    p.line(100, 76, 100, 145, c(DARK_GRAY));
    p.fill_rect(34, 144, 107, 150, c(DARK_GRAY));
    let g = c(DARK_GRAY);
    // R
    p.polyline(
        &[
            (52, 96),
            (52, 82),
            (58, 82),
            (60, 84),
            (60, 87),
            (58, 89),
            (52, 89),
        ],
        g,
    );
    p.line(55, 89, 60, 96, g);
    // I
    p.line(68, 82, 68, 96, g);
    p.line(66, 82, 70, 82, g);
    p.line(66, 96, 70, 96, g);
    // P
    p.polyline(
        &[
            (78, 96),
            (78, 82),
            (85, 82),
            (87, 84),
            (87, 87),
            (85, 89),
            (78, 89),
        ],
        g,
    );
    // Crack and epitaph scratches.
    p.polyline(&[(92, 100), (88, 108), (91, 114), (86, 124)], g);
    for y in [106, 112, 118, 124, 130] {
        p.line(50, y, 82, y, g);
    }
    // The culprit, at large.
    p.fill_ellipse(112, 150, 5, 4, c(YELLOW));
    p.plot(110, 148, c(WHITE));
    // Bare tree.
    p.fill_polygon(&[(8, 150), (14, 150), (12, 64), (10, 64)], c(BROWN));
    p.line(11, 90, 26, 72, c(BROWN));
    p.line(12, 104, 2, 92, c(BROWN));
    p.line(20, 80, 24, 62, c(BROWN));
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::actor::can_stand;
    use crate::engine::gfx::priority_for_y;

    #[test]
    fn start_and_approach_points_are_walkable() {
        let s = GameState::new();
        let pic = draw_bar(&s);
        assert!(can_stand(&pic, s.hero.0, s.hero.1), "start position");
        for h in hotspots(&s) {
            if h.noun == Noun::Sign {
                continue;
            }
            let (x, y) = h.approach;
            assert!(can_stand(&pic, x, y), "{:?} approach {x},{y}", h.noun);
            assert!(
                h.near.contains(x, y),
                "{:?} approach outside near-rect",
                h.noun
            );
        }
    }

    #[test]
    fn counter_hides_gus_legs_and_stools_occlude_hero() {
        let pic = draw_bar(&GameState::new());
        // Gus's feet are below the counter top -> hidden.
        let gus_pri = priority_for_y(GUS_POS.1);
        assert!(pic.priority_at(GUS_POS.0, 84).unwrap() > gus_pri);
        // His head (above the counter) is visible.
        assert!(pic.priority_at(GUS_POS.0, 66).unwrap() <= gus_pri);
        // Hero standing just behind a stool is hidden by its pole...
        let sx = STOOLS[1];
        assert!(pic.priority_at(sx, 125).unwrap() > priority_for_y(122));
        // ...but in front of it when standing south of the stool.
        assert!(pic.priority_at(sx, 125).unwrap() <= priority_for_y(140));
        // The counter itself is not walkable.
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
        let pic = draw_bar(&s);
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

    #[test]
    fn other_pictures_render() {
        let t = draw_title();
        assert_eq!(t.visual_at(262, 34), Some(YELLOW));
        let d = draw_tomb();
        assert_eq!(d.visual_at(45, 100), Some(LIGHT_GRAY));
    }
}
