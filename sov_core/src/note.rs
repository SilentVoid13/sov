use std::path::PathBuf;

use serde::Deserialize;

use crate::error::Result;

pub struct SovNote {
    pub filename: String,
    pub path: PathBuf,
    pub yaml: YamlMetadata,
    pub links: Vec<Link>,
}

#[derive(Debug)]
pub struct Link {
    pub value: String,
    pub start: usize,
    pub end: usize,
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
        let yaml: YamlMetadata = match s
            .split("---")
            .nth(1)
            .and_then(|s| s.split("---").nth(0))
        {
            Some(metadata) => serde_yaml::from_str(metadata)?,
            None => YamlMetadata {
                aliases: None,
                tags: Vec::new(),
            },
        };
        Ok(yaml)
    }

    pub fn parse_links(s: &str) -> Result<Vec<Link>> {
        let mut chars = s.chars().peekable().enumerate();
        let mut links = Vec::new();
        let mut is_escaped = false;

        while let Some((i, c)) = chars.next() {
            match c {
                '\\' => is_escaped = true,
                '[' if !is_escaped => {
                    let start_off = i;
                    if let Some((_, '[')) = chars.next() {
                        let s: String = chars
                            .by_ref()
                            .take_while(|(_, c)| *c != ']')
                            .map(|(_, c)| c)
                            .collect();
                        let link = match s.split_once('|') {
                            Some((link, _)) => link.to_string(),
                            None => s,
                        };
                        // TODO: should we return an error or continue?
                        let Some((end_off, ']')) = chars.next() else {
                            continue;
                            //return Err(SovError::InvalidLink(link));
                        };
                        links.push(Link {
                            value: link,
                            start: start_off,
                            end: end_off,
                        });
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
