//! Magic lines: while something is dragged, its centre and edges snap to the frame's centre and edges,
//! the safe zone and other items, and a red line shows what it snapped to.

use crate::app::App;
use crate::project::Project;
use crate::theme::ACCENT;
use eframe::egui::{Painter, Rect, Stroke, pos2};

const REACH_PX: f32 = 7.0;

pub struct Snapped {
    pub value: f32,
    pub line: Option<f32>,
}

/// Snaps a box (`center`, `half` size, all as fractions of the frame) so its centre or either edge meets a target.
pub fn snap_axis(center: f32, half: f32, targets: &[f32], reach: f32) -> Snapped {
    let best = targets
        .iter()
        .flat_map(|&t| [(t - center, t), (t - (center - half), t), (t - (center + half), t)])
        .filter(|(d, _)| d.abs() <= reach)
        .min_by(|a, b| a.0.abs().total_cmp(&b.0.abs()));
    match best {
        Some((d, line)) => Snapped {
            value: center + d,
            line: Some(line),
        },
        None => Snapped {
            value: center,
            line: None,
        },
    }
}

/// Where things can snap on each axis: frame centre and edges, safe-zone edges, other playing items.
pub fn targets(app: &App, skip: Option<u64>) -> (Vec<f32>, Vec<f32>) {
    let (mut xs, mut ys) = (vec![0.5, 0.0, 1.0], vec![0.5, 0.0, 1.0]);
    if app.safe_zones {
        let [l, t, r, b] = app.project.aspect.safe_margins();
        xs.extend([l, 1.0 - r]);
        ys.extend([t, 1.0 - b]);
    }
    let p: &Project = &app.project;
    for it in p
        .items()
        .filter(|i| i.kind.is_visual() && Some(i.id) != skip && i.at <= app.playhead && app.playhead < i.end_at())
    {
        let (hx, hy) = it.half_extent(p.aspect);
        xs.extend([it.x, it.x - hx, it.x + hx]);
        ys.extend([it.y, it.y - hy, it.y + hy]);
    }
    (xs, ys)
}

pub fn reach(frame: Rect) -> (f32, f32) {
    (REACH_PX / frame.width(), REACH_PX / frame.height())
}

/// Draws full-height / full-width lines at the snapped positions.
pub fn paint(p: &Painter, frame: Rect, (x, y): (Option<f32>, Option<f32>)) {
    let stroke = Stroke::new(1.5, ACCENT);
    if let Some(x) = x {
        let px = frame.left() + x * frame.width();
        p.line_segment([pos2(px, frame.top()), pos2(px, frame.bottom())], stroke);
    }
    if let Some(y) = y {
        let py = frame.top() + y * frame.height();
        p.line_segment([pos2(frame.left(), py), pos2(frame.right(), py)], stroke);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centre_snaps_to_the_middle() {
        let s = snap_axis(0.504, 0.2, &[0.5, 0.0, 1.0], 0.01);
        assert_eq!((s.value, s.line), (0.5, Some(0.5)));
    }

    #[test]
    fn edge_snaps_to_the_frame_edge() {
        let s = snap_axis(0.79, 0.2, &[0.5, 1.0], 0.02);
        assert!(
            (s.value - 0.8).abs() < 1e-6 && s.line == Some(1.0),
            "right edge meets the frame edge"
        );
    }

    #[test]
    fn nothing_near_leaves_it_alone() {
        let s = snap_axis(0.3, 0.05, &[0.5, 1.0], 0.01);
        assert_eq!((s.value, s.line), (0.3, None));
    }
}
