//! Searching, auditioning and placing free sounds.

use crate::app::{App, Event, Selection};
use crate::media::{probe::probe, stream::decode_file};
use crate::sounds::{self, Sound, Source};
use std::time::Duration;

const PREVIEW_SECS: u32 = 60;

impl App {
    /// Searches for the text in the search box: in one source when a suggestion of that kind was clicked, in both
    /// (`None`) for a typed search. The other group is cleared for a one-source search so the list matches the click.
    pub fn search_sounds(&mut self, scope: Option<Source>) {
        let query = self.sounds.query.trim().to_string();
        if query.is_empty() {
            return;
        }
        self.sounds.error = None;
        match scope {
            Some(Source::Effects) => self.sounds.music.clear(),
            Some(Source::Music) => self.sounds.effects.clear(),
            None => {}
        }
        for source in scope.map_or(vec![Source::Effects, Source::Music], |s| vec![s]) {
            self.sounds.loading += 1;
            let query = query.clone();
            self.spawn(move |tx| {
                let _ = tx.send(Event::SoundResults(source, sounds::search(&query, source)));
            });
        }
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
                self.sounds.loading = self.sounds.loading.saturating_sub(1);
                self.sounds.searched = true;
                let (list, error) = match result {
                    Ok(list) => (list, None),
                    Err(e) => (vec![], Some(e)),
                };
                match source {
                    Source::Effects => self.sounds.effects = list,
                    Source::Music => self.sounds.music = list,
                }
                self.sounds.error = self.sounds.error.take().or(error); // the first failure wins
            }
            Event::SoundPcm(id, pcm) => {
                self.sounds.busy.remove(&id);
                let mixer = self.audio.as_ref().map(|d| d.mixer());
                self.sounds.preview.play(mixer, pcm);
                self.sounds.playing = Some(id);
            }
            Event::SoundItem(id, item) => {
                self.sounds.busy.remove(&id);
                self.stop();
                self.project.register(&item);
                let (lane, time) = self.sounds.pending.remove(&id).unwrap_or((self.track, self.playhead));
                let ids = self.project.place_batch(vec![*item], lane, time, true);
                if let Some(&id) = ids.first() {
                    self.track = self.project.find(id).map_or(self.track, |(t, _)| t);
                    self.select(Selection::Item(id));
                }
            }
            Event::SoundFailed(id, message) => {
                self.sounds.busy.remove(&id);
                self.sounds.pending.remove(&id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    fn sound(id: &str) -> Sound {
        Sound {
            id: id.into(),
            title: id.into(),
            creator: "c".into(),
            license: "CC0 1.0".into(),
            secs: 1.0,
            url: String::new(),
            ext: "mp3".into(),
            credit: String::new(),
        }
    }

    #[test]
    fn effects_and_music_land_in_their_own_groups_and_loading_counts_down() {
        let mut app = App::new(Context::default());
        app.sounds.loading = 2; // a typed search asks both sources
        app.on_sound_event(Event::SoundResults(Source::Effects, Ok(vec![sound("whoosh")])));
        assert_eq!(app.sounds.loading, 1, "still waiting for the music answer");
        app.on_sound_event(Event::SoundResults(
            Source::Music,
            Ok(vec![sound("lofi"), sound("chill")]),
        ));
        assert_eq!(app.sounds.loading, 0);
        assert_eq!((app.sounds.effects.len(), app.sounds.music.len()), (1, 2));
        assert!(app.sounds.searched && app.sounds.error.is_none());
    }

    #[test]
    fn one_failing_source_does_not_hide_the_other() {
        let mut app = App::new(Context::default());
        app.sounds.loading = 2;
        app.on_sound_event(Event::SoundResults(Source::Effects, Err("offline".into())));
        app.on_sound_event(Event::SoundResults(Source::Music, Ok(vec![sound("lofi")])));
        assert_eq!(app.sounds.error.as_deref(), Some("offline"));
        assert_eq!(app.sounds.music.len(), 1, "music still shows");
        assert_eq!(app.sounds.loading, 0, "never goes below zero either");
        app.on_sound_event(Event::SoundResults(Source::Music, Ok(vec![])));
        assert_eq!(app.sounds.loading, 0);
    }

    #[test]
    fn a_blank_search_does_nothing_and_a_one_source_search_clears_the_other_group() {
        let mut app = App::new(Context::default());
        app.sounds.music = vec![sound("lofi")];
        app.search_sounds(None);
        assert_eq!(app.sounds.loading, 0, "empty query: no request");
        app.sounds.query = "whoosh".into();
        app.search_sounds(Some(Source::Effects));
        assert_eq!(app.sounds.loading, 1, "one request for one source");
        assert!(
            app.sounds.music.is_empty(),
            "the old music list would not match the whoosh the user clicked"
        );
    }
}
