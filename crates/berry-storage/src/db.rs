//! SQLite database connection and migration runner.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use berry_domain::{
    Container, ExtractedMetadata, FileSortField, Folder, ImageFile, SearchCriteria, SortDirection,
};
use rusqlite::{params, Connection, OpenFlags};

use crate::migrations::{LATEST_VERSION, MIGRATIONS};

/// Errors produced by the storage layer.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown container id stored in database: {0}")]
    UnknownContainer(String),
    #[error("no folder with id {0}")]
    FolderNotFound(i64),
    #[error("no file with id {0}")]
    FileNotFound(i64),
    #[error("rating must be between 1 and 10, got {0}")]
    InvalidRating(u8),
    #[error("failed to open database at {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
}

/// SQL that inserts or updates a file row keyed by its unique path.
const UPSERT_FILE_SQL: &str =
    "INSERT INTO files (folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
     ON CONFLICT(path) DO UPDATE SET
         folder_id       = excluded.folder_id,
         container       = excluded.container,
         size_bytes      = excluded.size_bytes,
         modified_at     = excluded.modified_at,
         metadata        = excluded.metadata,
         rating          = coalesce(excluded.rating, files.rating),
         aesthetic_score = coalesce(excluded.aesthetic_score, files.aesthetic_score)";

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

    // --- Folders ---

    /// Insert a new folder and return it (with its generated id and timestamp).
    ///
    /// Fails with a SQLite `UNIQUE` constraint error if the path already
    /// exists; callers should check [`find_folder_by_path`](Self::find_folder_by_path)
    /// first or surface the conflict to the user.
    pub fn add_folder(&self, path: &str) -> Result<Folder, DatabaseError> {
        self.conn
            .execute("INSERT INTO folders (path) VALUES (?1)", [path])?;
        let id = self.conn.last_insert_rowid();
        let added_at: String =
            self.conn
                .query_row("SELECT added_at FROM folders WHERE id = ?1", [id], |row| {
                    row.get(0)
                })?;
        Ok(Folder {
            id,
            path: path.to_string(),
            added_at,
        })
    }

    /// All folders, ordered by id.
    pub fn list_folders(&self) -> Result<Vec<Folder>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id, path, added_at FROM folders ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
                added_at: row.get(2)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    /// The folder registered at `path`, if any.
    pub fn find_folder_by_path(&self, path: &str) -> Result<Option<Folder>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id, path, added_at FROM folders WHERE path = ?1")?;
        let mut rows = stmt.query([path])?;
        match rows.next()? {
            Some(row) => Ok(Some(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
                added_at: row.get(2)?,
            })),
            None => Ok(None),
        }
    }

    /// Delete a folder and cascade-delete its indexed files.
    ///
    /// Errors with [`DatabaseError::FolderNotFound`] if the id does not exist.
    pub fn remove_folder(&self, id: i64) -> Result<(), DatabaseError> {
        let affected = self
            .conn
            .execute("DELETE FROM folders WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(DatabaseError::FolderNotFound(id));
        }
        Ok(())
    }

    // --- Files ---

    /// Insert or update a file row keyed by its unique path, returning the id.
    ///
    /// `file.metadata` is serialized to JSON and stored in the `metadata`
    /// column; re-upserting the same path keeps a single row and refreshes its
    /// (size, mtime, metadata) — the incremental-scan cache.
    pub fn upsert_file(&self, file: &ImageFile) -> Result<i64, DatabaseError> {
        let metadata = file
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        self.conn.execute(
            UPSERT_FILE_SQL,
            params![
                file.folder_id,
                file.path,
                file.container.id(),
                file.size_bytes as i64,
                file.modified_at,
                metadata,
                file.rating.map(|r| r as i64),
                file.aesthetic_score,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert or update many files in a single transaction, returning the
    /// number of rows written.
    ///
    /// Prefer this over repeated [`upsert_file`](Self::upsert_file) calls when
    /// inserting in bulk (e.g. a folder scan): one commit per batch instead of
    /// one fsync per file.
    pub fn upsert_files(&self, files: &[ImageFile]) -> Result<u64, DatabaseError> {
        if files.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached(UPSERT_FILE_SQL)?;
            for file in files {
                let metadata = file
                    .metadata
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?;
                stmt.execute(params![
                    file.folder_id,
                    file.path,
                    file.container.id(),
                    file.size_bytes as i64,
                    file.modified_at,
                    metadata,
                    file.rating.map(|r| r as i64),
                    file.aesthetic_score,
                ])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Delete every file of `folder_id` whose path is not in `seen`.
    ///
    /// Used at the end of a scan to drop rows for files that were removed from
    /// disk. `seen` is passed as a JSON array and matched with SQLite's
    /// `json_each`, so any number of paths works without dynamic SQL.
    pub fn delete_files_not_in(
        &self,
        folder_id: i64,
        seen: &[String],
    ) -> Result<u64, DatabaseError> {
        let seen_json = serde_json::to_string(seen)?;
        let affected = self.conn.execute(
            "DELETE FROM files
             WHERE folder_id = ?1
               AND path NOT IN (SELECT value FROM json_each(?2))",
            params![folder_id, seen_json],
        )?;
        Ok(affected as u64)
    }

    /// Helper to deserialize an image file row.
    fn map_row(row: &rusqlite::Row<'_>) -> Result<ImageFile, DatabaseError> {
        let id: i64 = row.get(0)?;
        let folder_id: i64 = row.get(1)?;
        let path: String = row.get(2)?;
        let container_id: String = row.get(3)?;
        let size_bytes: i64 = row.get(4)?;
        let modified_at: i64 = row.get(5)?;
        let metadata: Option<String> = row.get(6)?;
        let rating: Option<i64> = row.get(7)?;
        let aesthetic_score: Option<f64> = row.get(8)?;

        let container = Container::from_id(&container_id)
            .ok_or_else(|| DatabaseError::UnknownContainer(container_id))?;
        let metadata = metadata
            .map(|json| serde_json::from_str::<ExtractedMetadata>(&json))
            .transpose()?;

        Ok(ImageFile {
            id: Some(id),
            folder_id,
            path,
            size_bytes: size_bytes as u64,
            modified_at,
            container,
            metadata,
            rating: rating.map(|r| r as u8),
            aesthetic_score,
        })
    }

    /// Query files with optional folder filtering and multi-criteria sorting.
    pub fn query_files(
        &self,
        folder_id: Option<i64>,
        sort: FileSortField,
        direction: SortDirection,
    ) -> Result<Vec<ImageFile>, DatabaseError> {
        let criteria = SearchCriteria {
            folder_id,
            sort: Some(sort),
            direction: Some(direction),
            ..Default::default()
        };
        self.search_files(&criteria)
    }

    /// Search files matching various criteria (text, parameters, ratings, folders, sorting, pagination).
    pub fn search_files(&self, criteria: &SearchCriteria) -> Result<Vec<ImageFile>, DatabaseError> {
        let mut conditions = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(fid) = criteria.folder_id {
            conditions.push("folder_id = ?".to_string());
            params.push(rusqlite::types::Value::Integer(fid));
        }

        if let Some(text) = &criteria.text {
            let pattern = format!("%{}%", text);
            conditions.push(
                "(path LIKE ? OR json_extract(metadata, '$.prompt') LIKE ? OR json_extract(metadata, '$.negative_prompt') LIKE ? OR json_extract(metadata, '$.model_name') LIKE ?)".to_string(),
            );
            params.push(rusqlite::types::Value::Text(pattern.clone()));
            params.push(rusqlite::types::Value::Text(pattern.clone()));
            params.push(rusqlite::types::Value::Text(pattern.clone()));
            params.push(rusqlite::types::Value::Text(pattern));
        }

        if let Some(prompt) = &criteria.prompt {
            conditions.push("json_extract(metadata, '$.prompt') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", prompt)));
        }

        if let Some(negative_prompt) = &criteria.negative_prompt {
            conditions.push("json_extract(metadata, '$.negative_prompt') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!(
                "%{}%",
                negative_prompt
            )));
        }

        if let Some(model_name) = &criteria.model_name {
            conditions.push("json_extract(metadata, '$.model_name') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", model_name)));
        }

        if let Some(model_hash) = &criteria.model_hash {
            conditions.push("json_extract(metadata, '$.model_hash') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", model_hash)));
        }

        if let Some(sampler) = &criteria.sampler {
            conditions.push("json_extract(metadata, '$.sampler') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", sampler)));
        }

        if let Some(min_steps) = criteria.min_steps {
            conditions.push("CAST(json_extract(metadata, '$.steps') AS INTEGER) >= ?".to_string());
            params.push(rusqlite::types::Value::Integer(min_steps as i64));
        }

        if let Some(max_steps) = criteria.max_steps {
            conditions.push("CAST(json_extract(metadata, '$.steps') AS INTEGER) <= ?".to_string());
            params.push(rusqlite::types::Value::Integer(max_steps as i64));
        }

        if let Some(min_cfg) = criteria.min_cfg {
            conditions.push("CAST(json_extract(metadata, '$.cfg_scale') AS REAL) >= ?".to_string());
            params.push(rusqlite::types::Value::Real(min_cfg));
        }

        if let Some(max_cfg) = criteria.max_cfg {
            conditions.push("CAST(json_extract(metadata, '$.cfg_scale') AS REAL) <= ?".to_string());
            params.push(rusqlite::types::Value::Real(max_cfg));
        }

        if let Some(min_rating) = criteria.min_rating {
            conditions.push("rating >= ?".to_string());
            params.push(rusqlite::types::Value::Integer(min_rating as i64));
        }

        if let Some(max_rating) = criteria.max_rating {
            conditions.push("rating <= ?".to_string());
            params.push(rusqlite::types::Value::Integer(max_rating as i64));
        }

        if let Some(min_aesthetic) = criteria.min_aesthetic {
            conditions.push("aesthetic_score >= ?".to_string());
            params.push(rusqlite::types::Value::Real(min_aesthetic));
        }

        if let Some(max_aesthetic) = criteria.max_aesthetic {
            conditions.push("aesthetic_score <= ?".to_string());
            params.push(rusqlite::types::Value::Real(max_aesthetic));
        }

        let sort = criteria.sort.unwrap_or(FileSortField::ModifiedAt);
        let direction = criteria.direction.unwrap_or(SortDirection::Desc);
        let order_clause = match (sort, direction) {
            (FileSortField::ModifiedAt, SortDirection::Asc) => "modified_at ASC, id ASC",
            (FileSortField::ModifiedAt, SortDirection::Desc) => "modified_at DESC, id DESC",
            (FileSortField::Path, SortDirection::Asc) => "path ASC",
            (FileSortField::Path, SortDirection::Desc) => "path DESC",
            (FileSortField::SizeBytes, SortDirection::Asc) => "size_bytes ASC, id ASC",
            (FileSortField::SizeBytes, SortDirection::Desc) => "size_bytes DESC, id DESC",
            (FileSortField::Rating, SortDirection::Asc) => {
                "rating ASC NULLS LAST, modified_at DESC, id DESC"
            }
            (FileSortField::Rating, SortDirection::Desc) => {
                "rating DESC NULLS LAST, modified_at DESC, id DESC"
            }
            (FileSortField::AestheticScore, SortDirection::Asc) => {
                "aesthetic_score ASC NULLS LAST, modified_at DESC, id DESC"
            }
            (FileSortField::AestheticScore, SortDirection::Desc) => {
                "aesthetic_score DESC NULLS LAST, modified_at DESC, id DESC"
            }
        };

        let limit_clause = match (criteria.limit, criteria.offset) {
            (Some(limit), Some(offset)) => {
                params.push(rusqlite::types::Value::Integer(limit as i64));
                params.push(rusqlite::types::Value::Integer(offset as i64));
                " LIMIT ? OFFSET ?"
            }
            (Some(limit), None) => {
                params.push(rusqlite::types::Value::Integer(limit as i64));
                " LIMIT ?"
            }
            (None, Some(offset)) => {
                params.push(rusqlite::types::Value::Integer(offset as i64));
                " LIMIT -1 OFFSET ?"
            }
            (None, None) => "",
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT id, folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score
             FROM files{where_clause} ORDER BY {order_clause}{limit_clause}"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_and_then(rusqlite::params_from_iter(&params), Self::map_row)?;

        let mut files = Vec::new();
        for file in rows {
            files.push(file?);
        }
        Ok(files)
    }

    /// List distinct non-empty model names present in indexed metadata.
    pub fn list_distinct_models(&self) -> Result<Vec<String>, DatabaseError> {
        let sql = "SELECT DISTINCT json_extract(metadata, '$.model_name') AS model
                   FROM files
                   WHERE json_extract(metadata, '$.model_name') IS NOT NULL
                     AND json_extract(metadata, '$.model_name') != ''
                   ORDER BY model ASC";
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut models = Vec::new();
        for model in rows {
            models.push(model?);
        }
        Ok(models)
    }

    /// List distinct non-empty sampler names present in indexed metadata.
    pub fn list_distinct_samplers(&self) -> Result<Vec<String>, DatabaseError> {
        let sql = "SELECT DISTINCT json_extract(metadata, '$.sampler') AS sampler
                   FROM files
                   WHERE json_extract(metadata, '$.sampler') IS NOT NULL
                     AND json_extract(metadata, '$.sampler') != ''
                   ORDER BY sampler ASC";
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut samplers = Vec::new();
        for sampler in rows {
            samplers.push(sampler?);
        }
        Ok(samplers)
    }

    /// Files of a folder, ordered by path, with `metadata` deserialized.
    pub fn list_files(&self, folder_id: i64) -> Result<Vec<ImageFile>, DatabaseError> {
        self.query_files(Some(folder_id), FileSortField::Path, SortDirection::Asc)
    }

    /// Update user rating (1–10, or None to clear) for an image file.
    pub fn set_file_rating(&self, file_id: i64, rating: Option<u8>) -> Result<(), DatabaseError> {
        if let Some(r) = rating {
            if !(1..=10).contains(&r) {
                return Err(DatabaseError::InvalidRating(r));
            }
        }
        let affected = self.conn.execute(
            "UPDATE files SET rating = ?1 WHERE id = ?2",
            params![rating.map(|r| r as i64), file_id],
        )?;
        if affected == 0 {
            return Err(DatabaseError::FileNotFound(file_id));
        }
        Ok(())
    }

    /// Update user rating (1–10, or None to clear) for multiple image files in a single transaction.
    pub fn set_files_rating(
        &self,
        file_ids: &[i64],
        rating: Option<u8>,
    ) -> Result<usize, DatabaseError> {
        if let Some(r) = rating {
            if !(1..=10).contains(&r) {
                return Err(DatabaseError::InvalidRating(r));
            }
        }
        if file_ids.is_empty() {
            return Ok(0);
        }

        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached("UPDATE files SET rating = ?1 WHERE id = ?2")?;
            let r_val = rating.map(|r| r as i64);
            for &id in file_ids {
                count += stmt.execute(params![r_val, id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Number of indexed files in a folder.
    pub fn count_files(&self, folder_id: i64) -> Result<i64, DatabaseError> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM files WHERE folder_id = ?1",
            [folder_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Total number of indexed files across all folders.
    pub fn count_all_files(&self) -> Result<i64, DatabaseError> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Counts of indexed files per folder.
    pub fn get_folder_file_counts(&self) -> Result<HashMap<i64, i64>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT folder_id, COUNT(*) FROM files GROUP BY folder_id")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
        let mut counts = HashMap::new();
        for row in rows {
            let (folder_id, count) = row?;
            counts.insert(folder_id, count);
        }
        Ok(counts)
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

    // --- Folder repository ---

    fn image(folder_id: i64, path: &str) -> ImageFile {
        ImageFile {
            id: None,
            folder_id,
            path: path.to_string(),
            size_bytes: 1,
            modified_at: 1,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
        }
    }

    #[test]
    fn folder_crud_roundtrip() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/tmp/img").unwrap();
        assert_eq!(folder.path, "/tmp/img");
        assert!(folder.id > 0);
        assert!(!folder.added_at.is_empty());

        assert_eq!(db.list_folders().unwrap(), vec![folder.clone()]);
        assert_eq!(
            db.find_folder_by_path("/tmp/img").unwrap(),
            Some(folder.clone())
        );
        assert_eq!(db.find_folder_by_path("/nope").unwrap(), None);

        db.remove_folder(folder.id).unwrap();
        assert!(db.list_folders().unwrap().is_empty());
    }

    #[test]
    fn add_folder_rejects_duplicate_path() {
        let db = Database::connect_in_memory().unwrap();
        db.add_folder("/dup").unwrap();
        let err = db.add_folder("/dup").unwrap_err();
        assert!(matches!(err, DatabaseError::Sqlite(_)));
    }

    #[test]
    fn remove_missing_folder_errors() {
        let db = Database::connect_in_memory().unwrap();
        assert!(matches!(
            db.remove_folder(42),
            Err(DatabaseError::FolderNotFound(42))
        ));
    }

    // --- File repository ---

    #[test]
    fn file_upsert_and_list() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();

        let id_a = db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();
        let id_b = db.upsert_file(&image(folder.id, "/img/b.png")).unwrap();
        assert_ne!(id_a, id_b);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "/img/a.png");
        assert_eq!(files[0].id, Some(id_a));
        assert_eq!(files[0].container, Container::Png);
        assert_eq!(files[1].path, "/img/b.png");

        assert_eq!(db.count_files(folder.id).unwrap(), 2);
    }

    #[test]
    fn file_upsert_is_idempotent_by_path() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();

        let mut file = image(folder.id, "/img/a.png");
        file.size_bytes = 100;
        let first = db.upsert_file(&file).unwrap();

        file.size_bytes = 200;
        let second = db.upsert_file(&file).unwrap();
        assert_eq!(first, second, "re-upserting the same path keeps one row");

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].size_bytes, 200);
    }

    #[test]
    fn metadata_roundtrips_through_json_column() {
        use berry_domain::MetadataFormat;

        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let meta = ExtractedMetadata {
            format: MetadataFormat::A1111,
            parameters: Some("a masterpiece, Steps: 20, Seed: 123".to_string()),
            raw: None,
            prompt: Some("a masterpiece".to_string()),
            negative_prompt: Some("blurry".to_string()),
            width: Some(512),
            height: Some(768),
            seed: Some("123".to_string()),
            steps: Some(20),
            cfg_scale: Some(7.0),
            sampler: Some("DPM++ 2M Karras".to_string()),
            model_name: Some("dreamshaper".to_string()),
            model_hash: Some("abc123".to_string()),
        };

        let mut file = image(folder.id, "/img/a.png");
        file.metadata = Some(meta.clone());
        db.upsert_file(&file).unwrap();

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].metadata, Some(meta));
    }

    #[test]
    fn remove_folder_cascades_to_files() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();

        db.remove_folder(folder.id).unwrap();
        assert!(db.list_files(folder.id).unwrap().is_empty());
    }

    #[test]
    fn delete_files_not_in_removes_orphans() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();
        db.upsert_file(&image(folder.id, "/img/b.png")).unwrap();

        let removed = db
            .delete_files_not_in(folder.id, &["/img/a.png".to_string()])
            .unwrap();
        assert_eq!(removed, 1);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "/img/a.png");
    }

    #[test]
    fn query_files_sorts_and_filters() {
        let db = Database::connect_in_memory().unwrap();
        let f1 = db.add_folder("/folder1").unwrap();
        let f2 = db.add_folder("/folder2").unwrap();

        let mut f1_a = image(f1.id, "/folder1/a.png");
        f1_a.modified_at = 100;
        f1_a.size_bytes = 500;
        f1_a.rating = Some(8);
        f1_a.aesthetic_score = Some(7.5);

        let mut f1_b = image(f1.id, "/folder1/b.png");
        f1_b.modified_at = 200;
        f1_b.size_bytes = 300;
        f1_b.rating = Some(9);
        f1_b.aesthetic_score = None;

        let mut f2_c = image(f2.id, "/folder2/c.png");
        f2_c.modified_at = 150;
        f2_c.size_bytes = 800;
        f2_c.rating = None;
        f2_c.aesthetic_score = Some(8.2);

        db.upsert_files(&[f1_a, f1_b, f2_c]).unwrap();

        // 1. Filter by folder
        let folder1_files = db
            .query_files(Some(f1.id), FileSortField::Path, SortDirection::Asc)
            .unwrap();
        assert_eq!(folder1_files.len(), 2);
        assert_eq!(folder1_files[0].path, "/folder1/a.png");
        assert_eq!(folder1_files[1].path, "/folder1/b.png");

        // 2. All folders (folder_id: None), sorted by modified_at DESC
        let all_by_date_desc = db
            .query_files(None, FileSortField::ModifiedAt, SortDirection::Desc)
            .unwrap();
        assert_eq!(all_by_date_desc.len(), 3);
        assert_eq!(all_by_date_desc[0].path, "/folder1/b.png"); // 200
        assert_eq!(all_by_date_desc[1].path, "/folder2/c.png"); // 150
        assert_eq!(all_by_date_desc[2].path, "/folder1/a.png"); // 100

        // 3. Sorted by size ASC
        let all_by_size = db
            .query_files(None, FileSortField::SizeBytes, SortDirection::Asc)
            .unwrap();
        assert_eq!(all_by_size[0].path, "/folder1/b.png"); // 300
        assert_eq!(all_by_size[1].path, "/folder1/a.png"); // 500
        assert_eq!(all_by_size[2].path, "/folder2/c.png"); // 800

        // 4. Sorted by rating DESC (NULLs last)
        let all_by_rating = db
            .query_files(None, FileSortField::Rating, SortDirection::Desc)
            .unwrap();
        assert_eq!(all_by_rating[0].path, "/folder1/b.png"); // rating 9
        assert_eq!(all_by_rating[1].path, "/folder1/a.png"); // rating 8
        assert_eq!(all_by_rating[2].path, "/folder2/c.png"); // rating None (nulls last)

        // 5. Sorted by aesthetic_score DESC (NULLs last)
        let all_by_aesthetic = db
            .query_files(None, FileSortField::AestheticScore, SortDirection::Desc)
            .unwrap();
        assert_eq!(all_by_aesthetic[0].path, "/folder2/c.png"); // 8.2
        assert_eq!(all_by_aesthetic[1].path, "/folder1/a.png"); // 7.5
        assert_eq!(all_by_aesthetic[2].path, "/folder1/b.png"); // None (nulls last)
    }

    #[test]
    fn rating_updates_and_preservation_on_reupsert() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let id = db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();

        // Initially no rating
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, None);

        // Set valid rating
        db.set_file_rating(id, Some(7)).unwrap();
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, Some(7));

        // Invalid rating rejected
        assert!(matches!(
            db.set_file_rating(id, Some(0)),
            Err(DatabaseError::InvalidRating(0))
        ));
        assert!(matches!(
            db.set_file_rating(id, Some(11)),
            Err(DatabaseError::InvalidRating(11))
        ));

        // Setting rating on nonexistent file returns error
        assert!(matches!(
            db.set_file_rating(999, Some(5)),
            Err(DatabaseError::FileNotFound(999))
        ));

        // Re-scanning (upserting without rating) preserves the previously set rating
        let re_scan_file = image(folder.id, "/img/a.png"); // has rating: None
        db.upsert_file(&re_scan_file).unwrap();
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, Some(7), "re-upsert preserves user rating");

        // Clearing rating with None works
        db.set_file_rating(id, None).unwrap();
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, None);
    }

    #[test]
    fn file_counts_per_folder_and_total() {
        let db = Database::connect_in_memory().unwrap();
        let f1 = db.add_folder("/f1").unwrap();
        let f2 = db.add_folder("/f2").unwrap();

        assert_eq!(db.count_all_files().unwrap(), 0);

        db.upsert_file(&image(f1.id, "/f1/1.png")).unwrap();
        db.upsert_file(&image(f1.id, "/f1/2.png")).unwrap();
        db.upsert_file(&image(f2.id, "/f2/1.png")).unwrap();

        assert_eq!(db.count_all_files().unwrap(), 3);
        let counts = db.get_folder_file_counts().unwrap();
        assert_eq!(counts.get(&f1.id), Some(&2));
        assert_eq!(counts.get(&f2.id), Some(&1));
    }

    fn sample_meta(
        prompt: &str,
        neg: &str,
        model: &str,
        sampler: &str,
        steps: u32,
        cfg: f64,
        seed: &str,
    ) -> ExtractedMetadata {
        ExtractedMetadata {
            format: berry_domain::MetadataFormat::A1111,
            parameters: None,
            raw: None,
            prompt: Some(prompt.to_string()),
            negative_prompt: Some(neg.to_string()),
            width: Some(512),
            height: Some(768),
            seed: Some(seed.to_string()),
            steps: Some(steps),
            cfg_scale: Some(cfg),
            sampler: Some(sampler.to_string()),
            model_name: Some(model.to_string()),
            model_hash: Some("abc12345".to_string()),
        }
    }

    #[test]
    fn search_files_multi_criteria() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/library").unwrap();

        let mut file1 = image(folder.id, "/library/cyberpunk_cat.png");
        file1.rating = Some(9);
        file1.aesthetic_score = Some(8.5);
        file1.metadata = Some(sample_meta(
            "a neon cyberpunk cat, 8k resolution, photorealistic",
            "blurry, low quality",
            "dreamshaper_xl",
            "Euler a",
            25,
            7.0,
            "12345",
        ));

        let mut file2 = image(folder.id, "/library/nature_forest.png");
        file2.rating = Some(6);
        file2.aesthetic_score = Some(6.2);
        file2.metadata = Some(sample_meta(
            "serene green forest, river stream, sunlight",
            "ugly, artifacts",
            "realisticVision_v5",
            "DPM++ 2M Karras",
            30,
            5.5,
            "67890",
        ));

        let mut file3 = image(folder.id, "/library/anime_portrait.png");
        file3.rating = Some(8);
        file3.aesthetic_score = Some(7.8);
        file3.metadata = Some(sample_meta(
            "anime girl in city, colorful lights",
            "bad anatomy",
            "dreamshaper_xl",
            "Euler a",
            20,
            8.0,
            "99999",
        ));

        db.upsert_files(&[file1, file2, file3]).unwrap();

        // 1. Broad text search for "cyberpunk"
        let res = db
            .search_files(&SearchCriteria {
                text: Some("cyberpunk".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].path, "/library/cyberpunk_cat.png");

        // 2. Broad text search matching model name "dreamshaper"
        let res = db
            .search_files(&SearchCriteria {
                text: Some("dreamshaper".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2);

        // 3. Prompt specific search
        let res = db
            .search_files(&SearchCriteria {
                prompt: Some("forest".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].path, "/library/nature_forest.png");

        // 4. Rating range >= 8
        let res = db
            .search_files(&SearchCriteria {
                min_rating: Some(8),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2);

        // 5. Steps between 22 and 35
        let res = db
            .search_files(&SearchCriteria {
                min_steps: Some(22),
                max_steps: Some(35),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2); // file1 (25) and file2 (30)

        // 6. CFG scale >= 7.0 and model_name dreamshaper_xl
        let res = db
            .search_files(&SearchCriteria {
                model_name: Some("dreamshaper_xl".to_string()),
                min_cfg: Some(7.0),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2);

        // 7. Sampler filter
        let res = db
            .search_files(&SearchCriteria {
                sampler: Some("DPM++".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].path, "/library/nature_forest.png");

        // 8. Pagination (limit 1, offset 1) sorted by rating DESC
        let res = db
            .search_files(&SearchCriteria {
                sort: Some(FileSortField::Rating),
                direction: Some(SortDirection::Desc),
                limit: Some(1),
                offset: Some(1),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].rating, Some(8)); // 2nd highest rating (after 9)

        // 9. Distinct models and samplers
        let models = db.list_distinct_models().unwrap();
        assert_eq!(models, vec!["dreamshaper_xl", "realisticVision_v5"]);

        let samplers = db.list_distinct_samplers().unwrap();
        assert_eq!(samplers, vec!["DPM++ 2M Karras", "Euler a"]);
    }

    #[test]
    fn set_files_rating_batch() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let id1 = db.upsert_file(&image(folder.id, "/img/1.png")).unwrap();
        let id2 = db.upsert_file(&image(folder.id, "/img/2.png")).unwrap();
        let _id3 = db.upsert_file(&image(folder.id, "/img/3.png")).unwrap();

        // Batch update id1 and id2 to rating 9
        let updated = db.set_files_rating(&[id1, id2], Some(9)).unwrap();
        assert_eq!(updated, 2);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, Some(9));
        assert_eq!(files[1].rating, Some(9));
        assert_eq!(files[2].rating, None);

        // Batch clear ratings
        let cleared = db.set_files_rating(&[id1, id2], None).unwrap();
        assert_eq!(cleared, 2);
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, None);
        assert_eq!(files[1].rating, None);
    }
}
