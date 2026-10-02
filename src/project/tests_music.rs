//! Music: ending with the picture, fading out, and the caption transcript.

use super::tests::project;
use super::*;

fn music(len: f64, at: f64) -> Item {
    let mut i = Item::new("m.mp3".into(), "m".into(), Kind::Audio, (1, 1), len, true);
    i.at = at;
    i
}

#[test]
fn music_ends_with_the_picture_and_fades_out() {
    let (mut p, ..) = project(); // pictures end at 6 s
    assert!((p.visual_end() - 6.0).abs() < 1e-9);
    let long = p.add(1, music(30.0, 0.0));
    assert!((p.visual_end() - 6.0).abs() < 1e-9, "music does not extend the picture");
    assert!(p.fade_out_at_end(long, 2.0));
    let m = p.get(long).unwrap();
    assert!(
        (m.end_at() - 6.0).abs() < 1e-9 && (m.fade_out - 2.0).abs() < 1e-6,
        "cut at the picture, 2 s fade"
    );
}

#[test]
fn short_or_late_music_is_handled_sensibly() {
    let (mut p, ..) = project();
    let short = p.add(1, music(4.0, 0.0));
    assert!(p.fade_out_at_end(short, 1.5));
    assert!(
        (p.get(short).unwrap().len() - 4.0).abs() < 1e-9,
        "shorter than the picture: keeps its length"
    );
    let tiny = p.add(1, music(1.0, 5.0));
    assert!(p.fade_out_at_end(tiny, 3.0));
    assert!(p.get(tiny).unwrap().fade_out <= 1.0, "the fade never exceeds the sound");
    let late = p.add(1, music(5.0, 6.0));
    assert!(!p.fade_out_at_end(late, 1.0), "starts after the picture ends");
}

#[test]
fn fade_gain_follows_the_intro_and_outro() {
    let mut m = music(10.0, 0.0);
    (m.fade_in, m.fade_out) = (2.0, 4.0);
    let at = |t: f64| m.fade_gain_at(t);
    assert!(at(0.0) == 0.0 && (at(1.0) - 0.5).abs() < 1e-6 && at(3.0) == 1.0);
    assert!(at(6.0) == 1.0 && (at(8.0) - 0.5).abs() < 1e-6 && at(10.0) == 0.0);
    assert_eq!(music(5.0, 0.0).fade_gain_at(2.5), 1.0, "no fades, no change");
}

#[test]
fn the_transcript_is_every_caption_on_its_own_line_without_blanks() {
    let mut p = Project::default();
    assert_eq!(p.transcript(), "");
    let caption = |text: &str| Caption {
        start: 0.0,
        end: 1.0,
        text: text.into(),
    };
    p.captions = vec![caption("Hello there"), caption("  "), caption(" how are you? ")];
    assert_eq!(p.transcript(), "Hello there\nhow are you?");
}
