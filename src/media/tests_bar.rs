//! The lower third fades in from left to right.

use super::frame::frame;
use super::testutil::{image, near, pixel};
use crate::project::{Project, Transition};

const SIZE: (u32, u32) = (108, 192);

#[test]
fn a_lower_third_fades_in_softly_from_the_left() {
    let mut p = Project::default();
    let mut item = image("full", "red", SIZE);
    (item.end, item.transition, item.fade_in, item.bar) = (3.0, Transition::WipeRight, 1.0, true);
    p.add(0, item);
    let at = |t: f64, fx: f32| pixel(&frame(&p, t, SIZE.0, SIZE.1).unwrap(), SIZE, fx, 0.5);
    assert!(near(at(1.5, 0.5), [255, 0, 0]), "done");
    let (left, right) = (at(0.1, 0.1), at(0.1, 0.9));
    assert!(
        left[0] > right[0],
        "the left is further along than the right: {left:?} vs {right:?}"
    );
    let soft = at(0.15, 0.5)[0];
    assert!((10..250).contains(&soft), "the edge is a ramp, not a cut: {soft}");
}
