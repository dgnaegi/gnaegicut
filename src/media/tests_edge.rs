//! The soft fade and the border at the edge of a picture.

use super::frame::frame;
use super::testutil::{image, near, pixel};
use crate::project::{Edge, Project};

const SIZE: (u32, u32) = (108, 108);

fn shown(edge: Edge) -> impl Fn(f32, f32) -> [u8; 3] {
    let mut p = Project::default();
    let mut item = image("full", "red", SIZE);
    (item.end, item.edge) = (3.0, edge);
    p.add(0, item);
    let rgba = frame(&p, 1.0, SIZE.0, SIZE.1).unwrap();
    move |fx, fy| pixel(&rgba, SIZE, fx, fy)
}

#[test]
fn the_feather_fades_the_picture_towards_its_edge_and_leaves_the_middle() {
    let at = shown(Edge {
        feather: 0.4,
        ..Edge::default()
    });
    assert!(near(at(0.5, 0.5), [255, 0, 0]), "the middle is untouched");
    let (near_edge, edge) = (at(0.1, 0.5)[0], at(0.01, 0.5)[0]);
    assert!(
        near_edge > edge && near_edge < 255,
        "darker towards the edge: {near_edge} then {edge}"
    );
}

#[test]
fn the_border_is_drawn_on_the_edge_in_its_colour() {
    let at = shown(Edge {
        border: 0.1,
        border_color: [0, 0, 255],
        ..Edge::default()
    });
    assert!(near(at(0.02, 0.5), [0, 0, 255]), "the border");
    assert!(near(at(0.5, 0.5), [255, 0, 0]), "the picture inside");
}
