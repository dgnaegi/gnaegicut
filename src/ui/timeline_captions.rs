//! The captions lane: every caption is a block. Click selects it, drag an end to make it longer or shorter, drag the
//! middle to move it, Backspace removes it. With the magnet on, the edges snap to seams of clips and other captions.

use super::timeline_items::{GAP, Layout, snap};
use super::timeline_parts::CAPTION_LANE;
use crate::app::{App, Selection};
use crate::theme::{ACCENT, BLACK, BORDER, CAPTION, WHITE, bold};
use eframe::egui::{Align2, CursorIcon, Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

const GRIP: f32 = 7.0; // width of the draggable strip at each end of a block

enum Change {
    Start(f64),
    End(f64),
    Move(f64),
}

pub fn show(ui: &mut Ui, app: &mut App, lay: &Layout) {
    let top = lay.captions_top() + GAP;
    let lane = Rect::from_min_max(pos2(lay.x(0.0), top), pos2(lay.rect.right(), top + CAPTION_LANE));
    if ui.interact(lane, Id::new("captions-lane"), Sense::CLICK).clicked() {
        app.select(Selection::Captions);
    }
    let p = ui.painter_at(lay.rect);
    let (mut picked, mut change) = (None, None);
    for (i, c) in app.project.captions.iter().enumerate() {
        let r = Rect::from_min_max(pos2(lay.x(c.start), top), pos2(lay.x(c.end), top + CAPTION_LANE));
        let chosen = app.selection == Selection::Caption(i);
        let (fill, ink) = if chosen { (ACCENT, WHITE) } else { (WHITE, BLACK) };
        p.rect_filled(r, 0.0, fill);
        p.rect_stroke(r, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
        p.with_clip_rect(r).text(
            r.left_center() + vec2(8.0, 0.0),
            Align2::LEFT_CENTER,
            &c.text,
            bold(CAPTION),
            ink,
        );

        let body = ui.interact(r, Id::new(("caption", i)), Sense::click_and_drag());
        let left = grip(ui, r, i, false);
        let right = grip(ui, r, i, true);
        if body.clicked() || body.drag_started() || left.drag_started() || right.drag_started() {
            picked = Some(i);
        }
        let time = |resp: &Response| resp.interact_pointer_pos().map(|pos| lay.time(pos.x));
        if let (true, Some(t)) = (left.dragged(), time(&left)) {
            change = Some((i, Change::Start(t)));
        } else if let (true, Some(t)) = (right.dragged(), time(&right)) {
            change = Some((i, Change::End(t)));
        } else if let (true, Some(t)) = (body.dragged(), time(&body)) {
            let key = Id::new(("caption-grab", i));
            if body.drag_started() {
                ui.data_mut(|d| d.insert_temp(key, t - c.start)); // where in the block it was grabbed
            }
            change = Some((i, Change::Move(t - ui.data(|d| d.get_temp(key)).unwrap_or(0.0))));
        }
        if chosen {
            for x in [r.left(), r.right()] {
                let knob = Rect::from_center_size(pos2(x, r.center().y), vec2(6.0, 16.0));
                p.rect_filled(knob, 0.0, WHITE);
                p.rect_stroke(knob, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
            }
        }
    }
    if let Some(i) = picked {
        app.select(Selection::Caption(i));
    }
    if let Some((i, change)) = change {
        app.pause_for_edit();
        apply(app, lay, i, change);
    }
}

fn grip(ui: &mut Ui, r: Rect, index: usize, right: bool) -> Response {
    let strip = if right {
        Rect::from_min_max(pos2(r.right() - GRIP, r.top()), r.right_bottom())
    } else {
        Rect::from_min_max(r.left_top(), pos2(r.left() + GRIP, r.bottom()))
    };
    ui.interact(strip, Id::new(("caption-grip", index, right)), Sense::drag())
        .on_hover_cursor(CursorIcon::ResizeHorizontal)
}

/// Applies a drag, snapping the moving edge (or the block's start) to nearby seams when the magnet is on.
fn apply(app: &mut App, lay: &Layout, index: usize, change: Change) {
    let Some(own) = app.project.captions.get(index).map(|c| (c.start, c.end)) else {
        return;
    };
    let seams: Vec<f64> = app
        .project
        .edges()
        .into_iter()
        .filter(|e| (e - own.0).abs() > 1e-6 && (e - own.1).abs() > 1e-6) // not its own edges, or it would stick
        .collect();
    let snapped = |t: f64| if app.magnet { snap(t, 0.0, &seams, lay.zoom) } else { t };
    match change {
        Change::Start(t) => app.project.resize_caption_start(index, snapped(t)),
        Change::End(t) => app.project.resize_caption_end(index, snapped(t)),
        Change::Move(t) => app.project.move_caption(index, snapped(t)),
    };
}
