//! SQLite database connection and migration runner.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::migrations::{LATEST_VERSION, MIGRATIONS};

/// Errors produced by the storage layer.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("failed to open database at {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
}

/// A SQLite database with a fully migrated schema.
pub struct Database {
    conn: Connection,
    path: Option<PathBuf>,
}

impl Database {
    /// Open (creating if necessary) the database file at `path` and migrate
    /// it to the latest schema version.
    pub fn connect(path: &Path) -> Result<Self, DatabaseError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|source| DatabaseError::Open {
            path: path.to_path_buf(),
            source,
        })?;

        Self::init(conn, Some(path.to_path_buf()))
    }

    /// Open an in-memory database, primarily for tests.
    pub fn connect_in_memory() -> Result<Self, DatabaseError> {
        Self::init(Connection::open_in_memory()?, None)
    }

    fn init(conn: Connection, path: Option<PathBuf>) -> Result<Self, DatabaseError> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;

        let mut db = Self { conn, path };
        db.migrate()?;
        Ok(db)
    }

    /// The path the database was opened from, if any (in-memory has none).
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Apply any pending migrations, advancing `PRAGMA user_version`.
    fn migrate(&mut self) -> Result<(), DatabaseError> {
        let current = self.user_version()?;
        debug_assert!(
            current <= LATEST_VERSION,
            "database schema (v{current}) is newer than this build (v{LATEST_VERSION})"
        );

        for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
            let target = i as i64 + 1;
            self.apply_migration(sql, target)?;
        }
        Ok(())
    }

    /// Apply a single migration script and bump `user_version`, atomically.
    fn apply_migration(&mut self, sql: &str, target_version: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", target_version)?;
        tx.commit()?;
        Ok(())
    }

    /// The current `PRAGMA user_version` of the database.
    pub fn user_version(&self) -> Result<i64, DatabaseError> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        Ok(version)
    }

    /// Read a value from the `meta` table, if present.
    pub fn meta_get(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT value FROM meta WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_database_migrates_to_latest() {
        let db = Database::connect_in_memory().unwrap();
        assert_eq!(db.user_version().unwrap(), LATEST_VERSION);
        assert!(db.path().is_none());
    }

    #[test]
    fn migrations_are_idempotent() {
        // Re-running the runner on an already-migrated DB must be a no-op.
        let mut db = Database::connect_in_memory().unwrap();
        db.migrate().unwrap();
        assert_eq!(db.user_version().unwrap(), LATEST_VERSION);
    }

    #[test]
    fn migration_created_meta_table() {
        let db = Database::connect_in_memory().unwrap();
        assert_eq!(db.meta_get("anything").unwrap(), None);
    }

    #[test]
    fn file_database_persists_and_tracks_path() {
        let dir = std::env::temp_dir().join(format!("berry-storage-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");

        {
            let db = Database::connect(&path).unwrap();
            assert_eq!(db.path(), Some(path.as_path()));
            assert_eq!(db.user_version().unwrap(), LATEST_VERSION);
        }

        // Reopening the same file sees the persisted schema.
        let db = Database::connect(&path).unwrap();
        assert_eq!(db.user_version().unwrap(), LATEST_VERSION);

        // Drop the connection so SQLite releases its file locks before the
        // directory is removed (required on Windows).
        drop(db);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
