//! Timeline tools that change the selected clip: freeze frame and reverse.

use crate::app::{App, Selection};
use crate::media::{frame, probe::probe};

impl App {
    /// Holds the picture at the playhead for two seconds.
    pub fn freeze(&mut self) {
        self.stop();
        let Some(id) = self.selected_item() else {
            self.status = "Select a video clip first".into();
            return;
        };
        let (playhead, path) = (
            self.playhead,
            self.project.get(id).map(|i| i.path.clone()).unwrap_or_default(),
        );
        let out = std::env::temp_dir().join(format!("gnaegicut-freeze-{id}-{}.png", (playhead * 1000.0) as i64));
        let made = self.project.freeze(id, playhead, |at| {
            frame::still(&path, at, &out).ok()?;
            probe(&out).ok()
        });
        match made {
            Some(still) => self.select(Selection::Item(still)),
            None => self.status = "Put the playhead inside the selected video clip".into(),
        }
    }

    /// Plays the selected video or sound backwards (again: forwards).
    pub fn reverse(&mut self) {
        self.pause_for_edit();
        let item = self.selected_item().and_then(|id| self.project.get_mut(id));
        match item.filter(|i| !i.kind.is_still()) {
            Some(i) => i.reversed = !i.reversed,
            None => self.status = "Select a video or sound clip first".into(),
        }
    }
}
