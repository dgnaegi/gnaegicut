use super::font_picker;
use crate::app::{App, Selection};
use crate::project::CaptionStyle;
use crate::widgets::{Kind, button, caps, chip, segmented, warn};
use eframe::egui::{DragValue, Id, Slider, TextEdit, Ui, vec2};

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
    transcript(ui, app);
    split_all(ui, app);
    style(ui, app);
    list(ui, app);
}

/// The non-blank, trimmed lines of a text: what matters when comparing a typed transcript with the captions.
fn trimmed_lines(text: &str) -> Vec<&str> {
    text.lines().map(str::trim).filter(|l| !l.is_empty()).collect()
}

/// Every caption at once in one field, one per line, to read, copy and edit. A line break or a double space inside a
/// line splits that caption, removing a line break joins two. While typing the field keeps exactly what was typed
/// (a space at the end of a word must survive), and is only rewritten when a caption was split or joined.
fn transcript(ui: &mut Ui, app: &mut App) {
    if app.project.captions.is_empty() {
        return;
    }
    transcript_field(ui, app);
    if button(ui, "Copy all", Kind::Plain).clicked() {
        app.copy_transcript();
    }
}

/// The editable text of all captions (see `transcript`).
pub(super) fn transcript_field(ui: &mut Ui, app: &mut App) {
    let (field, store) = (Id::new("transcript-field"), Id::new("transcript-buffer"));
    let canonical = app.project.transcript();
    let typing = ui.memory(|m| m.has_focus(field));
    let mut buffer: String = if typing {
        ui.data(|d| d.get_temp(store)).unwrap_or_else(|| canonical.clone())
    } else {
        canonical
    };
    let edit = TextEdit::multiline(&mut buffer)
        .id(field)
        .desired_rows(6)
        .desired_width(f32::INFINITY);
    if ui.add(edit).changed() {
        app.project.apply_transcript(&buffer);
        let now = app.project.transcript();
        if trimmed_lines(&buffer) != trimmed_lines(&now) {
            buffer = now; // a split or a join happened: show the new lines
        }
    }
    ui.data_mut(|d| d.insert_temp(store, buffer));
}

/// Break every caption into pieces of at most this many words.
fn split_all(ui: &mut Ui, app: &mut App) {
    if app.project.captions.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        caps(ui, "Words per caption");
        for words in [1, 2, 3, 4] {
            if chip(ui, &words.to_string()).clicked() {
                app.project.split_captions(words);
            }
        }
    });
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
    let mut y = layout.position(app.project.aspect).1;
    let height = Slider::new(&mut y, 0.0..=1.0)
        .text("height")
        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0));
    if ui.add(height).changed() {
        layout.place(layout.x, y);
    }
    if !layout.auto_y && button(ui, "Reset position", Kind::Plain).clicked() {
        layout.reset_position();
    }
    let kind = if matches!(app.selection, Selection::Captions | Selection::Caption(_)) {
        Kind::Active
    } else {
        Kind::Plain
    };
    if button(ui, "Move on preview", kind).clicked() {
        app.select(Selection::Captions);
    }
}

fn list(ui: &mut Ui, app: &mut App) {
    let (mut remove, mut split) = (None, None);
    for (i, c) in app.project.captions.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut c.start).speed(0.05).fixed_decimals(1).suffix("s"));
            ui.add(DragValue::new(&mut c.end).speed(0.05).fixed_decimals(1).suffix("s"));
            if chip(ui, "Split").clicked() {
                split = Some(i);
            }
            if button(ui, "x", Kind::Plain).clicked() {
                remove = Some(i);
            }
        });
        ui.add(TextEdit::singleline(&mut c.text).desired_width(f32::INFINITY));
    }
    if let Some(i) = remove {
        app.project.captions.remove(i);
    } else if let Some(i) = split {
        app.project.split_caption(i);
    }
}
