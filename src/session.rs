//! Everything about the project as a file and as a session: save, open, autosave and restore,
//! plus the quiet-time housekeeping (resume playback after an edit, autosave, re-render text).

use crate::app::App;
use crate::persist;
use crate::project::{Kind, Project};
use eframe::egui::ViewportCommand;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const RESUME_AFTER: Duration = Duration::from_millis(250);
const AUTOSAVE_AFTER: Duration = Duration::from_secs(2);

fn dialog() -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter("GnaegiCut project", &[persist::EXTENSION])
}

impl App {
    pub fn save(&mut self) {
        match self.current_file.clone() {
            Some(path) => self.save_to(path),
            None => self.save_as(),
        }
    }

    pub fn save_as(&mut self) {
        if let Some(path) = dialog().set_file_name("untitled.gcut").save_file() {
            self.save_to(path);
        }
    }

    fn save_to(&mut self, path: PathBuf) {
        match persist::save(&self.project, &path) {
            Ok(()) => {
                self.status = format!("Saved {}", path.display());
                self.set_file(Some(path));
            }
            Err(e) => self.status = e,
        }
    }

    pub fn open(&mut self) {
        if let Some(path) = dialog().pick_file() {
            self.open_path(path);
        }
    }

    /// Opens a project file; also used for `gnaegicut project.gcut` on the command line.
    pub fn open_path(&mut self, path: PathBuf) {
        match persist::load(&path) {
            Ok(project) => self.adopt(project, Some(path)),
            Err(e) => self.status = e,
        }
    }

    /// Brings back the last session if there is one.
    pub fn restore_autosave(&mut self) {
        if let Some(project) = persist::load(&persist::autosave_path()).ok().filter(|p| !p.is_empty()) {
            self.adopt(project, None);
            self.status = "Restored your last session".into();
        }
    }

    fn adopt(&mut self, project: Project, file: Option<PathBuf>) {
        self.stop();
        let mut project = project;
        project.sync_media();
        self.captions_fp = Some(project.timing_fingerprint());
        self.saved_fp = project.fingerprint();
        self.pending_text = project.items().any(|i| i.kind == Kind::Text);
        let missing = persist::missing_media(&project);
        self.project = project;
        self.history.reset(&self.project);
        (self.playhead, self.track) = (0.0, 0);
        self.select(crate::app::Selection::None);
        self.set_file(file);
        if !missing.is_empty() {
            self.status = format!("Missing media: {}", missing.join(", "));
        }
    }

    /// Steps back through the edits (Cmd+Z).
    pub fn undo(&mut self) {
        self.stop();
        if self.history.undo(&mut self.project) {
            self.after_history_change("Undo");
        }
    }

    /// Steps forward again (Shift+Cmd+Z).
    pub fn redo(&mut self) {
        self.stop();
        if self.history.redo(&mut self.project) {
            self.after_history_change("Redo");
        }
    }

    /// Keeps the view valid after the project was replaced: no selection of an item that no longer exists.
    fn after_history_change(&mut self, what: &str) {
        self.status = what.into();
        self.playhead = self.playhead.min(self.project.total());
        self.track = self.track.min(self.project.tracks.len().saturating_sub(1));
        if matches!(self.selection, crate::app::Selection::Item(id) if self.project.get(id).is_none()) {
            self.select(crate::app::Selection::None);
        }
    }

    fn set_file(&mut self, file: Option<PathBuf>) {
        let name = file
            .as_deref()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned());
        let title = name.map_or("GnaegiCut".to_string(), |n| format!("GnaegiCut — {n}"));
        self.ctx.send_viewport_cmd(ViewportCommand::Title(title));
        self.current_file = file;
    }

    /// Runs every frame: cheap checks that act once the user has stopped editing.
    pub fn housekeeping(&mut self) {
        let (now, fp) = (Instant::now(), self.project.fingerprint());
        let pointer_down = self.ctx.input(|i| i.pointer.any_down());
        if self.history.observe(&self.project, now, pointer_down) {
            self.ctx.request_repaint_after(Duration::from_millis(200)); // come back once the edit has settled
        }
        self.edits.observe(fp, now);
        self.watch_sound_preview();
        let quiet = self.edits.quiet_for(now);

        if self.pending_text && self.fonts.is_some() {
            self.pending_text = false;
            let ids: Vec<_> = self
                .project
                .items()
                .filter(|i| i.kind == Kind::Text)
                .map(|i| i.id)
                .collect();
            ids.into_iter().for_each(|id| self.refresh_text(id)); // the saved PNGs live in temp and may be gone
            self.history.reset(&self.project); // re-rendering is not something to undo
        }
        if self.resume {
            let dragging = self.ctx.input(|i| i.pointer.any_down());
            if !dragging && quiet >= RESUME_AFTER {
                self.resume = false;
                self.toggle_play();
            } else {
                self.ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
        if fp != self.saved_fp {
            if quiet < AUTOSAVE_AFTER {
                self.ctx.request_repaint_after(AUTOSAVE_AFTER);
            } else {
                self.saved_fp = fp;
                if !self.project.is_empty() || !self.project.captions.is_empty() {
                    let _ = persist::save(&self.project, &persist::autosave_path());
                }
            }
        }
    }
}
