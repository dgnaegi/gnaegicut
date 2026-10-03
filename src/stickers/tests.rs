use super::*;

const JSON: &str = r#"{"icons":[
  {"codepoint":"1f638","tags":[":smile-cat:"],"popularity":5},
  {"codepoint":"1f483","tags":[":dancer:",":woman-dancing:"],"popularity":2},
  {"codepoint":"1f483_1f3fb","tags":[":dancer:"],"popularity":3},
  {"codepoint":"1f600","tags":[":smile:"],"popularity":1}
]}"#;

#[test]
fn the_list_drops_skin_tone_copies_and_turns_tags_into_words() {
    let all = parse_index(JSON).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].words, "smile cat");
    assert!(all.iter().all(|s| !s.code.contains("1f3fb")));
}

#[test]
fn search_ignores_endings_and_puts_the_popular_first() {
    let all = parse_index(JSON).unwrap();
    let names = |q: &str| search(&all, q).into_iter().map(|s| s.code).collect::<Vec<_>>();
    assert_eq!(names("dance"), ["1f483"], "dance finds dancing");
    assert_eq!(names("smile"), ["1f600", "1f638"], "most popular first");
    assert_eq!(names("smile cat"), ["1f638"], "every word has to match");
    assert_eq!(names("").len(), 3, "an empty search shows everything");
    assert!(names("zebra").is_empty());
}

#[test]
fn urls_point_at_the_preview_and_the_animation() {
    let s = &parse_index(JSON).unwrap()[0];
    assert!(s.preview_url().ends_with("/1f638/128.png") && s.gif_url().ends_with("/1f638/512.gif"));
}

#[test]
#[ignore = "needs the network"]
fn a_real_sticker_downloads_probes_and_loops_beyond_its_own_length() {
    use crate::project::{Item, Project};
    let all = load_index().unwrap();
    assert!(all.len() > 300, "the whole list: {}", all.len());
    let cat = search(&all, "smile cat").remove(0);
    let (pixels, w, h) = load_preview(&cat).unwrap();
    assert_eq!(pixels.len(), (w * h * 4) as usize);
    let mut item: Item = crate::media::probe::probe(&download(&cat).unwrap()).unwrap();
    let once = item.end;
    assert!(once > 0.5, "the animation has a length: {once}");
    (item.looped, item.end, item.scale) = (true, once * 4.0, 0.5);
    let mut p = Project::default();
    p.add(1, item);
    let frame = crate::media::frame::frame(&p, once * 3.0, 108, 192).unwrap();
    assert!(
        frame.as_chunks::<4>().0.iter().any(|px| px[0] > 40 || px[1] > 40),
        "the sticker is drawn after its first turn"
    );
}

#[test]
#[ignore = "needs the network"]
fn a_placed_sticker_really_moves() {
    use crate::project::Project;
    let all = load_index().unwrap();
    let dancer = search(&all, "dancer").remove(0);
    let mut item = crate::media::probe::probe(&download(&dancer).unwrap()).unwrap();
    let once = item.end;
    (item.looped, item.end, item.scale) = (true, once * 3.0, 0.5);
    let mut p = Project::default();
    p.add(1, item);
    let at = |t: f64| crate::media::frame::frame(&p, t, 108, 192).unwrap();
    let differing = at(0.1)
        .iter()
        .zip(at(0.1 + once * 0.4).iter())
        .filter(|(a, b)| a != b)
        .count();
    println!("animation {once}s, {differing} bytes differ");
    assert!(differing > 200, "two moments of the animation differ: {differing}");
}
