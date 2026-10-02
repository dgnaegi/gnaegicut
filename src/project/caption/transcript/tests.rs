use super::*;

/// "hello big world" 0-3 s, "how are you" 3-6 s, "bye" 6-7 s.
fn project() -> Project {
    let caption = |text: &str, start, end| Caption {
        start,
        end,
        text: text.into(),
    };
    Project {
        captions: vec![
            caption("hello big world", 0.0, 3.0),
            caption("how are you", 3.0, 6.0),
            caption("bye", 6.0, 7.0),
        ],
        ..Default::default()
    }
}

fn texts(p: &Project) -> Vec<&str> {
    p.captions.iter().map(|c| c.text.as_str()).collect()
}

#[test]
fn the_unchanged_transcript_changes_nothing_even_with_blank_lines_and_stray_spaces() {
    let mut p = project();
    let before = p.captions.clone();
    assert!(!p.apply_transcript(&p.transcript()));
    assert!(!p.apply_transcript("hello big world\n\n  how are you  \nbye\n"));
    assert_eq!(p.captions, before);
}

#[test]
fn editing_words_changes_the_text_and_keeps_the_times() {
    let mut p = project();
    assert!(p.apply_transcript("hello huge world\nhow are you\nbye"));
    assert_eq!(texts(&p), ["hello huge world", "how are you", "bye"]);
    assert_eq!((p.captions[0].start, p.captions[0].end), (0.0, 3.0));
}

#[test]
fn a_line_break_inside_a_caption_splits_it_and_the_time_is_shared() {
    let mut p = project();
    assert!(p.apply_transcript("hello big\nworld\nhow are you\nbye"));
    assert_eq!(texts(&p), ["hello big", "world", "how are you", "bye"]);
    assert_eq!(p.captions[0].start, 0.0);
    assert_eq!(p.captions[0].end, p.captions[1].start, "no gap between the halves");
    assert_eq!(p.captions[1].end, 3.0, "together they fill the original 3 s");
    assert_eq!(
        (p.captions[2].start, p.captions[2].end),
        (3.0, 6.0),
        "the others did not move"
    );
}

#[test]
fn a_double_space_splits_just_like_a_line_break() {
    let mut p = project();
    assert!(p.apply_transcript("hello big  world\nhow are you\nbye"));
    assert_eq!(texts(&p), ["hello big", "world", "how are you", "bye"]);
    let mut single = project();
    assert!(
        !single.apply_transcript("hello big world\nhow are you\nbye"),
        "a single space is just a space"
    );
}

#[test]
fn trailing_spaces_while_typing_do_not_split_anything() {
    let mut p = project();
    assert!(!p.apply_transcript("hello big world  \nhow are you   \nbye "));
    assert_eq!(p.captions.len(), 3);
}

#[test]
fn taking_out_a_line_break_joins_two_captions() {
    let mut p = project();
    assert!(p.apply_transcript("hello big world how are you\nbye"));
    assert_eq!(texts(&p), ["hello big world how are you", "bye"]);
    assert_eq!(
        (p.captions[0].start, p.captions[0].end),
        (0.0, 6.0),
        "the joined caption covers both"
    );
}

#[test]
fn deleting_a_line_removes_that_caption_only() {
    let mut p = project();
    assert!(p.apply_transcript("hello big world\nbye"));
    assert_eq!(texts(&p), ["hello big world", "bye"]);
    assert_eq!((p.captions[1].start, p.captions[1].end), (6.0, 7.0));
    assert!(p.apply_transcript(""));
    assert!(p.captions.is_empty(), "an empty field removes everything");
}

#[test]
fn a_new_line_at_the_end_becomes_a_caption_after_the_last() {
    let mut p = project();
    assert!(p.apply_transcript("hello big world\nhow are you\nbye\nsee you"));
    assert_eq!(p.captions[3].text, "see you");
    assert_eq!((p.captions[3].start, p.captions[3].end), (7.0, 8.0));
}

#[test]
fn a_line_added_in_the_middle_shares_the_time_of_the_caption_after_it() {
    let mut p = project();
    assert!(p.apply_transcript("hello big world\nwait\nhow are you\nbye"));
    assert_eq!(texts(&p), ["hello big world", "wait", "how are you", "bye"]);
    assert_eq!(p.captions[1].start, 3.0);
    assert_eq!(p.captions[1].end, p.captions[2].start);
    assert_eq!(
        p.captions[2].end, 6.0,
        "nothing overlaps and nothing moved past its old end"
    );
    assert_eq!(p.captions[3].start, 6.0);
}

#[test]
fn several_edits_at_once_are_all_applied() {
    let mut p = project();
    assert!(p.apply_transcript("hello\nbig world\nhow are you\nbye now"));
    assert_eq!(texts(&p), ["hello", "big world", "how are you", "bye now"]);
    assert_eq!(p.captions[2].end, 6.0);
}
