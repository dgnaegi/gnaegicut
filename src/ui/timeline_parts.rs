//! Pieces of the timeline panel: the ruler, the captions lane, scrubbing and the drop highlight.

use super::timeline_items::{ADD_ROW, Layout, RULER, snap};
use crate::app::{App, Selection};
use crate::theme::{ACCENT, BLACK, BORDER, WHITE, bold};
use eframe::egui::{Align2, Id, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

pub(super) const CAPTION_LANE: f32 = 28.0;
const GAP: f32 = super::timeline_items::GAP;

pub(super) fn ruler(ui: &Ui, lay: &Layout, seconds: f64) {
    let p = ui.painter_at(lay.rect);
    for s in 0..seconds as i64 {
        let x = lay.x(s as f64);
        let long = s % 5 == 0;
        p.line_segment(
            [
                pos2(x, lay.rect.top()),
                pos2(x, lay.rect.top() + if long { 12.0 } else { 6.0 }),
            ],
            Stroke::new(1.0, BLACK),
        );
        if long {
            p.text(
                pos2(x + 3.0, lay.rect.top() + 1.0),
                Align2::LEFT_TOP,
                format!("{s}"),
                bold(10.0),
                BLACK,
            );
        }
    }
}

pub(super) fn captions_lane(ui: &mut Ui, app: &mut App, lay: &Layout) {
    let top = lay.captions_top() + GAP;
    let lane = Rect::from_min_max(pos2(lay.x(0.0), top), pos2(lay.rect.right(), top + CAPTION_LANE));
    let resp = ui.interact(lane, Id::new("captions-lane"), Sense::CLICK);
    if resp.clicked() {
        app.select(Selection::Captions);
    }
    let p = ui.painter_at(lay.rect);
    let selected = app.selection == Selection::Captions;
    for c in &app.project.captions {
        let r = Rect::from_min_max(pos2(lay.x(c.start), top), pos2(lay.x(c.end), top + CAPTION_LANE));
        p.rect_filled(r, 0.0, if selected { ACCENT } else { WHITE });
        p.rect_stroke(r, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
        p.with_clip_rect(r).text(
            r.left_center() + vec2(4.0, 0.0),
            Align2::LEFT_CENTER,
            &c.text,
            bold(10.0),
            if selected { WHITE } else { BLACK },
        );
    }
}

/// Click or drag on the ruler or empty space to move the playhead; clicking empty lanes deselects.
pub(super) fn scrub(app: &mut App, bg: &eframe::egui::Response, lay: &Layout) {
    let Some(pos) = bg
        .interact_pointer_pos()
        .filter(|_| bg.is_pointer_button_down_on() || bg.clicked())
    else {
        return;
    };
    app.stop();
    let t = lay.time(pos.x).clamp(0.0, app.project.total());
    // Like a magnet: the playhead jumps onto a nearby seam between clips, text or captions. Alt moves it freely.
    let free = bg.ctx.input(|i| i.modifiers.alt);
    app.playhead = if app.magnet && !free {
        snap(t, 0.0, &app.project.edges(), lay.zoom).clamp(0.0, app.project.total())
    } else {
        t
    };
    if bg.clicked() && pos.y > lay.rect.top() + RULER {
        app.select(Selection::None);
    }
}

/// While media is dragged over the timeline, outlines the lanes it would land on (the add-track bar for a new one).
pub(super) fn highlight_drop(ui: &Ui, app: &App, lay: &Layout) {
    let (Some((_, pointer)), Some(view)) = (app.drag_hover(), app.timeline_view) else {
        return;
    };
    if !view.contains(pointer) {
        return;
    }
    let first = view.lane_at(pointer.y);
    // A new track (the bar above the lanes) for layers, or the one lane the clips will run along.
    let lanes: Vec<usize> = vec![first];
    let p = ui.painter_at(lay.rect);
    for lane in lanes {
        let row = if lane >= lay.lanes {
            Rect::from_min_size(
                pos2(lay.rect.left(), lay.rect.top() + RULER),
                vec2(lay.rect.width(), ADD_ROW),
            )
        } else {
            Rect::from_min_size(
                pos2(lay.rect.left(), lay.lane_top(lane)),
                vec2(lay.rect.width(), lay.lane),
            )
        };
        p.rect_stroke(row, 0.0, Stroke::new(3.0, ACCENT), StrokeKind::Inside);
    }
}
