//! The Transition tab: pick an effect and length for the join between this clip and the one before it.

use crate::app::App;
use crate::project::{Join, JoinEffect};
use crate::widgets::{Kind, button, section, warn};
use eframe::egui::{Slider, Ui};

const MIN_LEN: f32 = 0.2;

pub fn show(ui: &mut Ui, app: &mut App, id: u64) {
    let previous = app
        .project
        .predecessor(id)
        .and_then(|a| app.project.get(a))
        .map(|a| a.name.clone());
    let Some(item) = app.project.get(id) else { return };
    let (name, current) = (item.name.clone(), item.join);
    let Some(previous) = previous else {
        if current.is_some() && button(ui, "Remove transition", Kind::Plain).clicked() {
            app.project.set_join(id, None);
        }
        return;
    };
    section(ui, &format!("{previous} to {name}"));

    let mut choice = current;
    ui.horizontal_wrapped(|ui| {
        let kind = |on| if on { Kind::Active } else { Kind::Plain };
        if button(ui, "None", kind(current.is_none())).clicked() {
            choice = None;
        }
        for effect in JoinEffect::ALL {
            if button(ui, effect.label(), kind(current.is_some_and(|j| j.effect == effect))).clicked() {
                choice = Some(Join {
                    effect,
                    len: current.map_or(0.3, |j| j.len),
                });
            }
        }
    });

    let max = app.project.max_join_len(id);
    match choice {
        Some(join) if max >= MIN_LEN => {
            let mut len = join.len.min(max);
            if ui
                .add(Slider::new(&mut len, MIN_LEN..=max).text("length").suffix("s"))
                .changed()
            {
                choice = Some(Join { len, ..join });
            }
        }
        Some(_) => warn(ui, "Both clips need to be longer to carry a transition"),
        None => {}
    }
    if choice != current {
        app.project.set_join(id, choice);
    }
}
