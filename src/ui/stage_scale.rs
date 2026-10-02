//! Resizing from the preview: the mouse wheel and the pinch gesture scale whatever is selected, and handles stay
//! reachable even when an element is larger than the picture.

use eframe::egui::{Pos2, Rect, Ui, Vec2};

/// One step of the wheel or a pinch as a scale factor. Limited, so a fast flick cannot make an element vanish.
pub fn wheel_factor(pinch: f32, scroll_y: f32) -> f32 {
    (pinch * (1.0 + scroll_y * 0.0025)).clamp(0.8, 1.25)
}

/// The scale factor asked for this frame by the wheel or a pinch over `area` (1.0 when there is none).
pub fn input_factor(ui: &Ui, area: Rect) -> f32 {
    if !ui.rect_contains_pointer(area) {
        return 1.0;
    }
    wheel_factor(ui.input(|i| i.zoom_delta()), ui.input(|i| i.smooth_scroll_delta.y))
}

/// Pulls a handle position back inside the stage, so a handle of an element that is bigger than the picture can
/// still be grabbed.
pub fn reachable(p: Pos2, area: Rect, margin: f32) -> Pos2 {
    p.clamp(area.min + Vec2::splat(margin), area.max - Vec2::splat(margin))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::project::{Item, Kind};
    use eframe::egui::{Context, pos2};

    #[test]
    fn wheel_and_pinch_scale_in_the_expected_direction_and_are_limited() {
        assert_eq!(wheel_factor(1.0, 0.0), 1.0);
        assert!(wheel_factor(1.0, 40.0) > 1.0 && wheel_factor(1.0, -40.0) < 1.0);
        assert!(wheel_factor(1.1, 0.0) > 1.0, "a pinch outwards grows");
        assert_eq!(wheel_factor(1.0, 5000.0), 1.25);
        assert_eq!(wheel_factor(0.1, 0.0), 0.8);
    }

    #[test]
    fn handles_outside_the_stage_are_pulled_back_in() {
        let area = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 300.0));
        assert_eq!(reachable(pos2(-50.0, 500.0), area, 10.0), pos2(10.0, 290.0));
        assert_eq!(
            reachable(pos2(200.0, 100.0), area, 10.0),
            pos2(200.0, 100.0),
            "inside stays put"
        );
    }

    #[test]
    fn scaling_an_image_changes_its_size_within_limits_and_text_changes_its_font() {
        let mut app = App::new(Context::default());
        let image = Item::new("p.png".into(), "p".into(), Kind::Image, (100, 100), 0.0, false);
        let image = app.project.add(0, image);
        app.scale_item(image, 2.0);
        assert!((app.project.get(image).unwrap().scale - 2.0).abs() < 1e-6);
        app.scale_item(image, 100.0);
        assert_eq!(app.project.get(image).unwrap().scale, 8.0, "never beyond the limit");
        app.scale_item(image, 0.0001);
        assert_eq!(app.project.get(image).unwrap().scale, 0.05, "never vanishing");

        let text = Item::new("t.png".into(), "t".into(), Kind::Text, (100, 50), 0.0, false);
        let text = app.project.add(1, text);
        let before = app.project.get(text).unwrap().font_size;
        app.scale_item(text, 1.5);
        let it = app.project.get(text).unwrap();
        assert!(
            (it.font_size - before * 1.5).abs() < 1e-3,
            "text stays sharp by growing the font, not the picture"
        );
        assert_eq!(it.scale, 1.0);
    }
}
