//! Editing all captions as one text, one caption per line. A line break or a double space inside a caption splits
//! it, taking out a line break joins two captions, deleting a line removes the caption, and editing words changes
//! its text. The captions keep their times wherever the text did not change.

use super::Caption;
use crate::project::Project;

const NEW_CAPTION_SECS: f64 = 1.0;

/// The text without any whitespace, to recognise lines that were split or glued together.
fn squash(s: &str) -> String {
    s.split_whitespace().collect()
}

/// The lines of an edited transcript: broken at line breaks and at runs of two or more spaces, trimmed, blanks dropped.
fn lines(text: &str) -> Vec<String> {
    text.lines()
        .flat_map(|l| l.split("  "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

impl Project {
    /// Brings the captions in line with an edited transcript. Returns whether anything changed.
    pub fn apply_transcript(&mut self, text: &str) -> bool {
        let new = lines(text);
        let old = std::mem::take(&mut self.captions);
        let mut out: Vec<Caption> = vec![];
        let (mut i, mut j) = (0, 0);
        while i < old.len() || j < new.len() {
            match (old.get(i), new.get(j)) {
                (Some(o), Some(l)) if o.text.trim() == l => {
                    out.push(o.clone());
                    (i, j) = (i + 1, j + 1);
                }
                (Some(o), Some(l)) => {
                    // One caption turned into several lines: it was split.
                    let split = (2..=new.len() - j).find(|&k| squash(&new[j..j + k].concat()) == squash(&o.text));
                    // Several captions turned into one line: they were joined.
                    let join = (2..=old.len() - i).find(|&k| {
                        let glued: String = old[i..i + k].iter().map(|c| c.text.as_str()).collect();
                        squash(&glued) == squash(l)
                    });
                    if let Some(k) = split {
                        out.extend(Caption::evenly(o.start, o.end, new[j..j + k].to_vec()));
                        (i, j) = (i + 1, j + k);
                    } else if let Some(k) = join {
                        let text = old[i..i + k]
                            .iter()
                            .map(|c| c.text.trim())
                            .collect::<Vec<_>>()
                            .join(" ");
                        out.push(Caption {
                            start: o.start,
                            end: old[i + k - 1].end,
                            text,
                        });
                        (i, j) = (i + k, j + 1);
                    } else if old.get(i + 1).is_some_and(|next| next.text.trim() == l) {
                        i += 1; // this old caption was deleted
                    } else if new.get(j + 1).is_some_and(|next| o.text.trim() == next) {
                        // A line was added in front of this caption: it shares the caption's time.
                        out.extend(Caption::evenly(
                            o.start,
                            o.end,
                            vec![l.clone(), o.text.trim().to_string()],
                        ));
                        (i, j) = (i + 1, j + 2);
                    } else {
                        out.push(Caption {
                            text: l.clone(),
                            ..o.clone()
                        }); // the words were edited
                        (i, j) = (i + 1, j + 1);
                    }
                }
                (Some(_), None) => i += 1, // the remaining old captions were deleted
                (None, Some(l)) => {
                    let start = out.last().map_or(0.0, |c| c.end);
                    out.push(Caption {
                        start,
                        end: start + NEW_CAPTION_SECS,
                        text: l.clone(),
                    });
                    j += 1;
                }
                (None, None) => break,
            }
        }
        let changed = out != old;
        self.captions = out;
        changed
    }
}

#[cfg(test)]
mod tests;
