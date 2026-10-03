//! The inspector tabs that edit one item: placement, motion, audio and timing.

use crate::app::App;
use crate::project::edge::{MAX_BORDER, MAX_FEATHER};
use crate::project::{Crop, Kind, Look, Transition, ZoomEffect, crop};
use crate::widgets::{Kind as Btn, button, caps, chip, choice, percent, percent_slider, section};
use eframe::egui::{Slider, Ui, vec2};

pub fn placement(ui: &mut Ui, app: &mut App, id: u64) {
    let aspect = app.project.aspect;
    let Some(it) = app.project.get_mut(id) else { return };
    caps(ui, &it.name);
    percent_slider(ui, &mut it.x, -0.5..=1.5, "x");
    percent_slider(ui, &mut it.y, -0.5..=1.5, "y");
    ui.add(
        Slider::new(&mut it.scale, 0.05..=6.0)
            .logarithmic(true)
            .text("size")
            .custom_formatter(percent),
    );
    ui.add(Slider::new(&mut it.rotation, -180.0..=180.0).text("rotate").suffix("°"));
    ui.horizontal_wrapped(|ui| {
        if button(ui, "Fit", Btn::Plain).clicked() {
            it.scale = 1.0;
        }
        if button(ui, "Fill", Btn::Plain).clicked() {
            it.scale = it.cover_scale(aspect);
        }
        if button(ui, "Center", Btn::Plain).clicked() {
            (it.x, it.y) = (0.5, 0.5);
        }
        if button(ui, "Straighten", Btn::Plain).clicked() {
            it.rotation = 0.0;
        }
    });
    if it.kind != Kind::Text && it.kind != Kind::Audio {
        crop(ui, it);
        edge(ui, it);
    }
}

/// A soft fade around the picture, and a border.
fn edge(ui: &mut Ui, it: &mut crate::project::Item) {
    ui.add_space(12.0);
    section(ui, "Edge");
    ui.add(
        Slider::new(&mut it.edge.feather, 0.0..=MAX_FEATHER)
            .text("fade")
            .custom_formatter(percent),
    );
    ui.add(
        Slider::new(&mut it.edge.border, 0.0..=MAX_BORDER)
            .text("border")
            .custom_formatter(percent),
    );
    if it.edge.border > 0.0 {
        ui.color_edit_button_srgb(&mut it.edge.border_color);
    }
}

/// Cutting away the edges of a picture or video: one slider per side.
fn crop(ui: &mut Ui, it: &mut crate::project::Item) {
    ui.add_space(12.0);
    section(ui, "Crop");
    let before = it.crop;
    for (side, label) in [
        (&mut it.crop.left, "left"),
        (&mut it.crop.right, "right"),
        (&mut it.crop.top, "top"),
        (&mut it.crop.bottom, "bottom"),
    ] {
        ui.add(
            Slider::new(side, 0.0..=crop::MOST)
                .text(label)
                .custom_formatter(percent),
        );
    }
    if it.crop != before {
        it.crop = it.crop.limited();
    }
    if !it.crop.is_none() && button(ui, "Remove crop", Btn::Plain).clicked() {
        it.crop = Crop::default();
    }
}

/// Everything that makes a clip move or feel punchy: zoom, look, shake, and its own intro and outro.
pub fn motion(ui: &mut Ui, app: &mut App, id: u64) {
    let Some(it) = app.project.get_mut(id) else { return };
    section(ui, "Zoom");
    ui.add(
        Slider::new(&mut it.zoom, 1.0..=4.0)
            .text("zoom")
            .custom_formatter(|v, _| format!("{v:.2}x")),
    );
    choice(ui, &mut it.effect, &ZoomEffect::ALL, ZoomEffect::label);
    if it.effect != ZoomEffect::None {
        percent_slider(ui, &mut it.amount, 0.05..=1.0, "strength");
    }
    ui.add_space(12.0);
    section(ui, "Look and shake");
    choice(ui, &mut it.look, &Look::ALL, Look::label);
    percent_slider(ui, &mut it.shake, 0.0..=1.0, "shake");
    if it.kind == Kind::Video {
        ui.checkbox(&mut it.stabilize, "Stabilize shaky video");
    }
    ui.add_space(12.0);
    section(ui, "Intro");
    choice(ui, &mut it.transition, &Transition::INTRO, Transition::label);
    ui.add(Slider::new(&mut it.fade_in, 0.0..=2.0).text("length").suffix("s"));
    ui.add_space(12.0);
    section(ui, "Outro");
    let mut outro = it.outro_effect();
    if choice(ui, &mut outro, &Transition::OUTRO, Transition::label) {
        it.outro = Some(outro);
    }
    ui.add(Slider::new(&mut it.fade_out, 0.0..=2.0).text("length").suffix("s"));
}

/// One-click voice clean-up and gain for clips that have sound.
const FADE_PRESETS: [f32; 4] = [0.5, 1.0, 2.0, 3.0];
const DEFAULT_END_FADE: f32 = 2.0;

pub fn audio(ui: &mut Ui, app: &mut App, id: u64) {
    let Some(it) = app.project.get_mut(id) else { return };
    let kind = if it.enhance { Btn::Active } else { Btn::Cta };
    if button(
        ui,
        if it.enhance {
            "Enhanced voice: on"
        } else {
            "Enhance voice"
        },
        kind,
    )
    .clicked()
    {
        it.enhance = !it.enhance;
    }
    percent_slider(ui, &mut it.volume, 0.0..=2.0, "volume");
    if it.kind != Kind::Audio {
        return;
    }
    ui.add_space(12.0);
    section(ui, "Fade out");
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        for secs in FADE_PRESETS {
            if chip(ui, &format!("{secs}s")).clicked() {
                it.fade_out = secs.min(it.len() as f32);
            }
        }
    });
    ui.add(Slider::new(&mut it.fade_out, 0.0..=10.0).text("fade out").suffix("s"));
    ui.add(Slider::new(&mut it.fade_in, 0.0..=10.0).text("fade in").suffix("s"));
    // Cuts the music where the picture ends and fades it out there.
    let to_end = button(ui, "Fade out at end of video", Btn::Cta).clicked();
    if to_end {
        let secs = if it.fade_out > 0.0 {
            it.fade_out
        } else {
            DEFAULT_END_FADE
        };
        app.project.fade_out_at_end(id, secs);
    }
}

pub fn timing(ui: &mut Ui, app: &mut App, id: u64) {
    let Some(it) = app.project.get_mut(id) else { return };
    ui.add(Slider::new(&mut it.at, 0.0..=600.0).text("starts at").suffix("s"));
    if it.kind == Kind::Video {
        let src = it.src_len;
        ui.add(
            Slider::new(&mut it.start, 0.0..=(it.end - 0.1).max(0.0))
                .text("in")
                .suffix("s"),
        );
        ui.add(
            Slider::new(&mut it.end, (it.start + 0.1).min(src)..=src)
                .text("out")
                .suffix("s"),
        );
    } else {
        let mut len = it.len();
        if ui
            .add(Slider::new(&mut len, 0.2..=60.0).text("length").suffix("s"))
            .changed()
        {
            it.end = it.start + len;
        }
    }
}
