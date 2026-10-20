//! Actor movement over the picture's control (walkable) plane.

use super::gfx::Picture;
use super::sprite::Dir;

/// Pixels per tick horizontally / vertically (vertical is slower for a
/// faux-perspective feel, like the old interpreters).
pub const STEP_X: i32 = 2;
pub const STEP_Y: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Idle,
    /// Keep walking in a direction until blocked or told to stop.
    Heading(Dir),
    /// Walk toward a point.
    Target(i32, i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Actor {
    /// Feet position.
    pub x: i32,
    pub y: i32,
    pub dir: Dir,
    pub frame: u8,
    pub motion: Motion,
}

/// Half-width of the hero's footprint used for collision.
pub const FOOT_HALF: i32 = 3;

pub fn can_stand(pic: &Picture, x: i32, y: i32) -> bool {
    pic.walkable(x - FOOT_HALF, y) && pic.walkable(x, y) && pic.walkable(x + FOOT_HALF, y)
}

impl Actor {
    pub fn new(x: i32, y: i32) -> Self {
        Actor { x, y, dir: Dir::Down, frame: 0, motion: Motion::Idle }
    }

    pub fn is_moving(&self) -> bool {
        self.motion != Motion::Idle
    }

    pub fn stop(&mut self) {
        self.motion = Motion::Idle;
        self.frame = 0;
    }

    /// AGI-style arrow handling: pressing the direction you're already
    /// walking stops you; any other direction turns you that way.
    pub fn press_dir(&mut self, d: Dir) {
        if self.motion == Motion::Heading(d) {
            self.stop();
        } else {
            self.motion = Motion::Heading(d);
            self.dir = d;
        }
    }

    pub fn walk_to(&mut self, x: i32, y: i32) {
        self.motion = Motion::Target(x, y);
    }

    fn try_move(&mut self, pic: &Picture, dx: i32, dy: i32) -> bool {
        if (dx != 0 || dy != 0) && can_stand(pic, self.x + dx, self.y + dy) {
            self.x += dx;
            self.y += dy;
            true
        } else {
            false
        }
    }

    /// Advance one tick. Returns true if the actor moved.
    pub fn tick(&mut self, pic: &Picture) -> bool {
        let moved = match self.motion {
            Motion::Idle => false,
            Motion::Heading(d) => {
                let (dx, dy) = match d {
                    Dir::Up => (0, -STEP_Y),
                    Dir::Down => (0, STEP_Y),
                    Dir::Left => (-STEP_X, 0),
                    Dir::Right => (STEP_X, 0),
                };
                let ok = self.try_move(pic, dx, dy);
                if !ok {
                    self.stop();
                }
                ok
            }
            Motion::Target(tx, ty) => {
                let ddx = tx - self.x;
                let ddy = ty - self.y;
                if ddx.abs() <= 1 && ddy == 0 || (ddx == 0 && ddy == 0) {
                    self.stop();
                    return false;
                }
                let dx = ddx.signum() * ddx.abs().min(STEP_X);
                let dy = ddy.signum() * ddy.abs().min(STEP_Y);
                // Facing follows the dominant axis (vertical counts double,
                // since it moves at half speed).
                self.dir = if ddx.abs() >= ddy.abs() * 2 {
                    if ddx > 0 { Dir::Right } else { Dir::Left }
                } else if ddy > 0 {
                    Dir::Down
                } else {
                    Dir::Up
                };
                let ok = self.try_move(pic, dx, dy)
                    || self.try_move(pic, dx, 0)
                    || self.try_move(pic, 0, dy);
                if !ok {
                    self.stop();
                }
                ok
            }
        };
        if moved {
            self.frame = (self.frame + 1) % 4;
        }
        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::gfx::{Pen, CTRL_BLOCK, CTRL_WALK};

    fn floor() -> Picture {
        let mut p = Picture::new();
        p.fill_rect(0, 100, 320, 180, Pen::control_only(CTRL_WALK));
        // A table you can't walk through.
        p.fill_rect(150, 120, 170, 140, Pen::control_only(CTRL_BLOCK));
        p
    }

    #[test]
    fn walks_to_target() {
        let pic = floor();
        let mut a = Actor::new(20, 110);
        a.walk_to(60, 115);
        for _ in 0..200 {
            a.tick(&pic);
        }
        assert!((a.x - 60).abs() <= 1 && a.y == 115, "at {},{}", a.x, a.y);
        assert!(!a.is_moving());
    }

    #[test]
    fn blocked_by_obstacle_and_edges() {
        let pic = floor();
        let mut a = Actor::new(100, 130);
        a.press_dir(Dir::Right);
        for _ in 0..200 {
            a.tick(&pic);
        }
        assert!(a.x + FOOT_HALF < 150, "stopped before table, x={}", a.x);
        assert!(!a.is_moving());
        let mut b = Actor::new(100, 102);
        b.press_dir(Dir::Up);
        for _ in 0..50 {
            b.tick(&pic);
        }
        assert_eq!(b.y, 100);
    }

    #[test]
    fn same_arrow_twice_stops() {
        let mut a = Actor::new(100, 130);
        a.press_dir(Dir::Left);
        assert!(a.is_moving());
        a.press_dir(Dir::Left);
        assert!(!a.is_moving());
    }

    #[test]
    fn slides_along_walls() {
        let pic = floor();
        let mut a = Actor::new(100, 101);
        // Target is above the walkable area: should slide horizontally.
        a.walk_to(130, 90);
        for _ in 0..100 {
            a.tick(&pic);
        }
        assert!(a.x > 110);
    }
}
