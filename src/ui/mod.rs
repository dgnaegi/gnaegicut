//! Layout: toolbar on top, timeline below, inspector right, stage in the middle.

mod captions;
mod font_picker;
mod inspector;
mod item_panel;
mod library;
mod sounds_panel;
mod stage;
mod stage_edit;
mod stage_geometry;
mod stage_guides;
mod stage_overlay;
mod text_panel;
mod timeline;
mod timeline_gutter;
mod timeline_items;
mod timeline_joins;
mod toolbar;
mod transition_panel;

use crate::app::App;
use crate::drop::LibraryDrag;
use crate::theme::{ACCENT, BLACK, BORDER, PAD, WHITE, bold};
use eframe::egui::{
    Align2, Context, DragAndDrop, Frame, Id, Key, LayerId, Margin, Order, Panel, Rect, Stroke, StrokeKind, Ui, vec2,
};

/// White panel with a thick black border: the visible grid.
fn frame() -> Frame {
    Frame::new()
        .fill(WHITE)
        .stroke(Stroke::new(BORDER, BLACK))
        .inner_margin(Margin::same(PAD as i8))
}

fn shortcuts(ctx: &Context, app: &mut App) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let pressed = |k| ctx.input(|i| i.key_pressed(k));
    if ctx.input(|i| i.modifiers.command) {
        // Cmd+S saves, Shift+Cmd+S saves as, Cmd+O opens. Plain S below must not fire for these.
        if pressed(Key::S) {
            if ctx.input(|i| i.modifiers.shift) {
                app.save_as()
            } else {
                app.save()
            }
        } else if pressed(Key::O) {
            app.open();
        }
        return;
    }
    if pressed(Key::Space) {
        app.toggle_play();
    }
    if pressed(Key::S) {
        app.split();
    }
    if pressed(Key::Delete) || pressed(Key::Backspace) {
        app.delete();
    }
}

/// Media dropped onto the window, or a library entry dragged out of the library, is placed where it was released.
/// While dragging, a frame and a label say what will happen; the timeline highlights the target lanes.
fn drag_and_drop(ctx: &Context, app: &mut App) {
    let pointer = ctx.pointer_latest_pos();
    let files: Vec<_> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
    if !files.is_empty() {
        app.add_files(&files, pointer);
    }
    let over_work_area =
        |p| app.timeline_view.is_some_and(|v| v.contains(p)) || app.stage_rect.is_some_and(|r| r.contains(p));
    let released = ctx.input(|i| i.pointer.any_released());
    let dragged = if released {
        DragAndDrop::take_payload::<LibraryDrag>(ctx)
    } else {
        None
    };
    if let Some(drag) = dragged.filter(|_| pointer.is_some_and(over_work_area)) {
        app.add_from_library(drag.0, pointer);
    }
    cue(ctx, app);
}

fn cue(ctx: &Context, app: &App) {
    let Some((count, pointer)) = app.drag_hover() else {
        return;
    };
    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("drop")));
    let area = ctx.content_rect();
    painter.rect_stroke(area, 0.0, Stroke::new(8.0, ACCENT), StrokeKind::Inside);
    let how = match (count > 1, app.stack_drops) {
        (false, _) => "ONE CLIP",
        (true, true) => "STACKED AS LAYERS",
        (true, false) => "ONE AFTER ANOTHER",
    };
    let label = format!("DROP {count} · {how}");
    let at = pointer + vec2(18.0, 18.0);
    let size = painter.layout_no_wrap(label.clone(), bold(13.0), WHITE).size();
    painter.rect_filled(Rect::from_min_size(at, size + vec2(20.0, 14.0)), 0.0, ACCENT);
    painter.text(at + vec2(10.0, 7.0), Align2::LEFT_TOP, label, bold(13.0), WHITE);
}

pub(crate) use timeline_items::snap;

pub fn draw(ui: &mut Ui, app: &mut App) {
    app.tick();
    let ctx = ui.ctx().clone();
    shortcuts(&ctx, app);
    Panel::top("toolbar")
        .frame(frame())
        .show(ui, |ui| toolbar::show(ui, app));
    Panel::bottom("timeline")
        .frame(frame())
        .min_size(230.0)
        .show(ui, |ui| timeline::show(ui, app));
    Panel::right("inspector")
        .frame(frame())
        .default_size(340.0)
        .show(ui, |ui| inspector::show(ui, app));
    if app.library_open {
        Panel::left("library")
            .frame(frame())
            .default_size(280.0)
            .show(ui, |ui| library::show(ui, app));
    }
    stage::show(ui, app);
    drag_and_drop(&ctx, app);
}
