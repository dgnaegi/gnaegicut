use super::*;

#[test]
fn the_automatic_position_keeps_a_two_line_caption_inside_the_safe_zone_of_every_format() {
    let layout = CaptionLayout::default();
    for aspect in Aspect::ALL {
        let (_, y) = layout.position(aspect);
        let (_, h) = aspect.size();
        let line = aspect.size().0.min(h) as f32 / 14.0;
        let bottom = y + 1.2 * line / h as f32; // lower edge of two lines, with a little outline
        assert!(
            bottom <= 1.0 - aspect.safe_margins()[3],
            "{}: bottom edge {bottom}",
            aspect.label()
        );
        assert!(
            y - 1.2 * line / h as f32 >= aspect.safe_margins()[1],
            "{} top edge",
            aspect.label()
        );
    }
    for aspect in Aspect::ALL {
        assert!(
            (layout.position(aspect).1 - 0.6).abs() < 1e-6,
            "{}: 60% of the height",
            aspect.label()
        );
    }
}

#[test]
fn it_follows_the_format_and_the_size_until_the_user_moves_it() {
    let mut layout = CaptionLayout::default();
    let normal = layout.position(Aspect::Vertical).1;
    layout.size = 4.0; // so big that two lines at 60% would reach the buttons at the bottom
    assert!(
        layout.position(Aspect::Vertical).1 < normal,
        "huge text is pulled up to stay inside the safe zone"
    );
    layout.place(0.4, 0.3);
    assert_eq!(
        layout.position(Aspect::Wide),
        (0.4, 0.3),
        "a hand-placed caption stays put"
    );
    layout.reset_position();
    assert!(layout.auto_y && layout.x == 0.5);
}

#[test]
fn projects_saved_before_the_automatic_position_get_it() {
    let json = r#"{"x":0.5,"y":0.78,"family":"Inter","bold":true,"size":1.0}"#;
    let layout: CaptionLayout = serde_json::from_str(json).unwrap();
    assert!(layout.auto_y, "old files have no flag and move up into the safe zone");
}
