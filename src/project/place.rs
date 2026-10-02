//! Putting several new items onto the timeline at once: stacked as layers, or one after another.

use super::{Item, Project};

impl Project {
    /// The first track at or above `from` with nothing playing during `[at, at + len)`. Tracks past the end
    /// count as free, so callers get a new track.
    pub fn free_track(&self, from: usize, at: f64, len: f64) -> usize {
        let free = |t: usize| {
            self.tracks.get(t).is_none_or(|track| {
                track
                    .items
                    .iter()
                    .all(|i| i.end_at() <= at + 1e-6 || i.at >= at + len - 1e-6)
            })
        };
        (from..).find(|&t| free(t)).unwrap_or(from)
    }

    /// Places `items` starting at `time`. With `stack`, each lands on its own track going upward from `track`
    /// (skipping occupied ones) and all start at `time`; otherwise they follow one another on `track`.
    /// Small overlays on upper tracks start at half size. Returns the new ids in order.
    pub fn place_batch(&mut self, items: Vec<Item>, track: usize, time: f64, stack: bool) -> Vec<u64> {
        let (mut ids, mut at, mut lane) = (vec![], time.max(0.0), track);
        for mut item in items {
            let target = if stack {
                self.free_track(lane, at, item.len())
            } else {
                track
            };
            item.at = at;
            if target > 0 && item.kind.is_still() {
                item.scale = 0.5;
            }
            if !stack {
                at = item.end_at();
            }
            ids.push(self.add(target, item));
            lane = target + 1;
        }
        ids
    }
}
