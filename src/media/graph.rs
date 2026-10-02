//! The one place that turns a Project into an ffmpeg filter graph. Preview, playback, export and
//! transcription all use it, so what you see is what you export.
//!
//! Layers are composited bottom to top over a black base. Each item is trimmed by input options
//! (fast seeking), scaled to its box, optionally zoomed inside it, shifted to its timeline
//! position with `setpts`, and overlaid. Audio is delayed into place and mixed.

use super::{ass, reveal};
use crate::project::{Item, Kind, Project};

pub const VIDEO: &str = "[vout]";
pub const AUDIO: &str = "[aout]";
const FPS: u32 = 30;
const RATE: u32 = 48_000;

#[derive(Clone, Copy, PartialEq)]
pub enum Streams {
    Both,
    Video,
    Audio,
    /// Only what is spoken: the sound of videos, not music or effects. For transcription.
    Speech,
}

pub struct Graph {
    /// Input options (`-ss .. -i ..`) in the order the filter graph indexes them.
    pub args: Vec<String>,
    pub filter: String,
}

pub fn build(p: &Project, w: u32, h: u32, streams: Streams) -> Graph {
    let resolved;
    let p = if p.linked {
        p
    } else {
        resolved = p.resolved();
        &resolved
    };
    let (want_v, want_a) = (
        streams != Streams::Audio && streams != Streams::Speech,
        streams != Streams::Video,
    );
    let total = p.total().max(0.1);
    let draws = |i: &Item| want_v && i.kind.is_visual();
    let sounds = |i: &Item| want_a && i.has_audio && (streams != Streams::Speech || i.kind == Kind::Video);
    let used: Vec<&Item> = p.items().filter(|i| draws(i) || sounds(i)).collect();
    let layers = used.iter().filter(|i| draws(i)).count();
    let (mut args, mut parts) = (vec![], vec![]);

    let captions = want_v && !p.captions.is_empty();
    let top = if captions { "vpre" } else { "vout" };
    if want_v {
        let first = if layers == 0 { top } else { "b0" };
        parts.push(format!("color=c=black:s={w}x{h}:r={FPS}:d={total:.3}[{first}]"));
    }
    let mut layer = 0;
    for (i, item) in used.iter().enumerate() {
        args.extend(item.input_args());
        if draws(item) {
            let out = if layer + 1 == layers {
                top.to_string()
            } else {
                format!("b{}", layer + 1)
            };
            parts.push(video_layer(i, layer, item, p, (w, h), &out));
            layer += 1;
        }
    }
    if captions {
        let file = ass::file_for(p, w, h);
        parts.push(format!(
            "[vpre]ass=filename={}:fontsdir={}[vout]",
            file.display(),
            super::asset("assets/fonts")
        ));
    }
    if want_a {
        parts.extend(audio_mix(&used, total, &sounds));
    }
    Graph {
        args,
        filter: parts.join(";"),
    }
}

fn video_layer(i: usize, layer: usize, item: &Item, p: &Project, (w, h): (u32, u32), out: &str) -> String {
    let (bw, bh) = item.pixels(p.aspect, w, h);
    let (rw, rh) = item.rotated((bw, bh));
    let (x, y) = item.motion(item.origin(w, h, (rw, rh)), (w, h));

    // Up to the item's box: decode format, timing, scale and zoom.
    let mut pre = vec![];
    if item.needs_alpha() {
        pre.push("format=rgba".to_string());
    }
    pre.push(format!("fps={FPS}"));
    pre.push("setpts=PTS-STARTPTS".into());
    pre.push(match item.zoom_expr() {
        None => format!("scale={bw}:{bh}"),
        Some(z) => format!("scale=w='{bw}*({z})':h='{bh}*({z})':eval=frame,crop={bw}:{bh}"),
    });
    pre.extend(item.fx_filters(w));
    // After the box: rotation, fades, and shifting to the item's place on the timeline.
    let mut post = vec![];
    if item.rotation != 0.0 || item.spins() {
        post.push(format!("rotate=a='{}':ow={rw}:oh={rh}:c=none", item.rotation_expr()));
    }
    post.extend(item.fade_filters());
    post.extend(["setsar=1".to_string(), format!("setpts=PTS+{:.3}/TB", item.at)]);

    let overlay = format!("[b{layer}][v{i}]overlay=x='{x}':y='{y}':eof_action=pass[{out}]");
    match reveal::mask(i, item, (bw, bh)) {
        None => format!(
            "[{i}:v]{}[v{i}];{overlay}",
            pre.into_iter().chain(post).collect::<Vec<_>>().join(",")
        ),
        Some(mask) => format!(
            "[{i}:v]{}[bx{i}];{mask};{};[bm{i}]{}[v{i}];{overlay}",
            pre.join(","),
            reveal::apply(i, item),
            post.join(",")
        ),
    }
}

/// Silence of the full length plus every item's audio, each delayed to its start.
fn audio_mix(used: &[&Item], total: f64, sounds: &dyn Fn(&Item) -> bool) -> Vec<String> {
    let silence = format!("anullsrc=r={RATE}:cl=stereo,atrim=duration={total:.3}");
    let sounding: Vec<_> = used.iter().enumerate().filter(|(_, it)| sounds(it)).collect();
    if sounding.is_empty() {
        return vec![format!("{silence}{AUDIO}")];
    }
    let mut parts = vec![format!("{silence}[a_base]")];
    let mut labels = String::from("[a_base]");
    for (i, item) in &sounding {
        parts.push(format!("[{i}:a]{}[a{i}]", audio_filters(item)));
        labels += &format!("[a{i}]");
    }
    parts.push(format!(
        "{labels}amix=inputs={}:duration=first:normalize=0{AUDIO}",
        sounding.len() + 1
    ));
    parts
}

/// Clip audio chain: normalise format, optional voice clean-up, gain, fades, then delay into place.
fn audio_filters(item: &Item) -> String {
    let mut f = vec![
        format!("aresample={RATE}"),
        "aformat=channel_layouts=stereo".into(),
        "asetpts=PTS-STARTPTS".into(),
    ];
    if item.enhance {
        // Cut rumble, reduce steady noise, even out loudness, then normalise to -16 LUFS (social media standard).
        f.extend(
            [
                "highpass=f=80",
                "afftdn=nr=12:nf=-35",
                "acompressor=threshold=-20dB:ratio=3:attack=5:release=100",
            ]
            .map(String::from),
        );
        f.extend(["loudnorm=I=-16:TP=-1.5:LRA=11".into(), format!("aresample={RATE}")]);
    }
    if (item.volume - 1.0).abs() > 1e-3 {
        f.push(format!("volume={:.3}", item.volume));
    }
    let (fade_in, fade_out) = (item.enter().len, item.exit().len);
    if fade_in > 0.0 || fade_out > 0.0 {
        // Same as the picture: fades run in item time, so a cut-off front does not restart them.
        f.push(format!("asetpts=PTS-STARTPTS+{:.3}/TB", item.cut));
        if fade_in > 0.0 {
            f.push(format!("afade=t=in:st=0:d={fade_in:.3}"));
        }
        if fade_out > 0.0 {
            let start = (item.len() + item.cut - fade_out as f64).max(0.0);
            f.push(format!("afade=t=out:st={start:.3}:d={fade_out:.3}"));
        }
        f.push("asetpts=PTS-STARTPTS".into());
    }
    let ms = (item.at * 1000.0).round() as i64;
    f.push(format!("adelay={ms}|{ms}"));
    f.join(",")
}
