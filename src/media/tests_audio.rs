//! Sound-only items: music, effects and voice-over.

use super::export::export;
use super::frame::frame;
use super::graph::{self, Streams};
use super::probe::probe;
use super::stream::{self, SAMPLE_RATE};
use super::testutil::{near, pixel, sound, video};
use crate::project::{Kind, Project};

fn peak(pcm: &[f32], from: f64, to: f64) -> f32 {
    let at = |t: f64| (t * SAMPLE_RATE as f64) as usize * 2;
    pcm[at(from)..at(to)].iter().fold(0.0, |m, s| m.max(s.abs()))
}

#[test]
fn audio_files_are_probed_as_sound_only_items() {
    let item = sound("probe", 2);
    assert!(item.kind == Kind::Audio && item.has_audio && !item.kind.is_visual());
    assert!((item.len() - 2.0).abs() < 0.1, "length {}", item.len());
}

#[test]
fn a_sound_starts_where_it_is_placed_and_draws_nothing() {
    let mut p = Project::default();
    p.add(0, video("silent_blue", "blue", (108, 192), 4, false));
    let mut music = sound("music", 2);
    music.at = 1.5;
    p.add(1, music);

    let pcm = stream::audio(&p).unwrap();
    assert!(peak(&pcm, 0.0, 1.4) < 0.001, "silence before the sound starts");
    assert!(peak(&pcm, 1.7, 3.3) > 0.05, "audible while it plays");

    let f = frame(&p, 2.0, 108, 192).unwrap();
    assert!(
        near(pixel(&f, (108, 192), 0.5, 0.5), [0, 0, 255]),
        "the picture is just the video"
    );
}

#[test]
fn transcription_only_hears_videos_not_music() {
    let mut p = Project::default();
    let spoken = video("speech", "red", (64, 64), 2, true);
    let spoken_path = spoken.path.clone();
    p.add(0, spoken);
    let music = sound("bed", 3);
    let music_path = music.path.clone();
    p.add(1, music);

    let speech = graph::build(&p, 2, 2, Streams::Speech);
    assert!(speech.args.contains(&spoken_path) && !speech.args.contains(&music_path));
    let all = graph::build(&p, 2, 2, Streams::Audio);
    assert!(all.args.contains(&music_path), "but export and playback do mix it in");
}

#[test]
fn a_project_of_only_sound_exports_as_black_video_with_audio() {
    let mut p = Project::default();
    p.add(0, sound("only", 2));
    let out = std::env::temp_dir().join(format!("gc_sound_only_{}.mp4", std::process::id()));
    export(&p, &out).unwrap();
    let got = probe(&out).unwrap();
    assert!(got.has_audio && (got.src_len - 2.0).abs() < 0.3);
}

#[test]
fn a_music_fade_out_ramps_the_sound_down_to_nothing() {
    let mut p = Project::default();
    let mut m = sound("fading", 4);
    m.fade_out = 2.0;
    p.add(0, m);
    let pcm = stream::audio(&p).unwrap();
    assert!(peak(&pcm, 0.3, 1.5) > 0.05, "full level before the fade");
    assert!(
        peak(&pcm, 2.9, 3.2) < peak(&pcm, 2.0, 2.2) * 0.7,
        "quieter half way through the fade"
    );
    assert!(peak(&pcm, 3.9, 3.99) < 0.03, "silent at the end");
}
