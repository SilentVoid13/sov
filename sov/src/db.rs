use std::collections::HashSet;
use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};
use tracing::info;

use crate::error::{Result, SovError};
use crate::SovNote;

pub struct SovDb {
    db: Connection,
}

impl SovDb {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let db = Connection::open(&path)?;
        Ok(Self { db })
    }

    pub fn init(&self) -> Result<()> {
        let sql = include_str!("db.sql");
        self.db.execute_batch(sql)?;
        Ok(())
    }

    pub fn insert_notes(&self, notes: &[SovNote]) -> Result<()> {
        for note in notes {
            let path = note.path.to_str().ok_or(SovError::InvalidPath(note.path.clone()))?;

            let sql = "SELECT note_id FROM note WHERE path = ?";
            let p = params![path];
            let id: Option<u64> = self.db.query_row(sql, p, |r| r.get(0)).optional()?;
            let id = if let Some(id) = id {
                id
            } else {
                let mut stmt = self.db.prepare(
                    "INSERT INTO note (filename, path) VALUES (?, ?) RETURNING(note_id)",
                )?;
                let p = params![
                    note.filename,
                    path,
                ];
                let id: u64 = stmt.query_row(p, |r| r.get(0))?;
                id
            };

            // clean up old metadata
            let sql = "DELETE FROM alias WHERE note_id = ?";
            let p = params![id];
            self.db.execute(sql, p)?;

            let sql = "DELETE FROM tag_note WHERE note_id = ?";
            let p = params![id];
            self.db.execute(sql, p)?;

            let sql = "DELETE FROM link WHERE src_note = ?";
            let p = params![id];
            self.db.execute(sql, p)?;

            // insert new metadata
            let mut stmt = self
                .db
                .prepare("INSERT INTO alias (alias_id, note_id) VALUES (?, ?)")?;
            if let Some(aliases) = &note.yaml.aliases {
                for alias in aliases {
                    let p = params![alias, id];
                    stmt.execute(p)?;
                }
            }

            for tag_name in &note.yaml.tags {
                let sql = "SELECT tag_id FROM tag WHERE name = ?";
                let p = params![tag_name];
                let tag_id: Option<u64> = self.db.query_row(sql, p, |r| r.get(0)).optional()?;
                let tag_id = if let Some(tag_id) = tag_id {
                    tag_id
                } else {
                    let mut stmt = self.db.prepare("INSERT INTO tag (name) VALUES (?) RETURNING(tag_id)")?;
                    let p = params![tag_name];
                    let id: u64 = stmt.query_row(p, |r| r.get(0))?;
                    id
                };
                let mut stmt = self
                    .db
                    .prepare("INSERT INTO tag_note (tag_id, note_id) VALUES (?, ?)")?;
                let p = params![tag_id, id];
                stmt.execute(p)?;
            }

            // multiple links to the same note in the same file is possible
            let mut stmt = self
                .db
                .prepare("INSERT OR REPLACE INTO link (src_note, dst_note) VALUES (?, ?)")?;
            for link in &note.links {
                let p = params![id, link];
                stmt.execute(p)?;
            }
        }

        Ok(())
    }

    pub fn get_unique_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare("SELECT name FROM tag")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    pub fn get_note_by_filename(&self, filename: &str) -> Result<PathBuf> {
        let mut stmt = self.db.prepare("SELECT path FROM note WHERE filename = ?")?;
        let p = params![filename];
        // TODO: handle multiple rows
        let path: String = stmt.query_row(p, |r| r.get(0))?;
        let path = PathBuf::from(path);
        Ok(path)
    }

    pub fn get_all_note_paths(&self) -> Result<HashSet<PathBuf>> {
        let mut stmt = self.db.prepare("SELECT path FROM note")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut paths = HashSet::new();
        for row in rows {
            let path: String = row?;
            paths.insert(PathBuf::from(path));
        }
        Ok(paths)
    }

    pub fn delete_note_by_path(&self, path: &PathBuf) -> Result<()> {
        let path = path.to_str().ok_or(SovError::InvalidPath(path.clone()))?;
        let sql = "DELETE FROM note WHERE path = ?";
        let p = params![path];
        self.db.execute(sql, p)?;
        Ok(())
    }

    pub fn clean_dead_tags(&self) -> Result<()> {
        let sql = "DELETE FROM tag WHERE tag_id NOT IN (SELECT tag_id FROM tag_note)";
        self.db.execute(sql, [])?;
        Ok(())
    }
}
