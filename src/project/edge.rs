//! What happens at the edge of a picture: a soft fade into the background, and a solid border.

use serde::{Deserialize, Serialize};
use std::hash::Hasher;

/// Both are fractions of the shorter side of the picture.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// How far the picture fades out towards its edges: 0 is a hard edge.
    pub feather: f32,
    /// Thickness of the border drawn on the edge: 0 is none.
    pub border: f32,
    pub border_color: [u8; 3],
}

pub const MAX_FEATHER: f32 = 0.5;
pub const MAX_BORDER: f32 = 0.15;

impl Default for Edge {
    fn default() -> Self {
        Self {
            feather: 0.0,
            border: 0.0,
            border_color: [255, 255, 255],
        }
    }
}

impl Edge {
    pub fn hash_into(&self, h: &mut impl Hasher) {
        h.write_u32(self.feather.to_bits());
        h.write_u32(self.border.to_bits());
        h.write(&self.border_color);
    }

    /// The ffmpeg filter that draws the border inside a box of `bw` x `bh` pixels.
    pub fn border_filter(&self, (bw, bh): (u32, u32)) -> Option<String> {
        let px = (self.border * bw.min(bh) as f32).round() as u32;
        let [r, g, b] = self.border_color;
        (px > 0).then(|| format!("drawbox=x=0:y=0:w=iw:h=ih:color=0x{r:02X}{g:02X}{b:02X}@1:t={px}:replace=1"))
    }

    /// Width of the fade in pixels, for a box of `bw` x `bh`.
    pub fn feather_px(&self, (bw, bh): (u32, u32)) -> f32 {
        self.feather * bw.min(bh) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_border_needs_a_thickness_and_takes_the_shorter_side_as_its_scale() {
        let edge = Edge {
            border: 0.1,
            border_color: [255, 0, 16],
            ..Edge::default()
        };
        assert_eq!(
            edge.border_filter((200, 100)).unwrap(),
            "drawbox=x=0:y=0:w=iw:h=ih:color=0xFF0010@1:t=10:replace=1"
        );
        assert!(Edge::default().border_filter((200, 100)).is_none());
        assert_eq!(
            Edge {
                feather: 0.2,
                ..Edge::default()
            }
            .feather_px((200, 100)),
            20.0
        );
    }
}
