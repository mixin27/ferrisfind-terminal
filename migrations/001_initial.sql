CREATE TABLE sources (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    source_type TEXT NOT NULL CHECK (source_type IN ('seed', 'json', 'csv')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    record_count INTEGER NOT NULL DEFAULT 0 CHECK (record_count >= 0)
);

CREATE TABLE records (
    rowid INTEGER PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    category TEXT NOT NULL CHECK (length(trim(category)) > 0),
    content TEXT NOT NULL CHECK (length(trim(content)) > 0),
    metadata_json TEXT NOT NULL DEFAULT '{}'
        CHECK (json_valid(metadata_json) AND json_type(metadata_json) = 'object'),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_records_source_category ON records(source_id, category);

CREATE VIRTUAL TABLE records_fts USING fts5(
    title, content, content='records', content_rowid='rowid', tokenize='unicode61'
);

CREATE TABLE import_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT REFERENCES sources(id) ON DELETE SET NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed')),
    inserted_count INTEGER NOT NULL DEFAULT 0 CHECK (inserted_count >= 0),
    rejected_count INTEGER NOT NULL DEFAULT 0 CHECK (rejected_count >= 0),
    message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TRIGGER records_ai AFTER INSERT ON records BEGIN
    INSERT INTO records_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
    UPDATE sources SET record_count = record_count + 1 WHERE id = new.source_id;
END;

CREATE TRIGGER records_ad AFTER DELETE ON records BEGIN
    INSERT INTO records_fts(records_fts, rowid, title, content)
        VALUES ('delete', old.rowid, old.title, old.content);
    UPDATE sources SET record_count = record_count - 1 WHERE id = old.source_id;
END;

CREATE TRIGGER records_au AFTER UPDATE ON records BEGIN
    INSERT INTO records_fts(records_fts, rowid, title, content)
        VALUES ('delete', old.rowid, old.title, old.content);
    INSERT INTO records_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
    UPDATE sources SET record_count = record_count - 1
        WHERE id = old.source_id AND old.source_id != new.source_id;
    UPDATE sources SET record_count = record_count + 1
        WHERE id = new.source_id AND old.source_id != new.source_id;
END;

-- Establish a rebuilt index before recording schema version 1.
INSERT INTO records_fts(records_fts) VALUES ('rebuild');
