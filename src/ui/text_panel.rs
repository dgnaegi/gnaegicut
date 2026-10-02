use super::font_picker;
use crate::app::App;
use crate::widgets::segmented;
use eframe::egui::{Slider, TextEdit, Ui};

pub fn show(ui: &mut Ui, app: &mut App, id: u64) {
    let Some(it) = app.project.get_mut(id) else { return };
    let mut changed = ui
        .add(
            TextEdit::multiline(&mut it.text)
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        )
        .changed();
    if let Some(font) = font_picker::pick(ui, app.fonts.as_ref(), &it.font, "text-font") {
        it.font = font;
        changed = true;
    }
    changed |= ui
        .add(Slider::new(&mut it.font_size, 24.0..=300.0).text("font size"))
        .changed();
    ui.horizontal(|ui| {
        changed |= ui.color_edit_button_srgb(&mut it.color).changed();
        changed |= segmented(ui, &mut it.outline, &[(true, "Outline"), (false, "Plain")]);
    });
    if changed {
        app.refresh_text(id);
    }
}
