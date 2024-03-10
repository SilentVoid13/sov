use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Result, SovError};

pub struct SovNote {
    pub id: String,
    pub links: Vec<String>,
    pub yaml: YamlMetadata,
}

#[derive(Debug, Deserialize)]
pub struct YamlMetadata {
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
}

impl SovNote {
    pub fn extract_note_id(filename: &str) -> Option<String> {
        let id = filename.split(" - ").last()?;
        if id.len() != 12 {
            return None;
        }
        if !id.chars().all(|c| c.is_digit(10)) {
            return None;
        }
        Some(id.to_string())
    }

    pub fn from_file(path: &PathBuf, note_id: String) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        let yaml: YamlMetadata = match content.split("---").nth(1) {
            Some(metadata) => serde_yaml::from_str(metadata)?,
            None => YamlMetadata {
                aliases: Vec::new(),
                tags: Vec::new(),
            },
        };

        let mut chars = content.chars().peekable();
        let mut is_escaped = false;
        let mut in_code_block = false;

        let mut links = Vec::new();
        while let Some(c) = chars.next() {
            match c {
                '\\' => is_escaped = true,
                // TODO: this is a hack that handles inline code blocks
                // TODO: handle multiline code blocks
                '`' if !is_escaped => in_code_block = !in_code_block,
                '[' if !is_escaped && !in_code_block => {
                    if let Some('[') = chars.next() {
                        let s: String = chars.by_ref().take_while(|c| *c != ']').collect();
                        if chars.next() != Some(']') {
                            return Err(SovError::InvalidLink(path.display().to_string(), s));
                        }
                        // TODO: handle links with no ID (i.e. dead links)
                        let Some(link_id) = Self::extract_note_id(&s) else {
                            continue;
                            //return Err(SovError::InvalidLink(path.display().
                            // to_string(), s));
                        };
                        links.push(link_id);
                    }
                }
                _ => is_escaped = false,
            }
        }

        Ok(Self {
            id: note_id.to_string(),
            yaml,
            links,
        })
    }
}
