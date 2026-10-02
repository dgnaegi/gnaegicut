//! Free sound and music search inside the library: one search, results grouped, drag onto the timeline.

use crate::app::App;
use crate::drop::SoundDrag;
use crate::sounds::{Sound, Source};
use crate::theme::{ACCENT, BLACK, BORDER, MUTED, WHITE, bold};
use crate::widgets::{Kind, button, caps, chip, warn};
use eframe::egui::{Align2, Color32, Id, Key, Rect, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, pos2, vec2};

const ROW: f32 = 48.0;

pub fn show(ui: &mut Ui, app: &mut App) {
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 100.0).max(60.0);
        let edit = TextEdit::singleline(&mut app.sounds.query)
            .hint_text("Search sounds and music")
            .desired_width(width);
        let enter = ui.add(edit).lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if button(ui, "Search", Kind::Cta).clicked() || enter {
            app.search_sounds(None); // a typed search looks in both
        }
    });
    suggestions(ui, app, "Sound", Source::Effects);
    suggestions(ui, app, "Music", Source::Music);
    status(ui, app);

    let (effects, music) = (app.sounds.effects.clone(), app.sounds.music.clone());
    ScrollArea::vertical().show(ui, |ui| {
        group(ui, app, "Sound effects", effects);
        group(ui, app, "Music", music);
        credits(ui, app);
    });
}

/// One-tap searches of a kind: the label, then the words.
fn suggestions(ui: &mut Ui, app: &mut App, label: &str, source: Source) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        caps(ui, label);
        for word in source.suggestions() {
            if chip(ui, word).clicked() {
                app.sounds.query = word.to_string();
                app.search_sounds(Some(source));
            }
        }
    });
}

fn status(ui: &mut Ui, app: &App) {
    let s = &app.sounds;
    if s.loading > 0 {
        caps(ui, "Searching…");
    } else if let Some(e) = &s.error {
        warn(ui, e);
    } else if s.searched && s.effects.is_empty() && s.music.is_empty() {
        caps(ui, "No results");
    }
}

/// A heading with the number of results, then the rows. Nothing at all for an empty group.
fn group(ui: &mut Ui, app: &mut App, title: &str, list: Vec<Sound>) {
    if list.is_empty() {
        return;
    }
    ui.add_space(6.0);
    caps(ui, &format!("{title} ({})", list.len()));
    for sound in list {
        row(ui, app, sound);
    }
}

/// One result. Drag it onto the timeline to use it; the square on the right auditions it.
fn row(ui: &mut Ui, app: &mut App, sound: Sound) {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click_and_drag());
    if resp.dragged() {
        resp.dnd_set_drag_payload(SoundDrag(sound.clone()));
    }
    let play = Rect::from_min_size(rect.right_top() + vec2(-ROW + 8.0, 8.0), vec2(ROW - 16.0, ROW - 16.0));
    let play_resp = ui.interact(play, Id::new(("sound-play", &sound.id)), Sense::CLICK);

    let busy = app.sounds.busy.contains(&sound.id);
    let playing = app.sounds.playing.as_ref() == Some(&sound.id);
    let hot = play_resp.hovered();
    let (symbol, fill, ink) = match (busy, playing, hot) {
        (true, ..) => ("…", WHITE, BLACK),
        (_, true, _) => ("■", BLACK, WHITE),
        (_, _, true) => ("▶", ACCENT, WHITE),
        _ => ("▶", WHITE, BLACK),
    };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, if resp.hovered() { MUTED } else { WHITE });
    p.rect_stroke(rect, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
    p.rect_filled(play, 0.0, fill);
    p.rect_stroke(play, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
    p.text(play.center(), Align2::CENTER_CENTER, symbol, bold(13.0), ink);

    let text_area = Rect::from_min_max(rect.min, pos2(play.left() - 4.0, rect.bottom())).shrink(4.0);
    let meta = format!("{} · {} · {}", sound.duration_label(), sound.license, sound.creator);
    let clipped = p.with_clip_rect(text_area);
    clipped.text(
        pos2(rect.left() + 8.0, rect.top() + 8.0),
        Align2::LEFT_TOP,
        sound.title.to_uppercase(),
        bold(11.0),
        BLACK,
    );
    clipped.text(
        pos2(rect.left() + 8.0, rect.top() + 26.0),
        Align2::LEFT_TOP,
        meta,
        bold(10.0),
        Color32::from_gray(110),
    );
    if play_resp.clicked() && !busy {
        app.preview_sound(sound);
    }
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
