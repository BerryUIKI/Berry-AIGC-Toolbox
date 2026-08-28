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
];

/// The schema version the current code migrates databases to.
pub const LATEST_VERSION: i64 = MIGRATIONS.len() as i64;
