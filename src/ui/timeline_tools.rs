//! The tool bar above the tracks: undo, edit tools for the selected clip, magnet, zoom and the clock.

use super::timeline_zoom::{MAX_ZOOM, MIN_ZOOM};
use crate::app::App;
use crate::widgets::{Kind, button, caps};
use eframe::egui::{Slider, Ui};

fn tool(ui: &mut Ui, label: &str, on: bool) -> bool {
    button(ui, label, if on { Kind::Active } else { Kind::Plain }).clicked()
}

pub(super) fn show(ui: &mut Ui, app: &mut App) {
    ui.horizontal_wrapped(|ui| {
        if tool(ui, "Undo", false) {
            app.undo();
        }
        if tool(ui, "Redo", false) {
            app.redo();
        }
        ui.separator();
        if tool(ui, "Split (S)", false) {
            app.split();
        }
        if tool(ui, "Delete", false) {
            app.delete();
        }
        ui.separator();
        if tool(ui, "Freeze", false) {
            app.freeze();
        }
        let reversed = app
            .selected_item()
            .and_then(|id| app.project.get(id))
            .is_some_and(|i| i.reversed);
        if tool(ui, "Reverse", reversed) {
            app.reverse();
        }
        ui.separator();
        if tool(ui, "Magnet", app.magnet) {
            app.magnet = !app.magnet;
        }
        ui.add(
            Slider::new(&mut app.zoom, MIN_ZOOM..=MAX_ZOOM)
                .logarithmic(true)
                .text("zoom"),
        );
        caps(ui, &format!("{:.1}s / {:.1}s", app.playhead, app.project.total()));
    });
}
