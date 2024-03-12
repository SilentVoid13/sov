use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{Result, SovError};

pub struct SovNote {
    pub filename: String,
    pub path: PathBuf,
    pub yaml: YamlMetadata,
    pub links: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct YamlMetadata {
    // TODO: should I make this mandatory?
    pub aliases: Option<Vec<String>>,
    pub tags: Vec<String>,
}

impl SovNote {
    pub fn new(path: PathBuf, filename: String) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let yaml = SovNote::parse_yaml(&content)?;
        let links = SovNote::parse_links(&content)?;

        Ok(Self {
            filename,
            path,
            yaml,
            links,
        })
    }

    pub fn parse_yaml(s: &str) -> Result<YamlMetadata> {
        let yaml: YamlMetadata = match s.split("---").nth(1).map(|s| s.split("---").nth(0)).flatten() {
            Some(metadata) => serde_yaml::from_str(metadata)?,
            None => YamlMetadata {
                aliases: None,
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
                // TODO: handle code blocks
                // this is a hack that tries to handle inline code blocks (not working)
                //'`' if !is_escaped => in_code_block = !in_code_block,
                '[' if !is_escaped && !in_code_block => {
                    if let Some('[') = chars.next() {
                        let s: String = chars.by_ref().take_while(|c| *c != ']').collect();
                        let link = match s.split_once('|') {
                            Some((link, _)) => link.to_string(),
                            None => s,
                        };
                        // TODO: should we return an error or continue?
                        if chars.next() != Some(']') {
                            continue;
                            //return Err(SovError::InvalidLink(link));
                        }
                        links.push(link);
                    }
                }
                _ => is_escaped = false,
            }
        }
        Ok(links)
    }

    /*
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
    */
}
