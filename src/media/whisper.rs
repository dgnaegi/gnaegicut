//! Auto-captions: render the timeline's audio, run whisper.cpp on it, parse the SRT it writes.

use super::graph::{self, Streams};
use super::{asset, ffmpeg, run};
use crate::project::{Caption, Project};
use std::{fs, process::Command};

pub fn model_path() -> String {
    asset("models/ggml-base.bin")
}

pub fn transcribe(p: &Project) -> Result<Vec<Caption>, String> {
    if !std::path::Path::new(&model_path()).exists() {
        return Err(format!(
            "Whisper model missing: run scripts/get-model.sh ({})",
            model_path()
        ));
    }
    let dir = std::env::temp_dir();
    let wav = dir.join("gnaegicut-audio.wav");
    let base = dir.join("gnaegicut-captions");

    let g = graph::build(p, 2, 2, Streams::Speech);
    let mut cmd = ffmpeg();
    cmd.args(&g.args)
        .args([
            "-filter_complex",
            &g.filter,
            "-map",
            graph::AUDIO,
            "-ar",
            "16000",
            "-ac",
            "1",
        ])
        .arg(&wav);
    run(&mut cmd)?;

    // -ml/-sow: short lines that break on word boundaries, which suits vertical video.
    let mut w = Command::new("whisper-cli");
    w.args([
        "-m",
        &model_path(),
        "-l",
        "auto",
        "-osrt",
        "-ml",
        "28",
        "-sow",
        "-np",
        "-f",
    ])
    .arg(&wav)
    .arg("-of")
    .arg(&base);
    run(&mut w)?;
    let srt = fs::read_to_string(base.with_extension("srt")).map_err(|e| e.to_string())?;
    Ok(parse_srt(&srt))
}

fn seconds(s: &str) -> Option<f64> {
    let (hms, ms) = s.trim().split_once(',')?;
    let mut parts = hms.split(':').map(|x| x.parse::<f64>());
    let (h, m, sec) = (parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?);
    Some(h * 3600.0 + m * 60.0 + sec + ms.parse::<f64>().ok()? / 1000.0)
}

pub fn parse_srt(srt: &str) -> Vec<Caption> {
    srt.replace("\r\n", "\n")
        .split("\n\n")
        .filter_map(|block| {
            let mut lines = block.lines().skip_while(|l| !l.contains("-->"));
            let (a, b) = lines.next()?.split_once("-->")?;
            let text = lines.collect::<Vec<_>>().join(" ").trim().to_string();
            let (start, end) = (seconds(a)?, seconds(b)?);
            (!text.is_empty()).then_some(Caption { start, end, text })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_srt_blocks() {
        let srt = "1\n00:00:01,500 --> 00:00:03,000\nHello there\n\n2\n00:01:00,000 --> 00:01:02,250\nSecond\nline\n";
        let c = parse_srt(srt);
        assert_eq!(c.len(), 2);
        assert!((c[0].start - 1.5).abs() < 1e-9 && c[0].text == "Hello there");
        assert!((c[1].end - 62.25).abs() < 1e-9 && c[1].text == "Second line");
    }

    /// Needs the model and macOS `say`: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn transcribes_synthetic_speech() {
        let dir = std::env::temp_dir();
        let (aiff, mp4) = (dir.join("gc_say.aiff"), dir.join("gc_say.mp4"));
        run(Command::new("say")
            .args(["-o"])
            .arg(&aiff)
            .arg("Welcome to my channel. Today we cut videos."))
        .unwrap();
        run(ffmpeg()
            .args(["-f", "lavfi", "-i", "color=c=black:s=320x240:r=30", "-i"])
            .arg(&aiff)
            .args(["-shortest", "-pix_fmt", "yuv420p"])
            .arg(&mp4))
        .unwrap();
        let mut p = Project::default();
        p.add(0, crate::media::probe::probe(&mp4).unwrap());
        let text = transcribe(&p)
            .unwrap()
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        assert!(text.contains("channel"), "got: {text}");
    }
}
