//! Layout: toolbar on top, timeline below, inspector right, stage in the middle.

mod captions;
mod font_picker;
mod fullscreen;
mod inspector;
mod item_panel;
mod library;
mod sounds_panel;
mod stage;
mod stage_edit;
mod stage_geometry;
mod stage_guides;
mod stage_overlay;
mod stage_scale;
mod text_panel;
mod text_tab;
mod timeline;
mod timeline_audio;
mod timeline_captions;
mod timeline_gutter;
mod timeline_items;
mod timeline_joins;
mod timeline_parts;
mod timeline_tools;
mod timeline_trim;
mod timeline_zoom;
mod toolbar;
mod toolbar_icons;
mod transition_panel;

use crate::app::App;
use crate::drop::{Dragged, take_dragged};
use crate::theme::{ACCENT, BLACK, BORDER, PAD, WHITE, bold};
use eframe::egui::{
    Align2, Context, Event, Frame, Id, Key, LayerId, Margin, Modifiers, Order, Panel, Rect, Stroke, StrokeKind, Ui,
    vec2,
};

/// White panel with a thick black border: the visible grid.
fn frame() -> Frame {
    Frame::new()
        .fill(WHITE)
        .stroke(Stroke::new(BORDER, BLACK))
        .inner_margin(Margin::same(PAD as i8))
}

/// Keyboard shortcuts. A key is *consumed* when it is used, so a button that happens to have keyboard focus does
/// not also react to it (Space on a focused button would otherwise press the button and toggle playback at once).
fn shortcuts(ctx: &Context, app: &mut App) {
    // Only typing in a text field silences the shortcuts. `egui_wants_keyboard_input` is true for *any* focused
    // widget (a slider you just dragged, a drop-down), which made Space work only now and then.
    if ctx.text_edit_focused() {
        return;
    }
    let pressed = |mods: Modifiers, key: Key| ctx.input_mut(|i| i.consume_key(mods, key));
    // On macOS egui reports Cmd+C / Cmd+X / Cmd+V as copy, cut and paste events instead of key presses.
    let event = |wanted: fn(&Event) -> bool| ctx.input(|i| i.events.iter().any(wanted));
    if event(|e| matches!(e, Event::Copy)) || pressed(Modifiers::COMMAND, Key::C) {
        return app.copy_selection();
    }
    if event(|e| matches!(e, Event::Cut)) || pressed(Modifiers::COMMAND, Key::X) {
        return app.cut_selection();
    }
    if event(|e| matches!(e, Event::Paste(_))) || pressed(Modifiers::COMMAND, Key::V) {
        return app.paste();
    }
    // The most specific first: Shift+Cmd+Z before Cmd+Z, Shift+Cmd+S before Cmd+S.
    if pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z) || pressed(Modifiers::COMMAND, Key::Y) {
        app.redo();
    } else if pressed(Modifiers::COMMAND, Key::Z) {
        app.undo();
    } else if pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::S) {
        app.save_as();
    } else if pressed(Modifiers::COMMAND, Key::S) {
        app.save();
    } else if pressed(Modifiers::COMMAND, Key::N) {
        app.new_project();
    } else if pressed(Modifiers::COMMAND, Key::O) {
        app.open();
    } else if pressed(Modifiers::COMMAND, Key::Plus) || pressed(Modifiers::COMMAND, Key::Equals) {
        app.zoom_timeline(1.5, None);
    } else if pressed(Modifiers::COMMAND, Key::Minus) {
        app.zoom_timeline(1.0 / 1.5, None);
    } else if pressed(Modifiers::COMMAND, Key::Num0) {
        app.fit_timeline();
    } else if pressed(Modifiers::NONE, Key::Escape) && app.fullscreen {
        app.set_fullscreen(false);
    } else if pressed(Modifiers::NONE, Key::F) {
        app.set_fullscreen(!app.fullscreen);
    } else if pressed(Modifiers::NONE, Key::Space) {
        app.toggle_play();
    } else if pressed(Modifiers::NONE, Key::S) {
        app.split();
    } else if pressed(Modifiers::NONE, Key::Delete) || pressed(Modifiers::NONE, Key::Backspace) {
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
    let accept = pointer.is_some_and(over_work_area);
    let released = ctx.input(|i| i.pointer.any_released());
    let dragged = if released { take_dragged(ctx) } else { None };
    match dragged.filter(|_| accept) {
        Some(Dragged::Library(index)) => app.add_from_library(index, pointer),
        Some(Dragged::Sound(sound)) => app.drop_sound(sound, pointer),
        None => {}
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
    let how = match (count > 1, app.drops_stack(Some(pointer))) {
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
    app.confirm_new_dialog(&ctx);
    if app.fullscreen {
        return fullscreen::show(ui, app);
    }
    Panel::top("toolbar")
        .frame(frame())
        .show(ui, |ui| toolbar::show(ui, app));
    Panel::bottom("timeline")
        .frame(frame())
        .resizable(false) // the grip inside the panel does the resizing and the lanes follow
        .exact_size(app.timeline_height)
        .show(ui, |ui| timeline::show(ui, app));
    Panel::right("inspector")
        .frame(frame())
        .default_size(340.0)
        .show(ui, |ui| inspector::show(ui, app));
    Panel::left("library")
        .frame(frame())
        .default_size(280.0)
        .show(ui, |ui| library::show(ui, app));
    stage::show(ui, app);
    drag_and_drop(&ctx, app);
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_more;
