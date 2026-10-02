//! Splitting captions: one long caption becomes several shorter ones, which is how short-form video usually shows
//! speech (a few words at a time).

use super::Caption;
use crate::project::Project;

/// How many words each of `pieces` pieces gets for `words` words: as even as possible, bigger pieces first.
fn sizes(words: usize, pieces: usize) -> Vec<usize> {
    let (base, extra) = (words / pieces, words % pieces);
    (0..pieces).map(|i| base + usize::from(i < extra)).collect()
}

impl Caption {
    /// Pieces of at most `max_words` words (at least 1). The time is shared out by the number of characters in each
    /// piece, so a long word gets longer on screen, and the pieces tile the original span exactly. A caption that
    /// already fits comes back unchanged.
    pub fn split_into(&self, max_words: usize) -> Vec<Caption> {
        let words: Vec<&str> = self.text.split_whitespace().collect();
        let max_words = max_words.max(1);
        if words.len() <= max_words {
            return vec![self.clone()];
        }
        let mut next = 0;
        let texts: Vec<String> = sizes(words.len(), words.len().div_ceil(max_words))
            .into_iter()
            .map(|n| {
                let piece = words[next..next + n].join(" ");
                next += n;
                piece
            })
            .collect();
        let total: usize = texts.iter().map(|t| t.chars().count()).sum();
        let (mut at, mut seen) = (self.start, 0usize);
        let last = texts.len() - 1;
        texts
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                seen += text.chars().count();
                let end = if i == last {
                    self.end
                } else {
                    self.start + (self.end - self.start) * seen as f64 / total as f64
                };
                let piece = Caption { start: at, end, text };
                at = end;
                piece
            })
            .collect()
    }

    /// Two halves, split at a word boundary near the middle. `None` for a single word.
    pub fn split_in_two(&self) -> Option<[Caption; 2]> {
        let words = self.text.split_whitespace().count();
        let mut pieces = (words >= 2).then(|| self.split_into(words.div_ceil(2)))?.into_iter();
        Some([pieces.next()?, pieces.next()?])
    }
}

impl Project {
    /// Splits caption `index` in two. Returns false if it is a single word or does not exist.
    pub fn split_caption(&mut self, index: usize) -> bool {
        let Some([a, b]) = self.captions.get(index).and_then(Caption::split_in_two) else {
            return false;
        };
        self.captions.splice(index..=index, [a, b]);
        true
    }

    /// Splits every caption that has more than `max_words` words. Returns whether anything changed.
    pub fn split_captions(&mut self, max_words: usize) -> bool {
        let before = self.captions.len();
        self.captions = self.captions.iter().flat_map(|c| c.split_into(max_words)).collect();
        self.captions.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caption(text: &str, start: f64, end: f64) -> Caption {
        Caption {
            start,
            end,
            text: text.into(),
        }
    }

    #[test]
    fn one_word_per_caption_tiles_the_span_and_gives_longer_words_more_time() {
        let pieces = caption("one two three", 2.0, 5.0).split_into(1);
        assert_eq!(
            pieces.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(),
            ["one", "two", "three"]
        );
        assert_eq!(pieces[0].start, 2.0);
        assert_eq!(pieces[2].end, 5.0, "ends exactly where the original ended");
        assert!(
            pieces.windows(2).all(|w| (w[0].end - w[1].start).abs() < 1e-12),
            "no gaps, no overlaps"
        );
        assert!(
            pieces[2].end - pieces[2].start > pieces[0].end - pieces[0].start,
            "'three' is longer than 'one'"
        );
    }

    #[test]
    fn pieces_are_balanced_and_never_larger_than_asked() {
        let sizes: Vec<usize> = caption("a b c d e", 0.0, 5.0)
            .split_into(2)
            .iter()
            .map(|p| p.text.split_whitespace().count())
            .collect();
        assert_eq!(sizes, [2, 2, 1]);
        let balanced: Vec<usize> = caption("a b c d e f g", 0.0, 7.0)
            .split_into(4)
            .iter()
            .map(|p| p.text.split_whitespace().count())
            .collect();
        assert_eq!(balanced, [4, 3], "4 + 3 rather than 4 + 2 + 1");
    }

    #[test]
    fn a_caption_that_fits_stays_as_it_is() {
        let original = caption("just three words", 1.0, 2.0);
        let same = original.split_into(3);
        assert_eq!(same.len(), 1);
        assert_eq!(
            (same[0].start, same[0].end, same[0].text.as_str()),
            (1.0, 2.0, "just three words")
        );
        assert_eq!(caption("", 0.0, 1.0).split_into(1).len(), 1, "empty text is left alone");
        assert_eq!(
            original.split_into(0).len(),
            3,
            "zero means one word at a time, not a crash"
        );
    }

    #[test]
    fn splitting_in_two_needs_two_words_and_halves_the_words() {
        assert!(caption("solo", 0.0, 1.0).split_in_two().is_none());
        let [a, b] = caption("one two three four", 0.0, 4.0).split_in_two().unwrap();
        assert_eq!((a.text.as_str(), b.text.as_str()), ("one two", "three four"));
        assert_eq!(a.end, b.start);
        let [a, b] = caption("one two three", 0.0, 3.0).split_in_two().unwrap();
        assert_eq!((a.text.as_str(), b.text.as_str()), ("one two", "three"));
    }

    #[test]
    fn the_project_splits_one_caption_or_all_of_them() {
        let mut p = Project {
            captions: vec![
                caption("hello there my friend", 0.0, 2.0),
                caption("ok", 2.0, 3.0),
                caption("see you soon again", 3.0, 5.0),
            ],
            ..Default::default()
        };
        assert!(p.split_caption(0));
        assert_eq!(p.captions.len(), 4);
        assert_eq!(
            (p.captions[0].text.as_str(), p.captions[1].text.as_str()),
            ("hello there", "my friend")
        );
        assert!(!p.split_caption(2), "'ok' is a single word");
        assert!(!p.split_caption(99), "no such caption");

        assert!(p.split_captions(1));
        assert!(p.captions.iter().all(|c| c.text.split_whitespace().count() == 1));
        assert_eq!(p.captions.first().map(|c| c.start), Some(0.0));
        assert_eq!(p.captions.last().map(|c| c.end), Some(5.0));
        assert!(!p.split_captions(1), "already one word each: nothing more to do");
    }
}
