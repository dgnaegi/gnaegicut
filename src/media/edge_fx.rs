//! The soft fade around a picture, and the one way masks are applied.

use crate::project::Item;

/// Filter statements that produce the picture's feather mask as `[fm{i}]`: white inside, fading to black towards
/// the edges. It is one frame, repeated for as long as the item lasts. `None` without a feather.
pub fn mask(i: usize, item: &Item, size: (u32, u32)) -> Option<String> {
    let span = item.edge.feather_px(size);
    if span < 1.0 {
        return None;
    }
    let (bw, bh) = size;
    let near = "min(min(X,W-1-X),min(Y,H-1-Y))"; // distance to the closest edge
    let secs = item.len() + 1.0;
    Some(format!(
        "nullsrc=s={bw}x{bh}:r=30:d=0.04,format=gray,geq=lum='255*clip({near}/{span:.2},0,1)',\
         loop=loop=-1:size=1,setpts=N/(30*TB),trim=duration={secs:.3}[fm{i}]"
    ))
}

/// Multiplies the transparency of picture `[input]` with the gray mask `[mask]`, giving `[out]`. Pictures may
/// already have transparency (text, PNGs, an earlier mask), so it is multiplied in rather than replaced.
pub fn multiply(input: &str, mask: &str, out: &str) -> String {
    format!(
        "[{input}]split[{out}c][{out}a];[{out}a]alphaextract[{out}e];\
         [{out}e][{mask}]blend=all_mode=multiply:shortest=1[{out}m];[{out}c][{out}m]alphamerge[{out}]"
    )
}
