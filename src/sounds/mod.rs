//! Free sound search. Sound effects come from Freesound and music from Jamendo, both through Openverse,
//! which needs no account or key. Only licences that allow commercial use are requested; every result carries
//! the attribution text the licence asks for.

mod source;

use serde_json::Value;
pub use source::Source;
use std::path::PathBuf;
use std::time::Duration;

const SEARCH_URL: &str = "https://api.openverse.org/v1/audio/";
const USER_AGENT: &str = "GnaegiCut/0.1 (video editor)";
const MAX_DOWNLOAD: u64 = 60 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    pub id: String,
    pub title: String,
    pub creator: String,
    pub license: String,
    pub secs: f64,
    pub url: String,
    pub ext: String,
    /// The attribution text to show or copy, e.g. `"Deep Whoosh" by Kinoton is marked with CC0 1.0`.
    pub credit: String,
}

impl Sound {
    pub fn duration_label(&self) -> String {
        format!("{}:{:02}", self.secs as u64 / 60, self.secs as u64 % 60)
    }
}

/// The query parameters for a search; separate so they can be checked without a network.
pub fn params(query: &str, source: Source) -> Vec<(&'static str, String)> {
    vec![
        ("q", query.trim().to_string()),
        ("source", source.provider().into()),
        ("license_type", "commercial".into()),
        ("page_size", "20".into()),
    ]
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into()
}

pub fn search(query: &str, source: Source) -> Result<Vec<Sound>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    let mut request = agent().get(SEARCH_URL).header("User-Agent", USER_AGENT);
    for (key, value) in params(query, source) {
        request = request.query(key, value);
    }
    let mut response = request.call().map_err(|e| format!("search failed: {e}"))?;
    parse(&response.body_mut().read_to_string().map_err(|e| e.to_string())?)
}

fn license_label(license: &str, version: &str) -> String {
    if license.eq_ignore_ascii_case("cc0") {
        return format!("CC0 {version}").trim().to_string();
    }
    format!("CC {} {version}", license.to_uppercase()).trim().to_string()
}

fn extension(filetype: &str) -> &'static str {
    match filetype.trim_end_matches(|c: char| c.is_ascii_digit()) {
        "wav" => "wav",
        "ogg" => "ogg",
        "flac" => "flac",
        "m4a" | "aac" => "m4a",
        _ => "mp3", // mp3, mp32 (Jamendo), and anything unknown that ffmpeg can sniff
    }
}

pub fn parse(json: &str) -> Result<Vec<Sound>, String> {
    let root: Value = serde_json::from_str(json).map_err(|e| format!("unexpected answer: {e}"))?;
    let results = root
        .get("results")
        .and_then(Value::as_array)
        .ok_or("unexpected answer: no results")?;
    let text = |r: &Value, key: &str| r.get(key).and_then(Value::as_str).unwrap_or_default().to_string();
    Ok(results
        .iter()
        .filter_map(|r| {
            let (url, id) = (text(r, "url"), text(r, "id"));
            if url.is_empty() || id.is_empty() {
                return None;
            }
            let (title, creator) = (text(r, "title"), text(r, "creator"));
            let license = license_label(&text(r, "license"), &text(r, "license_version"));
            let credit = match text(r, "attribution") {
                a if a.is_empty() => format!(
                    "\"{title}\" by {creator} ({license}) {}",
                    text(r, "foreign_landing_url")
                ),
                a => a,
            };
            let secs = r.get("duration").and_then(Value::as_f64).unwrap_or(0.0) / 1000.0;
            Some(Sound {
                id,
                title,
                creator,
                license,
                secs,
                url,
                ext: extension(&text(r, "filetype")).into(),
                credit,
            })
        })
        .collect())
}

fn cache_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join("Library/Caches/GnaegiCut/sounds")
}

pub fn cache_path(sound: &Sound) -> PathBuf {
    let safe: String = sound
        .id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    cache_dir().join(format!("{safe}.{}", sound.ext))
}

/// Downloads a sound once; later calls return the cached file.
pub fn download(sound: &Sound) -> Result<PathBuf, String> {
    let path = cache_path(sound);
    if path.metadata().is_ok_and(|m| m.len() > 0) {
        return Ok(path);
    }
    let mut response = agent()
        .get(&sound.url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_DOWNLOAD)
        .read_to_vec()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(cache_dir()).map_err(|e| e.to_string())?;
    let partial = path.with_extension("part");
    std::fs::write(&partial, bytes)
        .and_then(|()| std::fs::rename(&partial, &path))
        .map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests;
