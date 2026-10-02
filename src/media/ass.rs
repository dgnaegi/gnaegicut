//! Caption burn-in via ASS subtitles, which ffmpeg's `ass` filter renders with libass.

use crate::project::{Aspect, Caption, CaptionLayout, CaptionStyle, Project};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

fn bgr(c: [u8; 3]) -> String {
    format!("&H00{:02X}{:02X}{:02X}", c[2], c[1], c[0])
}

fn stamp(t: f64) -> String {
    let cs = (t * 100.0).round() as u64;
    format!(
        "{}:{:02}:{:02}.{:02}",
        cs / 360_000,
        cs / 6000 % 60,
        cs / 100 % 60,
        cs % 100
    )
}

/// ASS script for a frame of `w`x`h` pixels. `layout.x/y` place the caption's centre.
pub fn render(
    captions: &[Caption],
    style: CaptionStyle,
    layout: &CaptionLayout,
    aspect: Aspect,
    (w, h): (u32, u32),
) -> String {
    let look = style.look();
    let size = (w.min(h) as f32 / 14.0 * layout.size).round() as u32;
    let (border_style, outline) = match (look.boxed, look.heavy) {
        (true, _) => (3, size / 5),
        (false, true) => (1, size / 9),
        (false, false) => (1, size / 18),
    };
    // Blur turns the heavy outline into a soft shadow (it is applied to the shadow layer only, see below).
    let blur = if look.heavy {
        format!("\\blur{}", (size / 22).max(2))
    } else {
        String::new()
    };
    let bold = if layout.bold { -1 } else { 0 };
    // Lines wrap inside the safe zone: as wide as the narrower of the two side margins allows, centred.
    let [left, _, right, _] = aspect.safe_margins();
    let margin = (left.max(right) * w as f32).round() as u32;
    let (cx, cy) = layout.position(aspect);
    let (x, y) = ((cx * w as f32).round() as i32, (cy * h as f32).round() as i32);
    let mut s = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {w}\nPlayResY: {h}\n\n[V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, \
         Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, \
         MarginR, MarginV, Encoding\n\
         Style: Default,{},{size},{},{},{},{},{bold},0,0,0,100,100,0,0,{border_style},{outline},0,5,{margin},{margin},0,1\n\n\
         [Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        layout.family,
        bgr(look.text),
        bgr(look.text),
        bgr(look.outline),
        bgr(look.outline),
    );
    for c in captions {
        let text = if look.uppercase {
            c.text.to_uppercase()
        } else {
            c.text.clone()
        };
        let text = text.replace(['{', '}'], "").replace('\n', "\\N");
        let (start, end) = (stamp(c.start), stamp(c.end));
        if look.heavy {
            // Two layers: a soft shadow below (the outline alone, blurred) and the letters on top, perfectly sharp.
            // Blurring the letters themselves is what made the text look pixelated.
            s += &format!("Dialogue: 0,{start},{end},Default,,0,0,0,,{{\\pos({x},{y}){blur}\\1a&HFF&}}{text}\n");
            s += &format!("Dialogue: 1,{start},{end},Default,,0,0,0,,{{\\pos({x},{y})\\bord0}}{text}\n");
        } else {
            s += &format!("Dialogue: 0,{start},{end},Default,,0,0,0,,{{\\pos({x},{y})}}{text}\n");
        }
    }
    s
}

/// Writes the project's captions to a temp file named by content (so concurrent graphs never clash).
pub fn file_for(p: &Project, w: u32, h: u32) -> PathBuf {
    let script = render(&p.captions, p.caption_style, &p.caption_layout, p.aspect, (w, h));
    let mut hasher = DefaultHasher::new();
    script.hash(&mut hasher);
    let path = std::env::temp_dir().join(format!("gnaegicut-{:x}.ass", hasher.finish()));
    if !path.exists() {
        let _ = std::fs::write(&path, script);
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_and_colours() {
        assert_eq!(stamp(3725.5), "1:02:05.50");
        assert_eq!(bgr([255, 48, 0]), "&H000030FF");
    }

    #[test]
    fn renders_positioned_uppercase_block_dialogue() {
        let c = [Caption {
            start: 1.0,
            end: 2.5,
            text: "hello {x}".into(),
        }];
        let layout = CaptionLayout {
            x: 0.5,
            y: 0.75,
            family: "Helvetica".into(),
            bold: false,
            size: 1.0,
            auto_y: false,
        };
        let out = render(&c, CaptionStyle::Block, &layout, Aspect::Vertical, (1080, 1920));
        assert!(out.contains("Dialogue: 0,0:00:01.00,0:00:02.50,Default,,0,0,0,,{\\pos(540,1440)}HELLO X"));
        assert!(out.contains("Style: Default,Helvetica,72,"));
        assert!(out.contains("PlayResY: 1920"));
    }

    #[test]
    fn pop_is_pink_with_a_heavy_soft_black_shadow_in_unica() {
        let c = [Caption {
            start: 0.0,
            end: 1.0,
            text: "Hello".into(),
        }];
        let layout = CaptionLayout::default();
        assert_eq!((layout.family.as_str(), layout.bold), ("AL Unica77 Black", true));
        let out = render(&c, CaptionStyle::Pop, &layout, Aspect::Vertical, (1080, 1920));
        assert!(out.contains("Style: Default,AL Unica77 Black,72,&H007519FF,&H007519FF,&H00000000,&H00000000,-1,"));
        assert!(
            out.contains(",1,8,0,5,"),
            "border style 1 with an outline of size/9 = 8 px"
        );
        let shadow = out
            .lines()
            .find(|l| l.starts_with("Dialogue: 0,"))
            .expect("a shadow layer");
        let letters = out
            .lines()
            .find(|l| l.starts_with("Dialogue: 1,"))
            .expect("a letters layer");
        assert!(
            shadow.contains("\\blur3") && shadow.contains("\\1a&HFF&"),
            "blurred outline only: {shadow}"
        );
        assert!(
            letters.contains("\\bord0") && !letters.contains("blur"),
            "sharp letters on top: {letters}"
        );
        assert!(!out.contains("HELLO"), "Pop keeps the speaker's capitalisation");
    }

    #[test]
    fn lines_wrap_inside_the_safe_zone_and_sit_where_the_layout_says() {
        let c = [Caption {
            start: 0.0,
            end: 1.0,
            text: "Hello".into(),
        }];
        let layout = CaptionLayout::default();
        let out = render(&c, CaptionStyle::Pop, &layout, Aspect::Vertical, (1080, 1920));
        let margin = (0.12f32 * 1080.0).round() as u32; // the right margin is the wider of the two
        assert!(
            out.contains(&format!(",5,{margin},{margin},0,1")),
            "margins {margin}: {out}"
        );
        let (_, y) = layout.position(Aspect::Vertical);
        assert!(out.contains(&format!("\\pos(540,{})", (y * 1920.0).round() as i32)));
        let wide = render(&c, CaptionStyle::Pop, &layout, Aspect::Wide, (1920, 1080));
        assert!(wide.contains("PlayResX: 1920"), "{wide}");
    }
}
