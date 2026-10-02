//! What the sound search panel shows and is waiting for.

use crate::sound_preview::SoundPreview;
use crate::sounds::{Sound, Source};
use std::collections::HashSet;

/// Which list the library panel shows.
#[derive(Clone, Copy, PartialEq)]
pub enum LibraryTab {
    Files,
    Effects,
    Music,
}

impl LibraryTab {
    pub fn source(self) -> Option<Source> {
        match self {
            LibraryTab::Files => None,
            LibraryTab::Effects => Some(Source::Effects),
            LibraryTab::Music => Some(Source::Music),
        }
    }
}

#[derive(Default)]
pub struct SoundsUi {
    pub query: String,
    pub results: Vec<Sound>,
    /// Which source the shown results came from, so switching tabs does not show the wrong list.
    pub results_for: Source,
    pub loading: bool,
    /// True once any search has answered, so "no results" is not shown before the first search.
    pub searched: bool,
    pub error: Option<String>,
    /// Sounds being downloaded right now, by id.
    pub busy: HashSet<String>,
    pub playing: Option<String>,
    pub preview: SoundPreview,
}

impl SoundsUi {
    pub fn stop_preview(&mut self) {
        self.preview.stop();
        self.playing = None;
    }
}
