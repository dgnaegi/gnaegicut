//! User-triggered operations. UI code calls these; they hold no layout.

use crate::app::{App, Event, Selection, Tab};
use crate::fonts::DEFAULT_FONT;
use crate::media::{export::export, whisper};
use crate::player::Player;
use crate::project::{Item, Kind};
use crate::text;
use std::sync::mpsc::Sender;

impl App {
    /// Runs `job` on a worker thread and wakes the UI when it finishes.
    pub(crate) fn spawn(&self, job: impl FnOnce(&Sender<Event>) + Send + 'static) {
        let (tx, ctx) = (self.tx.clone(), self.ctx.clone());
        std::thread::spawn(move || {
            job(&tx);
            ctx.request_repaint();
        });
    }

    pub fn toggle_play(&mut self) {
        if self.playing() || self.project.is_empty() {
            return self.stop();
        }
        self.sounds.stop_preview();
        if self.playhead >= self.project.total() - 0.05 {
            self.playhead = 0.0;
        }
        let (w, h) = self.preview_size();
        self.player = Some(Player::start(&self.project, self.playhead, w, h, self.mixer()));
    }

    /// Pick videos and images and place them one after another at the end of the active track.
    pub fn import(&mut self) {
        let exts: Vec<_> = crate::media::probe::VIDEO_EXTS
            .iter()
            .chain(&crate::media::probe::IMAGE_EXTS)
            .collect();
        self.import_files("video and images", &exts);
    }

    /// Pick sound or music files.
    pub fn import_sound(&mut self) {
        let exts: Vec<_> = crate::media::probe::AUDIO_EXTS.iter().collect();
        self.import_files("sound", &exts);
    }

    fn import_files(&mut self, label: &str, exts: &[&&str]) {
        if let Some(files) = rfd::FileDialog::new().add_filter(label, exts).pick_files() {
            self.add_files_in_sequence(&files);
        }
    }

    /// Selects the first use of a library file and moves the playhead there.
    pub fn focus_media(&mut self, path: &str) {
        self.stop();
        if let Some((id, at)) = self.project.first_use(path).map(|i| (i.id, i.at)) {
            self.playhead = at;
            self.select(Selection::Item(id));
        }
    }

    pub fn remove_from_library(&mut self, index: usize) {
        if index < self.project.media.len() {
            self.project.media.remove(index);
        }
    }

    /// Removes the picked track when it is empty. A track with clips in it, or the last one, stays.
    fn delete_track(&mut self, index: usize) {
        let empty = self.project.tracks.get(index).is_some_and(|t| t.items.is_empty());
        if !empty {
            self.status = "Only an empty track can be removed".into();
        } else if self.project.tracks.len() > 1 {
            self.remove_track(index);
            self.select(Selection::None);
        }
    }

    /// Removes an empty track (never the last one) and keeps the active track valid.
    pub fn remove_track(&mut self, index: usize) {
        if self.project.remove_empty_track(index) {
            self.track = self.track.min(self.project.tracks.len() - 1);
        }
    }

    /// Adds a text item on an overlay track at the playhead.
    pub fn add_text(&mut self) {
        self.stop();
        let Some(fonts) = &self.fonts else {
            self.status = "Fonts are still loading".into();
            return;
        };
        let mut item = Item::new(String::new(), String::new(), Kind::Text, (1, 1), 0.0, false);
        (item.text, item.font, item.end, item.at) = ("Your text".into(), DEFAULT_FONT.into(), 3.0, self.playhead);
        if let Err(e) = text::refresh(fonts, &mut item) {
            self.status = e;
            return;
        }
        let track = self.track.max(1);
        let id = self.project.add(track, item);
        self.select(Selection::Item(id));
        self.tab = Tab::Text;
        self.track = track;
    }

    /// Re-renders a text item after its text or look changed.
    pub fn refresh_text(&mut self, id: u64) {
        let (Some(fonts), Some(item)) = (&self.fonts, self.project.get_mut(id)) else {
            return;
        };
        if let Err(e) = text::refresh(fonts, item) {
            self.status = e;
        }
    }

    /// Scales an item by `grow`. Text changes its font size instead of its picture size, so it stays sharp.
    pub fn scale_item(&mut self, id: u64, grow: f32) {
        let Some(it) = self.project.get_mut(id) else { return };
        if it.kind == Kind::Text {
            it.font_size = (it.font_size * grow).clamp(12.0, 600.0);
            self.refresh_text(id);
        } else {
            it.scale = (it.scale * grow).clamp(0.05, 8.0);
        }
    }

    pub fn add_track(&mut self) {
        self.project.tracks.push(Default::default());
        self.track = self.project.tracks.len() - 1;
    }

    pub fn split(&mut self) {
        self.stop();
        if let Some(id) = self.selected_item() {
            self.project.split(id, self.playhead);
        }
    }

    pub fn delete(&mut self) {
        self.stop();
        if let Selection::Track(index) = self.selection {
            self.delete_track(index);
        } else if let Some(id) = self.selected_item() {
            if self.magnet {
                self.project.ripple_remove(id);
            } else {
                self.project.remove(id);
            }
            self.select(Selection::None);
            self.playhead = self.playhead.min(self.project.total());
        }
    }

    pub fn export(&mut self) {
        if self.project.is_empty() {
            return;
        }
        let Some(path) = rfd::FileDialog::new().set_file_name("export.mp4").save_file() else {
            return;
        };
        let project = self.project.snapshot();
        self.status = "Exporting…".into();
        self.spawn(move |tx| {
            let msg = match export(&project, &path) {
                Ok(()) => {
                    // Most free sounds require attribution, so the credits travel with the video.
                    let credits = project.credits();
                    if !credits.is_empty() {
                        let _ = std::fs::write(path.with_extension("credits.txt"), credits.join("\n"));
                    }
                    format!("Saved {}", path.display())
                }
                Err(e) => format!("Export failed: {e}"),
            };
            let _ = tx.send(Event::Status(msg));
        });
    }

    pub fn transcribe(&mut self) {
        if self.project.is_empty() {
            return;
        }
        let project = self.project.snapshot();
        let timing = project.timing_fingerprint();
        self.status = "Transcribing…".into();
        self.spawn(move |tx| match whisper::transcribe(&project) {
            Ok(captions) => {
                let _ = tx.send(Event::Status(format!("{} captions", captions.len())));
                let _ = tx.send(Event::Captions(captions, timing));
            }
            Err(e) => {
                let _ = tx.send(Event::Status(format!("Captions failed: {e}")));
            }
        });
    }
}
