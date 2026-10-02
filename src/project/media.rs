//! The project's library of source files. Items on the timeline reference these; deleting an item
//! does not delete its file from the library, so media can be placed again.

use super::{Item, Kind, Project};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MediaRef {
    pub path: String,
    pub name: String,
    pub kind: Kind,
    pub src_w: u32,
    pub src_h: u32,
    /// Duration in seconds for videos; 0 for images.
    pub len: f64,
    pub has_audio: bool,
    #[serde(default)]
    pub credit: Option<String>,
}

impl MediaRef {
    pub fn of(item: &Item) -> Self {
        let len = if item.kind.is_still() { 0.0 } else { item.src_len };
        Self {
            path: item.path.clone(),
            name: item.name.clone(),
            kind: item.kind,
            src_w: item.src_w,
            src_h: item.src_h,
            len,
            has_audio: item.has_audio,
            credit: item.credit.clone(),
        }
    }

    /// A fresh item for this file, trimmed to its full length and placed at 0.
    pub fn to_item(&self) -> Item {
        let size = (self.src_w, self.src_h);
        let mut item = Item::new(
            self.path.clone(),
            self.name.clone(),
            self.kind,
            size,
            self.len,
            self.has_audio,
        );
        item.credit = self.credit.clone();
        item
    }
}

impl Project {
    /// Adds the item's source file to the library unless it is already there. Text is not a file.
    pub fn register(&mut self, item: &Item) {
        if item.kind != Kind::Text && !self.media.iter().any(|m| m.path == item.path) {
            self.media.push(MediaRef::of(item));
        }
    }

    /// Makes sure every file used on the timeline is in the library (older projects had no library).
    pub fn sync_media(&mut self) {
        let items: Vec<Item> = self.items().cloned().collect();
        items.iter().for_each(|i| self.register(i));
    }

    /// How many items on the timeline play this file.
    pub fn usage(&self, path: &str) -> usize {
        self.items().filter(|i| i.path == path).count()
    }

    /// Attribution lines for every downloaded sound in the library, without duplicates.
    pub fn credits(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.media.iter().filter_map(|m| m.credit.clone()).collect();
        lines.dedup();
        lines.sort();
        lines.dedup();
        lines
    }

    /// The earliest item on the timeline that plays this file.
    pub fn first_use(&self, path: &str) -> Option<&Item> {
        self.items()
            .filter(|i| i.path == path)
            .min_by(|a, b| a.at.total_cmp(&b.at))
    }
}
