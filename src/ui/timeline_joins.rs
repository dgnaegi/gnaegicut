//! Markers on the timeline where two clips meet: a "+" to add a transition, or the transition itself.

use super::timeline_items::{LANE, Layout};
use crate::app::{App, Selection, Tab};
use crate::project::{Join, JoinEffect};
use crate::theme::{ACCENT, BLACK, BORDER, WHITE, bold};
use eframe::egui::{Align2, CursorIcon, Id, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

const MIN_WIDTH: f32 = 26.0;
const PLUS: f32 = 22.0;
/// A snappy default: a quick whip, not a slow dissolve.
const DEFAULT: Join = Join {
    effect: JoinEffect::WhipLeft,
    len: 0.3,
};

pub fn show(ui: &mut Ui, app: &mut App, lay: &Layout) {
    let mut clicked: Option<(u64, bool)> = None;
    let p = ui.painter_at(lay.rect);
    for (ti, track) in app.project.tracks.iter().enumerate() {
        for b in &track.items {
            let Some(a) = app.project.predecessor(b.id).and_then(|a| app.project.get(a)) else {
                continue;
            };
            let (top, joint) = (lay.lane_top(ti), lay.x(b.at));
            let (rect, label) = match b.join {
                Some(j) => {
                    let overlaps = j.effect.links(j.len).overlaps;
                    let (x0, x1) = if overlaps {
                        (joint, lay.x(a.end_at()))
                    } else {
                        (joint, joint + j.len * lay.zoom)
                    };
                    let width = (x1 - x0).max(MIN_WIDTH);
                    let left = if overlaps { x0 } else { x0 - width / 2.0 };
                    (
                        Rect::from_min_size(pos2(left, top), vec2(width, LANE)),
                        Some(j.effect.label()),
                    )
                }
                None => (
                    Rect::from_center_size(pos2(joint, top + LANE / 2.0), vec2(PLUS, PLUS)),
                    None,
                ),
            };
            let resp = ui
                .interact(rect, Id::new(("join", b.id)), Sense::click())
                .on_hover_cursor(CursorIcon::PointingHand);
            if resp.clicked() {
                clicked = Some((b.id, label.is_none()));
            }
            match label {
                Some(name) => {
                    p.rect_filled(rect, 0.0, ACCENT);
                    p.rect_stroke(rect, 0.0, Stroke::new(BORDER, WHITE), StrokeKind::Inside);
                    p.text(
                        rect.center_top() + vec2(0.0, 4.0),
                        Align2::CENTER_TOP,
                        "◆",
                        bold(11.0),
                        WHITE,
                    );
                    p.with_clip_rect(rect).text(
                        rect.center_bottom() - vec2(0.0, 4.0),
                        Align2::CENTER_BOTTOM,
                        name.to_uppercase(),
                        bold(8.0),
                        WHITE,
                    );
                }
                None => {
                    p.rect_filled(rect, 0.0, if resp.hovered() { ACCENT } else { WHITE });
                    p.rect_stroke(rect, 0.0, Stroke::new(BORDER, BLACK), StrokeKind::Inside);
                    p.text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "+",
                        bold(14.0),
                        if resp.hovered() { WHITE } else { BLACK },
                    );
                }
            }
        }
    }
    if let Some((id, add)) = clicked {
        app.pause_for_edit();
        if add {
            app.project.set_join(id, Some(DEFAULT));
        }
        app.select(Selection::Item(id));
        app.tab = Tab::Transition;
    }
}
