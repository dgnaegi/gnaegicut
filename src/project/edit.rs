//! Operations that restructure the timeline.

use super::{Caption, Item, Project, Track};

impl Project {
    /// Splits the item at timeline time `t`. Returns the id of the right-hand piece, or None
    /// if `t` is outside the item or too close to an edge.
    pub fn split(&mut self, id: u64, t: f64) -> Option<u64> {
        let (ti, ii) = self.find(id)?;
        let item = &self.tracks[ti].items[ii];
        let local = t - item.at;
        if local < 0.1 || local > item.len() - 0.1 {
            return None;
        }
        let mut right = item.clone();
        right.at = t;
        right.start += local;
        self.tracks[ti].items[ii].end = right.start;
        right.id = self.next_id;
        self.next_id += 1;
        let new_id = right.id;
        self.tracks[ti].items.insert(ii + 1, right);
        Some(new_id)
    }

    /// Removes a track if it is empty and not the last one. Returns whether it was removed.
    pub fn remove_empty_track(&mut self, index: usize) -> bool {
        let removable = self.tracks.len() > 1 && self.tracks.get(index).is_some_and(|t| t.items.is_empty());
        if removable {
            self.tracks.remove(index);
        }
        removable
    }

    /// Moves an item to `track` at timeline position `at` (clamped to >= 0).
    pub fn place(&mut self, id: u64, track: usize, at: f64) {
        let Some((ti, ii)) = self.find(id) else { return };
        let mut item = self.tracks[ti].items.remove(ii);
        item.at = at.max(0.0);
        let track = track.min(self.tracks.len() - 1);
        self.tracks[track].items.push(item);
        self.tracks[track].items.sort_by(|a, b| a.at.total_cmp(&b.at));
    }

    /// Removes an item and pulls everything after it on the same track left, closing the gap.
    pub fn ripple_remove(&mut self, id: u64) {
        let Some((ti, _)) = self.find(id) else { return };
        let Some((at, len)) = self.get(id).map(|i| (i.at, i.len())) else {
            return;
        };
        self.remove(id);
        for item in self.tracks[ti].items.iter_mut().filter(|i| i.at >= at) {
            item.at = (item.at - len).max(0.0);
        }
    }

    /// The part of the edit that plays from timeline time `t` onwards, re-based so it starts at 0.
    pub fn tail_from(&self, t: f64) -> Project {
        // Joins are resolved on the whole edit first: the overlap must be measured before the front is cut off.
        let mut rest = if self.linked { self.clone() } else { self.resolved() };
        for track in &mut rest.tracks {
            *track = Track {
                items: track
                    .items
                    .iter()
                    .filter(|i| i.end_at() > t + 0.05)
                    .map(|i| cut_front(i, t))
                    .collect(),
            };
        }
        rest.captions = self
            .captions
            .iter()
            .filter(|c| c.end > t)
            .map(|c| Caption {
                start: (c.start - t).max(0.0),
                end: c.end - t,
                text: c.text.clone(),
            })
            .collect();
        rest
    }

    /// The topmost item playing at `t` for which `hit` is true.
    pub fn topmost_at(&self, t: f64, hit: impl Fn(&Item) -> bool) -> Option<u64> {
        let playing = |i: &&Item| i.at <= t && t < i.end_at();
        self.items().filter(playing).filter(|i| hit(i)).last().map(|i| i.id)
    }
}

fn cut_front(item: &Item, t: f64) -> Item {
    let mut i = item.clone();
    if i.at < t {
        let cut = t - i.at;
        i.start += cut;
        i.cut += cut;
        i.at = 0.0;
    } else {
        i.at -= t;
    }
    i
}
