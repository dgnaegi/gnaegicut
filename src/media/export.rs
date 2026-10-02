use super::graph::{self, Streams};
use super::{ffmpeg, run};
use crate::project::Project;
use std::path::Path;

pub fn export(p: &Project, out_path: &Path) -> Result<(), String> {
    let (w, h) = p.aspect.size();
    let g = graph::build(p, w, h, Streams::Both);
    let mut cmd = ffmpeg();
    cmd.args(&g.args)
        .args(["-filter_complex", &g.filter, "-map", graph::VIDEO, "-map", graph::AUDIO])
        .args([
            "-c:v", "libx264", "-crf", "18", "-preset", "medium", "-pix_fmt", "yuv420p",
        ])
        .args(["-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart"])
        .arg(out_path);
    run(&mut cmd)
}
