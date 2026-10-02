//! Subtitle data and the styles shared by the preview and the export.

mod edit;
mod split;
mod transcript;

use super::Aspect;
use serde::{Deserialize, Serialize};

/// A subtitle line. Times are on the timeline, not in a source file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Caption {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CaptionStyle {
    Pop,     // hot pink text with a heavy black shadow all around: the default
    Classic, // white text, black outline
    Block,   // white text on a black box
    Accent,  // white text on a red box
}

/// Colours are RGB; shared by the egui preview and the ASS export so they cannot drift apart.
pub struct Look {
    pub text: [u8; 3],
    pub outline: [u8; 3],
    pub boxed: bool,
    pub uppercase: bool,
    /// A thick, softened outline: reads as a shadow all around the letters.
    pub heavy: bool,
}

/// The signature caption colour, #FF1975.
pub const POP_PINK: [u8; 3] = [0xFF, 0x19, 0x75];

impl CaptionStyle {
    pub const ALL: [CaptionStyle; 4] = [
        CaptionStyle::Pop,
        CaptionStyle::Classic,
        CaptionStyle::Block,
        CaptionStyle::Accent,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CaptionStyle::Pop => "Pop",
            CaptionStyle::Classic => "Classic",
            CaptionStyle::Block => "Block",
            CaptionStyle::Accent => "Accent",
        }
    }

    pub fn look(self) -> Look {
        let (white, black) = ([255, 255, 255], [0, 0, 0]);
        let plain = Look {
            text: white,
            outline: black,
            boxed: false,
            uppercase: false,
            heavy: false,
        };
        match self {
            CaptionStyle::Pop => Look {
                text: POP_PINK,
                heavy: true,
                ..plain
            },
            CaptionStyle::Classic => plain,
            CaptionStyle::Block => Look {
                boxed: true,
                uppercase: true,
                ..plain
            },
            CaptionStyle::Accent => Look {
                outline: [255, 48, 0],
                boxed: true,
                uppercase: true,
                ..plain
            },
        }
    }
}

/// Where and in which font captions are drawn. Position is the caption's centre as a fraction of the frame.
#[derive(Clone, Serialize, Deserialize)]
pub struct CaptionLayout {
    pub x: f32,
    pub y: f32,
    pub family: String,
    pub bold: bool,
    pub size: f32, // 1.0 = default size
    /// While true the caption rests at the lower edge of the platform safe zone of the current format and follows it
    /// when the format changes. Moving the caption by hand turns this off.
    #[serde(default = "yes")]
    pub auto_y: bool,
}

/// Where captions rest by default, as a fraction of the frame height.
const AUTO_HEIGHT: f32 = 0.6;

fn yes() -> bool {
    true
}

impl CaptionLayout {
    /// The caption's centre as fractions of the frame. Automatically: 60% of the way down, pulled back inside the
    /// platform safe zone if a large font or a second line would otherwise reach the buttons and the app's own caption
    /// (see `Aspect::safe_margins`).
    pub fn position(&self, aspect: Aspect) -> (f32, f32) {
        if !self.auto_y {
            return (self.x, self.y);
        }
        let (w, h) = aspect.size();
        let line = w.min(h) as f32 / 14.0 * self.size; // the caption font size in pixels (see media::ass)
        let half_block = 1.25 * line / h as f32; // half the height of a two-line caption
        let [_, top, _, bottom] = aspect.safe_margins();
        let (lowest, highest) = (top + half_block, (1.0 - bottom - half_block).max(top + half_block));
        (self.x, AUTO_HEIGHT.clamp(lowest, highest))
    }

    /// Puts the caption where the user dragged it.
    pub fn place(&mut self, x: f32, y: f32) {
        (self.x, self.y, self.auto_y) = (x, y, false);
    }

    /// Back to the automatic position.
    pub fn reset_position(&mut self) {
        (self.x, self.auto_y) = (0.5, true);
    }
}

impl Default for CaptionLayout {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: 0.7,
            family: "AL Unica77 Black".into(), // libass picks the Black cut by this name; "AL Unica77" alone gives Medium
            bold: true,
            size: 1.0,
            auto_y: true,
        }
    }
}

#[cfg(test)]
mod tests;
