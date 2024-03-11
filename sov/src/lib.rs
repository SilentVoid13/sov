pub mod config;
mod db;
pub mod error;
pub mod note;

use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use chrono::DateTime;
use config::SovConfig;
use db::SovDb;
use error::{Result, SovError};
use note::SovNote;
use walkdir::WalkDir;

pub struct Sov {
    config: SovConfig,
    db: SovDb,
}

impl Sov {
    pub fn new() -> Result<Self> {
        let config = SovConfig::load()?;
        let sov_db = SovDb::new(&config.db_path)?;

        let sov = Sov { config, db: sov_db };
        sov.init()?;
        Ok(sov)
    }

    pub fn init(&self) -> Result<()> {
        self.db.init()?;
        self.index()?;
        Ok(())
    }

    pub fn index(&self) -> Result<()> {
        let mut notes = Vec::new();

        let walker = WalkDir::new(&self.config.toml.notes_dir).into_iter();
        for entry in walker.filter_entry(|e| {
            let p = e.path();
            if self.config.toml.ignore_dirs.contains(&p.to_path_buf()) {
                return false;
            }
            if p.is_file() && p.extension().map(|s| s == "md").unwrap_or(false)  {
                return true;
            }
            if p.is_dir() {
                return true;
            }
            false
        }) {
            let entry = entry?;
            // Skip directories
            if !entry.path().is_file() {
                continue;
            }

            // Do not re-index notes that have not been modified
            let metadata = entry.metadata()?;
            let ctime = DateTime::from_timestamp(metadata.ctime(), metadata.ctime_nsec() as u32)
                .ok_or(SovError::InvalidTime)?;
            if ctime < self.config.last_update {
                continue;
            }
            println!("Indexing {:?} ...", entry.path());

            let path = entry.path().to_path_buf();
            let Some(filename) = path.file_stem() else {
                continue;
            };
            let Some(filename) = filename.to_str() else {
                continue;
            };
            let Some((note_id, note_name)) = SovNote::extract_note_id(&filename) else {
                continue;
            };
            let note = SovNote::new(path, note_id, note_name)?;
            notes.push(note);
        }
        self.db.insert_notes(&notes)?;
        self.config.update_last_update()?;

        Ok(())
    }

    pub fn resolve_note(&self, note: &str) -> Result<PathBuf> {
        let (id, _) =
            SovNote::extract_note_id(note).ok_or(SovError::NoteIdNotFound(note.into()))?;
        let note_path = self.db.get_note_by_id(&id)?;
        Ok(note_path)
    }

    pub fn list_tags(&self) -> Result<Vec<String>> {
        // TODO: display and sort by count
        let unique_tags = self.db.get_unique_tags()?;
        Ok(unique_tags)
    }
}
