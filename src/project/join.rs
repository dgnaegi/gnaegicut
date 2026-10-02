//! Transitions between two clips on the same track.
//!
//! A join belongs to the *later* clip and is applied against the clip before it. Most effects overlap the two
//! clips for the length of the transition (the later clip starts that much earlier); "Dip" does not overlap.
//! Rendering never changes the project: `resolved()` derives each clip's entrance and exit from the joins.

use super::join_effects::Join;
use super::kinds::Phase;
use super::{Item, Project};

const TOUCH: f64 = 0.05; // clips this close count as adjacent

impl Project {
    /// The clip directly before `id` on its track, if they touch or overlap.
    pub fn predecessor(&self, id: u64) -> Option<u64> {
        let (ti, ii) = self.find(id)?;
        let at = self.tracks[ti].items[ii].at;
        self.tracks[ti]
            .items
            .iter()
            .filter(|a| a.id != id && a.at < at && a.end_at() >= at - TOUCH)
            .max_by(|a, b| a.end_at().total_cmp(&b.end_at()))
            .map(|a| a.id)
    }

    /// Longest transition the two clips around `id` can carry.
    pub fn max_join_len(&self, id: u64) -> f32 {
        let (Some(b), Some(a)) = (self.get(id), self.predecessor(id).and_then(|a| self.get(a))) else {
            return 0.0;
        };
        (a.len().min(b.len()) - 0.1).max(0.0) as f32
    }

    /// Sets, changes or removes the transition into `id`, moving it (and everything after it on its track) so the
    /// overlap matches: overlapping effects pull the clip earlier, removing one puts it back.
    pub fn set_join(&mut self, id: u64, join: Option<Join>) {
        let Some(a) = self.predecessor(id).and_then(|a| self.get(a)).map(|a| a.end_at()) else {
            return;
        };
        let max = self.max_join_len(id);
        let join = join
            .map(|j| Join {
                len: j.len.min(max),
                ..j
            })
            .filter(|j| j.len > 0.05);
        let Some(item) = self.get_mut(id) else { return };
        let now = (a - item.at).max(0.0); // overlap currently in place
        let want = join
            .filter(|j| j.effect.links(j.len).overlaps)
            .map_or(0.0, |j| j.len as f64);
        item.join = join;
        let at = item.at;
        let shift = now - want; // positive: move later, negative: move earlier
        if let Some((ti, _)) = self.find(id) {
            for later in self.tracks[ti].items.iter_mut().filter(|i| i.at >= at - 1e-9) {
                later.at = (later.at + shift).max(0.0);
            }
        }
    }

    /// A copy where every clip knows how it enters and leaves because of its joins. Joins whose clips no longer
    /// touch or overlap are ignored. Idempotent.
    pub fn resolved(&self) -> Project {
        let mut p = self.clone();
        let ids: Vec<u64> = p.items().filter(|i| i.join.is_some()).map(|i| i.id).collect();
        for id in ids {
            let (Some(a_id), Some(join)) = (p.predecessor(id), p.get(id).and_then(|i| i.join)) else {
                continue;
            };
            let (a_end, b_at) = (p.get(a_id).map_or(0.0, Item::end_at), p.get(id).map_or(0.0, |i| i.at));
            let links = join.effect.links(join.len);
            let window = if links.overlaps {
                (a_end - b_at).min(join.len as f64)
            } else {
                join.len as f64
            };
            if links.overlaps && window <= TOUCH {
                continue; // dragged apart: no overlap left to transition across
            }
            let scale = (window as f32 / join.len).min(1.0);
            let sized = |ph: Phase| Phase {
                len: ph.len * scale,
                ..ph
            };
            if let Some(a) = p.get_mut(a_id) {
                a.link_out = Some(sized(links.out));
            }
            if let Some(b) = p.get_mut(id) {
                b.link_in = Some(sized(links.enter));
            }
        }
        p.linked = true;
        p
    }
}
