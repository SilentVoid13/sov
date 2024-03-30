pub mod config;
mod db;
pub mod error;
pub mod note;

use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use chrono::DateTime;
use config::SovConfig;
use db::SovDb;
use error::{Result, SovError};
use note::{Link, SovNote};
use tracing::info;
use walkdir::WalkDir;

pub struct Sov {
    config: SovConfig,
    db: SovDb,
}

#[derive(Debug)]
pub enum SovFeature {
    Index,
    Daily,
    ListNotes,
    ListTags,
    ListOrphans,
    ListDeadLinks,
    ListAliases,
    ListScripts,
    ResolveNote {
        note: String,
    },
    ResolveLinks {
        note: String,
    },
    ResolveDeadLinks {
        note: String,
    },
    ResolveBacklinks {
        note: String,
    },
    Rename {
        path: PathBuf,
        new_filename: String,
    },
    SearchTag {
        tag: String,
    },
    ScriptRun {
        script_name: String,
        args: Vec<String>,
    },
    ScriptCreate {
        note_name: String,
        script_name: String,
        args: Vec<String>,
    },
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
            if p.is_file() && p.extension().map(|s| s == "md").unwrap_or(false) {
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

    pub fn resolve_note(&self, filename: &str) -> Result<Option<PathBuf>> {
        let note_path = self.db.get_note_by_filename(filename)?;
        Ok(note_path)
    }

    pub fn resolve_backlinks(&self, filename: &str) -> Result<Vec<(PathBuf, Link)>> {
        let references = self.db.get_backlinks(filename)?;
        Ok(references)
    }

    pub fn resolve_links(&self, filename: &str) -> Result<Vec<Link>> {
        let links = self.db.get_links(filename)?;
        Ok(links)
    }

    pub fn resolve_dead_links(&self, filename: &str) -> Result<Vec<Link>> {
        let dead_links = self.db.get_dead_links(filename)?;
        Ok(dead_links)
    }

    pub fn list_note_names(&self) -> Result<Vec<String>> {
        let notes = self.db.get_all_note_names()?;
        Ok(notes)
    }

    pub fn list_note_aliases(&self) -> Result<Vec<(String, String)>> {
        let aliases = self.db.get_all_note_aliases()?;
        Ok(aliases)
    }

    pub fn list_tags(&self) -> Result<Vec<String>> {
        // TODO: display and sort by count
        let unique_tags = self.db.get_unique_tags()?;
        Ok(unique_tags)
    }

    pub fn list_orphans(&self) -> Result<Vec<PathBuf>> {
        let orphans = self.db.get_all_orphaned_notes()?;
        Ok(orphans)
    }

    pub fn list_dead_links(&self) -> Result<Vec<(PathBuf, String)>> {
        let dead_links = self.db.get_all_dead_links()?;
        Ok(dead_links)
    }

    pub fn list_scripts(&self) -> Result<Vec<String>> {
        let scripts = self.config.toml.scripts_dir.read_dir()?;
        let scripts = scripts
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.metadata().ok()?.is_dir() {
                    return None;
                }
                let path = entry.path();
                let filename = path.file_name()?.to_str()?.to_string();
                Some(filename)
            })
            .collect();
        Ok(scripts)
    }

    pub fn daily(&self) -> Result<PathBuf> {
        // TODO: add day offset to create notes for previous/next days?
        let now = chrono::Local::now();
        // TODO: add support for custom date format
        let date = now.format("%Y-%m-%d").to_string();
        if let Some(path) = self.db.get_note_by_filename(&date)? {
            return Ok(path);
        }
        let path = self
            .config
            .toml
            .daily_notes_dir
            .join(&date)
            .with_extension("md");
        info!("Creating new daily note: {:?}", path);
        // TODO: add template support
        std::fs::File::create(&path)?;
        Ok(path)
    }

    pub fn script_run(&self, script_name: &str, args: Vec<String>) -> Result<String> {
        let script_path = self.config.toml.scripts_dir.join(script_name);
        if !script_path.exists() {
            return Err(SovError::ScriptNotFound(script_name.to_string()));
        }
        let output = std::process::Command::new(script_path)
            .args(args)
            .output()?;
        if !output.status.success() {
            return Err(SovError::ScriptFailed(script_name.to_string()));
        }
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.to_string())
    }

    pub fn script_create(
        &self,
        note_name: &str,
        script_name: &str,
        args: Vec<String>,
    ) -> Result<PathBuf> {
        let note_path = self
            .config
            .toml
            .notes_dir
            .join(note_name)
            .with_extension("md");
        info!("Creating new note: {:?}", note_path);
        let note_content = self.script_run(script_name, args)?;
        std::fs::write(&note_path, note_content)?;
        Ok(note_path)
    }

    pub fn search_tag(&self, tag: &str) -> Result<Vec<PathBuf>> {
        let notes = self.db.find_notes_by_tag(tag)?;
        Ok(notes)
    }
}
