//! The Stickers tab: search the animated emoji and click one to put it on the timeline.

use crate::app::App;
use crate::stickers::{self, Sticker};
use crate::theme::{ACCENT, BLACK, BORDER, MUTED, SUBTLE, WHITE};
use crate::widgets::caps;
use eframe::egui::{Rect, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, vec2};

const CELL: f32 = 76.0;
const SHOWN: usize = 200;

pub fn show(ui: &mut Ui, app: &mut App) {
    app.ensure_stickers();
    ui.add(
        TextEdit::singleline(&mut app.sounds.stickers.query)
            .hint_text("Search: dance, cat, fire, heart")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(6.0);
    if let Some(error) = &app.sounds.stickers.error {
        crate::widgets::warn(ui, error);
    }
    let found: Vec<Sticker> = stickers::search(&app.sounds.stickers.all, &app.sounds.stickers.query)
        .into_iter()
        .take(SHOWN)
        .collect();
    if found.is_empty() && !app.sounds.stickers.all.is_empty() {
        caps(ui, "No sticker with that name");
    }
    let per_row = ((ui.available_width() / CELL) as usize).max(1);
    let rows = found.len().div_ceil(per_row);
    let mut chosen = None;
    ScrollArea::vertical()
        .auto_shrink(false)
        .show_rows(ui, CELL, rows, |ui, range| {
            for row in range {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    for sticker in found.iter().skip(row * per_row).take(per_row) {
                        if cell(ui, app, sticker) {
                            chosen = Some(sticker.clone());
                        }
                    }
                });
            }
        });
    ui.add_space(4.0);
    ui.label(
        eframe::egui::RichText::new("Noto Animated Emoji by Google, CC BY 4.0")
            .size(10.0)
            .color(SUBTLE),
    );
    if let Some(sticker) = chosen {
        app.add_sticker(sticker);
    }
}

/// One preview; returns true when it was clicked.
fn cell(ui: &mut Ui, app: &mut App, sticker: &Sticker) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(CELL, CELL), Sense::click());
    let busy = app.sounds.busy.contains(&format!("sticker-{}", sticker.code));
    app.request_preview(sticker);
    let p = ui.painter();
    p.rect_filled(rect, 0.0, if resp.hovered() { MUTED } else { WHITE });
    p.rect_stroke(
        rect,
        0.0,
        Stroke::new(BORDER, if busy { ACCENT } else { BLACK }),
        StrokeKind::Inside,
    );
    if let Some(texture) = app.sounds.stickers.previews.get(&sticker.code) {
        let image = Rect::from_center_size(rect.center(), vec2(CELL - 16.0, CELL - 16.0));
        p.image(
            texture.id(),
            image,
            Rect::from_min_max(eframe::egui::pos2(0.0, 0.0), eframe::egui::pos2(1.0, 1.0)),
            WHITE,
        );
    }
    resp.on_hover_text(&sticker.words).clicked()
}
