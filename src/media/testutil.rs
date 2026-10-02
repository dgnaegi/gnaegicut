//! Generates small real media files so tests exercise the actual ffmpeg paths without fixtures.

use super::{ffmpeg, probe::probe, run};
use crate::project::Item;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Unique per call: tests run in parallel and must never share (or half-read) a file.
fn path(name: &str) -> std::path::PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("gc_test_{}_{n}_{name}", std::process::id()))
}

/// A solid-colour video, optionally with a sine tone.
pub fn video(name: &str, color: &str, (w, h): (u32, u32), secs: u32, audio: bool) -> Item {
    let out = path(&format!("{name}.mp4"));
    let mut c = ffmpeg();
    c.args(["-f", "lavfi", "-i", &format!("color=c={color}:s={w}x{h}:r=30:d={secs}")]);
    if audio {
        c.args(["-f", "lavfi", "-i", &format!("sine=duration={secs}")]);
    }
    run(c.args(["-t", &secs.to_string(), "-pix_fmt", "yuv420p"]).arg(&out)).unwrap();
    probe(&out).unwrap()
}

pub fn image(name: &str, color: &str, (w, h): (u32, u32)) -> Item {
    let out = path(&format!("{name}.png"));
    let mut c = ffmpeg();
    c.args([
        "-f",
        "lavfi",
        "-i",
        &format!("color=c={color}:s={w}x{h}"),
        "-frames:v",
        "1",
    ]);
    run(c.arg(&out)).unwrap();
    probe(&out).unwrap()
}

/// RGB of the pixel at fraction (fx, fy) of an RGBA frame.
pub fn pixel(rgba: &[u8], (w, h): (u32, u32), fx: f32, fy: f32) -> [u8; 3] {
    let i = (((fy * h as f32) as usize) * w as usize + (fx * w as f32) as usize) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2]]
}

pub fn near(got: [u8; 3], want: [u8; 3]) -> bool {
    got.iter().zip(want).all(|(g, w)| (*g as i32 - w as i32).abs() < 40)
}

/// A sine-tone audio file (no picture).
pub fn sound(name: &str, secs: u32) -> Item {
    let out = path(&format!("{name}.wav"));
    let mut c = ffmpeg();
    c.args(["-f", "lavfi", "-i", &format!("sine=frequency=440:duration={secs}")]);
    run(c.arg(&out)).unwrap();
    probe(&out).unwrap()
}
