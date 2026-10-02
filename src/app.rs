use crate::fonts::Fonts;
use crate::player::Player;
use crate::preview::{self, Preview};
use crate::project::Project;
use crate::settle::Settle;
use crate::sound_state::{LibraryTab, SoundsUi};
use crate::thumbs::Thumbs;
use eframe::egui::Context;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

pub use crate::app_types::{Event, Selection, Tab};

pub struct App {
    pub project: Project,
    pub selection: Selection,
    pub tab: Tab,
    pub track: usize, // where imports land
    pub playhead: f64,
    pub player: Option<Player>,
    pub zoom: f32, // timeline pixels per second
    pub safe_zones: bool,
    pub library_open: bool,
    pub library_tab: LibraryTab,
    pub sounds: SoundsUi,
    /// Several files dropped at once: stacked as layers (true) or one after another (false).
    pub stack_drops: bool,
    /// The timeline's last layout, so drops can find the lane and time under the pointer.
    pub timeline_view: Option<crate::drop::TimelineView>,
    /// The preview area of the last frame; library drags dropped here go to the playhead.
    pub stage_rect: Option<eframe::egui::Rect>,
    pub thumbs: Thumbs,
    /// Magnetic timeline: items snap to each other while dragging, and deleting closes the gap.
    pub magnet: bool,
    pub fonts: Option<Fonts>,
    pub preview: Preview,
    pub status: String,
    pub ctx: Context,
    pub tx: Sender<Event>,
    rx: Receiver<Event>,
    // Session: where the project lives and what has happened to it since.
    pub current_file: Option<PathBuf>,
    pub captions_fp: Option<u64>,
    pub(crate) edits: Settle,
    pub(crate) saved_fp: u64,
    pub(crate) resume: bool,
    pub(crate) pending_text: bool,
}

impl App {
    pub fn new(ctx: Context) -> Self {
        let (tx, rx) = channel();
        let loader = tx.clone();
        let wake = ctx.clone();
        std::thread::spawn(move || {
            let _ = loader.send(Event::Fonts(Box::new(Fonts::load())));
            wake.request_repaint();
        });
        Self {
            project: Project::default(),
            selection: Selection::None,
            tab: Tab::Place,
            track: 0,
            playhead: 0.0,
            player: None,
            zoom: 60.0,
            safe_zones: false,
            library_open: true,
            library_tab: LibraryTab::Files,
            sounds: SoundsUi::default(),
            stack_drops: true,
            timeline_view: None,
            stage_rect: None,
            thumbs: Thumbs::default(),
            magnet: true,
            fonts: None,
            preview: Preview::new(ctx.clone()),
            status: String::new(),
            ctx,
            tx,
            rx,
            current_file: None,
            captions_fp: None,
            edits: Settle::new(0),
            saved_fp: 0,
            resume: false,
            pending_text: false,
        }
    }

    /// Changes the selection and brings the inspector to the matching tab.
    pub fn select(&mut self, selection: Selection) {
        self.selection = selection;
        match selection {
            Selection::Captions => self.tab = Tab::Captions,
            Selection::Item(_) if self.tab == Tab::Captions => self.tab = Tab::Place,
            _ => {}
        }
    }

    pub fn selected_item(&self) -> Option<u64> {
        match self.selection {
            Selection::Item(id) => self.project.get(id).map(|i| i.id),
            _ => None,
        }
    }

    /// Advances playback and applies results from background jobs. Call once per frame.
    pub fn tick(&mut self) {
        self.poll_player();
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Status(s) => self.status = s,
                Event::Captions(c, fp) => (self.project.captions, self.captions_fp) = (c, Some(fp)),
                Event::Fonts(f) => self.fonts = Some(*f),
                Event::Thumb(path, px, w, h) => self.thumbs.insert(&self.ctx, path, &px, w, h),
                other => self.on_sound_event(other),
            }
        }
        self.housekeeping();
    }

    pub fn playing(&self) -> bool {
        self.player.is_some()
    }

    /// Stops playback for good (the user paused, scrubbed or changed the structure).
    pub fn stop(&mut self) {
        self.player = None;
        self.resume = false;
    }

    /// Stops playback because the edit changed; it picks up again once editing settles.
    pub fn pause_for_edit(&mut self) {
        let was_playing = self.playing();
        self.stop();
        self.resume = was_playing;
    }

    fn poll_player(&mut self) {
        let Some(player) = self.player.as_mut() else { return };
        if player.fingerprint != self.project.fingerprint() {
            return self.pause_for_edit(); // would play something stale; resumes after the edit settles
        }
        let update = player.poll();
        let total = self.project.total();
        self.playhead = player.position().min(total);
        let (w, h) = preview::size(self.project.aspect);
        match update {
            Ok(u) => {
                if let Some(frame) = u.frame {
                    self.preview.set_frame(&self.ctx, &frame, w, h);
                }
                if u.finished || self.playhead >= total {
                    self.stop();
                }
            }
            Err(e) => {
                self.status = e;
                self.stop();
            }
        }
        self.ctx.request_repaint();
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        crate::ui::draw(ui, self);
    }
}
