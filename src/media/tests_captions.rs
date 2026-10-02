//! What captions look like on the picture, measured on rendered frames.

use super::frame::frame;
use super::testutil::video;
use crate::project::{Aspect, Caption, Project};
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

#[test]
fn default_captions_are_ff1975_with_black_all_around() {
    let px = pixels(&captioned());
    let band = (SIZE.1 as f32 * 0.55) as u32..(SIZE.1 as f32 * 0.80) as u32;
    let pink: Vec<(u32, u32)> = band
        .flat_map(|y| (0..SIZE.0).map(move |x| (x, y)))
        .filter(|&(x, y)| is_pink(at(&px, x, y)))
        .collect();
    assert!(pink.len() > 80, "pink text expected, found {} pixels", pink.len());

    // Shadow all around: stepping outward from the text's extreme pixels in all four directions, black appears.
    let dark = |p: [u8; 3]| p.iter().all(|v| *v < 70);
    let outward = |(x, y): (u32, u32), (dx, dy): (i32, i32)| {
        (1..=7).any(|d| dark(at(&px, (x as i32 + dx * d) as u32, (y as i32 + dy * d) as u32)))
    };
    let extreme = |key: fn(&(u32, u32)) -> i64| *pink.iter().max_by_key(|p| key(p)).unwrap();
    assert!(outward(extreme(|p| -(p.0 as i64)), (-1, 0)), "shadow to the left");
    assert!(outward(extreme(|p| p.0 as i64), (1, 0)), "shadow to the right");
    assert!(outward(extreme(|p| -(p.1 as i64)), (0, -1)), "shadow above");
    assert!(outward(extreme(|p| p.1 as i64), (0, 1)), "shadow below");
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

/// The smallest rectangle (as fractions of the frame) holding every pink caption pixel.
fn text_bounds(px: &[u8], (w, h): (u32, u32)) -> Option<[f32; 4]> {
    let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            if is_pink([px[i], px[i + 1], px[i + 2]]) {
                let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);
                b = [b[0].min(fx), b[1].min(fy), b[2].max(fx), b[3].max(fy)];
            }
        }
    }
    (b[0] <= b[2]).then_some(b)
}

#[test]
fn captions_stay_inside_the_safe_zone_in_every_format() {
    for aspect in Aspect::ALL {
        let size = crate::preview::size(aspect);
        let mut p = Project::default();
        p.aspect = aspect;
        p.add(
            0,
            video(
                &format!("safe_{}", aspect.label().replace(':', "x")),
                "0x808080",
                size,
                2,
                false,
            ),
        );
        // As long as one line of transcript gets (whisper breaks at 28 characters); it wraps to two lines.
        p.captions = vec![Caption {
            start: 0.0,
            end: 2.0,
            text: "Once upon a time in a forest".into(),
        }];
        let px = frame(&p, 1.0, size.0, size.1).unwrap();
        let [x0, y0, x1, y1] = text_bounds(&px, size).expect("pink text");
        let [left, top, right, bottom] = aspect.safe_margins();
        let slack = 0.01; // the soft shadow and rounding
        assert!(
            x0 >= left - slack && x1 <= 1.0 - right + slack,
            "{}: sideways {x0}..{x1}",
            aspect.label()
        );
        assert!(
            y0 >= top - slack && y1 <= 1.0 - bottom + slack,
            "{}: height {y0}..{y1}",
            aspect.label()
        );
    }
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
