//! Indexed framebuffer plus AGI-style picture planes.
//!
//! A [`Picture`] has three planes, like Sierra's AGI interpreter:
//! * `visual`   – the colors you see,
//! * `priority` – a depth value per pixel; a sprite pixel is only drawn where
//!   the sprite's own priority is >= the picture's priority (so the hero can
//!   walk *behind* the bar stools),
//! * `control`  – walkability: [`CTRL_BLOCK`] or [`CTRL_WALK`].
//!
//! Drawing ops take a [`Pen`] that says which planes to write to.

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;

/// Control-plane values.
pub const CTRL_BLOCK: u8 = 0;
pub const CTRL_WALK: u8 = 1;

/// Which planes a drawing operation writes to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pen {
    pub visual: Option<u8>,
    pub priority: Option<u8>,
    pub control: Option<u8>,
}

impl Pen {
    pub fn color(c: u8) -> Self {
        Pen {
            visual: Some(c),
            ..Default::default()
        }
    }
    pub fn pri(mut self, p: u8) -> Self {
        self.priority = Some(p);
        self
    }
    pub fn ctrl(mut self, c: u8) -> Self {
        self.control = Some(c);
        self
    }
    pub fn priority_only(p: u8) -> Self {
        Pen {
            priority: Some(p),
            ..Default::default()
        }
    }
    pub fn control_only(c: u8) -> Self {
        Pen {
            control: Some(c),
            ..Default::default()
        }
    }
}

/// Priority for an actor whose feet are at screen row `y`. Higher = nearer.
/// Mirrors AGI's banded priorities: rows are grouped into 12-pixel bands.
pub fn priority_for_y(y: i32) -> u8 {
    let band = (y.clamp(0, HEIGHT as i32 - 1) / 12) as u8;
    band + 1
}

#[derive(Clone)]
pub struct Picture {
    pub visual: Vec<u8>,
    pub priority: Vec<u8>,
    pub control: Vec<u8>,
}

impl Default for Picture {
    fn default() -> Self {
        Self::new()
    }
}

impl Picture {
    pub fn new() -> Self {
        Picture {
            visual: vec![0; WIDTH * HEIGHT],
            priority: vec![0; WIDTH * HEIGHT],
            control: vec![CTRL_BLOCK; WIDTH * HEIGHT],
        }
    }

    #[inline]
    fn idx(x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            None
        } else {
            Some(y as usize * WIDTH + x as usize)
        }
    }

    #[inline]
    pub fn plot(&mut self, x: i32, y: i32, pen: Pen) {
        if let Some(i) = Self::idx(x, y) {
            if let Some(c) = pen.visual {
                self.visual[i] = c;
            }
            if let Some(p) = pen.priority {
                self.priority[i] = p;
            }
            if let Some(c) = pen.control {
                self.control[i] = c;
            }
        }
    }

    pub fn visual_at(&self, x: i32, y: i32) -> Option<u8> {
        Self::idx(x, y).map(|i| self.visual[i])
    }
    pub fn priority_at(&self, x: i32, y: i32) -> Option<u8> {
        Self::idx(x, y).map(|i| self.priority[i])
    }
    pub fn control_at(&self, x: i32, y: i32) -> u8 {
        Self::idx(x, y)
            .map(|i| self.control[i])
            .unwrap_or(CTRL_BLOCK)
    }
    pub fn walkable(&self, x: i32, y: i32) -> bool {
        self.control_at(x, y) == CTRL_WALK
    }

    /// Bresenham line.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pen: Pen) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.plot(x, y, pen);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Connected line segments.
    pub fn polyline(&mut self, pts: &[(i32, i32)], pen: Pen) {
        for w in pts.windows(2) {
            self.line(w[0].0, w[0].1, w[1].0, w[1].1, pen);
        }
    }

    /// Filled axis-aligned rectangle, inclusive of (x0,y0), exclusive of (x1,y1).
    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pen: Pen) {
        for y in y0.max(0)..y1.min(HEIGHT as i32) {
            for x in x0.max(0)..x1.min(WIDTH as i32) {
                self.plot(x, y, pen);
            }
        }
    }

    /// Rectangle outline.
    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pen: Pen) {
        self.line(x0, y0, x1 - 1, y0, pen);
        self.line(x0, y1 - 1, x1 - 1, y1 - 1, pen);
        self.line(x0, y0, x0, y1 - 1, pen);
        self.line(x1 - 1, y0, x1 - 1, y1 - 1, pen);
    }

    /// Scanline polygon fill (even-odd rule), including the outline.
    pub fn fill_polygon(&mut self, pts: &[(i32, i32)], pen: Pen) {
        if pts.len() < 3 {
            self.polyline(pts, pen);
            return;
        }
        let min_y = pts.iter().map(|p| p.1).min().unwrap().max(0);
        let max_y = pts
            .iter()
            .map(|p| p.1)
            .max()
            .unwrap()
            .min(HEIGHT as i32 - 1);
        let n = pts.len();
        let mut xs: Vec<i32> = Vec::with_capacity(8);
        for y in min_y..=max_y {
            xs.clear();
            let fy = y as f32 + 0.5;
            for i in 0..n {
                let (ax, ay) = pts[i];
                let (bx, by) = pts[(i + 1) % n];
                let (ay_f, by_f) = (ay as f32, by as f32);
                if (ay_f <= fy && by_f > fy) || (by_f <= fy && ay_f > fy) {
                    let t = (fy - ay_f) / (by_f - ay_f);
                    xs.push((ax as f32 + t * (bx - ax) as f32).round() as i32);
                }
            }
            xs.sort_unstable();
            for pair in xs.chunks(2) {
                if let [a, b] = pair {
                    for x in *a..=*b {
                        self.plot(x, y, pen);
                    }
                }
            }
        }
        let mut closed = pts.to_vec();
        closed.push(pts[0]);
        self.polyline(&closed, pen);
    }

    /// Filled ellipse (approximated by a 24-gon).
    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, pen: Pen) {
        let pts: Vec<(i32, i32)> = (0..24)
            .map(|i| {
                let a = i as f32 / 24.0 * std::f32::consts::TAU;
                (
                    cx + (a.cos() * rx as f32).round() as i32,
                    cy + (a.sin() * ry as f32).round() as i32,
                )
            })
            .collect();
        self.fill_polygon(&pts, pen);
    }

    /// Fill the priority plane with AGI-style horizontal bands.
    pub fn priority_bands(&mut self) {
        for y in 0..HEIGHT as i32 {
            let p = priority_for_y(y);
            for x in 0..WIDTH as i32 {
                self.plot(x, y, Pen::priority_only(p));
            }
        }
    }

    /// AGI-style visual flood fill: replaces the contiguous region of the
    /// seed's color with `pen` (4-connected). Priority/control planes in the
    /// pen are written for every filled pixel too.
    pub fn flood_fill(&mut self, x: i32, y: i32, pen: Pen) {
        let Some(start) = Self::idx(x, y) else { return };
        let target = self.visual[start];
        if pen.visual == Some(target) {
            // Only non-visual planes would change; mark via a visited set.
            let mut seen = vec![false; WIDTH * HEIGHT];
            self.flood_inner(x, y, target, pen, Some(&mut seen));
        } else {
            self.flood_inner(x, y, target, pen, None);
        }
    }

    fn flood_inner(
        &mut self,
        x: i32,
        y: i32,
        target: u8,
        pen: Pen,
        mut seen: Option<&mut Vec<bool>>,
    ) {
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            let Some(i) = Self::idx(cx, cy) else { continue };
            if self.visual[i] != target {
                continue;
            }
            if let Some(s) = seen.as_deref_mut() {
                if s[i] {
                    continue;
                }
                s[i] = true;
            }
            self.plot(cx, cy, pen);
            stack.push((cx + 1, cy));
            stack.push((cx - 1, cy));
            stack.push((cx, cy + 1));
            stack.push((cx, cy - 1));
        }
    }
}

/// The per-frame composited screen (palette indices).
pub struct Frame {
    pub pixels: Vec<u8>,
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            pixels: vec![0; WIDTH * HEIGHT],
        }
    }

    pub fn copy_from(&mut self, pic: &Picture) {
        self.pixels.copy_from_slice(&pic.visual);
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && (x as usize) < WIDTH && (y as usize) < HEIGHT {
            self.pixels[y as usize * WIDTH + x as usize] = c;
        }
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        self.pixels[y as usize * WIDTH + x as usize]
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        for y in y0.max(0)..y1.min(HEIGHT as i32) {
            for x in x0.max(0)..x1.min(WIDTH as i32) {
                self.set(x, y, c);
            }
        }
    }

    /// Sierra-style message box frame: filled box with a double border.
    pub fn message_box(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, bg: u8, border: u8) {
        self.fill_rect(x0, y0, x1, y1, bg);
        for (inset, c) in [(1, border), (3, border)] {
            let (a, b, cx, d) = (x0 + inset, y0 + inset, x1 - 1 - inset, y1 - 1 - inset);
            for x in a..=cx {
                self.set(x, b, c);
                self.set(x, d, c);
            }
            for y in b..=d {
                self.set(a, y, c);
                self.set(cx, y, c);
            }
        }
    }

    /// Expand to RGBA bytes for the browser's ImageData.
    pub fn to_rgba(&self, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(WIDTH * HEIGHT * 4);
        for &p in &self.pixels {
            let [r, g, b] = super::palette::EGA[(p & 15) as usize];
            out.extend_from_slice(&[r, g, b, 255]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_fill_and_bounds() {
        let mut p = Picture::new();
        p.fill_rect(-10, -10, 5, 5, Pen::color(4));
        assert_eq!(p.visual_at(0, 0), Some(4));
        assert_eq!(p.visual_at(4, 4), Some(4));
        assert_eq!(p.visual_at(5, 5), Some(0));
        assert_eq!(p.visual_at(-1, 0), None);
    }

    #[test]
    fn polygon_fill_covers_interior_only() {
        let mut p = Picture::new();
        p.fill_polygon(
            &[(10, 10), (30, 10), (30, 30), (10, 30)],
            Pen::color(2).pri(5),
        );
        assert_eq!(p.visual_at(20, 20), Some(2));
        assert_eq!(p.priority_at(20, 20), Some(5));
        assert_eq!(p.visual_at(9, 20), Some(0));
        assert_eq!(p.visual_at(31, 20), Some(0));
    }

    #[test]
    fn flood_fill_stops_at_boundary() {
        let mut p = Picture::new();
        p.rect(10, 10, 20, 20, Pen::color(15));
        p.flood_fill(15, 15, Pen::color(4).ctrl(CTRL_WALK));
        assert_eq!(p.visual_at(15, 15), Some(4));
        assert!(p.walkable(15, 15));
        assert_eq!(p.visual_at(5, 5), Some(0));
        assert!(!p.walkable(5, 5));
        assert_eq!(p.visual_at(10, 15), Some(15));
    }

    #[test]
    fn flood_fill_same_color_only_touches_other_planes() {
        let mut p = Picture::new();
        p.rect(0, 0, 4, 4, Pen::color(1));
        p.flood_fill(2, 2, Pen::color(0).ctrl(CTRL_WALK));
        assert!(p.walkable(2, 2));
        assert!(!p.walkable(0, 0));
    }

    #[test]
    fn line_endpoints() {
        let mut p = Picture::new();
        p.line(0, 0, 10, 5, Pen::color(9));
        assert_eq!(p.visual_at(0, 0), Some(9));
        assert_eq!(p.visual_at(10, 5), Some(9));
    }

    #[test]
    fn priority_bands_increase_downward() {
        assert!(priority_for_y(150) > priority_for_y(100));
        assert_eq!(priority_for_y(0), 1);
    }

    #[test]
    fn rgba_expansion() {
        let mut f = Frame::new();
        f.set(0, 0, 15);
        let mut out = Vec::new();
        f.to_rgba(&mut out);
        assert_eq!(out.len(), WIDTH * HEIGHT * 4);
        assert_eq!(&out[0..4], &[255, 255, 255, 255]);
        assert_eq!(&out[4..8], &[0, 0, 0, 255]);
    }
}
