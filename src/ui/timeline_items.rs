//! Geometry of the timeline and the draggable item blocks.

use crate::project::{Item, Kind};
use crate::theme::{ACCENT, BLACK, BORDER, MUTED, WHITE, bold};
use eframe::egui::{Align2, Color32, CursorIcon, Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

pub const GUTTER: f32 = 56.0;
pub const RULER: f32 = 22.0;
pub const LANE: f32 = 44.0;
pub const GAP: f32 = 4.0;
pub const ADD_ROW: f32 = 26.0; // the "+ new track" bar above the top lane
const SNAP_PX: f32 = 8.0;

#[derive(Clone, Copy)]
pub struct Layout {
    pub rect: Rect,
    pub zoom: f32,
    pub lanes: usize,
}

impl Layout {
    pub fn x(&self, t: f64) -> f32 {
        self.rect.left() + GUTTER + t as f32 * self.zoom
    }

    pub fn time(&self, x: f32) -> f64 {
        ((x - self.rect.left() - GUTTER) / self.zoom) as f64
    }

    /// Higher tracks are drawn higher up, like layers.
    pub fn lane_top(&self, track: usize) -> f32 {
        self.lanes_top() + (self.lanes - 1 - track) as f32 * (LANE + GAP)
    }

    /// Top of the highest lane: below the ruler and the add-track bar.
    pub fn lanes_top(&self) -> f32 {
        self.rect.top() + RULER + ADD_ROW + GAP
    }

    pub fn track_at(&self, y: f32) -> usize {
        let row = ((y - self.lanes_top()) / (LANE + GAP)).floor().max(0.0) as usize;
        (self.lanes - 1).saturating_sub(row.min(self.lanes - 1))
    }

    pub fn captions_top(&self) -> f32 {
        self.lanes_top() + self.lanes as f32 * (LANE + GAP)
    }

    pub fn item_rect(&self, track: usize, item: &Item) -> Rect {
        let top = self.lane_top(track);
        Rect::from_min_max(pos2(self.x(item.at), top), pos2(self.x(item.end_at()), top + LANE))
    }
}

/// Paints one item block and returns its click/drag response.
pub fn show(ui: &mut Ui, lay: &Layout, track: usize, item: &Item, selected: bool) -> Response {
    let rect = lay.item_rect(track, item);
    let resp = ui.interact(rect, Id::new(("item", item.id)), Sense::click_and_drag());
    let (fill, ink): (Color32, Color32) = match (selected, item.kind) {
        (true, _) => (ACCENT, WHITE),
        (_, Kind::Video) => (BLACK, WHITE),
        (_, Kind::Image) => (MUTED, BLACK),
        (_, Kind::Text) => (WHITE, BLACK),
        (_, Kind::Audio) => (MUTED, BLACK),
    };
    let p = ui.painter_at(lay.rect);
    p.rect_filled(rect, 0.0, fill);
    p.rect_stroke(
        rect,
        0.0,
        Stroke::new(BORDER, if selected { WHITE } else { BLACK }),
        StrokeKind::Inside,
    );
    let tag = match item.kind {
        Kind::Video => "",
        Kind::Image => "IMG ",
        Kind::Text => "TXT ",
        Kind::Audio => "♪ ",
    };
    let label = format!("{tag}{}", item.name.to_uppercase());
    if item.kind == Kind::Audio {
        p.rect_filled(
            Rect::from_min_max(rect.left_bottom() - vec2(0.0, 6.0), rect.right_bottom()),
            0.0,
            if selected { WHITE } else { ACCENT },
        );
    }
    p.with_clip_rect(rect).text(
        rect.left_top() + [8.0, 6.0].into(),
        Align2::LEFT_TOP,
        label,
        bold(11.0),
        ink,
    );
    resp.on_hover_cursor(CursorIcon::Grab)
}

/// Pulls `at` to a nearby edge (0, the playhead, or another item's start/end) when within a few pixels.
pub fn snap(at: f64, len: f64, edges: &[f64], zoom: f32) -> f64 {
    let limit = (SNAP_PX / zoom) as f64;
    edges
        .iter()
        .flat_map(|&e| [(e - at, e), (e - (at + len), e - len)])
        .filter(|(d, _)| d.abs() <= limit)
        .min_by(|a, b| a.0.abs().total_cmp(&b.0.abs()))
        .map_or(at, |(_, snapped)| snapped)
}

/// Handles dragging an item block; returns `(id, target track, new start)` while it moves.
/// The unsnapped position is tracked separately, so snapping never swallows slow drags.
pub fn drag(
    ui: &mut Ui,
    lay: &Layout,
    (track, item): (usize, &Item),
    resp: &Response,
    (zoom, magnet): (f32, bool),
    edges: &[(u64, f64)],
) -> Option<(u64, usize, f64)> {
    let key = Id::new(("drag-at", item.id));
    if resp.drag_started() {
        ui.data_mut(|d| d.insert_temp(key, item.at));
    }
    if !resp.dragged() {
        return None;
    }
    let raw: f64 = ui.data(|d| d.get_temp(key)).unwrap_or(item.at) + (resp.drag_delta().x / zoom) as f64;
    ui.data_mut(|d| d.insert_temp(key, raw));
    let others: Vec<f64> = edges
        .iter()
        .filter(|(owner, _)| *owner != item.id)
        .map(|(_, t)| *t)
        .collect();
    let at = if magnet {
        snap(raw, item.len(), &others, zoom)
    } else {
        raw
    };
    let target = resp.interact_pointer_pos().map_or(track, |p| lay.track_at(p.y));
    Some((item.id, target, at))
}

#[cfg(test)]
mod tests {
    use super::snap;

    #[test]
    fn snaps_start_or_end_to_the_nearest_edge_within_reach() {
        let edges = [0.0, 4.0];
        assert_eq!(snap(4.05, 2.0, &edges, 100.0), 4.0, "start meets an edge");
        assert_eq!(snap(1.96, 2.0, &edges, 100.0), 2.0, "end meets an edge");
        assert_eq!(snap(2.0, 1.0, &edges, 100.0), 2.0, "nothing near: unchanged");
    }

    #[test]
    fn reach_shrinks_when_zoomed_in() {
        assert_eq!(snap(4.05, 2.0, &[4.0], 200.0), 4.05, "8px at 200px/s is only 0.04s");
    }
}
