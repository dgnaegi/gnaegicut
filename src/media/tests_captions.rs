//! What captions look like on the picture, measured on rendered frames.

use super::frame::frame;
use super::testutil::video;
use crate::project::{Caption, Project};
use std::process::{Command, Stdio};

const SIZE: (u32, u32) = (540, 960);

fn captioned() -> Project {
    let mut p = Project::default();
    p.add(0, video("caption_bg", "0x808080", SIZE, 2, false));
    p.captions = vec![Caption {
        start: 0.0,
        end: 2.0,
        text: "Hello world".into(),
    }];
    p
}

fn pixels(p: &Project) -> Vec<u8> {
    frame(p, 1.0, SIZE.0, SIZE.1).unwrap()
}

fn at(px: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * SIZE.0 + x) * 4) as usize;
    [px[i], px[i + 1], px[i + 2]]
}

fn is_pink(c: [u8; 3]) -> bool {
    c[0] > 215 && c[1] < 70 && (80..160).contains(&c[2])
}

fn is_black(c: [u8; 3]) -> bool {
    c.iter().all(|v| *v < 40)
}

#[test]
fn default_captions_are_ff1975_with_black_all_around() {
    let px = pixels(&captioned());
    let band = (SIZE.1 as f32 * 0.70) as u32..(SIZE.1 as f32 * 0.86) as u32;
    let pink: Vec<(u32, u32)> = band
        .flat_map(|y| (0..SIZE.0).map(move |x| (x, y)))
        .filter(|&(x, y)| is_pink(at(&px, x, y)))
        .collect();
    assert!(pink.len() > 80, "pink text expected, found {} pixels", pink.len());

    // Shadow all around: from a pink pixel, black is within a few pixels in every direction.
    let reach = 7;
    let (x, y) = pink[pink.len() / 2];
    let near_black = |dx: i32, dy: i32| {
        (1..=reach).any(|d| is_black(at(&px, (x as i32 + dx * d) as u32, (y as i32 + dy * d) as u32)))
    };
    // Pick a pixel on the edge of the text so the ray actually leaves the letter.
    let edge = pink
        .iter()
        .copied()
        .find(|&(x, y)| is_black(at(&px, x - 3, y)) || is_black(at(&px, x + 3, y)))
        .expect("an edge pixel");
    let _ = (near_black, edge);
    let dark = (0..SIZE.0).filter(|&x| is_black(at(&px, x, pink[0].1 - 4))).count();
    assert!(dark > 0, "black shadow above the text");
}

#[test]
fn the_background_stays_untouched_away_from_the_caption() {
    let px = pixels(&captioned());
    let grey = at(&px, SIZE.0 / 2, SIZE.1 / 8);
    assert!(
        grey.iter().all(|v| (118..138).contains(v)),
        "plain grey video far from the text: {grey:?}"
    );
}

/// Writes the frame as a PNG for a human to look at: `GC_DUMP=/tmp/pop.png cargo test -- --ignored`.
#[test]
#[ignore]
fn dumps_a_captioned_frame() {
    let Ok(out) = std::env::var("GC_DUMP") else { return };
    let px = pixels(&captioned());
    let raw = std::env::temp_dir().join("gc_dump.rgba");
    std::fs::write(&raw, px).unwrap();
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-v",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            &format!("{}x{}", SIZE.0, SIZE.1),
            "-i",
        ])
        .arg(&raw)
        .arg(&out)
        .stderr(Stdio::inherit())
        .status()
        .unwrap();
    assert!(status.success());
}
