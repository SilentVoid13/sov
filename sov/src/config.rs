use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, SovError};

pub struct SovConfig {
    pub config_dir: PathBuf,
    pub last_update_path: PathBuf,
    pub last_update: DateTime<Utc>,
    pub db_path: PathBuf,
    pub toml_path: PathBuf,
    pub toml: SovConfigToml,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SovConfigToml {
    pub notes_dir: PathBuf,
    pub daily_notes: PathBuf,
    pub templates: PathBuf,
    pub ignore_dirs: Vec<PathBuf>,
}

impl SovConfig {
    pub const SOV_DIR: &'static str = "sov";
    pub const LAST_UPDATE_FILE: &'static str = "last_update";
    pub const MIN_DATE: &'static str = "1900-01-01T00:00:00+00:00";
    pub const DB_FILE: &'static str = "sov.db3";

    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir().ok_or(SovError::NoConfigDir)?;
        let config_dir = config_dir.join(Self::SOV_DIR);
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let last_update_path = config_dir.join(Self::LAST_UPDATE_FILE);
        let last_update = if !last_update_path.exists() {
            std::fs::write(&last_update_path, Self::MIN_DATE)?;
            Self::MIN_DATE.to_string()
        } else {
            std::fs::read_to_string(&last_update_path)?
                .trim()
                .to_string()
        };
        let last_update = DateTime::parse_from_rfc3339(&last_update)?.to_utc();

        let db_path = config_dir.join(Self::DB_FILE);

        let toml_path = config_dir.join("sov.toml");
        let mut toml = if toml_path.exists() {
            let toml_content = std::fs::read_to_string(&toml_path)?;
            let toml: SovConfigToml = toml::from_str(&toml_content)?;
            toml
        } else {
            let toml = SovConfigToml::default();
            std::fs::write(&toml_path, toml::to_string(&toml)?)?;
            toml
        };

        if toml.notes_dir.as_os_str().is_empty() {
            return Err(SovError::NoNotesDir);
        }
        if !toml.notes_dir.is_absolute() || !toml.notes_dir.exists() {
            return Err(SovError::InvalidNotesDir(toml.notes_dir));
        }
        // convert relative paths to absolute
        for p in toml.ignore_dirs.iter_mut() {
            if p.is_relative() {
                *p = toml.notes_dir.join(&p);
            }
        }
        if toml.daily_notes.is_relative() {
            toml.daily_notes = toml.notes_dir.join(&toml.daily_notes);
        }
        if toml.templates.is_relative() {
            toml.templates = toml.notes_dir.join(&toml.templates);
        }

        Ok(Self {
            config_dir,
            last_update_path,
            last_update,
            db_path,
            toml_path,
            toml,
        })
    }

    pub fn update_last_update(&mut self) -> Result<()> {
        let now = chrono::offset::Utc::now();
        std::fs::write(&self.last_update_path, now.to_rfc3339())?;
        self.last_update = now;
        Ok(())
    }
}
