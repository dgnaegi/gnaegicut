//! Adding text to the edit: plain text and the lower third (a name bar, "Bauchbinde").

use crate::app::{App, Selection, Tab};
use crate::fonts::DEFAULT_FONT;
use crate::project::{Item, Kind, Transition};
use crate::text;

/// Where a new lower third starts, as a fraction of the frame width.
const LEFT_MARGIN: f32 = 0.06;

impl App {
    /// Adds a text item on an overlay track at the playhead.
    pub fn add_text(&mut self) {
        self.add_text_item(|_| {});
    }

    /// Adds a lower third: two lines (name, role) on a coloured bar low in the picture, wiping in and fading out.
    pub fn add_lower_third(&mut self) {
        self.add_text_item(|item| {
            item.text = "Name\nRole or place".into();
            (item.bar, item.outline, item.font_size, item.color) = (true, false, 56.0, [0, 0, 0]);
            (item.x, item.y, item.end) = (0.4, 0.78, 4.0);
            (item.transition, item.fade_in, item.fade_out) = (Transition::WipeRight, 0.35, 0.25);
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
        if item.bar {
            item.x = LEFT_MARGIN + item.frac(self.project.aspect).0 / 2.0;
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
        let aspect = self.project.aspect;
        let (Some(fonts), Some(item)) = (&self.fonts, self.project.get_mut(id)) else {
            return;
        };
        let left = item.x - item.frac(aspect).0 / 2.0;
        if let Err(e) = text::refresh(fonts, item) {
            self.status = e;
        }
        if item.bar {
            item.x = left + item.frac(aspect).0 / 2.0; // a lower third grows to the right, its left edge stays
        }
    }
}
