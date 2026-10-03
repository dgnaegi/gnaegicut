//! How an item moves and changes over time, as ffmpeg expressions: zoom, rotation, slides, shake, fades and reveals.
//! Everything eases: fast at the start and settling softly, which is what makes cuts feel snappy.
//!
//! Two clocks appear in the expressions. *Item time* is `t + cut`: seconds since the item started, even when
//! playback began part way through it. *Output time* is `t`, used by the overlay, which sees the whole timeline.

use super::kinds::Phase;
use super::{Item, Transition, ZoomEffect};

pub(super) fn clamp01(v: &str) -> String {
    format!("min(1,max(0,{v}))")
}

/// How hard the easing snaps: the higher, the more of the move happens in the first moments, then it holds.
const SNAP: u32 = 4;

/// Fast start, soft landing: a tick, then a tock.
pub fn ease_out(p: &str) -> String {
    format!("(1-pow(1-({p}),{SNAP}))")
}

/// Slow start, fast exit.
pub(super) fn ease_in(p: &str) -> String {
    format!("pow({p},{SNAP})")
}

const PUNCH_SECS: f64 = 0.6; // how long a punch-in or punch-out takes
const BEAT_HZ: f32 = 2.5; // pulses per second of the Beat zoom
const ZOOM_IN_GROWTH: f32 = 1.0; // a zoom entrance starts twice as big
const SLAM_DAMP: f32 = 7.0; // how fast the slam settles
const SLAM_HZ: f32 = 16.0; // radians per second of its springing
const SPIN_RADIANS: f32 = -4.712_389; // three quarters of a turn, counter-clockwise

impl Item {
    /// Seconds since the item started, as seen by the item's own stream filters.
    pub(super) fn item_time(&self) -> String {
        format!("(t+{:.3})", self.cut)
    }

    /// Seconds since the item started, as seen by the overlay (which runs on output time).
    fn overlay_age(&self) -> String {
        format!("(t+{:.3}-{:.3})", self.cut, self.at)
    }

    /// Length of the item counting the part cut off its front, so animations tied to the end stay put.
    pub(super) fn full_len(&self) -> f64 {
        self.len() + self.cut
    }

    /// Whether the entrance spins the item.
    pub fn spins(&self) -> bool {
        let enter = self.enter();
        enter.effect == Transition::Spin && enter.len > 0.0
    }

    /// ffmpeg expression for the content magnification at local time `t`, if it ever differs from 1.
    pub fn zoom_expr(&self) -> Option<String> {
        let (z, a, now) = (self.zoom, self.amount, self.item_time());
        let punch = clamp01(&format!("{now}/{:.3}", self.full_len().clamp(0.1, PUNCH_SECS)));
        let base = match self.effect {
            ZoomEffect::None => ((z - 1.0).abs() > 1e-3).then(|| format!("{z:.4}")),
            ZoomEffect::In => Some(format!("{z:.4}*(1+{a:.3}*{})", ease_out(&punch))),
            ZoomEffect::Out => Some(format!("{z:.4}*(1+{a:.3}*(1-{}))", ease_out(&punch))),
            // A damped spring: shoots past the target, then settles.
            ZoomEffect::Slam => Some(format!(
                "{z:.4}*(1+{a:.3}*(1-exp(-{SLAM_DAMP}*{now})*cos({SLAM_HZ}*{now})))"
            )),
            ZoomEffect::Pulse => Some(format!("{z:.4}*(1+{a:.3}*0.5*(1-cos(2*PI*{BEAT_HZ}*{now})))")),
        };
        let enter = self.enter();
        if enter.effect != Transition::Spin || enter.len <= 0.0 {
            return base;
        }
        // A spin starts 80% closer, inside its box, and settles to normal.
        let settle = ease_out(&clamp01(&format!("{now}/{:.3}", enter.len)));
        Some(format!("({})*(1+0.8*(1-{settle}))", base.unwrap_or_else(|| "1".into())))
    }

    /// How much bigger than its box a zooming entrance is right now, as an expression: it starts too big for the
    /// frame, so it is not cut off at the box, and shrinks into place. `None` for every other item.
    pub fn grow_expr(&self) -> Option<String> {
        let enter = self.enter();
        if enter.effect != Transition::Zoom || enter.len <= 0.0 || self.rotation != 0.0 {
            return None;
        }
        let settle = ease_out(&clamp01(&format!("{}/{:.3}", self.item_time(), enter.len)));
        Some(format!("(1+{ZOOM_IN_GROWTH}*(1-{settle}))"))
    }

    /// The rotation as an ffmpeg expression in radians: the fixed angle, plus the spin of an entrance.
    pub fn rotation_expr(&self) -> String {
        let fixed = self.rotation.to_radians();
        if !self.spins() {
            return format!("{fixed:.5}");
        }
        let progress = ease_out(&clamp01(&format!("{}/{:.3}", self.item_time(), self.enter().len)));
        format!("{fixed:.5}+{SPIN_RADIANS:.5}*(1-{progress})")
    }

    /// Whether the picture needs an alpha channel (rotation corners, fades, masks, stills with transparency).
    pub fn needs_alpha(&self) -> bool {
        self.kind.is_still()
            || self.rotation != 0.0
            || self.spins()
            || self.fade_lens() != (0.0, 0.0)
            || self.reveal().is_some()
            || self.edge.feather > 0.0
            || self.edge.border > 0.0
    }

    /// Lengths of the alpha fades at the start and end of the item.
    fn fade_lens(&self) -> (f32, f32) {
        let (enter, exit) = (self.enter(), self.exit());
        let fades_in = matches!(enter.effect, Transition::Fade | Transition::Zoom | Transition::Spin);
        (
            if fades_in { enter.len } else { 0.0 },
            if exit.effect == Transition::Fade { exit.len } else { 0.0 },
        )
    }

    /// The mask effect that reveals the item as it enters, with its length.
    pub fn reveal(&self) -> Option<Phase> {
        Some(self.enter()).filter(|p| p.effect.is_reveal() && p.len > 0.0)
    }

    /// Alpha fades, in item time, so playing from the middle of an item looks identical to playing it from the start.
    pub fn fade_filters(&self) -> Vec<String> {
        let (fade_in, fade_out) = self.fade_lens();
        if fade_in <= 0.0 && fade_out <= 0.0 {
            return vec![];
        }
        let mut f = vec![format!("setpts=PTS-STARTPTS+{:.3}/TB", self.cut)];
        if fade_in > 0.0 {
            f.push(format!("fade=t=in:st=0:d={fade_in:.3}:alpha=1"));
        }
        if fade_out > 0.0 {
            let start = (self.full_len() - fade_out as f64).max(0.0);
            f.push(format!("fade=t=out:st={start:.3}:d={fade_out:.3}:alpha=1"));
        }
        f.push("setpts=PTS-STARTPTS".into());
        f
    }

    /// Overlay `x`/`y` expressions of the item's box origin: slides and whips ease in and out, a shake adds
    /// jitter, and a glitch adds sudden jumps during its entrance or exit.
    pub fn motion(&self, (ox, oy): (i32, i32), (w, h): (u32, u32)) -> (String, String) {
        let (enter, exit) = (self.enter(), self.exit());
        let age = self.overlay_age();
        let entered = if enter.len > 0.0 {
            ease_out(&clamp01(&format!("{age}/{:.3}", enter.len)))
        } else {
            "1".into()
        };
        let left_at = self.full_len() - exit.len as f64;
        let leaving = if exit.len > 0.0 {
            ease_in(&clamp01(&format!("({age}-{left_at:.3})/{:.3}", exit.len)))
        } else {
            "0".into()
        };

        let (din, dout) = (enter.effect.direction(), exit.effect.direction());
        let (mut xs, mut ys): (Vec<String>, Vec<String>) = (vec![], vec![]);
        // An item enters from the side it travels away from, and leaves towards the side it travels to.
        for (parts, vin, vout, span) in [(&mut xs, din.0, dout.0, w), (&mut ys, din.1, dout.1, h)] {
            if vin != 0.0 {
                parts.push(format!("({vin:.0}*{span})*(-(1-{entered}))"));
            }
            if vout != 0.0 {
                parts.push(format!("({vout:.0}*{span})*{leaving}"));
            }
        }
        if self.shake > 0.0 {
            let amp = |span: u32| self.shake * 0.025 * span as f32;
            xs.push(format!("{:.2}*(sin(t*47)+0.6*sin(t*91+1))", amp(w)));
            ys.push(format!("{:.2}*(sin(t*53+2)+0.6*sin(t*83))", amp(h)));
        }
        for (phase, entering) in [(enter, true), (exit, false)] {
            if phase.effect == Transition::Glitch && phase.len > 0.0 {
                let on = if entering {
                    format!("lt({age},{:.3})", phase.len)
                } else {
                    format!("gt({age},{left_at:.3})")
                };
                xs.push(format!("{on}*{:.1}*sin(t*140)", 0.035 * w as f32));
                ys.push(format!("{on}*{:.1}*sin(t*97)", 0.01 * h as f32));
            }
        }
        let join = |o: i32, parts: Vec<String>| {
            if parts.is_empty() {
                o.to_string()
            } else {
                format!("{o}+{}", parts.join("+"))
            }
        };
        (join(ox, xs), join(oy, ys))
    }
}
