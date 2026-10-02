use super::{captions, item_panel, text_panel, transition_panel};
use crate::app::{App, Tab};
use crate::patterns;
use crate::project::Kind;
use crate::widgets::tabs;
use eframe::egui::{ScrollArea, Ui};

/// The tabs that make sense for the current selection, in display order.
fn available(app: &App) -> Vec<(Tab, &'static str)> {
    let mut list = vec![];
    if let Some(item) = app.selected_item().and_then(|id| app.project.get(id)) {
        if item.kind == Kind::Text {
            list.push((Tab::Text, "Text"));
        }
        if item.kind.is_visual() {
            list.extend([(Tab::Place, "Place"), (Tab::Motion, "Motion")]);
        }
        if item.has_audio {
            list.push((Tab::Audio, "Audio"));
        }
        list.push((Tab::Time, "Time"));
        if item.kind.is_visual() && (item.join.is_some() || app.project.predecessor(item.id).is_some()) {
            list.push((Tab::Transition, "Transition"));
        }
    }
    list.push((Tab::Captions, "Captions"));
    list
}

pub fn show(ui: &mut Ui, app: &mut App) {
    patterns::dots(&ui.painter_at(ui.max_rect()), ui.max_rect());
    let list = available(app);
    if !list.iter().any(|(tab, _)| *tab == app.tab) {
        app.tab = list[0].0; // e.g. the Audio tab after selecting an image
    }
    tabs(ui, &mut app.tab, &list);
    ui.add_space(8.0);
    ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        let id = app.selected_item();
        match (app.tab, id) {
            (Tab::Text, Some(id)) => text_panel::show(ui, app, id),
            (Tab::Place, Some(id)) => item_panel::placement(ui, app, id),
            (Tab::Motion, Some(id)) => item_panel::motion(ui, app, id),
            (Tab::Audio, Some(id)) => item_panel::audio(ui, app, id),
            (Tab::Time, Some(id)) => item_panel::timing(ui, app, id),
            (Tab::Transition, Some(id)) => transition_panel::show(ui, app, id),
            _ => {
                captions::show(ui, app);
            }
        }
    });
}
