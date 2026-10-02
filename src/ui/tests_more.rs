//! More shortcut tests: tracks and full screen.

use super::tests::{app_with_a_clip, key};
use super::*;

#[test]
fn backspace_removes_a_selected_empty_track_but_never_one_with_clips_or_the_last() {
    use crate::app::Selection;
    let ctx = Context::default();
    let mut app = app_with_a_clip(&ctx); // track 0 holds a clip
    app.add_track(); // track 1, empty
    let backspace = |app: &mut App| {
        let _ = ctx.run_ui(key(Key::Backspace, true), |ui| shortcuts(&ui.ctx().clone(), app));
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

    let _ = ctx.run_ui(key(Key::Escape, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(!app.fullscreen && !app.playing(), "Esc leaves and pauses");
    assert_eq!(app.preview_size(), normal, "back to the editing size");

    let _ = ctx.run_ui(key(Key::F, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
    assert!(app.fullscreen, "F enters");
    let _ = ctx.run_ui(key(Key::F, true), |ui| shortcuts(&ui.ctx().clone(), &mut app));
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
