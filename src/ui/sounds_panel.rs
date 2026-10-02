//! Free sound search inside the library: search, audition, add to the timeline.

use crate::app::App;
use crate::sounds::Sound;
use crate::theme::{BLACK, BORDER, WHITE, bold};
use crate::widgets::{Kind, button, caps, chip, warn};
use eframe::egui::{Align2, Key, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, pos2, vec2};

const ROW: f32 = 46.0;

pub fn show(ui: &mut Ui, app: &mut App) {
    let Some(source) = app.library_tab.source() else { return };
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 100.0).max(60.0);
        let edit = TextEdit::singleline(&mut app.sounds.query)
            .hint_text("Search free sounds")
            .desired_width(width);
        let enter = ui.add(edit).lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if button(ui, "Search", Kind::Cta).clicked() || enter {
            app.search_sounds();
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        for word in source.suggestions() {
            if chip(ui, word).clicked() {
                app.sounds.query = word.to_string();
                app.search_sounds();
            }
        }
    });
    status(ui, app, source);

    let list: Vec<Sound> = if app.sounds.results_for == source {
        app.sounds.results.clone()
    } else {
        vec![]
    };
    ScrollArea::vertical().show(ui, |ui| {
        for sound in list {
            row(ui, app, sound);
        }
        credits(ui, app);
    });
}

fn status(ui: &mut Ui, app: &App, source: crate::sounds::Source) {
    let s = &app.sounds;
    if s.loading {
        caps(ui, "Searching…");
    } else if let Some(e) = &s.error {
        warn(ui, e);
    } else if s.searched && s.results_for == source && s.results.is_empty() {
        caps(ui, "No results");
    } else if s.results_for == source && !s.results.is_empty() {
        caps(ui, &format!("{} results", s.results.len()));
    }
}

fn row(ui: &mut Ui, app: &mut App, sound: Sound) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 0.0, WHITE);
    p.rect_stroke(rect, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
    p.with_clip_rect(rect.shrink(4.0)).text(
        pos2(rect.left() + 8.0, rect.top() + 8.0),
        Align2::LEFT_TOP,
        sound.title.to_uppercase(),
        bold(11.0),
        BLACK,
    );
    let meta = format!("{} · {} · {}", sound.duration_label(), sound.license, sound.creator);
    p.with_clip_rect(rect.shrink(4.0)).text(
        pos2(rect.left() + 8.0, rect.top() + 26.0),
        Align2::LEFT_TOP,
        meta,
        bold(10.0),
        eframe::egui::Color32::from_gray(110),
    );

    let busy = app.sounds.busy.contains(&sound.id);
    let playing = app.sounds.playing.as_ref() == Some(&sound.id);
    ui.horizontal(|ui| {
        let (label, kind) = if busy {
            ("…", Kind::Plain)
        } else if playing {
            ("Stop", Kind::Active)
        } else {
            ("Play", Kind::Plain)
        };
        if button(ui, label, kind).clicked() && !busy {
            app.preview_sound(sound.clone());
        }
        if button(ui, "+ Add", Kind::Plain).clicked() {
            app.add_sound(sound.clone());
        }
    });
    ui.add_space(4.0);
}

/// Most free licences require crediting the author; this copies the lines for every sound used.
fn credits(ui: &mut Ui, app: &mut App) {
    let n = app.project.credits().len();
    if n == 0 {
        return;
    }
    ui.add_space(8.0);
    if button(ui, &format!("Copy credits ({n})"), Kind::Plain).clicked() {
        app.copy_credits();
    }
}
