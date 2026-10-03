//! Pixel-level tests for transitions and rotation: what the user sees at specific moments.

use super::export::export;
use super::frame::frame;
use super::probe::probe;
use super::testutil::{image, near, pixel, video};
use crate::project::{Aspect, Item, Project, Transition};

const SIZE: (u32, u32) = (108, 192);
const RED: [u8; 3] = [255, 0, 0];
const BLACK: [u8; 3] = [0, 0, 0];

/// A red image that fills the 9:16 frame for 3 seconds, with the given transition lengths.
fn filling(transition: Transition, fade_in: f32, fade_out: f32) -> Project {
    let mut p = Project::default();
    let mut item = image("full", "red", (108, 192));
    (item.end, item.transition, item.fade_in, item.fade_out) = (3.0, transition, fade_in, fade_out);
    p.add(0, item);
    p
}

fn at(p: &Project, t: f64, fx: f32, fy: f32) -> [u8; 3] {
    pixel(&frame(p, t, SIZE.0, SIZE.1).unwrap(), SIZE, fx, fy)
}

#[test]
fn fade_dissolves_in_and_out() {
    let p = filling(Transition::Fade, 1.0, 1.0);
    let half = |c: [u8; 3]| (80..180).contains(&c[0]) && c[1] < 30;
    assert!(near(at(&p, 0.02, 0.5, 0.5), BLACK), "starts transparent");
    assert!(half(at(&p, 0.5, 0.5, 0.5)), "halfway through the fade-in");
    assert!(near(at(&p, 1.5, 0.5, 0.5), RED), "fully visible in the middle");
    assert!(half(at(&p, 2.5, 0.5, 0.5)), "halfway through the fade-out");
}

#[test]
fn fade_is_still_mid_way_when_playback_starts_inside_the_item() {
    // frame() re-bases the project at t, cutting off the item's front. The fade must keep running in item time.
    let p = filling(Transition::Fade, 1.0, 0.0);
    let mid = at(&p, 0.5, 0.5, 0.5);
    assert!((80..180).contains(&mid[0]), "expected a half-faded red, got {mid:?}");
    assert!(near(at(&p, 1.2, 0.5, 0.5), RED), "and fully visible after the fade");
}

#[test]
fn slide_left_enters_from_the_right_and_exits_to_the_left() {
    let p = filling(Transition::SlideLeft, 1.0, 1.0);
    assert!(
        near(at(&p, 0.02, 0.5, 0.5), BLACK),
        "starts fully off-screen to the right"
    );
    // Eased: at the halfway mark the image is 93.75% in, so only a thin strip on the left is still empty.
    assert!(
        near(at(&p, 0.5, 0.5, 0.5), RED) && near(at(&p, 0.5, 0.05, 0.5), BLACK),
        "mostly in halfway"
    );
    assert!(
        near(at(&p, 1.5, 0.1, 0.5), RED) && near(at(&p, 1.5, 0.9, 0.5), RED),
        "fully in"
    );
    assert!(
        near(at(&p, 2.5, 0.25, 0.5), RED) && near(at(&p, 2.5, 0.95, 0.5), BLACK),
        "leaving eases in: only 6.25% gone halfway"
    );
}

#[test]
fn slide_down_moves_vertically() {
    let p = filling(Transition::SlideDown, 1.0, 0.0);
    // Moving down: it enters from the top edge, so at the halfway point the upper half is covered.
    assert!(
        near(at(&p, 0.5, 0.5, 0.5), RED) && near(at(&p, 0.5, 0.5, 0.95), BLACK),
        "93.75% in halfway"
    );
}

/// A square image whose box is 54x54 px in the 108x192 preview, centred.
fn square(rotation: f32) -> Project {
    let mut p = Project::default();
    let mut item = image("square", "red", (200, 200));
    (item.scale, item.rotation, item.end) = (0.5, rotation, 2.0);
    p.add(0, item);
    p
}

#[test]
fn rotation_turns_the_corners_away() {
    // A point near the unrotated corner: inside the square, but outside the same square turned 45 degrees.
    let (fx, fy) = (0.5 + 24.0 / 108.0, 0.5 + 24.0 / 192.0);
    assert!(near(at(&square(0.0), 0.5, fx, fy), RED));
    assert!(
        near(at(&square(45.0), 0.5, fx, fy), BLACK),
        "corner is empty once rotated"
    );
    assert!(near(at(&square(45.0), 0.5, 0.5, 0.5), RED), "centre stays filled");
    // Along the diagonal the rotated square reaches further than the original box.
    let reach = 0.5 + 30.0 / 108.0;
    assert!(
        near(at(&square(45.0), 0.5, reach, 0.5), RED),
        "diamond tip pokes out past the box edge"
    );
    assert!(near(at(&square(0.0), 0.5, reach, 0.5), BLACK));
}

#[test]
fn exports_transitions_and_rotation_together() {
    let mut p = Project::default();
    let mut v: Item = video("motion", "blue", (320, 180), 3, true);
    (v.rotation, v.transition, v.fade_in, v.fade_out) = (-10.0, Transition::SlideRight, 0.5, 0.5);
    p.add(0, v);
    p.aspect = Aspect::Portrait;
    let out = std::env::temp_dir().join(format!("gc_motion_{}.mp4", std::process::id()));
    export(&p, &out).unwrap();
    let got = probe(&out).unwrap();
    assert_eq!((got.src_w, got.src_h), (1080, 1350));
    assert!((got.src_len - 3.0).abs() < 0.3);
}

#[test]
fn slam_in_overshoots_the_zoom_amount_before_settling() {
    let mut item = Item::new(
        "/x.mp4".into(),
        "x".into(),
        crate::project::Kind::Video,
        (160, 90),
        5.0,
        false,
    );
    (item.effect, item.amount) = (crate::project::ZoomEffect::Slam, 0.5);
    let expr = item.zoom_expr().unwrap();
    assert!(
        expr.contains("exp(") && expr.contains("cos("),
        "a damped spring: {expr}"
    );
}

#[test]
fn an_item_that_wipes_in_fades_out_instead_of_cutting() {
    let p = filling(Transition::WipeRight, 1.0, 1.0);
    assert!(near(at(&p, 0.02, 0.5, 0.5), BLACK), "nothing revealed at the start");
    assert!(near(at(&p, 1.5, 0.9, 0.5), RED), "fully revealed after the wipe");
    let leaving = at(&p, 2.5, 0.5, 0.5);
    assert!((80..180).contains(&leaving[0]), "half faded out, got {leaving:?}");
}

#[test]
fn a_zoom_entrance_fades_out_instead_of_cutting() {
    let p = filling(Transition::Zoom, 1.0, 1.0);
    let leaving = at(&p, 2.5, 0.5, 0.5);
    assert!((80..180).contains(&leaving[0]), "half faded out, got {leaving:?}");
}

#[test]
fn intro_and_outro_are_chosen_separately() {
    let mut p = filling(Transition::SlideLeft, 1.0, 1.0);
    p.tracks[0].items[0].outro = Some(Transition::Fade);
    assert!(
        near(at(&p, 1.5, 0.1, 0.5), RED) && near(at(&p, 1.5, 0.9, 0.5), RED),
        "in the middle"
    );
    let leaving = at(&p, 2.5, 0.5, 0.5);
    assert!(
        (80..180).contains(&leaving[0]),
        "fades out although it slid in, got {leaving:?}"
    );
    assert!(near(at(&p, 0.02, 0.5, 0.5), BLACK), "still slides in from the right");
}
