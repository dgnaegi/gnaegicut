use crate::project::Aspect;
use crate::theme::{ACCENT, BORDER, bold};
use eframe::egui::{Align2, Color32, Painter, Rect, Stroke, StrokeKind, pos2};

/// Dims the areas covered by platform UI and outlines what stays visible.
pub fn safe_zone(p: &Painter, frame: Rect, aspect: Aspect) -> Rect {
    let [l, t, r, b] = aspect.safe_margins();
    let safe = Rect::from_min_max(
        pos2(frame.left() + l * frame.width(), frame.top() + t * frame.height()),
        pos2(frame.right() - r * frame.width(), frame.bottom() - b * frame.height()),
    );
    let dim = Color32::from_black_alpha(110);
    p.rect_filled(Rect::from_min_max(frame.min, pos2(frame.right(), safe.top())), 0.0, dim);
    p.rect_filled(
        Rect::from_min_max(pos2(frame.left(), safe.bottom()), frame.max),
        0.0,
        dim,
    );
    p.rect_filled(
        Rect::from_min_max(pos2(frame.left(), safe.top()), pos2(safe.left(), safe.bottom())),
        0.0,
        dim,
    );
    p.rect_filled(
        Rect::from_min_max(pos2(safe.right(), safe.top()), pos2(frame.right(), safe.bottom())),
        0.0,
        dim,
    );
    p.rect_stroke(safe, 0.0, Stroke::new(BORDER, ACCENT), StrokeKind::Inside);
    p.text(
        safe.left_top() + [6.0, 4.0].into(),
        Align2::LEFT_TOP,
        "SAFE ZONE",
        bold(10.0),
        ACCENT,
    );
    safe
}
