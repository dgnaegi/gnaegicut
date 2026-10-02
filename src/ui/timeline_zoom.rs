//! Zooming the time axis of the tracks: pinch or Cmd+scroll at the pointer, Cmd +/- from the keyboard, Cmd+0 to
//! fit everything. The point under the pointer stays where it is while the rest stretches around it.

use super::timeline_items::GUTTER;
use crate::app::App;

pub const MIN_ZOOM: f32 = 2.0; // pixels per second
pub const MAX_ZOOM: f32 = 800.0;
const MARGIN: f32 = 32.0;

/// The scroll offset that keeps the time under `anchor_x` in place when the zoom changes.
/// `time_origin` is where second 0 is on screen now, `viewport_left` the left edge of the visible tracks.
pub fn zoomed_offset(time_origin: f32, viewport_left: f32, anchor_x: f32, old: f32, new: f32) -> f32 {
    let seconds_under_anchor = (anchor_x - time_origin) / old;
    (GUTTER + seconds_under_anchor * new - (anchor_x - viewport_left)).max(0.0)
}

impl App {
    /// Multiplies the zoom by `factor`, around `anchor_x` (the centre of the visible tracks when `None`).
    pub fn zoom_timeline(&mut self, factor: f32, anchor_x: Option<f32>) {
        let Some(view) = self.timeline_view else { return };
        let new = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        if (new - self.zoom).abs() < 1e-4 {
            return;
        }
        let anchor = anchor_x.unwrap_or_else(|| view.visible.center().x);
        self.scroll_to = Some(zoomed_offset(
            view.time_origin,
            view.visible.left(),
            anchor,
            self.zoom,
            new,
        ));
        self.zoom = new;
    }

    /// Fits the whole edit into the visible tracks.
    pub fn fit_timeline(&mut self) {
        let Some(view) = self.timeline_view else { return };
        let room = view.visible.width() - GUTTER - MARGIN;
        self.zoom = (room / self.project.total().max(1.0) as f32).clamp(MIN_ZOOM, MAX_ZOOM);
        self.scroll_to = Some(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drop::TimelineView;
    use crate::project::{Item, Kind};
    use eframe::egui::{Context, Pos2, Rect};

    fn view(time_origin: f32) -> TimelineView {
        TimelineView {
            visible: Rect::from_min_max(Pos2::new(100.0, 0.0), Pos2::new(1000.0, 300.0)),
            time_origin,
            zoom: 60.0,
            lanes: 1,
            lanes_top: 60.0,
            lane_pitch: 48.0,
        }
    }

    #[test]
    fn the_time_under_the_pointer_stays_put() {
        let (viewport_left, anchor, old, new) = (100.0, 500.0, 60.0, 180.0);
        let time_origin = viewport_left - 120.0 + GUTTER; // scrolled 120 px
        let seconds = (anchor - time_origin) / old;
        let offset = zoomed_offset(time_origin, viewport_left, anchor, old, new);
        let new_origin = viewport_left - offset + GUTTER;
        assert!(
            ((anchor - new_origin) / new - seconds).abs() < 1e-3,
            "same second under the pointer"
        );
    }

    #[test]
    fn zooming_changes_the_zoom_within_limits_and_asks_for_a_scroll() {
        let mut app = App::new(Context::default());
        assert!(app.timeline_view.is_none());
        app.zoom_timeline(2.0, None);
        assert_eq!(app.zoom, 60.0, "nothing to zoom before the timeline has been drawn");

        app.timeline_view = Some(view(156.0));
        app.zoom_timeline(2.0, Some(500.0));
        assert_eq!(app.zoom, 120.0);
        assert!(app.scroll_to.is_some());
        app.zoom_timeline(1000.0, None);
        assert_eq!(app.zoom, MAX_ZOOM);
        app.zoom_timeline(0.00001, None);
        assert_eq!(app.zoom, MIN_ZOOM);
    }

    #[test]
    fn fit_shows_the_whole_edit_from_the_start() {
        let mut app = App::new(Context::default());
        app.timeline_view = Some(view(156.0));
        let clip = Item::new("a.mp4".into(), "a".into(), Kind::Video, (160, 90), 10.0, false);
        app.project.add(0, clip);
        app.fit_timeline();
        let room = 900.0 - GUTTER - MARGIN;
        assert!(
            (app.zoom - room / 10.0).abs() < 1e-3,
            "10 seconds across the visible width: {}",
            app.zoom
        );
        assert_eq!(app.scroll_to, Some(0.0));
    }
}
