//! Click-to-walk path-finding: a breadth-first search over the same step
//! lattice the actor moves on (2 px horizontally, 1 px vertically, or both
//! at once), so a found path can be followed step by step.

use std::collections::VecDeque;

use super::actor::{can_stand, STEP_X, STEP_Y};
use super::gfx::{Picture, HEIGHT, WIDTH};

const W: i32 = WIDTH as i32;
const H: i32 = HEIGHT as i32;
const NONE: u32 = u32::MAX;

/// Find a walking route from `from` to `to`. Positions where `avoid`
/// returns true are treated as walls (used to keep the hero from stumbling
/// through a doorway on the way to somewhere else). If `to` can't be
/// reached, the route ends at the reachable spot closest to it. Returns the
/// steps after `from`, in order; empty if already there or boxed in.
pub fn find_path(
    pic: &Picture,
    from: (i32, i32),
    to: (i32, i32),
    avoid: impl Fn(i32, i32) -> bool,
) -> Vec<(i32, i32)> {
    let idx = |x: i32, y: i32| (y * W + x) as usize;
    let inside = |x: i32, y: i32| (0..W).contains(&x) && (0..H).contains(&y);
    if !inside(from.0, from.1) {
        return vec![];
    }
    let cost = |x: i32, y: i32| {
        let dx = (x - to.0) as i64;
        let dy = ((y - to.1) * 2) as i64;
        dx * dx + dy * dy
    };
    let mut parent = vec![NONE; (W * H) as usize];
    let start = idx(from.0, from.1);
    parent[start] = start as u32;
    let mut q = VecDeque::from([from]);
    let mut best = (cost(from.0, from.1), from);
    // Straight moves first, so ties favor natural-looking routes.
    const MOVES: [(i32, i32); 8] = [
        (STEP_X, 0),
        (-STEP_X, 0),
        (0, STEP_Y),
        (0, -STEP_Y),
        (STEP_X, STEP_Y),
        (-STEP_X, STEP_Y),
        (STEP_X, -STEP_Y),
        (-STEP_X, -STEP_Y),
    ];
    while let Some((x, y)) = q.pop_front() {
        let c = cost(x, y);
        if c < best.0 {
            best = (c, (x, y));
        }
        if (x - to.0).abs() <= 1 && y == to.1 {
            break;
        }
        for (dx, dy) in MOVES {
            let (nx, ny) = (x + dx, y + dy);
            if !inside(nx, ny) || parent[idx(nx, ny)] != NONE {
                continue;
            }
            if !can_stand(pic, nx, ny) || avoid(nx, ny) {
                continue;
            }
            parent[idx(nx, ny)] = idx(x, y) as u32;
            q.push_back((nx, ny));
        }
    }
    let mut out = vec![];
    let mut cur = idx(best.1 .0, best.1 .1);
    while cur != start {
        out.push(((cur as i32) % W, (cur as i32) / W));
        cur = parent[cur] as usize;
    }
    out.reverse();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::gfx::{Pen, CTRL_BLOCK, CTRL_WALK};

    fn open_floor() -> Picture {
        let mut p = Picture::new();
        p.fill_rect(0, 100, 320, 170, Pen::control_only(CTRL_WALK));
        p
    }

    #[test]
    fn goes_around_a_wall() {
        let mut p = open_floor();
        // A wall from y 100 to 160 at x 150..160: must go under it.
        p.fill_rect(150, 100, 160, 160, Pen::control_only(CTRL_BLOCK));
        let path = find_path(&p, (100, 120), (200, 120), |_, _| false);
        assert_eq!(*path.last().unwrap(), (200, 120));
        assert!(path.iter().any(|&(_, y)| y >= 160));
        // Every step is a legal single actor step onto standable ground.
        let mut prev = (100, 120);
        for &(x, y) in &path {
            assert!((x - prev.0).abs() <= STEP_X && (y - prev.1).abs() <= STEP_Y);
            assert!(can_stand(&p, x, y));
            prev = (x, y);
        }
    }

    #[test]
    fn avoids_zones_and_stops_close_when_unreachable() {
        let p = open_floor();
        let path = find_path(&p, (100, 120), (200, 120), |x, _| {
            (140..160).contains(&x) && true
        });
        // Fully walled off by the avoid zone: end as close as possible.
        let end = *path.last().unwrap();
        assert!(end.0 < 140 && end.0 >= 136, "{end:?}");
        assert!(find_path(&p, (100, 120), (100, 120), |_, _| false).is_empty());
    }
}
