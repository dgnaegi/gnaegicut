//! Picture effects applied to the item's own frames: its look, glitches, flashes and whip blur.
//! Windows are measured in item time (`t + cut`), so they behave the same wherever playback starts.

use super::item_motion::{clamp01, ease_out};
use super::{Item, Transition};

const WHIP_BLUR: f32 = 0.026; // horizontal blur radius as a fraction of the frame width

impl Item {
    /// Filters between the box scale and the mask / rotation: the look first, then transition effects.
    pub fn fx_filters(&self, frame_w: u32) -> Vec<String> {
        let mut f: Vec<String> = self.look.filters().into_iter().map(String::from).collect();
        let (now, full) = (self.item_time(), self.full_len());
        for (phase, entering) in [(self.enter(), true), (self.exit(), false)] {
            if phase.len <= 0.0 {
                continue;
            }
            let len = phase.len as f64;
            let window = if entering {
                format!("lt({now},{len:.3})")
            } else {
                format!("gt({now},{:.3})", full - len)
            };
            let progress = clamp01(&format!(
                "({now}-{:.3})/{len:.3}",
                if entering { 0.0 } else { full - len }
            ));
            match phase.effect {
                Transition::Glitch => {
                    let on = format!("{window}*gt(sin({now}*61),-0.35)"); // flickers on and off inside the window
                    f.push(format!("chromashift=crh=-22:cbh=22:crv=6:cbv=-6:enable='{on}'"));
                    f.push(format!("noise=alls=55:allf=t:enable='{on}'"));
                    f.push(format!("eq=saturation=2.2:contrast=1.25:enable='{on}'"));
                }
                Transition::Flash => {
                    let white = if entering {
                        format!("1-{}", ease_out(&progress))
                    } else {
                        format!("pow({progress},2)")
                    };
                    f.push(format!("eq=brightness='{white}':eval=frame"));
                }
                effect if effect.blurs() => {
                    let sigma = frame_w as f32 * WHIP_BLUR;
                    f.push(format!("gblur=sigma={sigma:.1}:sigmaV=0.01:enable='{window}'"));
                }
                _ => {}
            }
        }
        f
    }
}
