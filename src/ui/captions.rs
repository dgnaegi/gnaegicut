use super::font_picker;
use crate::app::{App, Selection};
use crate::project::CaptionStyle;
use crate::widgets::{Kind, button, segmented, warn};
use eframe::egui::{DragValue, Slider, TextEdit, Ui};

pub fn show(ui: &mut Ui, app: &mut App) {
    if button(ui, "Generate captions", Kind::Cta).clicked() {
        app.transcribe();
    }
    if !app.project.captions.is_empty() && app.captions_fp != Some(app.project.timing_fingerprint()) {
        warn(
            ui,
            "The timeline changed since these captions were made. Generate again to re-time them",
        );
    }
    style(ui, app);
    list(ui, app);
}

fn style(ui: &mut Ui, app: &mut App) {
    let styles: Vec<_> = CaptionStyle::ALL.iter().map(|s| (*s, s.label())).collect();
    segmented(ui, &mut app.project.caption_style, &styles);
    let layout = &mut app.project.caption_layout;
    let current = app.fonts.as_ref().and_then(|f| {
        f.faces
            .iter()
            .find(|f| f.names.contains(&layout.family) && f.bold == layout.bold)
    });
    let shown = current.map_or_else(String::new, |f| f.ps_name.clone());
    let picked = font_picker::pick(ui, app.fonts.as_ref(), &shown, "caption-font");
    if let Some(face) = picked.and_then(|ps| app.fonts.as_ref()?.find(&ps)) {
        (layout.family, layout.bold) = (face.family.clone(), face.bold);
    }
    ui.add(Slider::new(&mut layout.size, 0.5..=2.5).text("size"));
    ui.add(
        Slider::new(&mut layout.y, 0.0..=1.0)
            .text("height")
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
    );
    let kind = if app.selection == Selection::Captions {
        Kind::Active
    } else {
        Kind::Plain
    };
    if button(ui, "Move on preview", kind).clicked() {
        app.select(Selection::Captions);
    }
}

fn list(ui: &mut Ui, app: &mut App) {
    let mut remove = None;
    for (i, c) in app.project.captions.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut c.start).speed(0.05).fixed_decimals(1).suffix("s"));
            ui.add(DragValue::new(&mut c.end).speed(0.05).fixed_decimals(1).suffix("s"));
            if button(ui, "x", Kind::Plain).clicked() {
                remove = Some(i);
            }
        });
        ui.add(TextEdit::singleline(&mut c.text).desired_width(f32::INFINITY));
    }
    if let Some(i) = remove {
        app.project.captions.remove(i);
    }
}
