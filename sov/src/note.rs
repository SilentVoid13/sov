use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Result, SovError};

pub struct SovNote {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub yaml: YamlMetadata,
    pub links: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct YamlMetadata {
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
}

impl SovNote {
    pub fn new(path: PathBuf, id: String, name: String) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let yaml = SovNote::parse_yaml(&content)?;
        let links = SovNote::parse_links(&content)?;

        Ok(Self {
            id,
            name,
            path,
            yaml,
            links,
        })
    }

    pub fn parse_yaml(s: &str) -> Result<YamlMetadata> {
        let yaml: YamlMetadata = match s.split("---").nth(1) {
            Some(metadata) => serde_yaml::from_str(metadata)?,
            None => YamlMetadata {
                aliases: Vec::new(),
                tags: Vec::new(),
            },
        };
        Ok(yaml)
    }

    pub fn parse_links(s: &str) -> Result<Vec<String>> {
        let mut chars = s.chars().peekable();
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
                            return Err(SovError::InvalidLink(s));
                        }
                        // TODO: handle links with no ID (i.e. dead links)
                        let Some((link_id, _)) = Self::extract_note_id(&s) else {
                            continue;
                        };
                        links.push(link_id);
                    }
                }
                _ => is_escaped = false,
            }
        }
        Ok(links)
    }

    pub fn extract_note_id(s: &str) -> Option<(String, String)> {
        let (name, id) = s.rsplit_once(" - ")?;
        if id.len() != 12 {
            return None;
        }
        if !id.chars().all(|c| c.is_digit(10)) {
            return None;
        }
        Some((id.to_string(), name.to_string()))
    }
}
