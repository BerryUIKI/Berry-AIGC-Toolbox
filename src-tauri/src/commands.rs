//! Tauri IPC commands exposed to the frontend.
//!
//! Each command is a thin wrapper: parse the request, call into the core
//! crates, and serialize the result. Business logic lives in the `berry-*`
//! crates, not here.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;

use berry_domain::{
    Album, FileSortField, Folder, ImageFile, PromptStat, SearchCriteria, SortDirection, Tag,
};
use berry_scan::{ScanStats, Scanner};
use berry_storage::Database;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::AppState;

/// Lock and return the shared database connection.
fn db<'a>(state: &'a State<'a, AppState>) -> Result<MutexGuard<'a, Database>, String> {
    state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())
}

/// Resolve a user-supplied folder path to a canonical absolute path.
///
/// The Windows `\\?\` verbatim prefix produced by `canonicalize` is stripped so
/// stored paths (and the walk keys derived from them) stay readable.
fn canonicalize_folder(path: &str) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("cannot access path: {e}"))?;
    if !meta.is_dir() {
        return Err("path is not a directory".to_string());
    }
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
    let text = canonical.to_string_lossy();
    let stripped = text.strip_prefix(r"\\?\").unwrap_or(&text);
    Ok(stripped.to_string())
}

/// Diagnostics shown on the M1 shell to prove the full chain works
/// (frontend → IPC → Rust → SQLite).
#[derive(Serialize)]
pub struct AppInfo {
    /// Crate version from `Cargo.toml`.
    pub app_version: String,
    /// The SQLite `PRAGMA user_version` after migration.
    pub schema_version: i64,
    /// Absolute path of the opened database (empty for in-memory).
    pub database_path: String,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, String> {
    let db = db(&state)?;
    let schema_version = db.user_version().map_err(|e| e.to_string())?;
    let database_path = db
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    Ok(AppInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        database_path,
    })
}

/// Register a folder for scanning. Rejects paths that do not exist, are not
/// directories, or are already registered.
#[tauri::command]
pub fn add_folder(path: String, state: State<'_, AppState>) -> Result<Folder, String> {
    let path = canonicalize_folder(&path)?;
    let db = db(&state)?;
    if db
        .find_folder_by_path(&path)
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err("folder is already added".to_string());
    }
    db.add_folder(&path).map_err(|e| e.to_string())
}

/// All registered folders, ordered by id.
#[tauri::command]
pub fn list_folders(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    db(&state)?.list_folders().map_err(|e| e.to_string())
}

/// Remove a folder and its indexed files.
#[tauri::command]
pub fn remove_folder(folder_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .remove_folder(folder_id)
        .map_err(|e| e.to_string())
}

/// Files of a folder, ordered by path.
#[tauri::command]
pub fn list_files(folder_id: i64, state: State<'_, AppState>) -> Result<Vec<ImageFile>, String> {
    db(&state)?.list_files(folder_id).map_err(|e| e.to_string())
}

/// Query indexed files with optional folder filtering and multi-criteria sorting.
#[tauri::command]
pub fn query_files(
    folder_id: Option<i64>,
    sort: Option<FileSortField>,
    direction: Option<SortDirection>,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    let sort = sort.unwrap_or(FileSortField::ModifiedAt);
    let direction = direction.unwrap_or(SortDirection::Desc);
    db(&state)?
        .query_files(folder_id, sort, direction)
        .map_err(|e| e.to_string())
}

/// Search indexed files using structured criteria.
#[tauri::command]
pub fn search_files(
    criteria: SearchCriteria,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?
        .search_files(&criteria)
        .map_err(|e| e.to_string())
}

/// Search indexed files using a parsed query string.
#[tauri::command]
pub fn search_files_by_query(
    query: String,
    folder_id: Option<i64>,
    sort: Option<FileSortField>,
    direction: Option<SortDirection>,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    let mut criteria = SearchCriteria::from_query(&query);
    if folder_id.is_some() {
        criteria.folder_id = folder_id;
    }
    if sort.is_some() {
        criteria.sort = sort;
    }
    if direction.is_some() {
        criteria.direction = direction;
    }
    db(&state)?
        .search_files(&criteria)
        .map_err(|e| e.to_string())
}

/// List distinct model names present in indexed metadata.
#[tauri::command]
pub fn list_distinct_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    db(&state)?
        .list_distinct_models()
        .map_err(|e| e.to_string())
}

/// List distinct sampler names present in indexed metadata.
#[tauri::command]
pub fn list_distinct_samplers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    db(&state)?
        .list_distinct_samplers()
        .map_err(|e| e.to_string())
}

/// Update user rating (1–10, or null to clear) for an image file.
#[tauri::command]
pub fn set_file_rating(
    file_id: i64,
    rating: Option<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .set_file_rating(file_id, rating)
        .map_err(|e| e.to_string())
}

/// Update user rating (1–10, or null to clear) for multiple image files.
#[tauri::command]
pub fn set_files_rating(
    file_ids: Vec<i64>,
    rating: Option<u8>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .set_files_rating(&file_ids, rating)
        .map_err(|e| e.to_string())
}

// --- Albums ---

/// Create a new album.
#[tauri::command]
pub fn create_album(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Album, String> {
    db(&state)?
        .create_album(&name, description.as_deref())
        .map_err(|e| e.to_string())
}

/// Retrieve an album by ID.
#[tauri::command]
pub fn get_album(id: i64, state: State<'_, AppState>) -> Result<Option<Album>, String> {
    db(&state)?.get_album(id).map_err(|e| e.to_string())
}

/// List all albums.
#[tauri::command]
pub fn list_albums(state: State<'_, AppState>) -> Result<Vec<Album>, String> {
    db(&state)?.list_albums().map_err(|e| e.to_string())
}

/// Rename an album.
#[tauri::command]
pub fn rename_album(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .rename_album(id, &new_name)
        .map_err(|e| e.to_string())
}

/// Delete an album.
#[tauri::command]
pub fn delete_album(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?.delete_album(id).map_err(|e| e.to_string())
}

/// Add a file to an album.
#[tauri::command]
pub fn add_file_to_album(
    album_id: i64,
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .add_file_to_album(album_id, file_id)
        .map_err(|e| e.to_string())
}

/// Add multiple files to an album.
#[tauri::command]
pub fn add_files_to_album(
    album_id: i64,
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .add_files_to_album(album_id, &file_ids)
        .map_err(|e| e.to_string())
}

/// Remove a file from an album.
#[tauri::command]
pub fn remove_file_from_album(
    album_id: i64,
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .remove_file_from_album(album_id, file_id)
        .map_err(|e| e.to_string())
}

/// Remove multiple files from an album.
#[tauri::command]
pub fn remove_files_from_album(
    album_id: i64,
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .remove_files_from_album(album_id, &file_ids)
        .map_err(|e| e.to_string())
}

/// Count files in an album.
#[tauri::command]
pub fn count_album_files(album_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    db(&state)?
        .count_album_files(album_id)
        .map_err(|e| e.to_string())
}

/// List files in an album.
#[tauri::command]
pub fn list_album_files(
    album_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?
        .list_album_files(album_id)
        .map_err(|e| e.to_string())
}

// --- Tags ---

/// Create a tag.
#[tauri::command]
pub fn create_tag(
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    db(&state)?
        .create_tag(&name, color.as_deref())
        .map_err(|e| e.to_string())
}

/// List all tags.
#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    db(&state)?.list_tags().map_err(|e| e.to_string())
}

/// Delete a tag.
#[tauri::command]
pub fn delete_tag(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?.delete_tag(id).map_err(|e| e.to_string())
}

/// Tag a single file.
#[tauri::command]
pub fn tag_file(file_id: i64, tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .tag_file(file_id, tag_id)
        .map_err(|e| e.to_string())
}

/// Tag multiple files.
#[tauri::command]
pub fn tag_files(
    file_ids: Vec<i64>,
    tag_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .tag_files(&file_ids, tag_id)
        .map_err(|e| e.to_string())
}

/// Untag a file.
#[tauri::command]
pub fn untag_file(file_id: i64, tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .untag_file(file_id, tag_id)
        .map_err(|e| e.to_string())
}

/// Untag multiple files.
#[tauri::command]
pub fn untag_files(
    file_ids: Vec<i64>,
    tag_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .untag_files(&file_ids, tag_id)
        .map_err(|e| e.to_string())
}

/// Get tags attached to a file.
#[tauri::command]
pub fn get_file_tags(file_id: i64, state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    db(&state)?
        .get_file_tags(file_id)
        .map_err(|e| e.to_string())
}

/// List files with a specific tag.
#[tauri::command]
pub fn list_files_by_tag(
    tag_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?
        .list_files_by_tag(tag_id)
        .map_err(|e| e.to_string())
}

// --- Favorites & NSFW ---

/// Set favorite status for a single file.
#[tauri::command]
pub fn set_file_favorite(
    file_id: i64,
    is_favorite: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .set_file_favorite(file_id, is_favorite)
        .map_err(|e| e.to_string())
}

/// Set favorite status for multiple files.
#[tauri::command]
pub fn set_files_favorite(
    file_ids: Vec<i64>,
    is_favorite: bool,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .set_files_favorite(&file_ids, is_favorite)
        .map_err(|e| e.to_string())
}

/// Set NSFW status for a single file.
#[tauri::command]
pub fn set_file_nsfw(
    file_id: i64,
    is_nsfw: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .set_file_nsfw(file_id, is_nsfw)
        .map_err(|e| e.to_string())
}

/// Set NSFW status for multiple files.
#[tauri::command]
pub fn set_files_nsfw(
    file_ids: Vec<i64>,
    is_nsfw: bool,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .set_files_nsfw(&file_ids, is_nsfw)
        .map_err(|e| e.to_string())
}

// --- Prompt Stats ---

/// Get frequency statistics for prompt tags.
#[tauri::command]
pub fn get_prompt_stats(
    is_negative: bool,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<PromptStat>, String> {
    db(&state)?
        .get_prompt_stats(is_negative, limit)
        .map_err(|e| e.to_string())
}

/// Aggregated file counts across the library and per folder.
#[derive(Serialize)]
pub struct LibraryCounts {
    pub total: i64,
    pub folders: HashMap<i64, i64>,
}

/// Get file counts per folder plus total indexed files across all folders.
#[tauri::command]
pub fn get_library_counts(state: State<'_, AppState>) -> Result<LibraryCounts, String> {
    let db = db(&state)?;
    let total = db.count_all_files().map_err(|e| e.to_string())?;
    let folders = db.get_folder_file_counts().map_err(|e| e.to_string())?;
    Ok(LibraryCounts { total, folders })
}

/// Scan a folder on a blocking thread, emitting `scan-progress` events as it
/// goes. Returns the final [`ScanStats`].
///
/// The scanner opens its own database connection, so the shell's connection
/// (used by read commands) is never blocked by a long scan.
#[tauri::command]
pub async fn scan_folder(
    app: AppHandle,
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<ScanStats, String> {
    run_scan(app, folder_id, state, false).await
}

/// Re-extract metadata from every file in a folder, ignoring the incremental
/// cache. Use after a metadata-parser update so already-indexed files get the
/// current extractor's output.
#[tauri::command]
pub async fn rebuild_metadata(
    app: AppHandle,
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<ScanStats, String> {
    run_scan(app, folder_id, state, true).await
}

/// Shared scan runner: snapshots the folder path and database path while the
/// lock is held, then spawns a [`Scanner`] on a blocking thread. `forced` makes
/// the scanner bypass the incremental cache (used by `rebuild_metadata`).
async fn run_scan(
    app: AppHandle,
    folder_id: i64,
    state: State<'_, AppState>,
    forced: bool,
) -> Result<ScanStats, String> {
    // Snapshot the folder path and database path while the lock is held, then
    // release it before spawning the scan thread.
    let (root, db_path) = {
        let db = db(&state)?;
        let folder = db
            .list_folders()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.id == folder_id)
            .ok_or_else(|| format!("no folder with id {folder_id}"))?;
        let db_path = db
            .path()
            .ok_or_else(|| "in-memory database cannot be scanned".to_string())?
            .to_path_buf();
        (folder.path, db_path)
    };

    let scanner = if forced {
        Scanner::with_forced_extractor(db_path)
    } else {
        Scanner::with_default_extractor(db_path)
    };
    tauri::async_runtime::spawn_blocking(move || {
        scanner
            .scan_folder(folder_id, Path::new(&root), |progress| {
                let _ = app.emit("scan-progress", progress);
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
