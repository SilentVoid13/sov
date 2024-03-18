use std::collections::HashSet;
use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

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

    pub fn insert_notes(&mut self, notes: &[SovNote]) -> Result<()> {
        let tx = self.db.transaction()?;
        {
            // Preparing statements outside of the loop is more efficient
            let mut ins_note =
                tx.prepare("INSERT INTO note (filename, path) VALUES (?, ?) RETURNING(note_id)")?;
            let mut ins_alias =
                tx.prepare("INSERT INTO alias (alias_id, note_id) VALUES (?, ?)")?;
            let mut ins_tag = tx.prepare("INSERT INTO tag (name) VALUES (?) RETURNING(tag_id)")?;
            let mut ins_tag_note =
                tx.prepare("INSERT INTO tag_note (tag_id, note_id) VALUES (?, ?)")?;
            // multiple links to the same note in the same file is possible
            let mut ins_link =
                tx.prepare("INSERT OR REPLACE INTO link (src_note, dst_note) VALUES (?, ?)")?;

            for note in notes {
                let path = note
                    .path
                    .to_str()
                    .ok_or(SovError::InvalidPath(note.path.clone()))?;

                let sql = "SELECT note_id FROM note WHERE path = ?";
                let p = params![path];
                let id: Option<u64> = tx.query_row(sql, p, |r| r.get(0)).optional()?;
                let id = if let Some(id) = id {
                    id
                } else {
                    let p = params![note.filename, path,];
                    let id: u64 = ins_note.query_row(p, |r| r.get(0))?;
                    id
                };

                // clean up old metadata
                let sql = "DELETE FROM alias WHERE note_id = ?";
                let p = params![id];
                tx.execute(sql, p)?;

                let sql = "DELETE FROM tag_note WHERE note_id = ?";
                let p = params![id];
                tx.execute(sql, p)?;

                let sql = "DELETE FROM link WHERE src_note = ?";
                let p = params![id];
                tx.execute(sql, p)?;

                // insert new metadata
                if let Some(aliases) = &note.yaml.aliases {
                    for alias in aliases {
                        let p = params![alias, id];
                        ins_alias.execute(p)?;
                    }
                }

                for tag_name in &note.yaml.tags {
                    let sql = "SELECT tag_id FROM tag WHERE name = ?";
                    let p = params![tag_name];
                    let tag_id: Option<u64> = tx.query_row(sql, p, |r| r.get(0)).optional()?;
                    let tag_id = if let Some(tag_id) = tag_id {
                        tag_id
                    } else {
                        let p = params![tag_name];
                        let id: u64 = ins_tag.query_row(p, |r| r.get(0))?;
                        id
                    };
                    let p = params![tag_id, id];
                    ins_tag_note.execute(p)?;
                }

                for link in &note.links {
                    let p = params![id, link.value];
                    ins_link.execute(p)?;
                }
            }
        }
        tx.commit()?;

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

    pub fn get_note_by_filename(&self, filename: &str) -> Result<Option<PathBuf>> {
        let mut stmt = self
            .db
            .prepare("SELECT path FROM note WHERE filename = ?")?;
        let p = params![filename];
        // TODO: handle multiple rows
        let path = stmt
            .query_row(p, |r| r.get(0).map(|p: String| PathBuf::from(p)))
            .optional()?;
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

    pub fn get_all_note_names(&self) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare("SELECT filename FROM note")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        Ok(names)
    }

    pub fn get_orphaned_notes(&self) -> Result<Vec<PathBuf>> {
        let sql = "SELECT path FROM note WHERE filename NOT IN (SELECT dst_note FROM link)";
        let mut stmt = self.db.prepare(sql)?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut paths = Vec::new();
        for row in rows {
            let path: String = row?;
            paths.push(PathBuf::from(path));
        }
        Ok(paths)
    }

    pub fn get_dead_links(&self) -> Result<Vec<(PathBuf, String)>> {
        let sql = "
            SELECT path, dst_note FROM note t1
            JOIN link t2 ON t1.note_id = t2.src_note
            WHERE dst_note NOT IN (
                SELECT filename FROM note
            )";
        let mut stmt = self.db.prepare(sql)?;
        let mut rows = stmt.query([])?;
        let mut paths = Vec::new();
        while let Some(row) = rows.next()? {
            let path: String = row.get(0)?;
            let dead_link = row.get(1)?;
            paths.push((PathBuf::from(path), dead_link));
        }
        Ok(paths)
    }

    pub fn get_backlinks(&self, note: &str) -> Result<Vec<PathBuf>> {
        let sql =
            "SELECT path FROM note WHERE note_id IN (SELECT src_note FROM link WHERE dst_note = ?)";
        let mut stmt = self.db.prepare(sql)?;
        let p = params![note];
        let rows = stmt.query_map(p, |row| row.get(0).map(|p: String| PathBuf::from(p)))?;
        let mut refs = Vec::new();
        for row in rows {
            refs.push(row?);
        }
        Ok(refs)
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
