//! Subtitle data and the styles shared by the preview and the export.

use serde::{Deserialize, Serialize};

/// A subtitle line. Times are on the timeline, not in a source file.
#[derive(Clone, Serialize, Deserialize)]
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
}

impl Default for CaptionLayout {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: 0.78,
            family: "AL Unica77 Black".into(), // libass picks the Black cut by this name; "AL Unica77" alone gives Medium
            bold: true,
            size: 1.0,
        }
    }
}
