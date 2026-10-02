use crate::app::App;
use crate::logo;
use crate::project::Aspect;
use crate::theme::black;
use crate::widgets::{Kind, button, menu_item, segmented};
use eframe::egui::{Align, Layout, Popup, RichText, Sense, Ui, vec2};

/// The single add button: opens a small menu of what can be added.
fn add_menu(ui: &mut Ui, app: &mut App) {
    let plus = button(ui, "+", Kind::Plain);
    Popup::menu(&plus).show(|ui| {
        ui.set_min_width(200.0);
        if menu_item(ui, "Video or image…").clicked() {
            app.import();
            ui.close();
        }
        if menu_item(ui, "Sound or music…").clicked() {
            app.import_sound();
            ui.close();
        }
        if menu_item(ui, "Text").clicked() {
            app.add_text();
            ui.close();
        }
    });
}

pub fn show(ui: &mut Ui, app: &mut App) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
        logo::paint(ui.painter(), rect);
        ui.label(RichText::new("GNAEGICUT").font(black(22.0)));
        ui.add_space(24.0);

        if button(ui, "New", Kind::Plain).clicked() {
            app.new_project();
        }
        if button(ui, "Open", Kind::Plain).clicked() {
            app.open();
        }
        if button(ui, "Save", Kind::Plain).clicked() {
            app.save();
        }
        ui.add_space(8.0);
        let library = if app.library_open { Kind::Active } else { Kind::Plain };
        if button(ui, "Library", library).clicked() {
            app.library_open = !app.library_open;
        }
        add_menu(ui, app);
        ui.add_space(16.0);
        let aspects: Vec<_> = Aspect::ALL.iter().map(|a| (*a, a.label())).collect();
        segmented(ui, &mut app.project.aspect, &aspects);
        let kind = if app.safe_zones { Kind::Active } else { Kind::Plain };
        if button(ui, "Safe zones", kind).clicked() {
            app.safe_zones = !app.safe_zones;
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if button(ui, "Export MP4", Kind::Cta).clicked() {
                app.export();
            }
            ui.label(&app.status);
        });
    });
}
