----------------------------------------
-- note
----------------------------------------

CREATE TABLE IF NOT EXISTS note (
    note_id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    path TEXT NOT NULL
);

----------------------------------------
-- link
----------------------------------------

CREATE TABLE IF NOT EXISTS link (
    link_id INTEGER PRIMARY KEY AUTOINCREMENT,
    src_note TEXT NOT NULL REFERENCES note(note_id),
    link_value TEXT NOT NULL,
    start INTEGER NOT NULL,
    end INTEGER NOT NULL
);

----------------------------------------
-- tag
----------------------------------------

CREATE TABLE IF NOT EXISTS tag (
    tag_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tag_note (
    tag_id INTEGER NOT NULL REFERENCES tag(tag_id),
    note_id INTEGER NOT NULL REFERENCES note(note_id),
    PRIMARY KEY(tag_id, note_id)
);

----------------------------------------
-- alias
----------------------------------------

CREATE TABLE IF NOT EXISTS alias (
    alias_id TEXT NOT NULL,
    note_id INTEGER REFERENCES note(note_id),
    PRIMARY KEY(alias_id, note_id)
);


----------------------------------------
-- TRIGGERS
----------------------------------------

CREATE TRIGGER IF NOT EXISTS remove_dead_note_metadata BEFORE DELETE ON note
BEGIN
    DELETE FROM link WHERE src_note = OLD.note_id;
    DELETE FROM tag_note WHERE note_id = OLD.note_id;
    DELETE FROM alias WHERE note_id = OLD.note_id;
END;
