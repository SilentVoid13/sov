----------------------------------------
-- note
----------------------------------------

-- CREATE SEQUENCE IF NOT EXISTS note_seq;
CREATE TABLE IF NOT EXISTS note (
    note_id TEXT PRIMARY KEY,
);

----------------------------------------
-- link
----------------------------------------

-- CREATE SEQUENCE IF NOT EXISTS link_seq;
CREATE TABLE IF NOT EXISTS link (
    src_note TEXT NOT NULL REFERENCES note(note_id),
    dst_note TEXT NOT NULL REFERENCES note(note_id),
    PRIMARY KEY(src_note, dst_note)
);

----------------------------------------
-- tag
----------------------------------------

-- CREATE SEQUENCE IF NOT EXISTS tag_seq;
CREATE TABLE IF NOT EXISTS tag (
    tag_id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL REFERENCES note(note_id),
);

----------------------------------------
-- alias
----------------------------------------

-- CREATE SEQUENCE IF NOT EXISTS alias_seq;
CREATE TABLE IF NOT EXISTS alias (
    alias_id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL REFERENCES note(note_id),
);
