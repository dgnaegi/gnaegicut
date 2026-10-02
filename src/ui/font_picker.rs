use crate::fonts::{Face, Fonts};
use eframe::egui::{ComboBox, ScrollArea, Ui};

/// A searchable dropdown of every installed font face. Returns the PostScript name when one is chosen.
pub fn pick(ui: &mut Ui, fonts: Option<&Fonts>, current: &str, salt: &str) -> Option<String> {
    let Some(fonts) = fonts else {
        ui.label("Loading fonts…");
        return None;
    };
    let shown = fonts.find(current).map_or_else(|| current.to_string(), Face::label);
    let mut chosen = None;
    ComboBox::from_id_salt(salt)
        .selected_text(shown)
        .width(ui.available_width() - 8.0)
        .show_ui(ui, |ui| {
            let key = ui.id().with("filter");
            let mut filter: String = ui.data_mut(|d| d.get_temp(key).unwrap_or_default());
            ui.text_edit_singleline(&mut filter);
            ui.data_mut(|d| d.insert_temp(key, filter.clone()));
            let needle = filter.to_lowercase();
            ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                for face in fonts
                    .faces
                    .iter()
                    .filter(|f| f.label().to_lowercase().contains(&needle))
                {
                    if ui.selectable_label(face.ps_name == current, face.label()).clicked() {
                        chosen = Some(face.ps_name.clone());
                    }
                }
            });
        });
    chosen
}
