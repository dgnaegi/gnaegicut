//! The edit itself: plain data plus the pure operations on it. No UI, no ffmpeg.

mod aspect;
mod caption;
mod edit;
mod item;
mod item_audio;
mod item_fx;
mod item_geometry;
mod item_motion;
mod join;
mod join_effects;
mod kinds;
mod media;
mod place;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_join;
#[cfg(test)]
mod tests_music;
#[cfg(test)]
mod tests_place;

pub use aspect::Aspect;
pub use caption::{Caption, CaptionLayout, CaptionStyle};
pub use item::Item;
pub use join_effects::{Join, JoinEffect};
pub use kinds::{Kind, Look, Transition, ZoomEffect};
pub use media::MediaRef;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};

/// A layer. Higher tracks draw on top of lower ones.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Track {
    pub items: Vec<Item>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
    pub tracks: Vec<Track>,
    /// Every source file imported, whether or not it is on the timeline.
    #[serde(default)]
    pub media: Vec<MediaRef>,
    pub captions: Vec<Caption>,
    pub caption_style: CaptionStyle,
    pub caption_layout: CaptionLayout,
    pub aspect: Aspect,
    next_id: u64,
    /// True once `resolved()` has derived entrances and exits from the joins.
    #[serde(skip)]
    pub(crate) linked: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            tracks: vec![Track::default()],
            media: vec![],
            captions: vec![],
            caption_style: CaptionStyle::Pop,
            caption_layout: CaptionLayout::default(),
            aspect: Aspect::Vertical,
            next_id: 1,
            linked: false,
        }
    }
}

impl Project {
    /// A copy for background jobs, so the UI can keep editing meanwhile.
    pub fn snapshot(&self) -> Project {
        self.clone()
    }

    /// All items, bottom layer first (the order they are composited in).
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.tracks.iter().flat_map(|t| t.items.iter())
    }

    /// The times where something starts or ends: 0, every item's start and end (so the seam between two clips is one
    /// entry) and every caption's start and end. Sorted, without duplicates. The playhead and dragged items snap to these.
    pub fn edges(&self) -> Vec<f64> {
        let items = self.items().flat_map(|i| [i.at, i.end_at()]);
        let captions = self.captions.iter().flat_map(|c| [c.start, c.end]);
        let mut all: Vec<f64> = std::iter::once(0.0).chain(items).chain(captions).collect();
        all.sort_by(f64::total_cmp);
        all.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
        all
    }

    /// All caption texts in order, one caption per line: what to copy into a post or a description.
    pub fn transcript(&self) -> String {
        let lines: Vec<&str> = self
            .captions
            .iter()
            .map(|c| c.text.trim())
            .filter(|t| !t.is_empty())
            .collect();
        lines.join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.items().next().is_none()
    }

    pub fn total(&self) -> f64 {
        self.items().fold(0.0, |end, i| end.max(i.end_at()))
    }

    pub fn track_end(&self, track: usize) -> f64 {
        self.tracks
            .get(track)
            .map_or(0.0, |t| t.items.iter().fold(0.0, |end, i| end.max(i.end_at())))
    }

    /// Adds `item` to `track` (creating tracks as needed) and returns its id.
    pub fn add(&mut self, track: usize, mut item: Item) -> u64 {
        while self.tracks.len() <= track {
            self.tracks.push(Track::default());
        }
        item.id = self.next_id;
        self.next_id += 1;
        let id = item.id;
        self.tracks[track].items.push(item);
        id
    }

    pub fn find(&self, id: u64) -> Option<(usize, usize)> {
        self.tracks
            .iter()
            .enumerate()
            .find_map(|(t, tr)| tr.items.iter().position(|i| i.id == id).map(|i| (t, i)))
    }

    pub fn get(&self, id: u64) -> Option<&Item> {
        self.find(id).map(|(t, i)| &self.tracks[t].items[i])
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Item> {
        let (t, i) = self.find(id)?;
        Some(&mut self.tracks[t].items[i])
    }

    pub fn remove(&mut self, id: u64) {
        if let Some((t, i)) = self.find(id) {
            self.tracks[t].items.remove(i);
        }
    }

    /// Changes when anything that decides *when* speech happens changes, which is what captions depend on.
    pub fn timing_fingerprint(&self) -> u64 {
        let mut h = DefaultHasher::new();
        for i in self.items().filter(|i| i.has_audio) {
            (i.id, i.at.to_bits(), i.start.to_bits(), i.end.to_bits()).hash(&mut h);
        }
        h.finish()
    }

    /// Changes whenever anything affecting the picture or sound changes.
    pub fn fingerprint(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.aspect.hash(&mut h);
        let c = &self.caption_layout;
        (
            c.x.to_bits(),
            c.y.to_bits(),
            c.size.to_bits(),
            c.bold,
            c.auto_y,
            &c.family,
        )
            .hash(&mut h);
        (self.caption_style as u8, self.captions.len()).hash(&mut h);
        for cap in &self.captions {
            (cap.start.to_bits(), cap.end.to_bits(), &cap.text).hash(&mut h);
        }
        for item in self.items() {
            item.hash_into(&mut h);
        }
        h.finish()
    }
}
