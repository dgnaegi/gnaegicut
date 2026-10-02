//! The two small icon buttons at the top right: new project and save. Drawn with shapes, so no icon font is needed.

use crate::theme::{ACCENT, BLACK, BORDER, WHITE};
use eframe::egui::{CursorIcon, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

const SIZE: f32 = 30.0;

/// A square button; `draw` paints the symbol in the given colour inside the rectangle.
fn icon(ui: &mut Ui, draw: impl FnOnce(&Painter, Rect, eframe::egui::Color32)) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(SIZE, SIZE), Sense::CLICK);
    let hot = resp.hovered() || resp.is_pointer_button_down_on();
    let (bg, fg) = if hot { (ACCENT, WHITE) } else { (WHITE, BLACK) };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(
        rect,
        0.0,
        Stroke::new(BORDER, if hot { ACCENT } else { BLACK }),
        StrokeKind::Inside,
    );
    draw(p, rect.shrink(8.0), fg);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn plus(ui: &mut Ui) -> Response {
    icon(ui, |p, r, fg| {
        let s = Stroke::new(2.5, fg);
        p.line_segment([pos2(r.center().x, r.top()), pos2(r.center().x, r.bottom())], s);
        p.line_segment([pos2(r.left(), r.center().y), pos2(r.right(), r.center().y)], s);
    })
}

/// A floppy disk: the body, the shutter at the top and the label at the bottom.
pub fn floppy(ui: &mut Ui) -> Response {
    icon(ui, |p, r, fg| {
        let at = |fx: f32, fy: f32| -> Pos2 { pos2(r.left() + r.width() * fx, r.top() + r.height() * fy) };
        p.rect_stroke(r, 0.0, Stroke::new(2.0, fg), StrokeKind::Inside);
        p.rect_filled(Rect::from_two_pos(at(0.25, 0.0), at(0.75, 0.38)), 0.0, fg);
        p.rect_filled(Rect::from_two_pos(at(0.2, 0.58), at(0.8, 1.0)), 0.0, fg);
    })
}
