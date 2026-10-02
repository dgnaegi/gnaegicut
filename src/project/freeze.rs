//! Freeze frame: the picture stops for a moment while the clip waits.

use super::{Item, Project};

pub const FREEZE_SECS: f64 = 2.0;

impl Project {
    /// Holds the frame at timeline time `t` of item `id` for `FREEZE_SECS`: the clip is split there, a still is
    /// inserted into the gap and everything after it on the track moves later. `make` turns the source time of the
    /// frame into the still (an image item); if it fails nothing changes. Returns the id of the still.
    pub fn freeze(&mut self, id: u64, t: f64, make: impl FnOnce(f64) -> Option<Item>) -> Option<u64> {
        let item = self.get(id).filter(|i| i.kind == super::Kind::Video)?;
        let local = t - item.at;
        if local < 0.1 || local > item.len() - 0.1 {
            return None;
        }
        let source = if item.reversed {
            item.end - local
        } else {
            item.start + local
        };
        let (x, y, scale, rotation) = (item.x, item.y, item.scale, item.rotation);
        let mut still = make(source)?;
        self.split(id, t)?;
        let (ti, _) = self.find(id)?;
        for later in self.tracks[ti].items.iter_mut().filter(|i| i.id != id && i.at >= t) {
            later.at += FREEZE_SECS;
        }
        (still.at, still.start, still.end) = (t, 0.0, FREEZE_SECS);
        (still.x, still.y, still.scale, still.rotation) = (x, y, scale, rotation);
        still.id = self.next_id;
        self.next_id += 1;
        let new_id = still.id;
        self.tracks[ti].items.push(still);
        self.tracks[ti].items.sort_by(|a, b| a.at.total_cmp(&b.at));
        Some(new_id)
    }
}
