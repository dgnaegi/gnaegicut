//! Cropping cuts the edges of a picture, and the rest fills the box.

use super::frame::frame;
use super::testutil::{image, near, pixel};
use crate::project::{Crop, Project};

const SIZE: (u32, u32) = (108, 192);

#[test]
fn a_cropped_picture_is_smaller_in_the_frame_and_shows_only_what_is_left() {
    let mut p = Project::default();
    let mut item = image("full", "red", (108, 108));
    item.crop = Crop {
        top: 0.5,
        ..Crop::default()
    }; // 108 x 54 remains, fitted to the frame width
    p.add(0, item);
    let at = |fy: f32| pixel(&frame(&p, 1.0, SIZE.0, SIZE.1).unwrap(), SIZE, 0.5, fy);
    assert!(near(at(0.5), [255, 0, 0]), "the remaining part is in the middle");
    assert!(
        near(at(0.1), [0, 0, 0]),
        "uncropped it would be a square, now there is room above"
    );
}
