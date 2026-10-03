//! Swiss International design tokens and egui style. Single source of truth for look and feel.

use eframe::egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style, TextStyle,
    style::HandleShape,
};
use std::sync::Arc;

pub const WHITE: Color32 = Color32::WHITE;
pub const BLACK: Color32 = Color32::BLACK;
pub const MUTED: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xF2);
pub const ACCENT: Color32 = Color32::from_rgb(0xFF, 0x30, 0x00);
/// Secondary text: 5.9:1 on white, so it stays readable at caption size.
pub const SUBTLE: Color32 = Color32::from_gray(100);
/// Dims the parts of the picture that platform buttons cover.
pub const DIM: Color32 = Color32::from_black_alpha(110);
pub const BORDER: f32 = 2.0;
/// Type scale: every font size in the interface is one of these.
pub const CAPTION: f32 = 10.0;
pub const SMALL: f32 = 11.0;
pub const LABEL: f32 = 12.0;
pub const BODY: f32 = 13.0;
pub const LARGE: f32 = 14.0;
pub const TITLE: f32 = 22.0;
/// Side panels keep the stage usable: neither too narrow to read nor wide enough to squeeze the picture.
pub const PANEL_MIN: f32 = 260.0;
pub const PANEL_MAX: f32 = 400.0;
pub const PAD: f32 = 12.0;

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

pub fn black(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("black".into()))
}

fn fonts() -> FontDefinitions {
    let faces: [(&str, &[u8]); 3] = [
        ("regular", include_bytes!("../assets/fonts/Inter-Regular.ttf")),
        ("bold", include_bytes!("../assets/fonts/Inter-Bold.ttf")),
        ("black", include_bytes!("../assets/fonts/Inter-Black.ttf")),
    ];
    let mut defs = FontDefinitions::default();
    for (name, bytes) in faces {
        defs.font_data
            .insert(name.into(), Arc::new(FontData::from_static(bytes)));
        defs.families
            .entry(FontFamily::Name(name.into()))
            .or_default()
            .push(name.into());
    }
    defs.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "regular".into());
    defs
}

pub fn apply(ctx: &Context) {
    ctx.set_fonts(fonts());
    let text_styles = [
        (TextStyle::Heading, black(TITLE)),
        (TextStyle::Body, FontId::proportional(BODY)),
        (TextStyle::Button, bold(LABEL)),
        (TextStyle::Small, FontId::proportional(SMALL)),
        (TextStyle::Monospace, FontId::monospace(12.0)),
    ]
    .into();
    let mut style = Style {
        text_styles,
        ..Default::default()
    };
    style.spacing.item_spacing = [8.0, 8.0].into();
    style.spacing.interact_size.y = 28.0;

    let v = &mut style.visuals;
    v.dark_mode = false;
    v.override_text_color = Some(BLACK);
    v.panel_fill = WHITE;
    v.window_fill = WHITE;
    v.extreme_bg_color = WHITE;
    v.faint_bg_color = MUTED;
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(BORDER, WHITE);
    let w = &mut v.widgets;
    for s in [
        &mut w.noninteractive,
        &mut w.inactive,
        &mut w.hovered,
        &mut w.active,
        &mut w.open,
    ] {
        s.corner_radius = CornerRadius::ZERO;
        s.expansion = 0.0;
        s.bg_stroke = Stroke::new(BORDER, BLACK);
        s.fg_stroke = Stroke::new(BORDER, BLACK);
        s.bg_fill = WHITE;
        s.weak_bg_fill = WHITE;
    }
    w.inactive.bg_fill = MUTED; // slider rails and drag-value fields
    w.hovered.bg_fill = ACCENT;
    w.hovered.weak_bg_fill = ACCENT;
    w.hovered.bg_stroke = Stroke::new(BORDER, ACCENT);
    w.active.bg_fill = BLACK;
    w.active.weak_bg_fill = BLACK;
    style.visuals.slider_trailing_fill = true;
    style.visuals.handle_shape = HandleShape::Rect { aspect_ratio: 0.5 };
    style.spacing.slider_rail_height = 6.0;
    ctx.set_global_style(style);
}
