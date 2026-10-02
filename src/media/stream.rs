//! Streams for real-time playback: raw frames and raw audio piped out of ffmpeg.

use super::ffmpeg;
use super::graph::{self, Streams};
use crate::project::Project;
use std::io::Read;
use std::process::Stdio;
use std::sync::mpsc::{Receiver, sync_channel};

pub const FPS: f64 = 30.0;
pub const SAMPLE_RATE: u32 = 48_000;

/// Decodes the composite to RGBA frames of `w`x`h`, `FPS` per second. Dropping the receiver stops ffmpeg.
pub fn video(p: &Project, w: u32, h: u32) -> Receiver<Vec<u8>> {
    let (tx, rx) = sync_channel(8); // bounded: ffmpeg decodes only slightly ahead of playback
    let g = graph::build(p, w, h, Streams::Video);
    let mut cmd = ffmpeg();
    cmd.args(&g.args)
        .args(["-filter_complex", &g.filter, "-map", graph::VIDEO]);
    cmd.args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"]);
    cmd.stdout(Stdio::piped()).stderr(Stdio::null());
    std::thread::spawn(move || {
        let Ok(mut child) = cmd.spawn() else { return };
        let Some(mut out) = child.stdout.take() else { return };
        loop {
            let mut frame = vec![0u8; (w * h * 4) as usize];
            if out.read_exact(&mut frame).is_err() || tx.send(frame).is_err() {
                break;
            }
        }
        let _ = child.kill();
        let _ = child.wait();
    });
    rx
}

/// The mixed audio track as interleaved stereo f32 at `SAMPLE_RATE`.
pub fn audio(p: &Project) -> Result<Vec<f32>, String> {
    let g = graph::build(p, 2, 2, Streams::Audio);
    let mut cmd = ffmpeg();
    cmd.args(&g.args)
        .args(["-filter_complex", &g.filter, "-map", graph::AUDIO]);
    cmd.args(["-f", "f32le", "-ar", &SAMPLE_RATE.to_string(), "-ac", "2", "-"]);
    let out = cmd.stderr(Stdio::piped()).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr)
            .lines()
            .last()
            .unwrap_or("audio failed")
            .into());
    }
    Ok(out
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect())
}

/// Decodes any audio (or video) file to interleaved stereo f32 at `SAMPLE_RATE`, at most `max_secs` long.
/// Used for sound previews.
pub fn decode_file(path: &std::path::Path, max_secs: u32) -> Result<Vec<f32>, String> {
    let mut cmd = ffmpeg();
    cmd.args(["-t", &max_secs.to_string(), "-i"]).arg(path);
    cmd.args(["-vn", "-f", "f32le", "-ar", &SAMPLE_RATE.to_string(), "-ac", "2", "-"]);
    let out = cmd.stderr(Stdio::piped()).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr)
            .lines()
            .last()
            .unwrap_or("could not decode")
            .into());
    }
    Ok(out
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect())
}
