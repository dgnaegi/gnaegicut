//! Animated stickers: the Noto Animated Emoji by Google (CC BY 4.0), loaded from Google's font CDN on demand. The
//! list of emoji is one small JSON file; a preview is a 128 px PNG; the sticker itself is a 512 px animated GIF
//! that is downloaded once and kept.

mod state;

use serde_json::Value;
pub use state::StickersUi;
use std::path::PathBuf;
use std::time::Duration;

const INDEX_URL: &str = "https://googlefonts.github.io/noto-emoji-animation/data/api.json";
const ASSET_URL: &str = "https://fonts.gstatic.com/s/e/notoemoji/latest";
const USER_AGENT: &str = "GnaegiCut (video editor)";
/// The credit that has to travel with every video that uses one (CC BY 4.0).
pub const CREDIT: &str =
    "Noto Animated Emoji by Google, CC BY 4.0, https://googlefonts.github.io/noto-emoji-animation/";

#[derive(Clone, Debug, PartialEq)]
pub struct Sticker {
    /// The emoji's code points, e.g. `1f638` or `1f468_200d_1f4bb`: also the file name on the CDN.
    pub code: String,
    /// What it is called, as words: `smile cat`.
    pub words: String,
    pub popularity: u32,
}

impl Sticker {
    pub fn preview_url(&self) -> String {
        format!("{ASSET_URL}/{}/128.png", self.code)
    }

    fn gif_url(&self) -> String {
        format!("{ASSET_URL}/{}/512.gif", self.code)
    }

    /// Whether this sticker answers `query`: every word of it must occur in the name. Endings are ignored, so
    /// "dance" finds "dancing".
    pub fn matches(&self, query: &str) -> bool {
        query.split_whitespace().all(|w| self.words.contains(stem(w).as_str()))
    }
}

fn stem(word: &str) -> String {
    let w = word.to_lowercase();
    let w = w.strip_suffix("ing").or_else(|| w.strip_suffix("es")).unwrap_or(&w);
    w.trim_end_matches(['e', 's']).to_string()
}

/// The stickers that answer `query`, the most popular first; everything for an empty query.
pub fn search(all: &[Sticker], query: &str) -> Vec<Sticker> {
    let mut found: Vec<Sticker> = all.iter().filter(|s| s.matches(query)).cloned().collect();
    found.sort_by_key(|s| s.popularity);
    found
}

/// Reads the CDN's list. Skin tone variants are left out: they only repeat the same animation.
pub fn parse_index(json: &str) -> Result<Vec<Sticker>, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let icons = value["icons"].as_array().ok_or("the sticker list is empty")?;
    let tone = |code: &str| code.split('_').any(|c| ("1f3fb"..="1f3ff").contains(&c));
    Ok(icons
        .iter()
        .filter_map(|i| {
            let code = i["codepoint"].as_str()?.to_string();
            let words = i["tags"]
                .as_array()?
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>();
            let words = words.join(" ").replace([':', '-', '_'], " ").to_lowercase();
            let words = words.split_whitespace().collect::<Vec<_>>().join(" ");
            (!tone(&code)).then(|| Sticker {
                code,
                words,
                popularity: i["popularity"].as_u64().unwrap_or(u64::MAX) as u32,
            })
        })
        .collect())
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into()
}

fn get(url: &str, limit: u64) -> Result<Vec<u8>, String> {
    let mut response = agent()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    response
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .map_err(|e| e.to_string())
}

fn cache_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join("Library/Caches/GnaegiCut/stickers")
}

/// The list of stickers: downloaded, and kept for when there is no network.
pub fn load_index() -> Result<Vec<Sticker>, String> {
    let file = cache_dir().join("index.json");
    match get(INDEX_URL, 4 << 20).and_then(|b| String::from_utf8(b).map_err(|e| e.to_string())) {
        Ok(json) => {
            let list = parse_index(&json)?;
            let _ = std::fs::create_dir_all(cache_dir()).and_then(|()| std::fs::write(&file, json));
            Ok(list)
        }
        Err(e) => std::fs::read_to_string(&file)
            .map_err(|_| e)
            .and_then(|j| parse_index(&j)),
    }
}

/// The preview picture as RGBA pixels with its size.
pub fn load_preview(sticker: &Sticker) -> Result<(Vec<u8>, u32, u32), String> {
    decode_png(&get(&sticker.preview_url(), 1 << 20)?)
}

/// Downloads the animated GIF once; later calls return the file kept before.
pub fn download(sticker: &Sticker) -> Result<PathBuf, String> {
    let path = cache_dir().join(format!("{}.gif", sticker.code));
    if path.metadata().is_ok_and(|m| m.len() > 0) {
        return Ok(path);
    }
    let bytes = get(&sticker.gif_url(), 8 << 20)?;
    std::fs::create_dir_all(cache_dir()).map_err(|e| e.to_string())?;
    let partial = path.with_extension("part");
    std::fs::write(&partial, bytes)
        .and_then(|()| std::fs::rename(&partial, &path))
        .map_err(|e| e.to_string())?;
    Ok(path)
}

fn decode_png(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND); // palette and transparency become plain RGBA
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("picture too large")?];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        other => return Err(format!("unexpected picture type {other:?}")),
    };
    Ok((rgba, info.width, info.height))
}

#[cfg(test)]
mod tests;
