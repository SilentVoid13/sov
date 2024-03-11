use std::path::PathBuf;

use duckdb::{params, Connection};

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
        // TODO: handle note deletion
        // TODO: handle note rename?

        // Remove note metadata before re-inserting
        for note in notes {
            let sql = "DELETE FROM alias WHERE note_id = ?";
            let p = params![note.id];
            self.db.execute(sql, p)?;

            let sql = "DELETE FROM tag WHERE note_id = ?";
            let p = params![note.id];
            self.db.execute(sql, p)?;

            let sql = "DELETE FROM link WHERE src_note = ?";
            let p = params![note.id];
            self.db.execute(sql, p)?;
        }

        // First we insert all the notes
        for note in notes {
            let mut stmt = self
                .db
                .prepare("INSERT OR IGNORE INTO note (note_id, name, path) VALUES (?, ?, ?)")?;
            let p = params![
                note.id,
                note.name,
                note.path.to_str().ok_or(SovError::InvalidPath(note.path.clone()))?
            ];
            stmt.execute(p)?;

            let mut stmt = self
                .db
                .prepare("INSERT OR IGNORE INTO alias (alias_id, note_id) VALUES (?, ?)")?;
            for alias in &note.yaml.aliases {
                let p = params![alias, note.id];
                stmt.execute(p)?;
            }

            let mut stmt = self
                .db
                .prepare("INSERT OR IGNORE INTO tag (tag_id, note_id) VALUES (?, ?)")?;
            for tag in &note.yaml.tags {
                let p = params![tag, note.id];
                stmt.execute(p)?;
            }
        }

        // Then we link the notes
        for note in notes {
            let mut stmt = self
                .db
                .prepare("INSERT OR IGNORE INTO link (src_note, dst_note) VALUES (?, ?)")?;
            for link in &note.links {
                let p = params![note.id, link];
                stmt.execute(p)?;
            }
        }

        Ok(())
    }

    pub fn get_unique_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare("SELECT DISTINCT tag_id FROM tag")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    pub fn get_note_by_id(&self, id: &str) -> Result<PathBuf> {
        let mut stmt = self.db.prepare("SELECT path FROM note WHERE note_id = ?")?;
        let p = params![id];
        let path: String = stmt.query_row(p, |r| r.get(0))?;
        let path = PathBuf::from(path);
        Ok(path)
    }
}
