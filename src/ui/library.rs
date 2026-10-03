//! The library: every imported file with a thumbnail, how often it is used, and quick actions.

use super::{captions, sounds_panel, text_tab};
use crate::app::App;
use crate::drop::LibraryDrag;
use crate::patterns;
use crate::project::{Kind, MediaRef};
use crate::sound_state::LibraryTab;
use crate::theme::{ACCENT, BLACK, BORDER, MUTED, WHITE, bold};
use crate::thumbs::BOX;
use crate::widgets::{Kind as Style, button, section, tabs};
use eframe::egui::{Align2, Id, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

const ROW: f32 = 72.0;

fn duration(m: &MediaRef) -> String {
    match m.kind {
        Kind::Image | Kind::Text => "IMAGE".into(),
        Kind::Video | Kind::Audio => format!("{}:{:02}", m.len as u64 / 60, m.len as u64 % 60),
    }
}

pub fn show(ui: &mut Ui, app: &mut App) {
    patterns::dots(&ui.painter_at(ui.max_rect()), ui.max_rect());
    let list = [
        (LibraryTab::Files, "Media"),
        (LibraryTab::Sounds, "Sounds"),
        (LibraryTab::Text, "Text"),
        (LibraryTab::Captions, "Captions"),
    ];
    tabs(ui, &mut app.library_tab, &list);
    ui.add_space(6.0);
    match app.library_tab {
        LibraryTab::Files => files(ui, app),
        LibraryTab::Sounds => sounds_panel::show(ui, app),
        LibraryTab::Text => text_tab::show(ui, app),
        LibraryTab::Captions => {
            ScrollArea::vertical().show(ui, |ui| captions::show(ui, app));
        }
    }
}

fn files(ui: &mut Ui, app: &mut App) {
    if button(ui, "Paste screenshot", Style::Plain).clicked() && !app.paste_image() {
        app.status = "No image on the clipboard".into();
    }
    ui.add_space(6.0);
    section(ui, &format!("Media ({})", app.project.media.len()));
    if app.project.media.is_empty() {
        return;
    }
    let selected_path = app
        .selected_item()
        .and_then(|id| app.project.get(id))
        .map(|i| i.path.clone());
    let (mut focus, mut remove) = (None, None);
    ScrollArea::vertical().show(ui, |ui| {
        for (index, media) in app.project.media.clone().iter().enumerate() {
            let used = app.project.usage(&media.path);
            let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click_and_drag());
            if resp.dragged() {
                resp.dnd_set_drag_payload(LibraryDrag(index));
            }
            let active = selected_path.as_deref() == Some(media.path.as_str());
            let p = ui.painter();
            p.rect_filled(rect, 0.0, if resp.hovered() { MUTED } else { WHITE });
            p.rect_stroke(
                rect,
                0.0,
                Stroke::new(BORDER, if active { ACCENT } else { BLACK }),
                StrokeKind::Inside,
            );

            let thumb = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(ROW - 12.0, ROW - 12.0));
            p.rect_filled(thumb, 0.0, BLACK);
            let tex = if media.kind.is_visual() {
                app.thumbs.get(media, &app.tx, &app.ctx)
            } else {
                None
            };
            if media.kind == Kind::Audio {
                p.text(thumb.center(), Align2::CENTER_CENTER, "♪", bold(26.0), ACCENT);
            }
            if let Some(tex) = tex {
                let fitted = Rect::from_center_size(thumb.center(), tex.size_vec2() * (ROW - 12.0) / BOX as f32);
                p.image(
                    tex.id(),
                    fitted,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    WHITE,
                );
            }
            let text_x = thumb.right() + 10.0;
            p.with_clip_rect(rect.shrink(4.0)).text(
                pos2(text_x, rect.top() + 10.0),
                Align2::LEFT_TOP,
                media.name.to_uppercase(),
                bold(11.0),
                BLACK,
            );
            let meta = format!("{} · {}×{}", duration(media), media.src_w, media.src_h);
            p.text(
                pos2(text_x, rect.top() + 28.0),
                Align2::LEFT_TOP,
                meta,
                bold(10.0),
                eframe::egui::Color32::from_gray(110),
            );
            let usage = if used == 0 {
                "NOT USED".to_string()
            } else {
                format!("USED {used}×")
            };
            p.text(
                pos2(text_x, rect.top() + 42.0),
                Align2::LEFT_TOP,
                usage,
                bold(10.0),
                if used == 0 { ACCENT } else { BLACK },
            );
            // A small × in the corner takes the file out of the library.
            let cross = Rect::from_min_size(rect.right_top() + vec2(-24.0, 4.0), vec2(20.0, 20.0));
            let cross_resp = ui.interact(cross, Id::new(("library-remove", index)), Sense::CLICK);
            let hot = cross_resp.hovered();
            let p = ui.painter();
            p.rect_filled(cross, 0.0, if hot { ACCENT } else { WHITE });
            p.text(
                cross.center(),
                Align2::CENTER_CENTER,
                "×",
                bold(14.0),
                if hot { WHITE } else { BLACK },
            );
            if cross_resp.clicked() {
                remove = Some(index);
            } else if resp.clicked() {
                focus = Some(media.path.clone());
            }
            ui.add_space(4.0);
        }
    });
    if let Some(path) = focus {
        app.focus_media(&path);
    }
    if let Some(i) = remove {
        app.remove_from_library(i);
    }
}
