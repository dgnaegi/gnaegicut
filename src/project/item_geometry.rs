//! Where an item is drawn: box size, rotation and position. Used by both the egui overlay and the ffmpeg graph.

use super::{Aspect, Item};

fn even(v: f32) -> u32 {
    ((v.round() as i64).max(2) & !1) as u32
}

impl Item {
    fn contain(&self, aspect: Aspect) -> f32 {
        let (w, h) = aspect.size();
        (w as f32 / self.src_w as f32).min(h as f32 / self.src_h as f32)
    }

    /// The `scale` at which the item covers the whole frame.
    pub fn cover_scale(&self, aspect: Aspect) -> f32 {
        let (w, h) = aspect.size();
        let cover = (w as f32 / self.src_w as f32).max(h as f32 / self.src_h as f32);
        cover / self.contain(aspect)
    }

    /// The item's (unrotated) box as fractions of the frame's width and height.
    pub fn frac(&self, aspect: Aspect) -> (f32, f32) {
        let (w, h) = aspect.size();
        let k = self.contain(aspect) * self.scale;
        (self.src_w as f32 * k / w as f32, self.src_h as f32 * k / h as f32)
    }

    /// Box size in pixels of a `w`x`h` frame; even, because yuv420 requires it.
    pub fn pixels(&self, aspect: Aspect, w: u32, h: u32) -> (u32, u32) {
        let (fw, fh) = self.frac(aspect);
        (even(fw * w as f32), even(fh * h as f32))
    }

    /// Size of the box once rotated: the smallest axis-aligned rectangle that holds it. A spinning entrance
    /// passes through every angle, so it gets the square that holds the box at any rotation.
    pub fn rotated(&self, (bw, bh): (u32, u32)) -> (u32, u32) {
        if self.spins() {
            let diagonal = even((bw as f32).hypot(bh as f32));
            return (diagonal, diagonal);
        }
        if self.rotation == 0.0 {
            return (bw, bh);
        }
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        let (s, c) = (sin.abs(), cos.abs());
        (even(bw as f32 * c + bh as f32 * s), even(bw as f32 * s + bh as f32 * c))
    }

    /// Half the rotated bounding box as fractions of the frame; used for alignment guides.
    pub fn half_extent(&self, aspect: Aspect) -> (f32, f32) {
        let (w, h) = aspect.size();
        let (fw, fh) = self.frac(aspect);
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        let (bw, bh) = (fw * w as f32, fh * h as f32);
        let (rw, rh) = (bw * cos.abs() + bh * sin.abs(), bw * sin.abs() + bh * cos.abs());
        (rw / 2.0 / w as f32, rh / 2.0 / h as f32)
    }

    /// Top-left corner of the (rotated) box in pixels of a `w`x`h` frame.
    pub fn origin(&self, w: u32, h: u32, box_px: (u32, u32)) -> (i32, i32) {
        let left = self.x * w as f32 - box_px.0 as f32 / 2.0;
        let top = self.y * h as f32 - box_px.1 as f32 / 2.0;
        (left.round() as i32, top.round() as i32)
    }
}
