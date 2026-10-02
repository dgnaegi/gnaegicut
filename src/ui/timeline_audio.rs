//! What a block on the timeline shows about the sound: the waveform with its fades, and the fade handles.

use super::timeline_items::Layout;
use crate::media::waveform;
use crate::project::Item;
use crate::theme::{ACCENT, BLACK, BORDER, WHITE};
use eframe::egui::{Color32, CursorIcon, Id, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

const STEP: f32 = 3.0; // pixels between waveform bars

/// The sound of the item as mirrored bars under its name. Quiet sound is lifted (square root) so it stays visible,
/// and the volume setting scales the bars.
pub(super) fn paint_wave(ui: &Ui, lay: &Layout, rect: Rect, item: &Item, peaks: &[f32], ink: Color32) {
    let area = Rect::from_min_max(
        pos2(rect.left(), rect.top() + 20.0),
        pos2(rect.right(), rect.bottom() - 4.0),
    );
    let visible = area.intersect(ui.clip_rect());
    let (mid, half) = (area.center().y, area.height() / 2.0);
    let (p, stroke) = (ui.painter_at(visible), Stroke::new(1.5, ink.gamma_multiply(0.75)));
    let mut x = (visible.left() / STEP).floor() * STEP; // aligned so bars do not shimmer while scrolling
    while x < visible.right() {
        let from = item.start + ((x - rect.left()) / lay.zoom) as f64;
        let local = ((x - rect.left()) / lay.zoom) as f64;
        let peak = waveform::peak_between(peaks, from, from + (STEP / lay.zoom) as f64);
        let level = (peak * item.volume * item.fade_gain_at(local)).clamp(0.0, 1.0); // the fades show in the bars
        let h = (level.sqrt() * half).max(0.5);
        p.line_segment([pos2(x + 1.0, mid - h), pos2(x + 1.0, mid + h)], stroke);
        x += STEP;
    }
}

/// Handles on a sound's block for its fades: drag the left one for the fade-in, the right one for the fade-out.
/// Returns `(fade_out?, seconds)` while one is being dragged.
pub(super) fn fade_handles(ui: &mut Ui, lay: &Layout, track: usize, item: &Item) -> Option<(bool, f32)> {
    let rect = lay.item_rect(track, item);
    let top = rect.top() + 20.0;
    let length = item.len() as f32 * lay.zoom;
    let (fade_in, fade_out) = (item.enter().len * lay.zoom, item.exit().len * lay.zoom);
    let p = ui.painter_at(lay.rect);
    let mut change = None;
    for (out, x) in [(false, rect.left() + fade_in), (true, rect.right() - fade_out)] {
        let x = x.clamp(rect.left() + 6.0, rect.right() - 6.0);
        let knob = Rect::from_center_size(pos2(x, top), vec2(11.0, 11.0));
        let resp = ui
            .interact(knob, Id::new(("fade", item.id, out)), Sense::drag())
            .on_hover_cursor(CursorIcon::ResizeHorizontal);
        // The ramp itself: a line from full volume down to silence (or up from it).
        let (from, to) = if out {
            (pos2(x, top), pos2(rect.right(), rect.bottom() - 4.0))
        } else {
            (pos2(rect.left(), rect.bottom() - 4.0), pos2(x, top))
        };
        p.line_segment([from, to], Stroke::new(1.5, ACCENT));
        p.rect_filled(
            knob,
            0.0,
            if resp.hovered() || resp.dragged() {
                ACCENT
            } else {
                WHITE
            },
        );
        p.rect_stroke(knob, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
        if let (true, Some(pos)) = (resp.dragged(), resp.interact_pointer_pos()) {
            let secs = if out { rect.right() - pos.x } else { pos.x - rect.left() } / lay.zoom;
            change = Some((out, secs.clamp(0.0, length / lay.zoom)));
        }
    }
    change
}
