use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Aspect {
    Vertical, // 9:16 (TikTok / Reels / Shorts)
    Portrait, // 4:5 (Instagram feed)
    Wide,     // 16:9 (YouTube)
}

impl Aspect {
    pub const ALL: [Aspect; 3] = [Aspect::Vertical, Aspect::Portrait, Aspect::Wide];

    pub fn label(self) -> &'static str {
        match self {
            Aspect::Vertical => "9:16",
            Aspect::Portrait => "4:5",
            Aspect::Wide => "16:9",
        }
    }

    pub fn size(self) -> (u32, u32) {
        match self {
            Aspect::Vertical => (1080, 1920),
            Aspect::Portrait => (1080, 1350),
            Aspect::Wide => (1920, 1080),
        }
    }

    /// Where platform UI (buttons, captions, titles) covers the picture, as fractions of the frame:
    /// `[left, top, right, bottom]`. Keep important content inside the remaining rectangle.
    pub fn safe_margins(self) -> [f32; 4] {
        match self {
            Aspect::Vertical => [0.06, 0.13, 0.12, 0.25],
            Aspect::Portrait => [0.05, 0.05, 0.05, 0.12],
            Aspect::Wide => [0.05, 0.05, 0.05, 0.05],
        }
    }
}
