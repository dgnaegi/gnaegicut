use crate::project::{Item, Kind};
use std::path::Path;
use std::process::Command;

pub const IMAGE_EXTS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "bmp", "tiff"];
pub const AUDIO_EXTS: [&str; 6] = ["mp3", "wav", "m4a", "aac", "ogg", "flac"];

pub fn probe(path: &Path) -> Result<Item, String> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries"])
        .arg("format=duration:stream=codec_type,width,height:stream_side_data=rotation:stream_tags=rotate")
        .args(["-of", "default=nw=1"])
        .arg(path)
        .output()
        .map_err(|e| format!("ffprobe not found: {e}"))?;
    let info = parse(&String::from_utf8_lossy(&out.stdout));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let file = path.to_string_lossy().into_owned();
    let bad = || format!("{} is not a usable video, image or sound", path.display());
    if AUDIO_EXTS.contains(&ext.as_str()) {
        let usable = info.audio && info.duration > 0.0;
        return usable
            .then(|| Item::new(file, name, Kind::Audio, (1, 1), info.duration, true))
            .ok_or_else(bad);
    }
    let image = IMAGE_EXTS.contains(&ext.as_str());
    if info.size.0 == 0 || !(image || info.duration > 0.0) {
        return Err(bad());
    }
    let kind = if image { Kind::Image } else { Kind::Video };
    Ok(Item::new(file, name, kind, info.size, info.duration, info.audio))
}

#[derive(Default, PartialEq, Debug)]
struct Info {
    size: (u32, u32),
    duration: f64,
    audio: bool,
}

/// Parses ffprobe's `key=value` lines. Phone videos store portrait as landscape plus a rotation,
/// and ffmpeg applies it on decode, so the displayed size swaps for 90/270 degrees.
fn parse(text: &str) -> Info {
    let (mut info, mut in_video, mut rotation) = (Info::default(), false, 0.0f64);
    for line in text.lines() {
        match line.split_once('=') {
            Some(("codec_type", "video")) => in_video = true,
            Some(("codec_type", "audio")) => (info.audio, in_video) = (true, false),
            Some(("width", v)) if in_video && info.size.0 == 0 => info.size.0 = v.parse().unwrap_or(0),
            Some(("height", v)) if in_video && info.size.1 == 0 => info.size.1 = v.parse().unwrap_or(0),
            Some(("rotation" | "TAG:rotate", v)) if in_video => rotation = v.parse().unwrap_or(0.0),
            Some(("duration", v)) => info.duration = v.parse().unwrap_or(0.0),
            _ => {}
        }
    }
    if (rotation.abs() as i64) % 180 == 90 {
        info.size = (info.size.1, info.size.0);
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_audio_video_and_rotation() {
        let text = "codec_type=video\nwidth=1920\nheight=1080\nrotation=-90\ncodec_type=audio\nduration=12.5\n";
        assert_eq!(
            parse(text),
            Info {
                size: (1080, 1920),
                duration: 12.5,
                audio: true
            }
        );
    }

    #[test]
    fn ignores_audio_stream_dimensions_and_missing_duration() {
        let info = parse("codec_type=video\nwidth=800\nheight=600\nduration=N/A\n");
        assert_eq!(
            info,
            Info {
                size: (800, 600),
                duration: 0.0,
                audio: false
            }
        );
    }
}
