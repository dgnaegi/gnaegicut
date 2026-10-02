//! Copy, cut and paste of timeline items.

use crate::app::{App, Selection};

impl App {
    /// Copies the selected item.
    pub fn copy_selection(&mut self) {
        let Some(item) = self.selected_item().and_then(|id| self.project.get(id)) else {
            return;
        };
        self.clipboard = Some(item.clone());
        self.status = format!("Copied {}", item.name);
        // On macOS egui only reports a paste when the system clipboard holds text, so leave a note there. Without it
        // Cmd+V on a fresh clipboard produced no event at all and pasting silently did nothing.
        self.ctx.copy_text(format!("GnaegiCut: {}", item.name));
    }

    /// Copies the selected item and removes it from the timeline.
    pub fn cut_selection(&mut self) {
        self.copy_selection();
        if self.clipboard.is_some() {
            self.delete();
        }
    }

    /// Copies the text of all captions, one per line, to the system clipboard.
    pub fn copy_transcript(&mut self) {
        let text = self.project.transcript();
        self.status = format!("Copied {} lines", text.lines().count());
        self.ctx.copy_text(text);
    }

    /// Pastes the copied item at the playhead, on the selected item's track (or the active one).
    pub fn paste(&mut self) {
        let Some(item) = self.clipboard.clone() else { return };
        self.stop();
        let track = self
            .selected_item()
            .and_then(|id| self.project.find(id))
            .map_or(self.track, |(t, _)| t);
        let id = self.project.paste_item(item, track, self.playhead);
        self.track = self.project.find(id).map_or(track, |(t, _)| t);
        self.select(Selection::Item(id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Item, Kind};
    use eframe::egui::Context;

    fn app_with_a_clip() -> (App, u64) {
        let mut app = App::new(Context::default());
        let mut item = Item::new("a.mp4".into(), "a".into(), Kind::Video, (160, 90), 4.0, true);
        item.volume = 0.4;
        let id = app.project.add(0, item);
        app.select(Selection::Item(id));
        (app, id)
    }

    #[test]
    fn copy_then_paste_adds_a_second_identical_clip_at_the_playhead() {
        let (mut app, original) = app_with_a_clip();
        app.copy_selection();
        app.playhead = 6.0;
        app.paste();
        assert_eq!(app.project.items().count(), 2);
        let pasted = app.selected_item().unwrap();
        assert_ne!(pasted, original);
        let copy = app.project.get(pasted).unwrap();
        assert!((copy.at - 6.0).abs() < 1e-9 && (copy.volume - 0.4).abs() < 1e-6 && copy.path == "a.mp4");
        assert!(app.project.get(original).is_some(), "the original stays");
    }

    #[test]
    fn pasting_twice_at_the_same_spot_stacks_the_copies_on_free_tracks() {
        let (mut app, _) = app_with_a_clip();
        app.copy_selection();
        app.playhead = 1.0;
        app.paste();
        app.paste();
        let tracks: std::collections::BTreeSet<usize> = app
            .project
            .items()
            .filter_map(|i| app.project.find(i.id).map(|(t, _)| t))
            .collect();
        assert_eq!(app.project.items().count(), 3);
        assert_eq!(tracks.len(), 3, "three overlapping clips on three tracks");
    }

    #[test]
    fn cut_removes_the_clip_and_paste_brings_it_back() {
        let (mut app, original) = app_with_a_clip();
        app.cut_selection();
        assert!(app.project.get(original).is_none() && app.clipboard.is_some());
        app.playhead = 2.0;
        app.paste();
        assert_eq!(app.project.items().count(), 1);
        assert!((app.project.items().next().unwrap().at - 2.0).abs() < 1e-9);
    }

    #[test]
    fn paste_without_a_copy_does_nothing_and_copy_without_selection_keeps_the_clipboard() {
        let mut app = App::new(Context::default());
        app.paste();
        assert!(app.project.is_empty());
        let (mut app, _) = app_with_a_clip();
        app.copy_selection();
        app.select(Selection::None);
        app.copy_selection();
        assert!(
            app.clipboard.is_some(),
            "an empty selection does not clear what was copied"
        );
    }

    #[test]
    fn copy_all_puts_every_caption_on_the_clipboard_one_per_line() {
        let (mut app, _) = app_with_a_clip();
        let ctx = app.ctx.clone(); // copy_text writes to the app's own context
        app.project.captions = vec![
            crate::project::Caption {
                start: 0.0,
                end: 1.0,
                text: "First line".into(),
            },
            crate::project::Caption {
                start: 1.0,
                end: 2.0,
                text: "Second line".into(),
            },
        ];
        let output = ctx.run_ui(eframe::egui::RawInput::default(), |_| app.copy_transcript());
        let copied = output.platform_output.commands.iter().find_map(|c| match c {
            eframe::egui::OutputCommand::CopyText(text) => Some(text.clone()),
            _ => None,
        });
        assert_eq!(copied.as_deref(), Some("First line\nSecond line"));
        assert!(app.status.contains("2 lines"));
    }
}
