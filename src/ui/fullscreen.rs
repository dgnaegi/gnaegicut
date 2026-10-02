//! Watching the preview full screen: the picture fits the whole window, nothing else is shown.
//! Click the picture or press Space to play and pause, Esc or the cross (or F) to leave.

use crate::app::App;
use crate::theme::{ACCENT, BLACK, WHITE, bold};
use eframe::egui::{Align2, CentralPanel, Frame, Id, Rect, Sense, Ui, ViewportCommand, pos2, vec2};

impl App {
    /// Enters or leaves full screen. Entering starts playing right away from the playhead, with a sharper preview.
    pub fn set_fullscreen(&mut self, on: bool) {
        if self.fullscreen == on {
            return;
        }
        self.stop(); // the player renders at the old size
        self.fullscreen = on;
        self.ctx.send_viewport_cmd(ViewportCommand::Fullscreen(on));
        if on {
            self.toggle_play();
        }
    }
}

pub fn show(ui: &mut Ui, app: &mut App) {
    let ctx = ui.ctx().clone();
    CentralPanel::default().frame(Frame::new().fill(BLACK)).show(ui, |ui| {
        if !app.playing() {
            let size = app.preview_size();
            app.preview.show(&ctx, &app.project, app.playhead, size);
        }
        let area = ui.available_rect_before_wrap();
        let screen = ui.allocate_rect(area, Sense::click());
        let (w, h) = app.project.aspect.size();
        let scale = (area.width() / w as f32).min(area.height() / h as f32);
        let frame = Rect::from_center_size(area.center(), vec2(w as f32, h as f32) * scale);
        if let Some(tex) = &app.preview.texture {
            let whole = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().image(tex.id(), frame, whole, WHITE);
        }
        if screen.clicked() {
            app.toggle_play();
        }

        // The way out: a small cross in the corner.
        let cross = Rect::from_min_size(pos2(area.right() - 56.0, area.top() + 16.0), vec2(40.0, 40.0));
        let exit = ui.interact(cross, Id::new("fullscreen-exit"), Sense::CLICK);
        let hot = exit.hovered();
        let p = ui.painter();
        p.rect_filled(cross, 0.0, if hot { ACCENT } else { BLACK });
        p.text(cross.center(), Align2::CENTER_CENTER, "×", bold(22.0), WHITE);
        if exit.clicked() {
            app.set_fullscreen(false);
        }
    });
}
