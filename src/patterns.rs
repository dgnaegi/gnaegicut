//! Subtle CSS-style textures from the design system, painted with egui primitives.

use crate::theme::BLACK;
use eframe::egui::{Color32, Painter, Rect, Stroke, pos2};

const CELL: f32 = 24.0;

fn ink(alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(BLACK.r(), BLACK.g(), BLACK.b(), alpha)
}

/// 24px grid at ~3% opacity.
pub fn grid(p: &Painter, r: Rect) {
    let stroke = Stroke::new(1.0, ink(10));
    let mut x = r.left();
    while x <= r.right() {
        p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], stroke);
        x += CELL;
    }
    let mut y = r.top();
    while y <= r.bottom() {
        p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], stroke);
        y += CELL;
    }
}

/// 16px dot matrix at ~4% opacity.
pub fn dots(p: &Painter, r: Rect) {
    let mut y = r.top() + 8.0;
    while y < r.bottom() {
        let mut x = r.left() + 8.0;
        while x < r.right() {
            p.circle_filled(pos2(x, y), 1.0, ink(14));
            x += 16.0;
        }
        y += 16.0;
    }
}
