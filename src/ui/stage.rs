use super::{stage_edit, stage_overlay};
use crate::app::App;
use crate::patterns;
use crate::theme::{BLACK, BORDER, WHITE, black};
use crate::widgets::surface;
use eframe::egui::{Align2, CentralPanel, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

const INSET: f32 = 32.0;

pub fn show(ui: &mut Ui, app: &mut App) {
    CentralPanel::default()
        .frame(super::frame().inner_margin(0))
        .show(ui, |ui| {
            if !app.playing() {
                // While playing, frames are pushed by the player instead of requested.
                let ctx = ui.ctx().clone();
                app.preview.show(&ctx, &app.project, app.playhead);
            }
            let area = ui.available_rect_before_wrap();
            app.stage_rect = Some(area);
            ui.allocate_rect(area, Sense::hover());
            surface(ui, area, true);
            let painter = ui.painter_at(area);
            patterns::grid(&painter, area.shrink(BORDER));

            if app.project.is_empty() {
                let at = area.left_top() + vec2(INSET, INSET);
                painter.text(at, Align2::LEFT_TOP, "DROP\nVIDEO OR\nIMAGE", black(72.0), BLACK);
                return;
            }
            let (w, h) = app.project.aspect.size();
            let scale = ((area.width() - 2.0 * INSET) / w as f32).min((area.height() - 2.0 * INSET) / h as f32);
            let frame = Rect::from_center_size(area.center(), vec2(w as f32, h as f32) * scale);
            painter.rect_filled(frame, 0.0, BLACK);
            if let Some(tex) = &app.preview.texture {
                painter.image(
                    tex.id(),
                    frame,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    WHITE,
                );
            }
            painter.rect_stroke(frame, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Outside);
            if app.safe_zones {
                stage_overlay::safe_zone(&painter, frame, app.project.aspect);
            }
            stage_edit::interact(ui, app, frame);
        });
}
