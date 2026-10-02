//! An item's box on the preview: centre, half-size and rotation, with the maths for corners and hit tests.

use crate::project::{Aspect, Item};
use eframe::egui::{Pos2, Rect, Vec2, vec2};

const HANDLE_REACH: f32 = 30.0; // how far above the top edge the rotation handle sits

#[derive(Clone, Copy)]
pub struct Quad {
    pub center: Pos2,
    pub half: Vec2,
    pub angle: f32, // radians, clockwise
}

impl Quad {
    pub fn of(frame: Rect, it: &Item, aspect: Aspect) -> Self {
        let (fw, fh) = it.frac(aspect);
        Self {
            center: frame.min + vec2(it.x * frame.width(), it.y * frame.height()),
            half: vec2(fw * frame.width(), fh * frame.height()) / 2.0,
            angle: it.rotation.to_radians(),
        }
    }

    fn rotate(&self, v: Vec2, angle: f32) -> Vec2 {
        let (s, c) = angle.sin_cos();
        vec2(v.x * c - v.y * s, v.x * s + v.y * c)
    }

    /// Top-left, top-right, bottom-right, bottom-left.
    pub fn corners(&self) -> [Pos2; 4] {
        let h = self.half;
        [vec2(-h.x, -h.y), vec2(h.x, -h.y), vec2(h.x, h.y), vec2(-h.x, h.y)]
            .map(|c| self.center + self.rotate(c, self.angle))
    }

    pub fn contains(&self, p: Pos2) -> bool {
        let local = self.rotate(p - self.center, -self.angle);
        local.x.abs() <= self.half.x && local.y.abs() <= self.half.y
    }

    /// Where the rotation handle sits: above the middle of the top edge, in the item's own orientation.
    pub fn rotate_handle(&self) -> Pos2 {
        self.center + self.rotate(vec2(0.0, -self.half.y - HANDLE_REACH), self.angle)
    }

    pub fn top_middle(&self) -> Pos2 {
        self.center + self.rotate(vec2(0.0, -self.half.y), self.angle)
    }

    /// The rotation (degrees) that points the handle at `p`, snapped to 15° steps when close.
    pub fn angle_towards(&self, p: Pos2, snap: bool) -> f32 {
        let d = p - self.center;
        let deg = d.y.atan2(d.x).to_degrees() + 90.0;
        let deg = if deg > 180.0 { deg - 360.0 } else { deg };
        let nearest = (deg / 15.0).round() * 15.0;
        if snap && (deg - nearest).abs() < 4.0 {
            nearest
        } else {
            deg
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(angle_deg: f32) -> Quad {
        Quad {
            center: Pos2::new(100.0, 100.0),
            half: vec2(40.0, 20.0),
            angle: angle_deg.to_radians(),
        }
    }

    #[test]
    fn contains_follows_rotation() {
        let (flat, turned) = (quad(0.0), quad(90.0));
        let wide_point = Pos2::new(100.0 + 35.0, 100.0);
        assert!(flat.contains(wide_point) && !turned.contains(wide_point));
        assert!(
            turned.contains(Pos2::new(100.0, 100.0 + 35.0)),
            "the long side now points down"
        );
    }

    #[test]
    fn handle_points_away_from_the_top_edge() {
        let h = quad(0.0).rotate_handle();
        assert!((h.x - 100.0).abs() < 1e-3 && h.y < 100.0 - 20.0);
        let right = quad(90.0).rotate_handle();
        assert!(
            right.x > 100.0 + 20.0,
            "turning clockwise swings the handle to the right"
        );
    }

    #[test]
    fn angle_round_trips_and_snaps() {
        let q = quad(0.0);
        assert!(
            (q.angle_towards(Pos2::new(100.0, 0.0), false)).abs() < 1e-3,
            "straight up is 0°"
        );
        assert!((q.angle_towards(Pos2::new(200.0, 100.0), false) - 90.0).abs() < 1e-3);
        let near = Pos2::new(
            100.0 + 100.0 * 14.0_f32.to_radians().sin(),
            100.0 - 100.0 * 14.0_f32.to_radians().cos(),
        );
        assert_eq!(q.angle_towards(near, true), 15.0);
    }
}
