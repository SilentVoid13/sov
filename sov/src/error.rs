use std::path::PathBuf;

use thiserror::Error;

pub type Result<T, E = SovError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum SovError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("db error: {0}")]
    Db(#[from] duckdb::Error),
    #[error("walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("invalid note id: {0}")]
    InvalidNoteId(PathBuf),
    #[error("invalid link in file {0}: {1}")]
    InvalidLink(String, String),
}
