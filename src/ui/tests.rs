use super::*;
use crate::project::{Item, Kind, Project};
use eframe::egui::{Event, RawInput};

/// `run_ui` that discards the texture uploads of the frame: nothing renders them in a test, and egui panics when
/// a frame's output is dropped with uploads still pending.
pub(super) trait RunQuiet {
    fn run_quiet(&self, input: RawInput, ui: impl FnMut(&mut Ui));
}

impl RunQuiet for Context {
    fn run_quiet(&self, input: RawInput, ui: impl FnMut(&mut Ui)) {
        self.run_ui(input, ui).textures_delta.clear();
    }
}

pub(super) fn key(key: Key, pressed: bool) -> RawInput {
    let event = Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    RawInput {
        events: vec![event],
        ..Default::default()
    }
}

pub(super) fn app_with_a_clip(ctx: &Context) -> App {
    let mut app = App::new(ctx.clone());
    let clip = Item::new(
        "/nonexistent.mp4".into(),
        "x".into(),
        Kind::Video,
        (160, 90),
        5.0,
        false,
    );
    app.project.add(0, clip);
    app
}

#[test]
fn space_starts_and_stops_playback_and_is_consumed() {
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    let mut consumed = false;
    ctx.run_quiet(key(Key::Space, true), |ui| {
        shortcuts(&ui.ctx().clone(), &mut app);
        consumed = !ui.ctx().input(|i| i.key_pressed(Key::Space));
    });
    assert!(app.playing(), "Space starts playback");
    assert!(consumed, "and no widget gets to see the key afterwards");
    ctx.run_quiet(key(Key::Space, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(!app.playing(), "Space again pauses");
}

#[test]
fn space_still_works_after_touching_a_slider_or_other_focused_widget() {
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    let mut value = 0.5f32;
    ctx.run_quiet(RawInput::default(), |ui| {
        let slider = ui.add(eframe::egui::Slider::new(&mut value, 0.0..=1.0));
        slider.request_focus();
    });
    assert!(
        ctx.egui_wants_keyboard_input(),
        "egui counts a focused slider as wanting the keyboard"
    );
    ctx.run_quiet(key(Key::Space, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(app.playing(), "but Space must still play");
}

#[test]
fn space_is_left_alone_while_typing() {
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    ctx.run_quiet(RawInput::default(), |ui| {
        let mut text = String::new();
        let edit = ui.text_edit_singleline(&mut text);
        edit.request_focus();
    });
    ctx.run_quiet(key(Key::Space, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(!app.playing(), "a focused text field keeps its space bar");
}

#[test]
fn copy_and_paste_events_work_for_the_selected_clip() {
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    let id = app.project.items().next().map(|i| i.id).unwrap();
    app.select(crate::app::Selection::Item(id));
    let with = |event: Event| RawInput {
        events: vec![event],
        ..Default::default()
    };
    ctx.run_quiet(with(Event::Copy), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(app.clipboard.is_some(), "Cmd+C copies the selected clip");
    app.playhead = 7.0;
    ctx.run_quiet(with(Event::Paste(String::new())), |ui| {
        shortcuts(&ui.ctx().clone(), &mut app)
    });
    assert_eq!(app.project.items().count(), 2, "Cmd+V pastes it");
}

#[test]
fn copying_leaves_text_on_the_system_clipboard_so_the_paste_event_arrives() {
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    let id = app.project.items().next().map(|i| i.id).unwrap();
    app.select(crate::app::Selection::Item(id));
    let copy = RawInput {
        events: vec![Event::Copy],
        ..Default::default()
    };
    let mut output = ctx.run_ui(copy, |ui| shortcuts(&ui.ctx().clone(), &mut app));
    output.textures_delta.clear(); // nothing renders in a test
    let wrote_text = output
        .platform_output
        .commands
        .iter()
        .any(|c| matches!(c, eframe::egui::OutputCommand::CopyText(_)));
    assert!(
        wrote_text,
        "without text on the clipboard macOS never sends Cmd+V to the app"
    );
}

#[test]
fn cmd_z_undoes_and_shift_cmd_z_redoes_an_edit() {
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    app.history.reset(&Project::default()); // the clip counts as an edit made after the project was empty
    let key_with = |mods: Modifiers| {
        let event = Event::Key {
            key: Key::Z,
            physical_key: Some(Key::Z),
            pressed: true,
            repeat: false,
            modifiers: mods,
        };
        RawInput {
            events: vec![event],
            ..Default::default()
        }
    };
    ctx.run_quiet(key_with(Modifiers::COMMAND), |ui| {
        shortcuts(&ui.ctx().clone(), &mut app)
    });
    assert!(app.project.is_empty(), "Cmd+Z removed the clip");
    ctx.run_quiet(key_with(Modifiers::COMMAND | Modifiers::SHIFT), |ui| {
        shortcuts(&ui.ctx().clone(), &mut app)
    });
    assert_eq!(app.project.items().count(), 1, "Shift+Cmd+Z brought it back");
}
