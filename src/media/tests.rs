use super::export::export;
use super::frame::frame;
use super::probe::probe;
use super::stream::{self, SAMPLE_RATE};
use super::testutil::{image, near, pixel, video};
use crate::project::{Aspect, Caption, Kind, Project, ZoomEffect};

/// A blue 16:9 video for 4s with a red image overlaid on track 2 from 1s to 3s, up-left of centre.
fn layered() -> Project {
    let mut p = Project::default();
    p.add(0, video("blue", "blue", (640, 360), 4, true));
    let mut red = image("red", "red", (200, 200));
    (red.at, red.end, red.x, red.y, red.scale) = (1.0, 2.0, 0.25, 0.25, 0.3);
    p.add(1, red);
    p
}

#[test]
fn image_overlay_appears_only_during_its_time_and_place() {
    let p = layered();
    let size = (108, 192);
    let at = |t: f64, fx: f32, fy: f32| pixel(&frame(&p, t, size.0, size.1).unwrap(), size, fx, fy);
    assert!(near(at(0.5, 0.5, 0.5), [0, 0, 255]), "video visible in the middle");
    assert!(
        near(at(0.5, 0.25, 0.25), [0, 0, 0]),
        "letterbox is black before the image starts"
    );
    assert!(near(at(2.0, 0.25, 0.25), [255, 0, 0]), "image drawn at its position");
    assert!(near(at(3.5, 0.25, 0.25), [0, 0, 0]), "image gone after its end");
    assert!(near(at(2.0, 0.5, 0.5), [0, 0, 255]), "rest of the video untouched");
}

#[test]
fn exports_layers_with_zoom_effects_and_captions() {
    let mut p = layered();
    for item in p.tracks.iter_mut().flat_map(|t| t.items.iter_mut()) {
        item.effect = if item.kind == Kind::Video {
            ZoomEffect::In
        } else {
            ZoomEffect::Pulse
        };
        item.zoom = 1.2;
    }
    p.aspect = Aspect::Portrait;
    p.captions = vec![Caption {
        start: 0.5,
        end: 3.0,
        text: "Hello world".into(),
    }];
    p.caption_layout.family = "Helvetica".into();
    let out = std::env::temp_dir().join("gc_layers.mp4");
    export(&p, &out).unwrap();
    let got = probe(&out).unwrap();
    assert_eq!((got.src_w, got.src_h), (1080, 1350));
    assert!((got.src_len - 4.0).abs() < 0.2, "duration {}", got.src_len);
    assert!(got.has_audio);
}

#[test]
fn zoom_punches_in_over_time() {
    let mut p = Project::default();
    let mut v = video("zoom", "black", (640, 360), 2, false);
    v.scale = v.cover_scale(Aspect::Wide);
    p.aspect = Aspect::Wide;
    p.add(0, v);
    let mut stamp = image("white", "white", (64, 64));
    (stamp.at, stamp.end, stamp.scale) = (0.0, 2.0, 0.2);
    p.add(0, stamp); // a white square drawn into the same layer proves overlays survive zoom
    let size = (192, 108);
    let f = frame(&p, 0.5, size.0, size.1).unwrap();
    assert!(near(pixel(&f, size, 0.5, 0.5), [255, 255, 255]));
}

#[test]
fn streams_30fps_video_and_matching_audio() {
    let mut p = Project::default();
    p.add(0, video("stream", "green", (320, 180), 2, true));
    let frames = stream::video(&p, 90, 160).iter().count();
    assert!((59..=61).contains(&frames), "frames: {frames}");
    let samples = stream::audio(&p).unwrap().len();
    let expected = 2 * SAMPLE_RATE as usize * 2;
    assert!(
        samples.abs_diff(expected) < SAMPLE_RATE as usize / 10,
        "samples: {samples}"
    );
}

#[test]
fn audio_is_delayed_to_the_items_position() {
    let mut p = Project::default();
    let mut v = video("late", "red", (160, 90), 1, true);
    v.at = 2.0;
    p.add(0, v);
    let pcm = stream::audio(&p).unwrap();
    let loud = |from: f64, to: f64| {
        let (a, b) = (
            (from * SAMPLE_RATE as f64) as usize * 2,
            (to * SAMPLE_RATE as f64) as usize * 2,
        );
        pcm[a..b].iter().fold(0f32, |m, s| m.max(s.abs()))
    };
    assert!(loud(0.0, 1.9) < 0.001, "silent before the item starts");
    // the mono sine is upmixed to stereo (-3 dB), so it peaks near 0.09
    assert!(loud(2.1, 2.9) > 0.05, "audible while it plays");
}

#[test]
fn dropping_receiver_stops_the_stream() {
    let mut p = Project::default();
    p.add(0, video("stop", "green", (320, 180), 2, false));
    let rx = stream::video(&p, 90, 160);
    assert!(rx.recv().is_ok());
    drop(rx); // must not hang or leak the ffmpeg process
}

/// Peak level of the item's audio between `from` and `to` seconds, after the full graph.
fn peak(p: &Project, from: f64, to: f64) -> f32 {
    let pcm = stream::audio(p).unwrap();
    let at = |t: f64| (t * SAMPLE_RATE as f64) as usize * 2;
    pcm[at(from)..at(to)].iter().fold(0.0, |m, s| m.max(s.abs()))
}

fn tone(configure: impl FnOnce(&mut crate::project::Item)) -> Project {
    let mut p = Project::default();
    let mut v = video("tone", "red", (160, 90), 4, true);
    configure(&mut v);
    p.add(0, v);
    p
}

#[test]
fn enhance_voice_raises_quiet_audio_to_target_loudness() {
    let plain = peak(&tone(|_| {}), 1.0, 3.0);
    let enhanced = peak(&tone(|v| v.enhance = true), 1.0, 3.0);
    assert!(enhanced > plain * 1.5, "plain {plain}, enhanced {enhanced}");
    assert!(enhanced < 1.0, "must not clip: {enhanced}");
}

#[test]
fn volume_scales_and_zero_mutes() {
    let loud = peak(&tone(|_| {}), 1.0, 3.0);
    assert!(peak(&tone(|v| v.volume = 0.5), 1.0, 3.0) < loud * 0.6);
    assert!(peak(&tone(|v| v.volume = 0.0), 1.0, 3.0) < 0.001);
}

#[test]
fn fades_ramp_the_start_and_end() {
    let p = tone(|v| (v.fade_in, v.fade_out) = (1.5, 1.5));
    assert!(peak(&p, 0.0, 0.3) < 0.04, "starts silent");
    let mid = peak(&p, 1.8, 2.2);
    assert!(mid > 0.07, "full level in the middle: {mid}");
    assert!(peak(&p, 3.8, 3.99) < 0.03, "ends silent");
}
