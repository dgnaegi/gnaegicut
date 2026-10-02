//! Text items are rendered to a transparent PNG here, then composited like any image. That keeps the
//! look identical in preview and export for any installed font, with no filter-escaping pitfalls.

use crate::fonts::{DEFAULT_FONT, Fonts};
use crate::project::Item;
use crate::text_bar;
use fontdue::{Font, FontSettings};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{fs::File, io::BufWriter};

pub struct Rendered {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

/// Straight-alpha RGBA of centred, multi-line text.
pub fn render(font: &Font, text: &str, size: f32, color: [u8; 3], outline: bool, bar: bool) -> Rendered {
    let lines: Vec<&str> = if text.is_empty() {
        vec!["Text"]
    } else {
        text.lines().collect()
    };
    let metrics = font.horizontal_line_metrics(size);
    let (ascent, line_h) = metrics.map_or((size * 0.8, size * 1.2), |m| (m.ascent, m.new_line_size));
    let width = |l: &str| l.chars().map(|c| font.metrics(c, size).advance_width).sum::<f32>();
    let widest = lines.iter().map(|l| width(l)).fold(1.0, f32::max);
    let ring = if outline { (size / 14.0).max(1.0) } else { 0.0 };
    let plan = text_bar::Plan::new(size);
    let pad = (size / 8.0 + ring * 2.0).ceil();
    let (w, h) = if bar {
        plan.size(widest, lines.len(), line_h)
    } else {
        (
            (widest + pad * 2.0).ceil() as usize,
            (line_h * lines.len() as f32 + pad * 2.0).ceil() as usize,
        )
    };

    let (mut fill, mut edge) = (vec![0u8; w * h], vec![0u8; w * h]);
    let stamp = |buf: &mut Vec<u8>, bitmap: &[u8], gw: usize, left: f32, top: f32| {
        for (i, &cov) in bitmap.iter().enumerate() {
            let (x, y) = (
                (left as i32 + (i % gw) as i32) as usize,
                (top as i32 + (i / gw) as i32) as usize,
            );
            if x < w && y < h {
                buf[y * w + x] = buf[y * w + x].max(cov);
            }
        }
    };
    for (n, line) in lines.iter().enumerate() {
        let (mut x, top) = if bar {
            (plan.text_x(), plan.line_y(n, line_h))
        } else {
            (pad + (widest - width(line)) / 2.0, pad + n as f32 * line_h)
        };
        let baseline = top + ascent;
        for c in line.chars() {
            let (m, bitmap) = font.rasterize(c, size);
            let (left, top) = (x + m.xmin as f32, baseline - m.height as f32 - m.ymin as f32);
            if outline {
                for k in 0..16 {
                    let a = k as f32 * std::f32::consts::TAU / 16.0;
                    stamp(
                        &mut edge,
                        &bitmap,
                        m.width.max(1),
                        left + ring * a.cos(),
                        top + ring * a.sin(),
                    );
                }
            }
            stamp(&mut fill, &bitmap, m.width.max(1), left, top);
            x += m.advance_width;
        }
    }
    let rgba = if bar {
        let widths: Vec<f32> = lines.iter().map(|l| width(l)).collect();
        text_bar::paint(&plan, (w, h), &widths, line_h, &fill, color)
    } else {
        compose(&fill, &edge, color)
    };
    Rendered {
        rgba,
        w: w as u32,
        h: h as u32,
    }
}

/// Fill over a black outline, as straight (non-premultiplied) RGBA.
fn compose(fill: &[u8], edge: &[u8], color: [u8; 3]) -> Vec<u8> {
    let mut out = Vec::with_capacity(fill.len() * 4);
    for (&f, &e) in fill.iter().zip(edge) {
        let (f, e) = (f as f32 / 255.0, e as f32 / 255.0);
        let alpha = f + e * (1.0 - f);
        let mix = |c: u8| {
            if alpha > 0.0 {
                (c as f32 * f / alpha).round() as u8
            } else {
                0
            }
        };
        out.extend([
            mix(color[0]),
            mix(color[1]),
            mix(color[2]),
            (alpha * 255.0).round() as u8,
        ]);
    }
    out
}

/// Re-renders a text item's PNG after its text or look changed, and updates its geometry.
pub fn refresh(fonts: &Fonts, item: &mut Item) -> Result<(), String> {
    let name = if fonts.find(&item.font).is_some() {
        item.font.as_str()
    } else {
        DEFAULT_FONT
    };
    let loaded = fonts.with_data(name, |data, index| {
        Font::from_bytes(
            data,
            FontSettings {
                collection_index: index,
                ..Default::default()
            },
        )
    });
    let font = loaded.ok_or("font not found")?.map_err(|e| e.to_string())?;
    let img = render(&font, &item.text, item.font_size, item.color, item.outline, item.bar);

    let mut hasher = DefaultHasher::new();
    (
        &item.text,
        name,
        item.font_size.to_bits(),
        item.color,
        item.outline,
        item.bar,
    )
        .hash(&mut hasher);
    let path = std::env::temp_dir().join(format!("gnaegicut-text-{:x}.png", hasher.finish()));
    let mut enc = png::Encoder::new(
        BufWriter::new(File::create(&path).map_err(|e| e.to_string())?),
        img.w,
        img.h,
    );
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut w| w.write_image_data(&img.rgba))
        .map_err(|e| e.to_string())?;

    item.path = path.to_string_lossy().into_owned();
    (item.src_w, item.src_h) = (img.w, img.h);
    item.name = item.text.lines().next().unwrap_or("Text").chars().take(24).collect();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Kind;

    #[test]
    fn renders_visible_outlined_multiline_text() {
        let fonts = Fonts::load();
        assert!(fonts.find(DEFAULT_FONT).is_some(), "bundled font missing");
        let mut item = Item::new(String::new(), String::new(), Kind::Text, (1, 1), 0.0, false);
        item.text = "Hello\nWorld".into();
        item.font = DEFAULT_FONT.into();
        refresh(&fonts, &mut item).unwrap();
        assert!(item.src_w > 100 && item.src_h > item.src_w / 6);
        assert!(std::fs::metadata(&item.path).unwrap().len() > 100);
        assert_eq!(item.name, "Hello");
    }
}
