//! The inspector tabs that edit one item: placement, motion, audio and timing.

use crate::app::App;
use crate::project::{Kind, Look, Transition, ZoomEffect};
use crate::widgets::{Kind as Btn, button, caps, chip, choice_grid, section};
use eframe::egui::{Slider, Ui, vec2};

fn percent(v: f64, _: std::ops::RangeInclusive<usize>) -> String {
    format!("{:.0}%", v * 100.0)
}

pub fn placement(ui: &mut Ui, app: &mut App, id: u64) {
    let aspect = app.project.aspect;
    let Some(it) = app.project.get_mut(id) else { return };
    caps(ui, &it.name);
    ui.add(Slider::new(&mut it.x, -0.5..=1.5).text("x").custom_formatter(percent));
    ui.add(Slider::new(&mut it.y, -0.5..=1.5).text("y").custom_formatter(percent));
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
    let effects: Vec<_> = ZoomEffect::ALL.iter().map(|e| (*e, e.label())).collect();
    choice_grid(ui, &mut it.effect, &effects);
    if it.effect != ZoomEffect::None {
        ui.add(
            Slider::new(&mut it.amount, 0.05..=1.0)
                .text("strength")
                .custom_formatter(percent),
        );
    }
    ui.add_space(12.0);
    section(ui, "Look and shake");
    let looks: Vec<_> = Look::ALL.iter().map(|l| (*l, l.label())).collect();
    choice_grid(ui, &mut it.look, &looks);
    ui.add(
        Slider::new(&mut it.shake, 0.0..=1.0)
            .text("shake")
            .custom_formatter(percent),
    );
    ui.add_space(12.0);
    section(ui, "Intro / outro");
    let kinds: Vec<_> = Transition::BASIC.iter().map(|t| (*t, t.label())).collect();
    choice_grid(ui, &mut it.transition, &kinds);
    ui.add(Slider::new(&mut it.fade_in, 0.0..=2.0).text("in").suffix("s"));
    ui.add(Slider::new(&mut it.fade_out, 0.0..=2.0).text("out").suffix("s"));
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
    ui.add(
        Slider::new(&mut it.volume, 0.0..=2.0)
            .text("volume")
            .custom_formatter(percent),
    );
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
