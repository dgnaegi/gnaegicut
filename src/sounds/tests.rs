use super::*;

const SAMPLE: &str = r#"{"result_count":2,"results":[
  {"id":"eab8a6e2-0ac8-4615-9e0d-ebde30523783","title":"Deep Whoosh #1","creator":"Kinoton","license":"cc0",
   "license_version":"1.0","url":"https://cdn.freesound.org/previews/351/351256_2247456-hq.mp3","duration":3155,
   "filetype":"mp3","attribution":"\"Deep Whoosh #1\" by Kinoton is marked with CC0 1.0.",
   "foreign_landing_url":"https://freesound.org/people/Kinoton/sounds/351256"},
  {"id":"jam-1","title":"Lofi House","creator":"AstroShroom","license":"by-sa","license_version":"3.0",
   "url":"https://prod-1.storage.jamendo.com/?trackid=1356703&format=mp32","duration":123000,"filetype":"mp32",
   "attribution":null,"foreign_landing_url":"https://www.jamendo.com/track/1356703"},
  {"id":"broken","title":"No file","url":""}]}"#;

#[test]
fn parses_results_and_skips_entries_without_a_file() {
    let sounds = parse(SAMPLE).unwrap();
    assert_eq!(sounds.len(), 2);
    let first = &sounds[0];
    assert_eq!(
        (first.title.as_str(), first.creator.as_str(), first.license.as_str()),
        ("Deep Whoosh #1", "Kinoton", "CC0 1.0")
    );
    assert!((first.secs - 3.155).abs() < 1e-9 && first.duration_label() == "0:03");
    assert!(first.credit.contains("Kinoton") && first.ext == "mp3");
}

#[test]
fn builds_a_credit_when_the_service_gives_none_and_maps_jamendo_mp32() {
    let jamendo = &parse(SAMPLE).unwrap()[1];
    assert_eq!(jamendo.license, "CC BY-SA 3.0");
    assert_eq!(jamendo.ext, "mp3");
    assert_eq!(
        jamendo.credit,
        "\"Lofi House\" by AstroShroom (CC BY-SA 3.0) https://www.jamendo.com/track/1356703"
    );
    assert_eq!(jamendo.duration_label(), "2:03");
}

#[test]
fn rejects_garbage_answers() {
    assert!(parse("<html>rate limited</html>").is_err());
    assert!(parse("{}").is_err());
}

#[test]
fn asks_only_for_commercially_usable_sounds_from_the_right_source() {
    let effects = params("  whoosh ", Source::Effects);
    assert!(effects.contains(&("q", "whoosh".into())) && effects.contains(&("source", "freesound".into())));
    assert!(effects.contains(&("license_type", "commercial".into())));
    assert!(params("lofi", Source::Music).contains(&("source", "jamendo".into())));
    let size: u32 = effects
        .iter()
        .find(|(k, _)| *k == "page_size")
        .unwrap()
        .1
        .parse()
        .unwrap();
    assert!(
        size <= 20,
        "Openverse answers 401 above 20 results per page without an account"
    );
}

#[test]
fn cache_names_are_safe_and_stable() {
    let mut s = parse(SAMPLE).unwrap().remove(0);
    s.id = "../../etc/passwd-1".into();
    let path = cache_path(&s);
    assert_eq!(path.file_name().unwrap(), "etcpasswd-1.mp3");
    assert!(path.starts_with(cache_dir()));
    assert_eq!(
        search("   ", Source::Effects).unwrap(),
        vec![],
        "blank searches never hit the network"
    );
}

/// Needs the internet: `cargo test -- --ignored`.
#[test]
#[ignore]
fn finds_and_downloads_a_real_sound() {
    let found = search("whoosh", Source::Effects).unwrap();
    assert!(!found.is_empty());
    let path = download(&found[0]).unwrap();
    let item = crate::media::probe::probe(&path).unwrap();
    assert!(item.has_audio && item.len() > 0.1);
    assert_eq!(download(&found[0]).unwrap(), path, "second call uses the cache");
}
