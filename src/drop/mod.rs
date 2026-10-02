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

/// Dragging a sound search result onto the timeline.
pub struct SoundDrag(pub crate::sounds::Sound);

/// What the user was dragging when they let go.
pub enum Dragged {
    Library(usize),
    Sound(crate::sounds::Sound),
}

/// Takes whatever is being dragged. egui's `take_payload::<T>` empties the slot even when the type is not `T`, so
/// asking for one kind first would silently throw the other away; peek by type, then clear.
pub fn take_dragged(ctx: &eframe::egui::Context) -> Option<Dragged> {
    let found = match DragAndDrop::payload::<LibraryDrag>(ctx) {
        Some(drag) => Some(Dragged::Library(drag.0)),
        None => DragAndDrop::payload::<SoundDrag>(ctx).map(|drag| Dragged::Sound(drag.0.clone())),
    };
    if found.is_some() {
        DragAndDrop::clear_payload(ctx);
    }
    found
}

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
            let edges: Vec<f64> = self.project.edges().into_iter().chain([self.playhead]).collect();
            time = snap(time, 0.0, &edges, view.zoom);
        }
        Some((view.lane_at(pos.y), time))
    }

    /// What is being dragged over the window right now (files from the OS, or a library entry) and where.
    pub fn drag_hover(&self) -> Option<(usize, Pos2)> {
        let files = self.ctx.input(|i| i.raw.hovered_files.len());
        let dragging_one = DragAndDrop::has_payload_of_type::<LibraryDrag>(&self.ctx)
            || DragAndDrop::has_payload_of_type::<SoundDrag>(&self.ctx);
        let count = if files > 0 { files } else { usize::from(dragging_one) };
        (count > 0)
            .then(|| self.ctx.pointer_latest_pos().map(|p| (count, p)))
            .flatten()
    }

    /// Whether a drop at `pos` makes layers (true) or a row of clips (false). The place decides: on an existing lane
    /// clips follow one another along it; in the new-track area above the lanes, or on the preview, every file gets
    /// a track of its own and lies on top of the others.
    pub fn drops_stack(&self, pos: Option<Pos2>) -> bool {
        self.drop_target(pos)
            .is_none_or(|(lane, _)| lane >= self.project.tracks.len())
    }

    /// Places new items where they were dropped (see `drops_stack`). Away from the timeline they start at the playhead.
    pub fn drop_items(&mut self, items: Vec<Item>, pos: Option<Pos2>) {
        if items.is_empty() {
            return;
        }
        self.stop();
        let stack = self.drops_stack(pos);
        let (lane, time) = self.drop_target(pos).unwrap_or((self.track, self.playhead));
        let ids = self.project.place_batch(items, lane, time, stack);
        self.select_placed(ids, lane);
    }

    /// Several files one after another at the end of the active track: the import dialog and the command line.
    pub fn add_files_in_sequence(&mut self, files: &[PathBuf]) {
        let items = self.load_files(files);
        if items.is_empty() {
            return;
        }
        self.stop();
        let (lane, time) = (self.track, self.project.track_end(self.track));
        let ids = self.project.place_batch(items, lane, time, false);
        self.select_placed(ids, lane);
    }

    fn select_placed(&mut self, ids: Vec<u64>, fallback_lane: usize) {
        if let Some(&top) = ids.last() {
            self.track = self.project.find(top).map_or(fallback_lane, |(track, _)| track);
            self.select(Selection::Item(top));
        }
    }

    /// Probes media and adds it to the library; failures go to the status line.
    fn load_files(&mut self, files: &[PathBuf]) -> Vec<Item> {
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
        items
    }

    /// Probes media, adds it to the library and places it where it was dropped.
    pub fn add_files(&mut self, files: &[PathBuf], pos: Option<Pos2>) {
        let items = self.load_files(files);
        self.drop_items(items, pos);
    }

    /// A sound dropped from the search results: remember where, then download it and place it.
    pub fn drop_sound(&mut self, sound: crate::sounds::Sound, pos: Option<Pos2>) {
        let target = self.drop_target(pos).unwrap_or((self.track, self.playhead));
        self.sounds.pending.insert(sound.id.clone(), target);
        self.add_sound(sound);
    }

    /// Places a library file on the timeline again.
    pub fn add_from_library(&mut self, index: usize, pos: Option<Pos2>) {
        if let Some(item) = self.project.media.get(index).map(|m| m.to_item()) {
            self.drop_items(vec![item], pos);
        }
    }
}

#[cfg(test)]
mod tests;
