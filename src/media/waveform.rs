//! The loudness outline of a file's sound, for drawing on the timeline.

use super::ffmpeg;
use std::process::Stdio;

/// Buckets per second of audio.
pub const PER_SEC: usize = 25;
const RATE: usize = 5000;

/// Linear peak (0..1) of every bucket of `size` samples.
pub fn bucket(samples: &[i16], size: usize) -> Vec<f32> {
    samples
        .chunks(size.max(1))
        .map(|c| c.iter().map(|s| s.unsigned_abs() as f32).fold(0.0, f32::max) / 32768.0)
        .collect()
}

/// The peaks of a file's first audio stream, `PER_SEC` per second. Empty if the file has no sound.
pub fn peaks(path: &str) -> Result<Vec<f32>, String> {
    let out = ffmpeg()
        .args([
            "-i",
            path,
            "-vn",
            "-ac",
            "1",
            "-ar",
            &RATE.to_string(),
            "-f",
            "s16le",
            "-",
        ])
        .stderr(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    let samples: Vec<i16> = out
        .stdout
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_le_bytes(*b))
        .collect();
    Ok(bucket(&samples, RATE / PER_SEC))
}

/// The loudest bucket between `from` and `to` seconds (so zoomed-out timelines show peaks, not averages).
pub fn peak_between(peaks: &[f32], from: f64, to: f64) -> f32 {
    let first = (from.max(0.0) * PER_SEC as f64) as usize;
    let last = ((to.max(0.0) * PER_SEC as f64) as usize).max(first);
    (first..=last).filter_map(|i| peaks.get(i)).fold(0.0, |m, p| m.max(*p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::testutil::{sound, video};

    #[test]
    fn buckets_keep_the_loudest_sample_of_each_slice() {
        assert_eq!(bucket(&[0, 16384, -32768, 0, 100], 2), vec![0.5, 1.0, 100.0 / 32768.0]);
        assert!(bucket(&[], 4).is_empty());
    }

    #[test]
    fn peak_between_finds_the_maximum_in_the_range() {
        let p = [0.1, 0.9, 0.2, 0.3];
        assert_eq!(peak_between(&p, 0.0, 0.05), 0.9, "buckets are 40 ms wide");
        assert_eq!(peak_between(&p, 0.08, 0.2), 0.3, "past the end is ignored");
        assert_eq!(peak_between(&p, 5.0, 6.0), 0.0);
    }

    #[test]
    fn a_two_second_tone_gives_about_fifty_loud_buckets_and_silence_gives_none() {
        let tone = peaks(&sound("wave_tone", 2).path).unwrap();
        assert!(
            (tone.len() as i64 - 2 * PER_SEC as i64).abs() <= 2,
            "buckets: {}",
            tone.len()
        );
        assert!(
            tone.iter().skip(2).all(|p| *p > 0.05),
            "a steady tone is audible in every bucket"
        );
        let quiet = peaks(&video("wave_silent", "red", (64, 64), 1, false).path).unwrap();
        assert!(quiet.is_empty(), "no audio stream, no peaks");
    }
}
