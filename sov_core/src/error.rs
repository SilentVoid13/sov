use std::path::PathBuf;

use thiserror::Error;

pub type Result<T, E = SovError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum SovError {
    // third-party
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("chrono error: {0}")]
    Chrono(#[from] chrono::ParseError),
    #[error("toml deserialize error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("no config dir")]
    NoConfigDir,
    #[error("no notes dir, please set `notes_dir` in sov.toml")]
    NoNotesDir,
    #[error("script failed: {0}")]
    ScriptFailed(String),

    // Invalid
    #[error("invalid link: {0}")]
    InvalidLink(String),
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("file time error")]
    InvalidTime,
    #[error("invalid notes dir: {0}")]
    InvalidNotesDir(PathBuf),

    // Not Found
    #[error("note not found: {0}")]
    NoteNotFound(String),
    #[error("script not found: {0}")]
    ScriptNotFound(String),
}
