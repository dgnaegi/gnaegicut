//! What the sound search panel shows and is waiting for.

use crate::sound_preview::SoundPreview;
use crate::sounds::Sound;
use std::collections::HashSet;

/// Which list the library panel shows.
#[derive(Clone, Copy, PartialEq)]
pub enum LibraryTab {
    Files,
    Sounds,
}

#[derive(Default)]
pub struct SoundsUi {
    pub query: String,
    /// Results of the last search, kept apart: sound effects and music are shown as two groups.
    pub effects: Vec<Sound>,
    pub music: Vec<Sound>,
    /// How many searches are still waiting for an answer (a free search asks both sources).
    pub loading: usize,
    /// True once any search has answered, so "no results" is not shown before the first search.
    pub searched: bool,
    pub error: Option<String>,
    /// Sounds being downloaded right now, by id.
    pub busy: HashSet<String>,
    /// Where each downloading sound was dropped: (track, time). Sounds without an entry go to the playhead.
    pub pending: std::collections::HashMap<String, (usize, f64)>,
    pub playing: Option<String>,
    pub preview: SoundPreview,
}

impl SoundsUi {
    pub fn stop_preview(&mut self) {
        self.preview.stop();
        self.playing = None;
    }
}
