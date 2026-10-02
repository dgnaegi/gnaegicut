use super::timeline_items::{self as items, ADD_ROW, GAP, GUTTER, Layout, RULER, lane_height};
use super::timeline_parts::{CAPTION_LANE, highlight_drop, ruler, scrub};
use super::{timeline_gutter, timeline_joins};
use crate::app::{App, Selection};
use crate::drop::TimelineView;
use crate::patterns;
use crate::project::Kind as MediaKind;
use crate::theme::{ACCENT, BLACK, BORDER, MUTED};
use eframe::egui::{CursorIcon, Rect, ScrollArea, Sense, Stroke, Ui, pos2, vec2};

const GRIP: f32 = 10.0;
const MIN_PANEL: f32 = 170.0;
const MAX_PANEL_SHARE: f32 = 0.8; // the timeline may take at most this share of the window

pub fn show(ui: &mut Ui, app: &mut App) {
    grip(ui, app);
    super::timeline_tools::show(ui, app);
    // Pinch, or Cmd+scroll, over the tracks zooms the time axis around the pointer.
    let pinch = ui.input(|i| i.zoom_delta());
    let over = ui
        .ctx()
        .pointer_latest_pos()
        .filter(|p| app.timeline_view.is_some_and(|v| v.contains(*p)));
    if let (true, Some(p)) = (pinch != 1.0, over) {
        app.zoom_timeline(pinch, Some(p.x));
    }
    let mut area = ScrollArea::both().auto_shrink(false);
    if let Some(x) = app.scroll_to.take() {
        area = area.horizontal_scroll_offset(x);
    }
    area.show(ui, |ui| track(ui, app));
}

/// A bar along the top edge of the panel: drag it up and the timeline takes room from the preview above it.
fn grip(ui: &mut Ui, app: &mut App) {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), GRIP), Sense::drag());
    let hot = resp.hovered() || resp.dragged();
    let handle = Rect::from_center_size(rect.center(), vec2(56.0, 4.0));
    ui.painter().rect_filled(handle, 0.0, if hot { ACCENT } else { BLACK });
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
    }
    if resp.dragged() {
        let room = ui.ctx().content_rect().height() * MAX_PANEL_SHARE;
        app.timeline_height = (app.timeline_height - ui.input(|i| i.pointer.delta().y)).clamp(MIN_PANEL, room);
    }
}

fn track(ui: &mut Ui, app: &mut App) {
    let lanes = app.project.tracks.len();
    let seconds = app.project.total().max(10.0) + 5.0;
    let lane = lane_height(app.timeline_height, lanes);
    let size = vec2(
        (GUTTER + seconds as f32 * app.zoom).max(ui.available_width()),
        RULER + ADD_ROW + GAP + lanes as f32 * (lane + GAP) + CAPTION_LANE + GAP * 2.0,
    );
    let (rect, bg) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let lay = Layout {
        rect,
        zoom: app.zoom,
        lanes,
        lane,
    };
    app.timeline_view = Some(TimelineView {
        visible: ui.clip_rect(),
        time_origin: lay.x(0.0),
        zoom: app.zoom,
        lanes,
        lanes_top: lay.lanes_top(),
        lane_pitch: lay.lane + GAP,
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
    // Make sure every file with sound has its waveform on the way before drawing.
    for item in app.project.items().filter(|i| i.has_audio) {
        app.waveforms.request(&item.path, &app.tx, &app.ctx);
    }
    let (mut dragged, mut picked, mut fading, mut trimmed) = (None, None, None, None);
    for (ti, track) in app.project.tracks.iter().enumerate() {
        for item in &track.items {
            let selected = app.selection == Selection::Item(item.id);
            let wave = app.waveforms.get(&item.path);
            let resp = items::show(ui, &lay, ti, item, selected, wave);
            if item.kind == MediaKind::Audio {
                fading =
                    fading
                        .or(super::timeline_audio::fade_handles(ui, &lay, ti, item)
                            .map(|(out, secs)| (item.id, out, secs)));
            }
            trimmed = trimmed.or(super::timeline_trim::handles(ui, &lay, ti, item));
            if resp.clicked() || resp.drag_started() {
                picked = Some((item.id, ti));
            }
            if let Some(moved) = items::drag(ui, &lay, (ti, item), &resp, (app.zoom, app.magnet), &edges) {
                dragged = Some(moved);
            }
        }
    }
    if let Some((id, out, secs)) = fading {
        app.pause_for_edit();
        if let Some(it) = app.project.get_mut(id) {
            *(if out { &mut it.fade_out } else { &mut it.fade_in }) = secs;
        }
    }
    if let Some((id, front, mut t)) = trimmed {
        app.pause_for_edit();
        if app.magnet {
            // The edge pulls to the start or end of anything on any track, so lanes line up above one another.
            let others: Vec<f64> = edges.iter().filter(|(o, _)| *o != id).map(|(_, e)| *e).collect();
            t = items::snap(t, 0.0, &others, app.zoom);
        }
        app.project.trim_edge(id, front, t);
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
    super::timeline_captions::show(ui, app, &lay);
    scrub(app, &bg, &lay);
    let x = lay.x(app.playhead);
    p.line_segment(
        [pos2(x, rect.top()), pos2(x, rect.bottom())],
        Stroke::new(BORDER, ACCENT),
    );
    timeline_gutter::show(ui, app, &lay);
}
