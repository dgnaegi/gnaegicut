//! Everything that talks to ffmpeg, ffprobe or whisper. The UI never builds a command itself.

pub mod ass;
mod edge_fx;
pub mod export;
pub mod frame;
pub mod graph;
pub mod probe;
mod reveal;
pub mod stream;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_audio;
#[cfg(test)]
mod tests_bar;
#[cfg(test)]
mod tests_captions;
#[cfg(test)]
mod tests_crop;
#[cfg(test)]
mod tests_edge;
#[cfg(test)]
mod tests_fx;
#[cfg(test)]
mod tests_join;
#[cfg(test)]
mod tests_motion;
#[cfg(test)]
pub mod testutil;
pub mod waveform;
pub mod whisper;

use std::process::Command;

/// An ffmpeg command that only prints real errors.
pub fn ffmpeg() -> Command {
    let mut c = Command::new("ffmpeg");
    c.args(["-y", "-v", "error"]);
    c
}

/// Runs a command to completion; on failure returns the last line it printed to stderr.
pub fn run(cmd: &mut Command) -> Result<(), String> {
    let out = cmd
        .output()
        .map_err(|e| format!("could not start {:?}: {e}", cmd.get_program()))?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    Err(err.lines().last().unwrap_or("command failed").to_string())
}

/// Bundled assets live next to the source tree.
pub fn asset(rel: &str) -> String {
    let bundled = std::env::current_exe().ok().and_then(|exe| Some(exe.parent()?.parent()?.join("Resources").join(rel)));
    match bundled {
        Some(p) if p.exists() => p.to_string_lossy().into_owned(),
        _ => format!("{}/{rel}", env!("CARGO_MANIFEST_DIR")),
    }
}
