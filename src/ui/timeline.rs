use super::timeline_items::{self as items, ADD_ROW, GAP, GUTTER, LANE, Layout, RULER};
use super::{timeline_gutter, timeline_joins};
use crate::app::{App, Selection};
use crate::drop::TimelineView;
use crate::patterns;
use crate::theme::{ACCENT, BLACK, BORDER, MUTED, WHITE, bold};
use crate::widgets::{Kind, button, caps};
use eframe::egui::{Align2, Id, Rect, ScrollArea, Sense, Slider, Stroke, StrokeKind, Ui, pos2, vec2};

const CAPTION_LANE: f32 = 28.0;

pub fn show(ui: &mut Ui, app: &mut App) {
    controls(ui, app);
    ScrollArea::both().auto_shrink(false).show(ui, |ui| track(ui, app));
}

fn controls(ui: &mut Ui, app: &mut App) {
    ui.horizontal_wrapped(|ui| {
        if button(ui, if app.playing() { "Pause" } else { "Play" }, Kind::Active).clicked() {
            app.toggle_play();
        }
        if button(ui, "Split (S)", Kind::Plain).clicked() {
            app.split();
        }
        if button(ui, "Delete", Kind::Plain).clicked() {
            app.delete();
        }
        if button(ui, "+ Track", Kind::Plain).clicked() {
            app.add_track();
        }
        let magnet = if app.magnet { Kind::Active } else { Kind::Plain };
        if button(ui, "Magnet", magnet).clicked() {
            app.magnet = !app.magnet;
        }
        ui.add(Slider::new(&mut app.zoom, 10.0..=200.0).text("zoom"));
        caps(ui, &format!("{:.1}s / {:.1}s", app.playhead, app.project.total()));
    });
}

fn track(ui: &mut Ui, app: &mut App) {
    let lanes = app.project.tracks.len();
    let seconds = app.project.total().max(10.0) + 5.0;
    let size = vec2(
        (GUTTER + seconds as f32 * app.zoom).max(ui.available_width()),
        RULER + ADD_ROW + GAP + lanes as f32 * (LANE + GAP) + CAPTION_LANE + GAP * 2.0,
    );
    let (rect, bg) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let lay = Layout {
        rect,
        zoom: app.zoom,
        lanes,
    };
    app.timeline_view = Some(TimelineView {
        visible: ui.clip_rect(),
        time_origin: lay.x(0.0),
        zoom: app.zoom,
        lanes,
        lanes_top: lay.lanes_top(),
        lane_pitch: LANE + GAP,
    });
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 0.0, MUTED);
    patterns::grid(&p, rect);
    ruler(ui, &lay, seconds);

    // (owner, time): the owner is excluded so an item never snaps to its own edges. Owner 0 = the grid.
    let edges: Vec<(u64, f64)> = [(0, 0.0), (0, app.playhead)]
        .into_iter()
        .chain(app.project.items().flat_map(|i| [(i.id, i.at), (i.id, i.end_at())]))
        .collect();
    let (mut dragged, mut picked) = (None, None);
    for (ti, track) in app.project.tracks.iter().enumerate() {
        for item in &track.items {
            let selected = app.selection == Selection::Item(item.id);
            let resp = items::show(ui, &lay, ti, item, selected);
            if resp.clicked() || resp.drag_started() {
                picked = Some((item.id, ti));
            }
            if let Some(moved) = items::drag(ui, &lay, (ti, item), &resp, (app.zoom, app.magnet), &edges) {
                dragged = Some(moved);
            }
        }
    }
    if let Some((id, ti)) = picked {
        app.select(Selection::Item(id));
        app.track = ti;
    }
    if let Some((id, target, at)) = dragged {
        app.pause_for_edit();
        app.project.place(id, target, at);
        app.track = target;
    }

    highlight_drop(ui, app, &lay);
    timeline_joins::show(ui, app, &lay);
    captions_lane(ui, app, &lay);
    scrub(app, &bg, &lay);
    let x = lay.x(app.playhead);
    p.line_segment(
        [pos2(x, rect.top()), pos2(x, rect.bottom())],
        Stroke::new(BORDER, ACCENT),
    );
    timeline_gutter::show(ui, app, &lay);
}

fn ruler(ui: &Ui, lay: &Layout, seconds: f64) {
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

fn captions_lane(ui: &mut Ui, app: &mut App, lay: &Layout) {
    let top = lay.captions_top() + GAP;
    let lane = Rect::from_min_max(pos2(lay.x(0.0), top), pos2(lay.rect.right(), top + CAPTION_LANE));
    let resp = ui.interact(lane, Id::new("captions-lane"), Sense::click());
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
fn scrub(app: &mut App, bg: &eframe::egui::Response, lay: &Layout) {
    let Some(pos) = bg
        .interact_pointer_pos()
        .filter(|_| bg.is_pointer_button_down_on() || bg.clicked())
    else {
        return;
    };
    app.stop();
    app.playhead = lay.time(pos.x).clamp(0.0, app.project.total());
    if bg.clicked() && pos.y > lay.rect.top() + RULER {
        app.select(Selection::None);
    }
}

/// While media is dragged over the timeline, outlines the lanes it would land on (the add-track bar for a new one).
fn highlight_drop(ui: &Ui, app: &App, lay: &Layout) {
    let (Some((count, pointer)), Some(view)) = (app.drag_hover(), app.timeline_view) else {
        return;
    };
    if !view.contains(pointer) {
        return;
    }
    let first = view.lane_at(pointer.y);
    let lanes: Vec<usize> = if app.stack_drops {
        (first..first + count).collect()
    } else {
        vec![first]
    };
    let p = ui.painter_at(lay.rect);
    for lane in lanes {
        let row = if lane >= lay.lanes {
            Rect::from_min_size(
                pos2(lay.rect.left(), lay.rect.top() + RULER),
                vec2(lay.rect.width(), ADD_ROW),
            )
        } else {
            Rect::from_min_size(pos2(lay.rect.left(), lay.lane_top(lane)), vec2(lay.rect.width(), LANE))
        };
        p.rect_stroke(row, 0.0, Stroke::new(3.0, ACCENT), StrokeKind::Inside);
    }
}
