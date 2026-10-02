//! Where dropped or dragged media lands: onto the timeline lane under the pointer (at the time under it),
//! or, anywhere else, on the active track. Several files stack as layers or follow one another.

use crate::app::{App, Selection};
use crate::media::probe::probe;
use crate::project::Item;
use crate::ui::snap;
use eframe::egui::{DragAndDrop, Pos2, Rect};
use std::path::PathBuf;

/// Dragging a library entry (by its index) onto the timeline.
pub struct LibraryDrag(pub usize);

/// Enough of the timeline's last layout to turn a pointer position into a lane and a time.
#[derive(Clone, Copy)]
pub struct TimelineView {
    pub visible: Rect,
    pub time_origin: f32,
    pub zoom: f32,
    pub lanes: usize,
    pub lanes_top: f32,
    pub lane_pitch: f32,
}

impl TimelineView {
    pub fn contains(&self, p: Pos2) -> bool {
        self.visible.contains(p)
    }

    /// The track under `y`. Above the top lane (the ruler and the add-track bar) means "a new track on top".
    pub fn lane_at(&self, y: f32) -> usize {
        if y < self.lanes_top {
            return self.lanes;
        }
        let row = ((y - self.lanes_top) / self.lane_pitch).floor() as usize;
        self.lanes - 1 - row.min(self.lanes - 1)
    }

    pub fn time_at(&self, x: f32) -> f64 {
        (((x - self.time_origin) / self.zoom) as f64).max(0.0)
    }
}

impl App {
    /// The lane and time a drop at `pos` would use, if `pos` is over the timeline.
    pub fn drop_target(&self, pos: Option<Pos2>) -> Option<(usize, f64)> {
        let (view, pos) = (self.timeline_view?, pos?);
        if !view.contains(pos) {
            return None;
        }
        let mut time = view.time_at(pos.x);
        if self.magnet {
            let edges: Vec<f64> = [0.0, self.playhead]
                .into_iter()
                .chain(self.project.items().flat_map(|i| [i.at, i.end_at()]))
                .collect();
            time = snap(time, 0.0, &edges, view.zoom);
        }
        Some((view.lane_at(pos.y), time))
    }

    /// What is being dragged over the window right now (files from the OS, or a library entry) and where.
    pub fn drag_hover(&self) -> Option<(usize, Pos2)> {
        let files = self.ctx.input(|i| i.raw.hovered_files.len());
        let count = if files > 0 {
            files
        } else {
            usize::from(DragAndDrop::has_payload_of_type::<LibraryDrag>(&self.ctx))
        };
        (count > 0)
            .then(|| self.ctx.pointer_latest_pos().map(|p| (count, p)))
            .flatten()
    }

    /// Places new items. Over the timeline they go to the lane and time under `pos`; elsewhere to the active track,
    /// at the playhead when stacking or after the track's last item when sequencing.
    pub fn drop_items(&mut self, items: Vec<Item>, pos: Option<Pos2>) {
        if items.is_empty() {
            return;
        }
        self.stop();
        let (lane, time) = self.drop_target(pos).unwrap_or_else(|| {
            (
                self.track,
                if self.stack_drops {
                    self.playhead
                } else {
                    self.project.track_end(self.track)
                },
            )
        });
        let ids = self.project.place_batch(items, lane, time, self.stack_drops);
        if let Some(&top) = ids.last() {
            self.track = self.project.find(top).map_or(lane, |(track, _)| track);
            self.select(Selection::Item(top));
        }
    }

    /// Probes media, adds it to the library and places it. Used by the import dialog, OS drops and the command line.
    pub fn add_files(&mut self, files: &[PathBuf], pos: Option<Pos2>) {
        let mut items = vec![];
        for f in files {
            match probe(f) {
                Ok(item) => {
                    self.project.register(&item);
                    items.push(item);
                }
                Err(e) => self.status = e,
            }
        }
        self.drop_items(items, pos);
    }

    /// Places a library file on the timeline again.
    pub fn add_from_library(&mut self, index: usize, pos: Option<Pos2>) {
        if let Some(item) = self.project.media.get(index).map(|m| m.to_item()) {
            self.drop_items(vec![item], pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> TimelineView {
        TimelineView {
            visible: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(800.0, 300.0)),
            time_origin: 100.0,
            zoom: 50.0,
            lanes: 3,
            lanes_top: 60.0,
            lane_pitch: 48.0,
        }
    }

    #[test]
    fn the_top_row_is_the_highest_track_and_above_it_is_a_new_one() {
        let v = view();
        assert_eq!(v.lane_at(70.0), 2, "first row = top track");
        assert_eq!(v.lane_at(60.0 + 48.0 + 5.0), 1);
        assert_eq!(v.lane_at(60.0 + 48.0 * 2.0 + 5.0), 0);
        assert_eq!(v.lane_at(10.0), 3, "over the ruler: a new track");
        assert_eq!(v.lane_at(290.0), 0, "below the lanes stays on the lowest track");
    }

    #[test]
    fn time_follows_the_zoom_and_never_goes_negative() {
        let v = view();
        assert!((v.time_at(100.0 + 125.0) - 2.5).abs() < 1e-9);
        assert_eq!(v.time_at(20.0), 0.0);
    }
}
