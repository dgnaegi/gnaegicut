//! Searching, auditioning and placing free sounds.

use crate::app::{App, Event, Selection};
use crate::media::{probe::probe, stream::decode_file};
use crate::sounds::{self, Sound};
use std::time::Duration;

const PREVIEW_SECS: u32 = 60;

impl App {
    pub fn search_sounds(&mut self) {
        let (Some(source), query) = (self.library_tab.source(), self.sounds.query.trim().to_string()) else {
            return;
        };
        if query.is_empty() {
            return;
        }
        self.sounds.loading = true;
        self.sounds.error = None;
        self.spawn(move |tx| {
            let _ = tx.send(Event::SoundResults(source, sounds::search(&query, source)));
        });
    }

    /// Plays a sound, or stops it if it is the one already playing.
    pub fn preview_sound(&mut self, sound: Sound) {
        let was_playing = self.sounds.playing.as_ref() == Some(&sound.id);
        self.sounds.stop_preview();
        if was_playing || !self.sounds.busy.insert(sound.id.clone()) {
            return;
        }
        self.stop();
        self.spawn(move |tx| {
            let result = sounds::download(&sound).and_then(|path| decode_file(&path, PREVIEW_SECS));
            let _ = tx.send(match result {
                Ok(pcm) => Event::SoundPcm(sound.id, pcm),
                Err(e) => Event::SoundFailed(sound.id, e),
            });
        });
    }

    /// Downloads a sound and puts it on the timeline at the playhead, on the first free track.
    pub fn add_sound(&mut self, sound: Sound) {
        if !self.sounds.busy.insert(sound.id.clone()) {
            return;
        }
        self.spawn(move |tx| {
            let result = sounds::download(&sound).and_then(|path| probe(&path));
            let _ = tx.send(match result {
                Ok(mut item) => {
                    item.name = sound.title.chars().take(40).collect();
                    item.credit = Some(sound.credit.clone());
                    Event::SoundItem(sound.id, Box::new(item))
                }
                Err(e) => Event::SoundFailed(sound.id, e),
            });
        });
    }

    pub fn copy_credits(&mut self) {
        let credits = self.project.credits();
        self.ctx.copy_text(credits.join("\n"));
        self.status = format!("Copied {} credit lines", credits.len());
    }

    pub(crate) fn on_sound_event(&mut self, event: Event) {
        match event {
            Event::SoundResults(source, result) => {
                self.sounds.loading = false;
                self.sounds.searched = true;
                self.sounds.results_for = source;
                match result {
                    Ok(list) => (self.sounds.results, self.sounds.error) = (list, None),
                    Err(e) => (self.sounds.results, self.sounds.error) = (vec![], Some(e)),
                }
            }
            Event::SoundPcm(id, pcm) => {
                self.sounds.busy.remove(&id);
                self.sounds.preview.play(pcm);
                self.sounds.playing = Some(id);
            }
            Event::SoundItem(id, item) => {
                self.sounds.busy.remove(&id);
                self.stop();
                self.project.register(&item);
                let ids = self.project.place_batch(vec![*item], self.track, self.playhead, true);
                if let Some(&id) = ids.first() {
                    self.track = self.project.find(id).map_or(self.track, |(t, _)| t);
                    self.select(Selection::Item(id));
                }
            }
            Event::SoundFailed(id, message) => {
                self.sounds.busy.remove(&id);
                self.status = message;
            }
            _ => {}
        }
    }

    /// Clears the "playing" mark once a preview has finished by itself.
    pub(crate) fn watch_sound_preview(&mut self) {
        if self.sounds.playing.is_some() {
            if !self.sounds.preview.is_playing() {
                self.sounds.playing = None;
            }
            self.ctx.request_repaint_after(Duration::from_millis(250));
        }
    }
}
