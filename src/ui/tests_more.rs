//! More shortcut tests: tracks and full screen.

use super::tests::{RunQuiet, app_with_a_clip, key};
use super::*;

#[test]
fn backspace_removes_a_selected_empty_track_but_never_one_with_clips_or_the_last() {
    use crate::app::Selection;
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx); // track 0 holds a clip
    app.add_track(); // track 1, empty
    let backspace = |app: &mut App| {
        ctx.run_quiet(key(Key::Backspace, true), |ui| shortcuts(&ui.ctx().clone(), app));
    };
    app.select(Selection::Track(1));
    backspace(&mut app);
    assert_eq!(app.project.tracks.len(), 1, "the empty track is gone");
    assert!(app.selection == Selection::None);

    app.select(Selection::Track(0));
    backspace(&mut app);
    assert_eq!(app.project.tracks.len(), 1, "a track with a clip stays");
    assert_eq!(app.project.items().count(), 1, "and so does the clip");
    assert!(app.status.contains("empty"), "it says why: {}", app.status);

    let id = app.project.items().next().map(|i| i.id).unwrap();
    app.project.remove(id);
    app.select(Selection::Track(0));
    backspace(&mut app);
    assert_eq!(app.project.tracks.len(), 1, "the last track always stays");
}

#[test]
fn full_screen_plays_with_a_sharper_picture_and_esc_or_f_leaves_it() {
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    let normal = app.preview_size();
    app.set_fullscreen(true);
    assert!(app.fullscreen && app.playing(), "entering starts playing");
    assert!(app.preview_size().0 > normal.0, "and renders a larger picture");

    ctx.run_quiet(key(Key::Escape, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(!app.fullscreen && !app.playing(), "Esc leaves and pauses");
    assert_eq!(app.preview_size(), normal, "back to the editing size");

    ctx.run_quiet(key(Key::F, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(app.fullscreen, "F enters");
    ctx.run_quiet(key(Key::F, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(!app.fullscreen, "and leaves again");
}

#[test]
fn full_screen_with_nothing_to_play_just_shows_the_window() {
    let ctx = Context::default();
    let mut app = App::new(ctx.clone());
    app.set_fullscreen(true);
    assert!(app.fullscreen && !app.playing());
    app.set_fullscreen(true); // entering twice changes nothing
    assert!(app.fullscreen);
}

#[test]
fn a_space_typed_in_the_transcript_field_is_kept_and_does_not_split_by_itself() {
    use super::captions::transcript_field;
    use eframe::egui::{Event, Id, RawInput};
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    app.project.captions = vec![crate::project::Caption {
        start: 0.0,
        end: 2.0,
        text: "hello world".into(),
    }];
    ctx.run_quiet(RawInput::default(), |ui| {
        transcript_field(ui, &mut app);
        ui.memory_mut(|m| m.request_focus(Id::new("transcript-field")));
    });
    let typed = RawInput {
        events: vec![Event::Text(" ".into())],
        ..Default::default()
    };
    ctx.run_quiet(typed, |ui| transcript_field(ui, &mut app));
    let kept: String = ctx
        .data(|d| d.get_temp(Id::new("transcript-buffer")))
        .unwrap_or_default();
    assert_eq!(
        kept.len(),
        "hello world".len() + 1,
        "the typed space is still in the field: {kept:?}"
    );
    assert_eq!(app.project.captions.len(), 1, "one space changes nothing");
    assert_eq!(app.project.captions[0].text, "hello world");
}

#[test]
fn backspace_removes_only_the_selected_caption() {
    use crate::app::Selection;
    use crate::project::Caption;
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx);
    let caption = |text: &str, start, end| Caption {
        start,
        end,
        text: text.into(),
    };
    app.project.captions = vec![caption("a", 0.0, 1.0), caption("b", 1.0, 2.0), caption("c", 2.0, 3.0)];
    app.select(Selection::Caption(1));
    ctx.run_quiet(key(Key::Backspace, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert_eq!(
        app.project.captions.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
        ["a", "c"]
    );
    assert!(
        app.selection == Selection::None,
        "nothing stays selected that no longer exists"
    );
    assert_eq!(app.project.items().count(), 1, "the clips are untouched");
}
