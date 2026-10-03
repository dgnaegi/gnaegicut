//! Cutting away the edges of a picture.

use serde::{Deserialize, Serialize};
use std::hash::Hasher;

/// The share of the picture's width / height that is cut away on each side, 0 to `MOST`.
#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Crop {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Opposite sides together never cut away more than this, so something is always left.
pub const MOST: f32 = 0.9;

impl Crop {
    pub fn is_none(&self) -> bool {
        *self == Self::default()
    }

    /// The fractions of width and height that remain.
    pub fn kept(&self) -> (f32, f32) {
        (1.0 - self.left - self.right, 1.0 - self.top - self.bottom)
    }

    /// Pulls every side back so the two opposite ones leave at least `1 - MOST`.
    pub fn limited(self) -> Self {
        let pair = |a: f32, b: f32| {
            let (a, b) = (a.clamp(0.0, MOST), b.clamp(0.0, MOST));
            let over = (a + b - MOST).max(0.0);
            (a - over / 2.0, b - over / 2.0)
        };
        let ((left, right), (top, bottom)) = (pair(self.left, self.right), pair(self.top, self.bottom));
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// The ffmpeg filter that cuts the edges off, if any are cut.
    pub fn filter(&self) -> Option<String> {
        let (w, h) = self.kept();
        (!self.is_none()).then(|| format!("crop=iw*{w:.5}:ih*{h:.5}:iw*{:.5}:ih*{:.5}", self.left, self.top))
    }

    pub fn hash_into(&self, h: &mut impl Hasher) {
        for v in [self.left, self.top, self.right, self.bottom] {
            h.write_u32(v.to_bits());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_cut_means_no_filter_and_the_whole_picture() {
        assert!(Crop::default().filter().is_none());
        assert_eq!(Crop::default().kept(), (1.0, 1.0));
    }

    #[test]
    fn what_is_left_follows_the_cut_sides() {
        let crop = Crop {
            left: 0.1,
            top: 0.0,
            right: 0.2,
            bottom: 0.5,
        };
        assert_eq!(crop.kept(), (0.7, 0.5));
        assert!(
            crop.filter()
                .unwrap()
                .starts_with("crop=iw*0.70000:ih*0.50000:iw*0.10000:ih*0.00000")
        );
    }

    #[test]
    fn opposite_sides_never_cut_everything_away() {
        let crop = Crop {
            left: 0.9,
            right: 0.9,
            ..Crop::default()
        }
        .limited();
        assert!(crop.kept().0 >= 0.1 - 1e-6);
        assert_eq!(crop.left, crop.right, "the excess is shared");
    }
}
