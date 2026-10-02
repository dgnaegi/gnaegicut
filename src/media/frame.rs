use super::ffmpeg;
use super::graph::{self, Streams};
use crate::project::Project;
use std::process::Stdio;

/// One RGBA frame of the full composite at timeline time `t`, exactly as the export will draw it.
pub fn frame(p: &Project, t: f64, w: u32, h: u32) -> Option<Vec<u8>> {
    let rest = p.tail_from(t);
    let g = graph::build(&rest, w, h, Streams::Video);
    let out = ffmpeg()
        .args(&g.args)
        .args(["-filter_complex", &g.filter, "-map", graph::VIDEO, "-frames:v", "1"])
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    (out.stdout.len() == (w * h * 4) as usize).then_some(out.stdout)
}

/// A small RGBA picture of a file for the library: a frame from the middle of a video, or the image itself.
pub fn thumbnail(path: &str, at: f64, (w, h): (u32, u32)) -> Option<Vec<u8>> {
    let out = ffmpeg()
        .args([
            "-ss",
            &format!("{at:.2}"),
            "-i",
            path,
            "-frames:v",
            "1",
            "-vf",
            &format!("scale={w}:{h}"),
        ])
        .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    (out.stdout.len() == (w * h * 4) as usize).then_some(out.stdout)
}
