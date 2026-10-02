//! Reusable Swiss-style building blocks. Every panel composes these instead of styling ad hoc.

use crate::theme::{ACCENT, BLACK, BORDER, MUTED, WHITE, bold};
use eframe::egui::{Align2, Color32, CursorIcon, Rect, Response, Sense, Stroke, StrokeKind, Ui, vec2};

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Plain,  // white, inverts on hover
    Active, // black, goes red on hover (selected state)
    Cta,    // red, goes black on hover (primary call to action)
}

pub fn button(ui: &mut Ui, text: &str, kind: Kind) -> Response {
    let label = text.to_uppercase();
    let font = bold(12.0);
    let width = ui.painter().layout_no_wrap(label.clone(), font.clone(), BLACK).size().x;
    let (rect, resp) = ui.allocate_exact_size(vec2(width + 28.0, 36.0), Sense::CLICK);
    let hot = resp.hovered() || resp.is_pointer_button_down_on();
    let (bg, fg): (Color32, Color32) = match (kind, hot) {
        (Kind::Plain, false) => (WHITE, BLACK),
        (Kind::Plain, true) | (Kind::Active, false) | (Kind::Cta, true) => (BLACK, WHITE),
        (Kind::Active, true) | (Kind::Cta, false) => (ACCENT, WHITE),
    };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(
        rect,
        0.0,
        Stroke::new(BORDER, if bg == ACCENT { ACCENT } else { BLACK }),
        StrokeKind::Inside,
    );
    p.text(rect.center(), Align2::CENTER_CENTER, label, font, fg);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// A row of mutually exclusive buttons; returns true if the selection changed.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    let mut row = |ui: &mut Ui| {
        ui.spacing_mut().item_spacing.x = -BORDER; // borders overlap into one shared line
        for (value, label) in options {
            let kind = if current == value { Kind::Active } else { Kind::Plain };
            if button(ui, label, kind).clicked() && current != value {
                *current = *value;
                changed = true;
            }
        }
    };
    // Already inside a row (the toolbar)? Then just add the buttons; otherwise start one.
    if ui.layout().main_dir().is_horizontal() {
        ui.scope(&mut row);
    } else {
        ui.horizontal(row);
    }
    changed
}

/// A wrapped grid of mutually exclusive buttons, for lists too long for one row. Returns true if the choice changed.
pub fn choice_grid<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
        for (value, label) in options {
            let kind = if current == value { Kind::Active } else { Kind::Plain };
            if button(ui, label, kind).clicked() && current != value {
                *current = *value;
                changed = true;
            }
        }
    });
    changed
}

/// A small group heading with a rule beneath: a red marker and an uppercase title.
pub fn section(ui: &mut Ui, title: &str) {
    ui.horizontal(|ui| {
        let (marker, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
        ui.painter().rect_filled(marker, 0.0, ACCENT);
        ui.label(eframe::egui::RichText::new(title.to_uppercase()).font(bold(12.0)));
    });
    rule(ui);
}

/// Underlined tabs: the active one is black with a thick red bar, the rest are grey and turn red on hover.
/// Returns true if the selection changed.
pub fn tabs<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
        for (value, label) in options {
            let label = label.to_uppercase();
            let width = ui.painter().layout_no_wrap(label.clone(), bold(12.0), BLACK).size().x;
            let (rect, resp) = ui.allocate_exact_size(vec2(width + 20.0, 34.0), Sense::CLICK);
            let active = current == value;
            let ink = if active {
                BLACK
            } else if resp.hovered() {
                ACCENT
            } else {
                Color32::from_gray(120)
            };
            ui.painter().text(
                rect.center() - vec2(0.0, 2.0),
                Align2::CENTER_CENTER,
                label,
                bold(12.0),
                ink,
            );
            if active {
                let bar = Rect::from_min_max(rect.left_bottom() - vec2(0.0, 4.0), rect.right_bottom());
                ui.painter().rect_filled(bar, 0.0, ACCENT);
            }
            if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() && !active {
                *current = *value;
                changed = true;
            }
        }
    });
    rule(ui);
    changed
}

pub fn rule(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), BORDER), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, BLACK);
}

/// Small uppercase caption for secondary information.
pub fn caps(ui: &mut Ui, text: &str) {
    ui.label(
        eframe::egui::RichText::new(text.to_uppercase())
            .font(bold(10.0))
            .color(Color32::from_gray(90)),
    );
}

/// One line of a menu: full width, left-aligned, inverting on hover.
pub fn menu_item(ui: &mut Ui, text: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width().max(180.0), 34.0), Sense::CLICK);
    let hot = resp.hovered();
    let p = ui.painter();
    p.rect_filled(rect, 0.0, if hot { BLACK } else { WHITE });
    p.text(
        rect.left_center() + vec2(12.0, 0.0),
        Align2::LEFT_CENTER,
        text.to_uppercase(),
        bold(12.0),
        if hot { WHITE } else { BLACK },
    );
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// A small tag-like button, for quick suggestions.
pub fn chip(ui: &mut Ui, text: &str) -> Response {
    let label = text.to_uppercase();
    let width = ui.painter().layout_no_wrap(label.clone(), bold(10.0), BLACK).size().x;
    let (rect, resp) = ui.allocate_exact_size(vec2(width + 16.0, 24.0), Sense::CLICK);
    let (bg, fg) = if resp.hovered() {
        (ACCENT, WHITE)
    } else {
        (WHITE, BLACK)
    };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(
        rect,
        0.0,
        Stroke::new(BORDER, if resp.hovered() { ACCENT } else { BLACK }),
        StrokeKind::Inside,
    );
    p.text(rect.center(), Align2::CENTER_CENTER, label, bold(10.0), fg);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Small uppercase notice in the accent colour, for things the user should act on.
pub fn warn(ui: &mut Ui, text: &str) {
    ui.label(
        eframe::egui::RichText::new(text.to_uppercase())
            .font(bold(10.0))
            .color(ACCENT),
    );
}

/// A bordered surface, optionally on the muted background.
pub fn surface(ui: &mut Ui, rect: Rect, muted: bool) {
    let p = ui.painter();
    p.rect_filled(rect, 0.0, if muted { MUTED } else { WHITE });
    p.rect_stroke(rect, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
}
