//! Ordered SQL migrations.
//!
//! Each entry is applied inside a single transaction and advances
//! `PRAGMA user_version` by one. **Never reorder, edit, or remove an applied
//! migration** — append a new one instead. Deployed databases rely on this.

/// The ordered list of migrations. Index `i` migrates the schema from version
/// `i` to version `i + 1`.
pub const MIGRATIONS: &[&str] = &[
    // v1: foundation tables.
    r#"
    CREATE TABLE meta (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    ) STRICT;
    "#,
    // v2: folder and file indexing.
    r#"
    CREATE TABLE folders (
        id        INTEGER PRIMARY KEY,
        path      TEXT NOT NULL UNIQUE,
        added_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE TABLE files (
        id          INTEGER PRIMARY KEY,
        folder_id   INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
        path        TEXT NOT NULL UNIQUE,
        container   TEXT NOT NULL,      -- stable Container id: png|jpg|webp|mp4
        size_bytes  INTEGER NOT NULL,
        modified_at INTEGER NOT NULL,   -- unix seconds; incremental-scan cache
        metadata    TEXT,               -- JSON ExtractedMetadata, NULL until extracted
        indexed_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE INDEX idx_files_folder ON files(folder_id);
    "#,
    // v3: ratings, aesthetics score, and browsing indexes.
    r#"
    ALTER TABLE files ADD COLUMN rating INTEGER;
    ALTER TABLE files ADD COLUMN aesthetic_score REAL;

    CREATE INDEX idx_files_modified ON files(modified_at);
    CREATE INDEX idx_files_rating ON files(rating);
    CREATE INDEX idx_files_aesthetic ON files(aesthetic_score);
    "#,
];

/// The schema version the current code migrates databases to.
pub const LATEST_VERSION: i64 = MIGRATIONS.len() as i64;
