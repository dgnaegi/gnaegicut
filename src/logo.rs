//! The GnaegiCut mark: a geometric G cut by a red diagonal. Mirrors assets/logo.svg (48-unit grid).

use crate::theme::{ACCENT, BLACK, WHITE};
use eframe::egui::{Painter, Rect, Shape, Stroke, epaint::PathShape, pos2};

const START_DEG: f32 = -40.0;

pub fn paint(p: &Painter, rect: Rect) {
    let s = rect.width() / 48.0;
    let at = |x: f32, y: f32| pos2(rect.left() + x * s, rect.top() + y * s);
    p.rect_filled(rect, 0.0, BLACK);

    // G: three-quarter ring plus crossbar, as one polyline.
    let arc = (0..=60).map(|i| {
        let deg = START_DEG - 320.0 * i as f32 / 60.0;
        let (sin, cos) = deg.to_radians().sin_cos();
        at(24.0 + 13.0 * cos, 24.0 + 13.0 * sin)
    });
    let pts: Vec<_> = arc.chain([at(37.0, 24.0), at(24.0, 24.0)]).collect();
    p.add(Shape::Path(PathShape::line(pts, Stroke::new(6.0 * s, WHITE))));

    // The cut: a black gap with a red blade inside it.
    p.line_segment([at(4.0, 44.0), at(44.0, 4.0)], Stroke::new(8.0 * s, BLACK));
    p.line_segment([at(4.0, 44.0), at(44.0, 4.0)], Stroke::new(4.0 * s, ACCENT));
}
