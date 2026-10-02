//! The Text tab of the left panel: what text can be added.

use crate::app::App;
use crate::widgets::{Kind, button, caps};
use eframe::egui::Ui;

pub fn show(ui: &mut Ui, app: &mut App) {
    caps(ui, "Add at the playhead");
    ui.add_space(6.0);
    if button(ui, "Text", Kind::Plain).clicked() {
        app.add_text();
    }
    ui.add_space(6.0);
    if button(ui, "Bauchbinde", Kind::Plain).clicked() {
        app.add_lower_third();
    }
    ui.add_space(8.0);
    caps(ui, "Bauchbinde: name and role on a bar");
}
