//! Inputs and feedback built on the basic widgets: percentage sliders, label-driven choices, the busy bar.

use crate::theme::{ACCENT, BLACK, BORDER};
use crate::widgets::{caps, choice_grid};
use eframe::egui::emath::Numeric;
use eframe::egui::{Rect, Sense, Slider, Ui, pos2, vec2};

/// A fraction shown as a percentage, for sliders.
pub fn percent(v: f64, _: std::ops::RangeInclusive<usize>) -> String {
    format!("{:.0}%", v * 100.0)
}

/// A slider for a fraction (0.5 shows as 50%).
pub fn percent_slider<N: Numeric>(ui: &mut Ui, value: &mut N, range: std::ops::RangeInclusive<N>, label: &str) {
    ui.add(Slider::new(value, range).text(label).custom_formatter(percent));
}

/// Like `choice_grid`, for a list of values that know their own label (`Transition::INTRO`, `Look::ALL`, ...).
pub fn choice<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, all: &[T], label: fn(T) -> &'static str) -> bool {
    let options: Vec<_> = all.iter().map(|v| (*v, label(*v))).collect();
    choice_grid(ui, current, &options)
}

/// Something is running in the background: a short red block sweeps along a black rule. No percentages, because
/// ffmpeg cannot tell how far it is; it only says "still working".
pub fn busy(ui: &mut Ui, text: &str) {
    ui.ctx().request_repaint();
    let (rect, _) = ui.allocate_exact_size(vec2(96.0, 14.0), Sense::hover());
    let rail = Rect::from_center_size(rect.center(), vec2(rect.width(), BORDER));
    let p = ui.painter();
    p.rect_filled(rail, 0.0, BLACK);
    let phase = (ui.input(|i| i.time) * 1.2 % 1.0) as f32;
    let block = 28.0;
    let x = rect.left() + (rect.width() - block) * phase;
    p.rect_filled(
        Rect::from_min_size(pos2(x, rect.top() + 3.0), vec2(block, 8.0)),
        0.0,
        ACCENT,
    );
    caps(ui, text);
}
