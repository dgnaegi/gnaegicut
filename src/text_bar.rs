//! The look of a lower third (Bauchbinde): every line on its own white box. Geometry only; `text.rs` supplies the
//! letters.

const WHITE: [u8; 3] = [255, 255, 255];

/// A filled rectangle in pixels: left, top, right, bottom.
type Rect = [f32; 4];

/// Where everything of a lower third sits, relative to the font size.
pub struct Plan {
    /// Room between a box's edge and its text.
    pub hpad: f32,
    pub margin: f32,
    pub gap: f32, // between two boxes
    /// Extra space after every letter: lower thirds are set a little airy.
    pub tracking: f32,
}

impl Plan {
    pub fn new(size: f32) -> Self {
        Self {
            hpad: size * 0.3,
            margin: (size / 10.0).ceil(),
            gap: (size / 18.0).ceil(),
            tracking: size * 0.05,
        }
    }

    /// Where the text of a line starts, horizontally.
    pub fn text_x(&self) -> f32 {
        self.margin + self.hpad
    }

    /// Image size for lines of the given widest width and line height.
    pub fn size(&self, widest: f32, lines: usize, line_h: f32) -> (usize, usize) {
        let w = self.text_x() + widest + self.hpad + self.margin;
        let h = self.margin * 2.0 + lines as f32 * line_h + lines.saturating_sub(1) as f32 * self.gap;
        (w.ceil() as usize, h.ceil() as usize)
    }

    /// Top of line `n`.
    pub fn line_y(&self, n: usize, line_h: f32) -> f32 {
        self.margin + n as f32 * (line_h + self.gap)
    }

    fn boxes(&self, widths: &[f32], line_h: f32) -> Vec<Rect> {
        (0..widths.len())
            .map(|n| {
                let top = self.line_y(n, line_h);
                [
                    self.margin,
                    top,
                    self.margin + self.hpad * 2.0 + widths[n],
                    top + line_h,
                ]
            })
            .collect()
    }
}

/// Paints the boxes on a transparent image of `w` x `h`, then the letters (`fill` coverage) over them.
pub fn paint(plan: &Plan, (w, h): (usize, usize), widths: &[f32], line_h: f32, fill: &[u8], ink: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; w * h * 4];
    let mut rect = |r: Rect, color: [u8; 3]| {
        for y in (r[1].round() as usize)..(r[3].round() as usize).min(h) {
            for x in (r[0].round() as usize)..(r[2].round() as usize).min(w) {
                out[(y * w + x) * 4..][..4].copy_from_slice(&[color[0], color[1], color[2], 255]);
            }
        }
    };
    plan.boxes(widths, line_h).into_iter().for_each(|r| rect(r, WHITE));
    for (px, &cover) in out.as_chunks_mut::<4>().0.iter_mut().zip(fill) {
        let a = cover as f32 / 255.0;
        for c in 0..3 {
            px[c] = (ink[c] as f32 * a + px[c] as f32 * (1.0 - a)).round() as u8;
        }
        px[3] = px[3].max(cover);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_land_on_white_boxes_and_the_rest_stays_transparent() {
        let plan = Plan::new(40.0);
        let (w, h) = plan.size(100.0, 2, 48.0);
        let mut fill = vec![0u8; w * h];
        let (x, y) = (plan.text_x() as usize + 2, plan.margin as usize + 5);
        fill[y * w + x] = 255;
        let img = paint(&plan, (w, h), &[100.0, 60.0], 48.0, &fill, [0, 0, 0]);
        assert_eq!(img[(y * w + x) * 4..][..4], [0, 0, 0, 255], "ink on the box");
        let inside = (plan.margin as usize + 1, plan.margin as usize + 1);
        assert_eq!(
            img[(inside.1 * w + inside.0) * 4..][..4],
            [255, 255, 255, 255],
            "the box"
        );
        assert_eq!(img[3], 0, "the corner outside stays transparent");
    }
}
