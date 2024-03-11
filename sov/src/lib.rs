mod db;
pub mod error;
pub mod note;

use std::{os::unix::fs::MetadataExt, path::PathBuf};

use chrono::{DateTime, FixedOffset};
use db::SovDb;
use error::{Result, SovError};
use note::SovNote;
use walkdir::WalkDir;

pub struct Sov {
    config: SovConfig,
    db: SovDb,
}

pub struct SovConfig {
    pub last_update: DateTime<FixedOffset>,
    pub db_path: PathBuf,
    pub notes_path: PathBuf,
}

impl Sov {
    pub const MIN_DATE: &'static str = "1900-01-01T00:00:00+00:00";

    pub fn new() -> Result<Self> {
        // TODO: config file
        let last_update_f = PathBuf::from(".last_update");
        let last_update = if !last_update_f.exists() {
            std::fs::write(".last_update", Self::MIN_DATE)?;
            Self::MIN_DATE.to_string()
        } else {
            std::fs::read_to_string(".last_update")?.trim().to_string()
        };
        let last_update = DateTime::parse_from_rfc3339(&last_update)?;

        let config = SovConfig {
            last_update,
            db_path: "sov.db3".into(),
            notes_path: "/home/silent/quark".into(),
        };
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

    pub fn update_last_update(&self) -> Result<()> {
        let now = chrono::offset::Utc::now();
        std::fs::write(".last_update", now.to_rfc3339())?;
        Ok(())
    }

    pub fn index(&self) -> Result<()> {
        let mut notes = Vec::new();

        let walker = WalkDir::new(&self.config.notes_path).into_iter();
        for entry in walker.filter_entry(|e| {
            e.path().is_dir()
                || (e.path().is_file() && e.path().extension().map(|s| s == "md").unwrap_or(false))
        }) {
            let entry = entry?;
            // Skip directories
            if !entry.path().is_file() {
                continue;
            }

            // Do not re-index notes that have not been modified
            let metadata = entry.metadata()?;
            let ctime = DateTime::from_timestamp(metadata.ctime(), metadata.ctime_nsec() as u32).ok_or(SovError::InvalidTime)?;
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
        self.update_last_update()?;

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
