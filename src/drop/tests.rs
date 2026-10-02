use super::*;

fn view() -> TimelineView {
    TimelineView {
        visible: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(800.0, 300.0)),
        time_origin: 100.0,
        zoom: 50.0,
        lanes: 3,
        lanes_top: 60.0,
        lane_pitch: 48.0,
    }
}

#[test]
fn the_top_row_is_the_highest_track_and_above_it_is_a_new_one() {
    let v = view();
    assert_eq!(v.lane_at(70.0), 2, "first row = top track");
    assert_eq!(v.lane_at(60.0 + 48.0 + 5.0), 1);
    assert_eq!(v.lane_at(60.0 + 48.0 * 2.0 + 5.0), 0);
    assert_eq!(v.lane_at(10.0), 3, "over the ruler: a new track");
    assert_eq!(v.lane_at(290.0), 0, "below the lanes stays on the lowest track");
}

#[test]
fn time_follows_the_zoom_and_never_goes_negative() {
    let v = view();
    assert!((v.time_at(100.0 + 125.0) - 2.5).abs() < 1e-9);
    assert_eq!(v.time_at(20.0), 0.0);
}

fn sound(id: &str) -> crate::sounds::Sound {
    crate::sounds::Sound {
        id: id.into(),
        title: "Whoosh".into(),
        creator: "Someone".into(),
        license: "CC0 1.0".into(),
        secs: 2.0,
        url: String::new(),
        ext: "mp3".into(),
        credit: "credit".into(),
    }
}

#[test]
fn a_dragged_sound_survives_being_asked_for_after_a_library_card() {
    // Regression: asking for the library payload first used to wipe a sound payload.
    let ctx = eframe::egui::Context::default();
    DragAndDrop::set_payload(&ctx, SoundDrag(sound("abc")));
    match take_dragged(&ctx) {
        Some(Dragged::Sound(s)) => assert_eq!(s.id, "abc"),
        _ => panic!("the sound was lost"),
    }
    assert!(!DragAndDrop::has_any_payload(&ctx), "the slot is cleared afterwards");
    assert!(take_dragged(&ctx).is_none());
}

#[test]
fn a_dragged_library_card_is_found_too() {
    let ctx = eframe::egui::Context::default();
    DragAndDrop::set_payload(&ctx, LibraryDrag(3));
    assert!(matches!(take_dragged(&ctx), Some(Dragged::Library(3))));
}

#[test]
fn a_downloaded_sound_lands_on_the_lane_and_time_it_was_dropped_at() {
    use crate::app::Event;
    use crate::project::{Item, Kind};
    let mut app = App::new(eframe::egui::Context::default());
    app.sounds.pending.insert("abc".into(), (2, 3.5));
    let mut item = Item::new("/tmp/x.mp3".into(), "Whoosh".into(), Kind::Audio, (1, 1), 2.0, true);
    item.credit = Some("credit".into());
    app.tx.send(Event::SoundItem("abc".into(), Box::new(item))).unwrap();
    app.tick();
    let placed = app
        .project
        .items()
        .find(|i| i.kind == Kind::Audio)
        .expect("the sound is on the timeline");
    assert_eq!(
        app.project.find(placed.id).map(|(t, _)| t),
        Some(2),
        "on the dropped lane"
    );
    assert!((placed.at - 3.5).abs() < 1e-9, "at the dropped time");
    assert_eq!(app.project.media.len(), 1, "and in the library, with its credit");
    assert_eq!(app.project.credits(), vec!["credit".to_string()]);
    assert!(app.sounds.pending.is_empty() && app.sounds.busy.is_empty());
}
