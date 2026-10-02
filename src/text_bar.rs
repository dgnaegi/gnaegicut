//! The look of a lower third (Bauchbinde): every line on its own white box, with a bracket in the accent colour
//! on the left. Geometry only; `text.rs` supplies the letters.

/// The accent colour of the bracket, the same pink as the captions.
const PINK: [u8; 3] = [255, 25, 117];
const WHITE: [u8; 3] = [255, 255, 255];

/// A filled rectangle in pixels: left, top, right, bottom.
type Rect = [f32; 4];

/// Where everything of a lower third sits, relative to the font size.
pub struct Plan {
    /// Left margin before the text inside its box, and the room the bracket takes.
    pub indent: f32,
    pub hpad: f32,
    pub margin: f32,
    pub bracket: f32, // stroke width of the bracket
    pub gap: f32,     // between two boxes
}

impl Plan {
    pub fn new(size: f32) -> Self {
        Self {
            indent: size * 0.42,
            hpad: size * 0.3,
            margin: (size / 10.0).ceil(),
            bracket: (size * 0.14).ceil(),
            gap: (size / 18.0).ceil(),
        }
    }

    /// Where the text of line `n` starts, horizontally.
    pub fn text_x(&self) -> f32 {
        self.margin + self.indent + self.hpad
    }

    /// Top of the first box: the bracket's upper arm sits above it.
    pub fn top(&self) -> f32 {
        self.margin + self.bracket
    }

    /// Image size for lines of the given widths and height.
    pub fn size(&self, widest: f32, lines: usize, line_h: f32) -> (usize, usize) {
        let w = self.text_x() + widest + self.hpad + self.margin;
        let h =
            self.top() + lines as f32 * line_h + lines.saturating_sub(1) as f32 * self.gap + self.bracket + self.margin;
        (w.ceil() as usize, h.ceil() as usize)
    }

    /// Top of line `n`.
    pub fn line_y(&self, n: usize, line_h: f32) -> f32 {
        self.top() + n as f32 * (line_h + self.gap)
    }

    fn boxes(&self, widths: &[f32], line_h: f32) -> Vec<Rect> {
        let x = self.margin + self.indent;
        (0..widths.len())
            .map(|n| {
                [
                    x,
                    self.line_y(n, line_h),
                    x + self.hpad * 2.0 + widths[n],
                    self.line_y(n, line_h) + line_h,
                ]
            })
            .collect()
    }

    /// A bracket "[": the upright and an arm at the top and bottom.
    fn bracket(&self, h: f32) -> Vec<Rect> {
        let (m, b, arm) = (self.margin, self.bracket, self.indent + self.hpad * 2.0);
        vec![
            [m, m, m + b, h - m],
            [m, m, m + arm, m + b],
            [m, h - m - b, m + arm, h - m],
        ]
    }
}

/// Paints boxes and bracket on a transparent image of `w` x `h`, then the letters (`fill` coverage) over them.
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
    plan.bracket(h as f32).into_iter().for_each(|r| rect(r, PINK));
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
    fn the_image_holds_every_box_and_the_bracket_and_letters_land_on_white() {
        let plan = Plan::new(40.0);
        let (w, h) = plan.size(100.0, 2, 48.0);
        let mut fill = vec![0u8; w * h];
        let (x, y) = (plan.text_x() as usize + 2, plan.top() as usize + 5);
        fill[y * w + x] = 255;
        let img = paint(&plan, (w, h), &[100.0, 60.0], 48.0, &fill, [0, 0, 0]);
        assert_eq!(img[(y * w + x) * 4..][..4], [0, 0, 0, 255], "ink on the box");
        let m = plan.margin as usize + 1;
        assert_eq!(
            img[((h / 2) * w + m) * 4..][..4],
            [255, 25, 117, 255],
            "the upright of the bracket"
        );
        assert_eq!(img[3], 0, "the corner outside stays transparent");
    }
}
