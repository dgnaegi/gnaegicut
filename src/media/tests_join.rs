//! Pixel tests for transitions between two clips: A is red, B is blue, both fill the frame.

use super::frame::frame;
use super::testutil::{image, near, pixel};
use crate::project::{Join, JoinEffect, Project};

pub(super) const SIZE: (u32, u32) = (108, 192);
pub(super) const RED: [u8; 3] = [255, 0, 0];
pub(super) const BLUE: [u8; 3] = [0, 0, 255];
pub(super) const BLACK: [u8; 3] = [0, 0, 0];

/// Red for 3 s, then blue for 3 s, joined by `effect` over `len` seconds.
pub(super) fn joined(effect: JoinEffect, len: f32) -> Project {
    let mut p = Project::default();
    let mut a = image("join_a", "red", SIZE);
    a.end = 3.0;
    let mut b = image("join_b", "blue", SIZE);
    (b.end, b.at) = (3.0, 3.0);
    p.add(0, a);
    let b = p.add(0, b);
    p.set_join(b, Some(Join { effect, len }));
    p
}

pub(super) fn at(p: &Project, t: f64, fx: f32, fy: f32) -> [u8; 3] {
    pixel(&frame(p, t, SIZE.0, SIZE.1).unwrap(), SIZE, fx, fy)
}

#[test]
fn dissolve_blends_the_two_clips() {
    let p = joined(JoinEffect::Dissolve, 1.0); // overlap 2..3
    assert!(near(at(&p, 1.0, 0.5, 0.5), RED));
    let mid = at(&p, 2.5, 0.5, 0.5);
    assert!(mid[0] > 70 && mid[2] > 70, "both colours present mid-way: {mid:?}");
    assert!(near(at(&p, 3.5, 0.5, 0.5), BLUE));
}

#[test]
fn wipe_right_reveals_the_next_clip_from_the_left() {
    let p = joined(JoinEffect::WipeRight, 1.0);
    assert!(near(at(&p, 1.0, 0.5, 0.5), RED), "before the transition");
    // Halfway in time is 93.75% revealed: the movement eases out (fast start, soft landing).
    assert!(
        near(at(&p, 2.5, 0.25, 0.5), BLUE) && near(at(&p, 2.5, 0.95, 0.5), RED),
        "mostly revealed halfway"
    );
    assert!(
        near(at(&p, 2.1, 0.5, 0.5), RED),
        "a tenth in, the middle is still the old clip"
    );
    assert!(
        near(at(&p, 3.4, 0.25, 0.5), BLUE) && near(at(&p, 3.4, 0.75, 0.5), BLUE),
        "afterwards"
    );
}

#[test]
fn wipe_down_reveals_from_the_top() {
    let p = joined(JoinEffect::WipeDown, 1.0);
    assert!(
        near(at(&p, 2.5, 0.5, 0.25), BLUE) && near(at(&p, 2.5, 0.5, 0.95), RED),
        "93.75% revealed halfway"
    );
}

#[test]
fn push_left_slides_the_old_clip_out_and_the_new_one_in() {
    let p = joined(JoinEffect::PushLeft, 1.0);
    // Eased: the new clip is 93.75% in at the halfway mark and covers the old one, which is only 6.25% out.
    assert!(
        near(at(&p, 2.5, 0.05, 0.5), RED) && near(at(&p, 2.5, 0.5, 0.5), BLUE),
        "the new clip has taken most of the frame"
    );
    assert!(near(at(&p, 3.5, 0.1, 0.5), BLUE) && near(at(&p, 3.5, 0.9, 0.5), BLUE));
}

#[test]
fn circle_opens_from_the_centre() {
    let p = joined(JoinEffect::Circle, 1.0);
    assert!(near(at(&p, 2.5, 0.5, 0.5), BLUE), "centre is revealed first");
    assert!(near(at(&p, 2.5, 0.02, 0.02), RED), "corner still shows the old clip");
    assert!(near(at(&p, 3.2, 0.05, 0.05), BLUE), "and finally the corner too");
}

#[test]
fn dip_goes_through_black_without_overlapping() {
    let p = joined(JoinEffect::Dip, 1.0); // A fades out 2.5..3, B fades in 3..3.5
    assert!((p.total() - 6.0).abs() < 1e-9, "no overlap, so the length is unchanged");
    assert!(near(at(&p, 2.0, 0.5, 0.5), RED));
    assert!(near(at(&p, 3.0, 0.5, 0.5), BLACK), "fully dark at the cut");
    assert!(near(at(&p, 4.0, 0.5, 0.5), BLUE));
}

#[test]
fn zoom_fades_the_new_clip_in() {
    let p = joined(JoinEffect::Zoom, 1.0);
    let mid = at(&p, 2.5, 0.5, 0.5);
    assert!(mid[0] > 40 && mid[2] > 40, "cross-fading while it settles: {mid:?}");
    assert!(near(at(&p, 3.5, 0.5, 0.5), BLUE));
}

#[test]
fn every_effect_renders_in_the_middle_of_its_transition_and_when_playing_from_it() {
    for effect in JoinEffect::ALL {
        let p = joined(effect, 1.0);
        let mid = if effect == JoinEffect::Dip { 2.9 } else { 2.5 };
        assert!(frame(&p, mid, SIZE.0, SIZE.1).is_some(), "{} failed", effect.label());
    }
}

#[test]
fn starting_playback_inside_a_wipe_matches_the_paused_frame() {
    // frame() re-bases at t, so item time continues from the cut: the wipe must still be half way at 2.5 s.
    let p = joined(JoinEffect::WipeRight, 1.0);
    let left = at(&p, 2.5, 0.25, 0.5);
    let right = at(&p, 2.5, 0.95, 0.5);
    assert!(near(left, BLUE) && near(right, RED), "{left:?} / {right:?}");
}
