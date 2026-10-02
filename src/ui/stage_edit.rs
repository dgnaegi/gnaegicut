//! Direct manipulation on the preview: click to select; drag to move (with magic lines), corner handles to
//! scale, the round handle to rotate.

use super::stage_geometry::Quad;
use super::stage_guides as guides;
use super::stage_scale::{input_factor, reachable};
use crate::app::{App, Selection};
use crate::theme::{ACCENT, BORDER, WHITE};
use eframe::egui::{Id, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, vec2};

const HANDLE: f32 = 14.0;
type Lines = (Option<f32>, Option<f32>);

pub fn interact(ui: &mut Ui, app: &mut App, frame: Rect) {
    let bg = ui.interact(frame, Id::new("stage-bg"), Sense::CLICK);
    if let (true, Some(pos)) = (bg.clicked(), bg.interact_pointer_pos()) {
        pick(app, frame, pos);
    }
    match app.selection {
        Selection::Item(id) => edit_item(ui, app, frame, id),
        Selection::Captions => move_captions(ui, app, frame),
        Selection::None | Selection::Track(_) => {}
    }
}

fn pick(app: &mut App, frame: Rect, pos: Pos2) {
    let aspect = app.project.aspect;
    let hit = app.project.topmost_at(app.playhead, |i| {
        i.kind.is_visual() && Quad::of(frame, i, aspect).contains(pos)
    });
    app.select(hit.map_or(Selection::None, Selection::Item));
}

fn edit_item(ui: &mut Ui, app: &mut App, frame: Rect, id: u64) {
    let (aspect, t) = (app.project.aspect, app.playhead);
    let Some(quad) = app
        .project
        .get(id)
        .filter(|i| i.kind.is_visual() && i.at <= t && t < i.end_at())
        .map(|i| Quad::of(frame, i, aspect))
    else {
        return;
    };
    let corners = quad.corners();
    let free = ui.input(|i| i.modifiers.alt);

    let body = ui.interact(Rect::from_points(&corners), Id::new(("body", id)), Sense::drag());
    let mut lines: Lines = (None, None);
    if body.dragged() {
        lines = move_item(ui, app, frame, id, body.drag_delta(), body.drag_started());
    }
    let area = app.stage_rect.unwrap_or(frame);
    let mut grow = input_factor(ui, area); // the mouse wheel and pinch scale the selection
    for (n, corner) in corners.into_iter().enumerate() {
        let handle = ui.interact(
            Rect::from_center_size(reachable(corner, area, HANDLE), Vec2::splat(HANDLE)),
            Id::new(("handle", id, n)),
            Sense::drag(),
        );
        if let (true, Some(p)) = (handle.dragged(), handle.interact_pointer_pos()) {
            let (now, before) = (
                (p - quad.center).length(),
                (p - handle.drag_delta() - quad.center).length(),
            );
            if before > 1.0 {
                grow = now / before;
            }
        }
    }
    let rotate = ui.interact(
        Rect::from_center_size(quad.rotate_handle(), Vec2::splat(HANDLE + 4.0)),
        Id::new(("rot", id)),
        Sense::drag(),
    );
    let angle = rotate
        .interact_pointer_pos()
        .filter(|_| rotate.dragged())
        .map(|p| quad.angle_towards(p, !free));

    if grow != 1.0 || angle.is_some() {
        app.pause_for_edit();
        app.scale_item(id, grow);
        if let (Some(it), Some(angle)) = (app.project.get_mut(id), angle) {
            it.rotation = angle;
        }
    }
    if let Some(item) = app.project.get(id) {
        paint_selection(ui, &Quad::of(frame, item, aspect), area);
    }
    guides::paint(&ui.painter_at(frame), frame, lines);
}

/// Moves an item by `delta` pixels. The unsnapped position is tracked separately so snapping never swallows
/// slow drags; hold Alt to move freely.
fn move_item(ui: &Ui, app: &mut App, frame: Rect, id: u64, delta: Vec2, started: bool) -> Lines {
    let (xs, ys) = guides::targets(app, Some(id));
    let aspect = app.project.aspect;
    app.pause_for_edit();
    let Some(it) = app.project.get_mut(id) else {
        return (None, None);
    };
    let key = Id::new(("move-raw", id));
    if started {
        ui.data_mut(|d| d.insert_temp(key, (it.x, it.y)));
    }
    let (rx, ry): (f32, f32) = ui.data(|d| d.get_temp(key)).unwrap_or((it.x, it.y));
    let raw = (rx + delta.x / frame.width(), ry + delta.y / frame.height());
    ui.data_mut(|d| d.insert_temp(key, raw));
    if ui.input(|i| i.modifiers.alt) {
        (it.x, it.y) = raw;
        return (None, None);
    }
    let ((reach_x, reach_y), (hx, hy)) = (guides::reach(frame), it.half_extent(aspect));
    let (sx, sy) = (
        guides::snap_axis(raw.0, hx, &xs, reach_x),
        guides::snap_axis(raw.1, hy, &ys, reach_y),
    );
    (it.x, it.y) = (sx.value, sy.value);
    (sx.line, sy.line)
}

fn paint_selection(ui: &Ui, quad: &Quad, area: Rect) {
    let p = ui.painter();
    p.add(Shape::closed_line(quad.corners().to_vec(), Stroke::new(BORDER, ACCENT)));
    for corner in quad.corners() {
        let r = Rect::from_center_size(reachable(corner, area, HANDLE), Vec2::splat(HANDLE));
        p.rect_filled(r, 0.0, WHITE);
        p.rect_stroke(r, 0.0, Stroke::new(BORDER, ACCENT), StrokeKind::Inside);
    }
    let knob = quad.rotate_handle();
    p.line_segment([quad.top_middle(), knob], Stroke::new(BORDER, ACCENT));
    p.circle(knob, HANDLE / 2.0 + 1.0, WHITE, Stroke::new(BORDER, ACCENT));
}

/// With captions selected, dragging anywhere on the frame moves them, snapping to the centre lines.
fn move_captions(ui: &mut Ui, app: &mut App, frame: Rect) {
    let drag = ui.interact(frame, Id::new("caption-drag"), Sense::drag());
    let (xs, ys) = guides::targets(app, None);
    let mut lines: Lines = (None, None);
    let grow = input_factor(ui, app.stage_rect.unwrap_or(frame));
    if grow != 1.0 {
        app.pause_for_edit();
        let layout = &mut app.project.caption_layout;
        layout.size = (layout.size * grow).clamp(0.4, 4.0);
    }
    if drag.dragged() {
        app.pause_for_edit();
        let key = Id::new("caption-raw");
        let aspect = app.project.aspect;
        let c = &mut app.project.caption_layout;
        let (x, y) = c.position(aspect);
        if drag.drag_started() {
            ui.data_mut(|d| d.insert_temp(key, (x, y)));
        }
        let (rx, ry): (f32, f32) = ui.data(|d| d.get_temp(key)).unwrap_or((x, y));
        let d = drag.drag_delta();
        let raw = (
            (rx + d.x / frame.width()).clamp(0.0, 1.0),
            (ry + d.y / frame.height()).clamp(0.0, 1.0),
        );
        ui.data_mut(|dt| dt.insert_temp(key, raw));
        let (reach_x, reach_y) = guides::reach(frame);
        let (sx, sy) = (
            guides::snap_axis(raw.0, 0.0, &xs, reach_x),
            guides::snap_axis(raw.1, 0.0, &ys, reach_y),
        );
        let free = ui.input(|i| i.modifiers.alt);
        let (nx, ny) = if free { raw } else { (sx.value, sy.value) };
        c.place(nx, ny);
        lines = if free { (None, None) } else { (sx.line, sy.line) };
    }
    let (cx, cy) = app.project.caption_layout.position(app.project.aspect);
    let at = frame.min + vec2(cx * frame.width(), cy * frame.height());
    let guide = Rect::from_center_size(at, vec2(frame.width() * 0.84, frame.height() * 0.07));
    ui.painter()
        .rect_stroke(guide, 0.0, Stroke::new(BORDER, ACCENT), StrokeKind::Outside);
    guides::paint(&ui.painter_at(frame), frame, lines);
}
