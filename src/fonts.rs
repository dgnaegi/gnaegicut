//! The fonts available for text and captions: everything installed on the OS plus the bundled Inter.

use crate::media::asset;

pub const DEFAULT_FONT: &str = "Inter-Black";

pub struct Face {
    pub family: String,
    /// Every family name the font file declares ("AL Unica77" and "AL Unica77 Black", for instance).
    pub names: Vec<String>,
    pub style: String,
    pub ps_name: String,
    pub bold: bool,
}

impl Face {
    pub fn label(&self) -> String {
        format!("{} {}", self.family, self.style)
    }
}

pub struct Fonts {
    db: fontdb::Database,
    pub faces: Vec<Face>,
}

impl Fonts {
    pub fn load() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db.load_fonts_dir(asset("assets/fonts"));
        let mut faces: Vec<_> = db
            .faces()
            .filter_map(|f| {
                let family = f.families.first()?.0.clone();
                let bold = f.weight.0 >= 600;
                let style = match (bold, f.style) {
                    (true, fontdb::Style::Normal) => "Bold",
                    (false, fontdb::Style::Normal) => "Regular",
                    (true, _) => "Bold Italic",
                    (false, _) => "Italic",
                };
                Some((
                    f.weight.0,
                    Face {
                        family,
                        names: f.families.iter().map(|(n, _)| n.clone()).collect(),
                        style: style.into(),
                        ps_name: f.post_script_name.clone(),
                        bold,
                    },
                ))
            })
            .collect();
        faces.sort_by(|(wa, a), (wb, b)| (&a.family, wa, &a.ps_name).cmp(&(&b.family, wb, &b.ps_name)));
        faces.dedup_by(|(_, a), (_, b)| a.ps_name == b.ps_name);
        Self {
            db,
            faces: faces.into_iter().map(|(_, f)| f).collect(),
        }
    }

    pub fn find(&self, ps_name: &str) -> Option<&Face> {
        self.faces.iter().find(|f| f.ps_name == ps_name)
    }

    /// Runs `f` with the font file's bytes and the face index inside it (for .ttc collections).
    pub fn with_data<R>(&self, ps_name: &str, f: impl FnOnce(&[u8], u32) -> R) -> Option<R> {
        let id = self.db.faces().find(|i| i.post_script_name == ps_name)?.id;
        self.db.with_face_data(id, f)
    }
}
