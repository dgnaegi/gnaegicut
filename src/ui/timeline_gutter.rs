use super::timeline_items::{ADD_ROW, GAP, GUTTER, LANE, Layout, RULER};
use crate::app::App;
use crate::theme::{ACCENT, BLACK, BORDER, WHITE, bold};
use crate::widgets::{Kind, button};
use eframe::egui::{Align2, Id, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

/// Lane labels stay at the visible left edge; clicking one chooses where new media lands, right-click removes
/// an empty track. The bar above the top lane adds a track.
pub fn show(ui: &mut Ui, app: &mut App, lay: &Layout) {
    let left = ui.clip_rect().left().max(lay.rect.left());
    let p = ui.painter_at(lay.rect.intersect(ui.clip_rect()));

    let add = Rect::from_min_size(pos2(left, lay.rect.top() + RULER), vec2(GUTTER * 2.6, ADD_ROW));
    let hot = ui.interact(add, Id::new("add-track"), Sense::click());
    p.rect_filled(add, 0.0, if hot.hovered() { BLACK } else { WHITE });
    p.rect_stroke(add, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
    p.text(
        add.center(),
        Align2::CENTER_CENTER,
        "+ NEW TRACK",
        bold(11.0),
        if hot.hovered() { WHITE } else { BLACK },
    );
    if hot.on_hover_cursor(eframe::egui::CursorIcon::PointingHand).clicked() {
        app.add_track();
    }

    let mut remove = None;
    for ti in 0..lay.lanes {
        let r = Rect::from_min_size(pos2(left, lay.lane_top(ti)), vec2(GUTTER - GAP, LANE));
        let resp = ui.interact(r, Id::new(("lane", ti)), Sense::click());
        if resp.clicked() {
            app.track = ti;
        }
        resp.context_menu(|ui| {
            if button(ui, "Remove track", Kind::Plain).clicked() {
                remove = Some(ti);
                ui.close();
            }
        });
        let active = app.track == ti;
        p.rect_filled(r, 0.0, if active { ACCENT } else { WHITE });
        p.rect_stroke(r, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
        p.text(
            r.center(),
            Align2::CENTER_CENTER,
            format!("{}", ti + 1),
            bold(14.0),
            if active { WHITE } else { BLACK },
        );
    }
    if let Some(ti) = remove {
        app.remove_track(ti);
    }
}
