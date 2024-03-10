mod db;
pub mod error;
pub mod note;

use std::path::PathBuf;

use db::SovDb;
use error::{Result, SovError};
use note::SovNote;
use walkdir::WalkDir;

pub struct Sov {
    config: SovConfig,
    db: SovDb,
}

pub struct SovConfig {
    pub db_path: PathBuf,
    pub notes_path: PathBuf,
}

impl Sov {
    pub fn new() -> Result<Self> {
        let config = SovConfig {
            db_path: "sov.db3".into(),
            notes_path: "/home/silent/quark".into(),
        };
        let sov_db = SovDb::new(&config.db_path)?;
        sov_db.init()?;

        Ok(Self { config, db: sov_db })
    }

    pub fn index(&self) -> Result<()> {
        let mut notes = Vec::new();

        for entry in WalkDir::new(&self.config.notes_path) {
            let entry = entry?;
            let path = entry.path().to_path_buf();
            if path.is_dir() || path.extension().map(|s| s != "md").unwrap_or(true) {
                continue;
            }
            let Some(filename) = path.file_stem() else {
                continue;
            };
            let Some(filename) = filename.to_str() else {
                continue;
            };
            let Some(note_id) = SovNote::extract_note_id(&filename) else {
                continue;
            };
            let note = SovNote::from_file(&path, note_id)?;
            notes.push(note);
        }
        self.db.insert_notes(&notes)?;
        Ok(())
    }
}
