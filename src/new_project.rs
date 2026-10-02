//! Cmd+N: start over with an empty project, after a confirmation if there is unsaved work.

use crate::app::App;
use crate::persist;
use crate::project::Project;
use crate::widgets::{Kind, button};
use eframe::egui::{Context, Modal, RichText};

/// What the project is compared to before Cmd+N throws it away.
pub struct Unsaved {
    /// Fingerprint of the project as last saved or opened; it differs while there is unsaved work.
    pub file_fp: u64,
    /// The warning is showing.
    pub confirm: bool,
}

impl Default for Unsaved {
    fn default() -> Self {
        Self {
            file_fp: Project::default().fingerprint(),
            confirm: false,
        }
    }
}

impl App {
    /// True when the project differs from what is on disk (or was never saved and is not empty).
    pub fn has_unsaved_work(&self) -> bool {
        self.project.fingerprint() != self.unsaved.file_fp
    }

    /// Cmd+N: starts a new project right away, or asks first when work would be lost.
    pub fn new_project(&mut self) {
        if self.has_unsaved_work() {
            self.unsaved.confirm = true;
        } else {
            self.reset_project();
        }
    }

    fn reset_project(&mut self) {
        let _ = std::fs::remove_file(persist::autosave_path()); // or the old project comes back on next launch
        self.adopt(Project::default(), None);
        self.status = "New project".into();
    }

    /// The warning that has to be confirmed before unsaved work is thrown away.
    pub fn confirm_new_dialog(&mut self, ctx: &Context) {
        if !self.unsaved.confirm {
            return;
        }
        let modal = Modal::new("confirm_new".into()).show(ctx, |ui| {
            ui.set_width(340.0);
            ui.label(RichText::new("DISCARD UNSAVED WORK?").strong());
            ui.add_space(8.0);
            ui.label("This project has changes that are not saved. A new project deletes them for good.");
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if button(ui, "Discard and start new", Kind::Cta).clicked() {
                    self.unsaved.confirm = false;
                    self.reset_project();
                }
                if button(ui, "Cancel", Kind::Plain).clicked() {
                    self.unsaved.confirm = false;
                }
            });
        });
        if modal.should_close() {
            self.unsaved.confirm = false; // Escape or a click outside
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Item;

    fn app_with_a_clip() -> App {
        let mut app = App::new(Context::default());
        let clip = Item::new(
            "/x.mp4".into(),
            "x".into(),
            crate::project::Kind::Video,
            (160, 90),
            5.0,
            false,
        );
        app.project.add(0, clip);
        app
    }

    #[test]
    fn an_empty_project_starts_new_without_asking() {
        let mut app = App::new(Context::default());
        app.new_project();
        assert!(!app.unsaved.confirm);
    }

    #[test]
    fn unsaved_work_needs_confirmation_and_cancel_keeps_it() {
        let mut app = app_with_a_clip();
        app.new_project();
        assert!(app.unsaved.confirm);
        assert!(!app.project.is_empty());
    }

    #[test]
    fn saved_work_starts_new_without_asking() {
        let mut app = app_with_a_clip();
        app.unsaved.file_fp = app.project.fingerprint();
        app.new_project();
        assert!(!app.unsaved.confirm);
        assert!(app.project.is_empty());
    }
}
