//! Masks for wipe and circle transitions: a grey video that is white where the item is already revealed.
//! The mask is only animated for the first part of the item; after that it is plain white.

use crate::project::{Item, Kind, Transition, ease_out};

/// How wide the soft edge of a fading wipe is, as a fraction of the width.
const FEATHER: f64 = 0.35;

/// Filter statements that produce the item's reveal mask as `[m{i}]`, sized to the item's box.
pub fn mask(i: usize, item: &Item, (bw, bh): (u32, u32)) -> Option<String> {
    let phase = item.reveal()?;
    let head = phase.len as f64 - item.cut; // part of the animation still ahead of us
    if head <= 0.03 {
        return None;
    }
    let p = ease_out(&format!("clip((T+{:.3})/{:.3},0,1)", item.cut, phase.len));
    // A lower third fades in along the wipe: the edge is a soft ramp, so the text appears from left to right.
    let soft = |dist: String| format!("255*clip(({dist})/(W*{FEATHER}),0,1)");
    let lum = match phase.effect {
        Transition::WipeRight if item.bar => soft(format!("W*{p}*(1+{FEATHER})-X")),
        Transition::WipeRight => format!("255*lt(X,W*{p})"),
        Transition::WipeLeft => format!("255*gt(X,W*(1-{p}))"),
        Transition::WipeDown => format!("255*lt(Y,H*{p})"),
        Transition::WipeUp => format!("255*gt(Y,H*(1-{p}))"),
        Transition::Circle => format!("255*lt(hypot(X-W/2,Y-H/2),{p}*hypot(W/2,H/2))"),
        _ => return None,
    };
    let rest = item.len() + 1.0;
    Some(format!(
        "nullsrc=s={bw}x{bh}:r=30:d={head:.3},format=gray,geq=lum='{lum}'[mh{i}];\
         color=c=white:s={bw}x{bh}:r=30:d={rest:.3},format=gray[mr{i}];\
         [mh{i}][mr{i}]concat=n=2:v=1:a=0[m{i}]"
    ))
}

/// Applies the mask `[m{i}]` to the picture `[bx{i}]`, giving `[bm{i}]`. Stills may already have transparency
/// (text, PNGs), so their own alpha is multiplied in rather than replaced.
pub fn apply(i: usize, item: &Item) -> String {
    if item.kind == Kind::Video {
        format!("[bx{i}][m{i}]alphamerge[bm{i}]")
    } else {
        format!(
            "[bx{i}]split[bc{i}][ba{i}];[ba{i}]alphaextract[ae{i}];\
             [ae{i}][m{i}]blend=all_mode=multiply[m2{i}];[bc{i}][m2{i}]alphamerge[bm{i}]"
        )
    }
}
