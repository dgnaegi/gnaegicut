//! Adding text to the edit: plain text and the lower third (a name bar, "Bauchbinde").

use crate::app::{App, Selection, Tab};
use crate::fonts::DEFAULT_FONT;
use crate::project::{Item, Kind, Transition};
use crate::text;

impl App {
    /// Adds a text item on an overlay track at the playhead.
    pub fn add_text(&mut self) {
        self.add_text_item(|_| {});
    }

    /// Adds a lower third: two lines (name, role) on a coloured bar low in the picture, sliding in and out.
    pub fn add_lower_third(&mut self) {
        self.add_text_item(|item| {
            item.text = "NAME\nRole or place".into();
            (item.bar, item.outline, item.font_size) = (true, false, 64.0);
            (item.x, item.y, item.end) = (0.5, 0.78, 4.0);
            (item.transition, item.fade_in, item.fade_out) = (Transition::SlideRight, 0.4, 0.4);
        });
    }

    fn add_text_item(&mut self, style: impl FnOnce(&mut Item)) {
        self.stop();
        let Some(fonts) = &self.fonts else {
            self.status = "Fonts are still loading".into();
            return;
        };
        let mut item = Item::new(String::new(), String::new(), Kind::Text, (1, 1), 0.0, false);
        (item.text, item.font, item.end, item.at) = ("Your text".into(), DEFAULT_FONT.into(), 3.0, self.playhead);
        style(&mut item);
        if let Err(e) = text::refresh(fonts, &mut item) {
            self.status = e;
            return;
        }
        let track = self.track.max(1);
        item.at = self.project.fit_at(track, &item, item.at);
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
}
