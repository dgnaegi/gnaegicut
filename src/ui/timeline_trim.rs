//! Handles on the left and right edge of a block: drag them to shorten it, or to bring back a cut-off part.

use super::timeline_items::Layout;
use crate::project::Item;
use crate::theme::{ACCENT, BLACK, WHITE};
use eframe::egui::{CursorIcon, Id, Rect, Sense, Ui, pos2, vec2};

const GRIP: f32 = 8.0; // width of the grab area at each edge

/// Returns `(id, front?, new edge time)` while an edge is being dragged.
pub(super) fn handles(ui: &mut Ui, lay: &Layout, track: usize, item: &Item) -> Option<(u64, bool, f64)> {
    let rect = lay.item_rect(track, item);
    let p = ui.painter_at(lay.rect);
    let mut change = None;
    for front in [true, false] {
        let x = if front { rect.left() } else { rect.right() };
        let grip = Rect::from_center_size(pos2(x, rect.center().y), vec2(GRIP, rect.height()));
        let resp = ui
            .interact(grip, Id::new(("trim", item.id, front)), Sense::drag())
            .on_hover_cursor(CursorIcon::ResizeHorizontal);
        if resp.hovered() || resp.dragged() {
            let bar = Rect::from_center_size(pos2(x, rect.center().y), vec2(4.0, rect.height() * 0.5));
            p.rect_filled(bar, 0.0, if resp.dragged() { ACCENT } else { WHITE });
            p.rect_stroke(bar, 0.0, (1.0, BLACK), eframe::egui::StrokeKind::Outside);
        }
        if let (true, Some(pos)) = (resp.dragged(), resp.interact_pointer_pos()) {
            change = Some((item.id, front, lay.time(pos.x)));
        }
    }
    change
}
