//! Pasting a screenshot (or any image) from the system clipboard into the media.

use crate::app::App;
use crate::persist;
use std::path::{Path, PathBuf};
use std::{fs::File, io::BufWriter};

/// Writes RGBA pixels as a PNG into `dir`, named by the time, and returns its path.
fn save_png(dir: &Path, (w, h): (u32, u32), rgba: &[u8]) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let path = dir.join(format!("screenshot-{secs}.png"));
    let mut enc = png::Encoder::new(BufWriter::new(File::create(&path).map_err(|e| e.to_string())?), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut writer| writer.write_image_data(rgba))
        .map_err(|e| e.to_string())?;
    Ok(path)
}

/// Where pasted images are kept: next to the autosave, so projects can still find them later.
fn folder() -> PathBuf {
    persist::autosave_path().with_file_name("pastes")
}

/// The image on the clipboard, saved as a file. `None` when the clipboard holds no image.
fn clipboard_image() -> Option<Result<PathBuf, String>> {
    if cfg!(test) {
        return None; // tests must not touch (or crash on) the real system clipboard
    }
    let image = arboard::Clipboard::new().ok()?.get_image().ok()?;
    Some(save_png(
        &folder(),
        (image.width as u32, image.height as u32),
        &image.bytes,
    ))
}

impl App {
    /// Adds the clipboard's image to the media and lays it on top at the playhead. Returns false if there is none.
    pub fn paste_image(&mut self) -> bool {
        match clipboard_image() {
            None => false,
            Some(Err(e)) => {
                self.status = e;
                true
            }
            Some(Ok(path)) => {
                self.add_files(&[path], None);
                self.status = "Pasted the image from the clipboard".into();
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_become_a_png_file_of_the_same_size() {
        let dir = std::env::temp_dir().join("gnaegicut-screenshot-test");
        let path = save_png(&dir, (2, 1), &[255, 0, 0, 255, 0, 0, 255, 255]).unwrap();
        let decoder = png::Decoder::new(std::io::BufReader::new(File::open(&path).unwrap()));
        let reader = decoder.read_info().unwrap();
        assert_eq!((reader.info().width, reader.info().height), (2, 1));
        let _ = std::fs::remove_dir_all(dir);
    }
}
