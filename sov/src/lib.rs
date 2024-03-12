pub mod config;
mod db;
pub mod error;
pub mod note;

use std::{collections::HashSet, os::unix::fs::MetadataExt};
use std::path::PathBuf;

use chrono::DateTime;
use config::SovConfig;
use db::SovDb;
use error::{Result, SovError};
use note::SovNote;
use tracing::info;
use walkdir::WalkDir;

pub struct Sov {
    config: SovConfig,
    db: SovDb,
}

impl Sov {
    pub fn new() -> Result<Self> {
        let config = SovConfig::load()?;
        let sov_db = SovDb::new(&config.db_path)?;

        let mut sov = Sov { config, db: sov_db };
        sov.init()?;
        Ok(sov)
    }

    pub fn init(&mut self) -> Result<()> {
        self.db.init()?;
        self.index()?;
        Ok(())
    }

    pub fn index(&mut self) -> Result<()> {
        let mut notes = Vec::new();

        let walker = WalkDir::new(&self.config.toml.notes_dir).into_iter();
        let mut fs_paths = HashSet::new();

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

            let path = entry.path().to_path_buf();
            let Some(filename) = path.file_stem() else {
                continue;
            };
            let Some(filename) = filename.to_str() else {
                continue;
            };

            fs_paths.insert(entry.path().to_path_buf());

            // Do not re-index notes that have not been modified
            let metadata = entry.metadata()?;
            let ctime = DateTime::from_timestamp(metadata.ctime(), metadata.ctime_nsec() as u32)
                .ok_or(SovError::InvalidTime)?;
            if ctime < self.config.last_update {
                continue;
            }
            info!("Indexing new note: {:?} ...", entry.path());

            let filename = filename.to_string();
            let note = SovNote::new(path, filename)?;
            notes.push(note);
        }

        // Insert new notes

        self.db.insert_notes(&notes)?;
        self.config.update_last_update()?;

        // Clean up DB

        let db_paths = self.db.get_all_note_paths()?;
        let dead_notes = db_paths.difference(&fs_paths);
        for note in dead_notes {
            info!("Deleting dead note: {:?}", note);
            self.db.delete_note_by_path(note)?;
        }
        self.db.clean_dead_tags()?;

        Ok(())
    }

    pub fn resolve_note(&self, note: &str) -> Result<PathBuf> {
        let note_path = self.db.get_note_by_filename(&note)?;
        Ok(note_path)
    }

    pub fn list_tags(&self) -> Result<Vec<String>> {
        // TODO: display and sort by count
        let unique_tags = self.db.get_unique_tags()?;
        Ok(unique_tags)
    }
}
