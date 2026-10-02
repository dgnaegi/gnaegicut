//! The punchy effects, measured on rendered frames: flash, glitch, whip, spin, shake and looks.

use super::frame::frame;
use super::stream;
use super::tests_join::{BLACK, BLUE, RED, SIZE, at, joined};
use super::testutil::{image, near, pixel};
use crate::project::{JoinEffect, Look, Project};

fn bright(c: [u8; 3]) -> bool {
    c.iter().all(|v| *v > 150)
}

#[test]
fn flash_burns_out_to_white_at_the_cut_and_settles_into_the_next_clip() {
    let p = joined(JoinEffect::Flash, 1.0); // A fades to white 2.5..3, B starts white and settles 3..3.5
    assert!(
        (p.total() - 6.0).abs() < 1e-9,
        "a flash does not overlap, so the length stays"
    );
    assert!(near(at(&p, 2.0, 0.5, 0.5), RED), "before the flash");
    assert!(
        bright(at(&p, 2.93, 0.5, 0.5)),
        "going to white: {:?}",
        at(&p, 2.93, 0.5, 0.5)
    );
    assert!(
        bright(at(&p, 3.02, 0.5, 0.5)),
        "still white right after the cut: {:?}",
        at(&p, 3.02, 0.5, 0.5)
    );
    assert!(near(at(&p, 4.0, 0.5, 0.5), BLUE), "settled into the new clip");
}

#[test]
fn glitch_breaks_up_the_picture_only_during_its_window() {
    let p = joined(JoinEffect::Glitch, 1.0); // glitching 2.5..3 on A and 3..3.5 on B
    let spread = |t: f64| {
        let px = frame(&p, t, SIZE.0, SIZE.1).unwrap();
        let samples: Vec<[u8; 3]> = (0..40)
            .map(|i| pixel(&px, SIZE, 0.1 + i as f32 * 0.02, 0.4 + (i % 5) as f32 * 0.05))
            .collect();
        (0..3)
            .map(|c| samples.iter().map(|s| s[c]).max().unwrap() - samples.iter().map(|s| s[c]).min().unwrap())
            .max()
            .unwrap()
    };
    assert!(spread(4.0) < 12, "clean blue outside the window");
    assert!(
        spread(3.1) > 25,
        "noise and colour splitting inside it: {}",
        spread(3.1)
    );
}

#[test]
fn whip_pushes_like_a_slide_and_ends_clean() {
    let p = joined(JoinEffect::WhipLeft, 0.4);
    let mid = at(&p, 2.8, 0.5, 0.5); // overlap 2.6..3.0; most of the new clip is in
    assert!(
        mid[2] > 120 && mid[0] < 140,
        "mostly the new blue clip, smeared by the blur: {mid:?}"
    );
    assert!(
        near(at(&p, 3.5, 0.5, 0.5), BLUE) && near(at(&p, 3.5, 0.1, 0.5), BLUE),
        "no blur or offset afterwards"
    );
}

#[test]
fn spin_fades_the_new_clip_in_while_it_turns() {
    let p = joined(JoinEffect::Spin, 1.0);
    let early = at(&p, 2.1, 0.5, 0.5);
    let mid = at(&p, 2.5, 0.5, 0.5);
    assert!(
        early[0] > 190 && early[2] < 80,
        "mostly the old clip at first: {early:?}"
    );
    assert!(mid[0] > 40 && mid[2] > 40, "both clips visible half way: {mid:?}");
    assert!(
        near(at(&p, 3.5, 0.5, 0.5), BLUE) && near(at(&p, 3.5, 0.05, 0.05), BLUE),
        "upright and full-frame at the end"
    );
}

/// A small red square in a black 9:16 frame, optionally shaking.
fn square(shake: f32, look: Look) -> Project {
    let mut p = Project::default();
    let mut item = image("fx_square", "red", (200, 200));
    (item.scale, item.end, item.shake, item.look) = (0.5, 3.0, shake, look);
    p.add(0, item);
    p
}

/// Whether the red square covers the probe pixel (on its left edge) in each of the first 24 streamed frames.
fn probes(p: &Project) -> Vec<bool> {
    let frames: Vec<Vec<u8>> = stream::video(p, SIZE.0, SIZE.1).iter().take(24).collect();
    frames
        .iter()
        .map(|px| near(pixel(px, SIZE, 27.5 / 108.0, 0.5), RED))
        .collect()
}

#[test]
fn shake_moves_the_picture_around_while_a_still_one_stays_put() {
    // The left edge of the 54 px square sits at x = 27, so that pixel flips as the square wanders either way.
    assert!(
        probes(&square(0.0, Look::None)).iter().all(|inside| *inside),
        "no shake: always inside"
    );
    let moving = probes(&square(1.0, Look::None));
    assert!(
        moving.contains(&true) && moving.contains(&false),
        "the edge wanders past the probe: {moving:?}"
    );
}

#[test]
fn looks_change_the_colours() {
    let grey = at(&square(0.0, Look::Mono), 0.5, 0.5, 0.5);
    assert!(
        grey[0].abs_diff(grey[1]) < 25 && grey[0].abs_diff(grey[2]) < 25,
        "mono is grey: {grey:?}"
    );
    let plain = at(&square(0.0, Look::None), 0.5, 0.5, 0.5);
    assert!(near(plain, RED));
    let outside = at(&square(0.0, Look::Vignette), 0.5, 0.05, 0.05);
    assert!(near(outside, BLACK), "the vignette does not paint outside the picture");
}

#[test]
fn a_reversed_clip_still_renders_picture_and_sound() {
    use super::testutil::video;
    let mut p = Project::default();
    let id = p.add(0, video("rev", "red", (320, 180), 2, true));
    p.get_mut(id).unwrap().reversed = true;
    assert!(frame(&p, 1.0, 90, 160).is_some(), "the reverse filter chain is valid");
    let samples = stream::audio(&p).unwrap();
    assert!(samples.iter().any(|s| s.abs() > 0.05), "areverse keeps the tone");
}
