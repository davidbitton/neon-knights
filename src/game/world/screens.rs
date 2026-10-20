//! Full-screen backdrops: title, tombstones (one per death), and the ending.

use super::{c, stars, Lcg, PIC_BOTTOM, STATUS_H};
use crate::engine::gfx::Picture;
use crate::engine::palette::*;
use crate::game::state::DeathKind;

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
/// Death screen backdrop: a lonely headstone (left half; text goes right),
/// with the culprit of this particular death resting on the grass.
pub fn draw_tomb(kind: DeathKind) -> Picture {
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
    p.line(68, 82, 68, 96, g);
    p.line(66, 82, 70, 82, g);
    p.line(66, 96, 70, 96, g);
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
    p.polyline(&[(92, 100), (88, 108), (91, 114), (86, 124)], g);
    for y in [106, 112, 118, 124, 130] {
        p.line(50, y, 82, y, g);
    }
    // Bare tree.
    p.fill_polygon(&[(8, 150), (14, 150), (12, 64), (10, 64)], c(BROWN));
    p.line(11, 90, 26, 72, c(BROWN));
    p.line(12, 104, 2, 92, c(BROWN));
    p.line(20, 80, 24, 62, c(BROWN));
    // The culprit, at large.
    match kind {
        DeathKind::Egg => {
            p.fill_ellipse(112, 150, 5, 4, c(YELLOW));
            p.plot(110, 148, c(WHITE));
        }
        DeathKind::Dumpster => {
            p.fill_ellipse(114, 150, 8, 6, c(BLACK));
            p.fill_rect(112, 142, 116, 145, c(DARK_GRAY));
            p.line(110, 141, 118, 141, c(DARK_GRAY));
            p.plot(122, 140, c(WHITE)); // a fly, paying respects
            p.plot(125, 138, c(WHITE));
        }
        DeathKind::Slushie => {
            p.fill_polygon(&[(107, 140), (119, 140), (117, 155), (109, 155)], c(WHITE));
            p.fill_rect(108, 145, 119, 148, c(BLUE));
            p.fill_ellipse(113, 139, 6, 3, c(LIGHT_BLUE));
            p.line(115, 139, 119, 128, c(LIGHT_RED));
        }
        DeathKind::Jackpot => {
            for (x, y) in [
                (108, 154),
                (116, 154),
                (124, 154),
                (112, 150),
                (120, 150),
                (116, 146),
                (130, 155),
            ] {
                p.fill_ellipse(x, y, 4, 2, c(YELLOW));
                p.line(x - 3, y, x + 3, y, c(BROWN));
            }
        }
        DeathKind::Pool => {
            p.fill_ellipse(114, 152, 7, 4, c(YELLOW));
            p.fill_ellipse(119, 145, 4, 4, c(YELLOW));
            p.fill_rect(123, 145, 126, 147, c(LIGHT_RED));
            p.plot(120, 144, c(BLACK));
        }
    }
    p
}

/// City skyline silhouettes with lit windows (shared by rooftop and ending).
pub fn skyline(p: &mut Picture, base: i32) {
    let mut rng = Lcg(77);
    let mut x = 0;
    while x < 320 {
        let w = 14 + rng.next(22);
        let h = 14 + rng.next(30);
        let col = if rng.next(2) == 0 { DARK_GRAY } else { BLUE };
        p.fill_rect(x, base - h, x + w, base, c(col));
        for wy in ((base - h + 3)..(base - 2)).step_by(4) {
            for wx in ((x + 2)..(x + w - 2)).step_by(4) {
                if rng.next(3) == 0 {
                    p.plot(wx, wy, c(YELLOW));
                }
            }
        }
        x += w + rng.next(4);
    }
}

/// The ending: a spotlight on the Crow's Nest stage.
pub fn draw_ending() -> Picture {
    let mut p = Picture::new();
    p.fill_rect(0, 0, 320, 200, c(BLACK));
    stars(&mut p, 5, 70, STATUS_H, 60);
    p.fill_ellipse(282, 28, 10, 10, c(WHITE));
    skyline(&mut p, 120);
    p.fill_rect(0, 120, 320, PIC_BOTTOM, c(BROWN));
    for y in (124..PIC_BOTTOM).step_by(8) {
        p.line(0, y, 319, y, c(RED));
    }
    // Spotlight cone (dithered) and pool of light.
    for y in STATUS_H..172 {
        let t = (y - STATUS_H) as f32 / 160.0;
        let half = 8.0 + t * 52.0;
        let (x0, x1) = ((160.0 - half) as i32, (160.0 + half) as i32);
        for x in x0..x1 {
            if (x + y) % 2 == 0 {
                p.plot(x, y, c(YELLOW));
            }
        }
    }
    p.fill_ellipse(160, 168, 58, 6, c(YELLOW));
    // Piano silhouette and a mic stand.
    p.fill_polygon(
        &[(14, 132), (70, 132), (80, 146), (58, 156), (14, 156)],
        c(BLACK),
    );
    p.fill_rect(18, 144, 60, 147, c(WHITE));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backdrops_render() {
        let t = draw_title();
        assert_eq!(t.visual_at(262, 34), Some(YELLOW));
        for k in DeathKind::ALL {
            let d = draw_tomb(k);
            assert_eq!(d.visual_at(45, 100), Some(LIGHT_GRAY), "{k:?}");
        }
        // Each death leaves a different culprit on the grass.
        let pics: Vec<Vec<u8>> = DeathKind::ALL
            .iter()
            .map(|k| draw_tomb(*k).visual)
            .collect();
        for i in 0..pics.len() {
            for j in (i + 1)..pics.len() {
                assert_ne!(pics[i], pics[j]);
            }
        }
        assert_eq!(draw_ending().visual_at(160, 168), Some(YELLOW));
    }
}
