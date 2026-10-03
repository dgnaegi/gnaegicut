use super::toolbar_icons;
use crate::app::App;
use crate::logo;
use crate::project::Aspect;
use crate::theme::{TITLE, black};
use crate::widgets::{Kind, busy, button, segmented};
use eframe::egui::{Align, Label, Layout, RichText, Sense, Ui, vec2};

pub fn show(ui: &mut Ui, app: &mut App) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
        logo::paint(ui.painter(), rect);
        ui.label(RichText::new("GNAEGICUT").font(black(TITLE)));
        ui.add_space(24.0);

        let aspects: Vec<_> = Aspect::ALL.iter().map(|a| (*a, a.label())).collect();
        segmented(ui, &mut app.project.aspect, &aspects);
        let kind = if app.safe_zones { Kind::Active } else { Kind::Plain };
        if button(ui, "Safe zones", kind).clicked() {
            app.safe_zones = !app.safe_zones;
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if toolbar_icons::floppy(ui).on_hover_text("Save (Cmd+S)").clicked() {
                app.save();
            }
            if toolbar_icons::plus(ui).on_hover_text("New project (Cmd+N)").clicked() {
                app.new_project();
            }
            ui.add_space(8.0);
            if button(ui, "Export MP4", Kind::Cta).clicked() {
                app.export();
            }
            match app.busy {
                Some(what) => busy(ui, what),
                None => {
                    ui.add(Label::new(&app.status).truncate()); // a long path must not push the buttons away
                }
            }
        });
    });
}
