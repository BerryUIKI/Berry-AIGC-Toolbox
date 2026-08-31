//! Tauri IPC commands exposed to the frontend.
//!
//! Each command is a thin wrapper: parse the request, call into the core
//! crates, and serialize the result. Business logic lives in the `berry-*`
//! crates, not here.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;

use berry_domain::{
    Album, CheckpointModelStat, DatabaseStats, FileSortField, Folder, ImageFile, ModelCacheEntry,
    PromptStat, SearchCriteria, SortDirection, Tag,
};
use berry_scan::{ScanStats, Scanner};
use berry_storage::Database;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

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
pub async fn list_files(
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?.list_files(folder_id).map_err(|e| e.to_string())
}

/// Query indexed files with optional folder filtering and multi-criteria sorting.
#[tauri::command]
pub async fn query_files(
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
pub async fn search_files(
    criteria: SearchCriteria,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?
        .search_files(&criteria)
        .map_err(|e| e.to_string())
}

/// Search indexed files using a parsed query string.
#[tauri::command]
pub async fn search_files_by_query(
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

// --- Checkpoints and Model Cache ---

/// Get list of indexed checkpoint models and their occurrence counts.
#[tauri::command]
pub fn get_checkpoint_models(
    state: State<'_, AppState>,
) -> Result<Vec<CheckpointModelStat>, String> {
    db(&state)?
        .get_checkpoint_models()
        .map_err(|e| e.to_string())
}

/// Import A1111 cache.json or custom model hash mapping JSON file.
#[tauri::command]
pub fn import_model_cache_file(path: String, state: State<'_, AppState>) -> Result<usize, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read cache file at {path}: {e}"))?;

    let root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse JSON in cache file: {e}"))?;

    let mut entries = Vec::new();

    if let Some(obj) = root.as_object() {
        for (key, val) in obj {
            if let Some(item_obj) = val.as_object() {
                // A1111 format: key = "checkpoint/name [hash]", val = { "model_name": ..., "hash": ..., "sha256": ... }
                let hash = item_obj
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        item_obj
                            .get("hashes")
                            .and_then(|h| h.get("SHA256"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or(key.as_str());

                let name = item_obj
                    .get("model_name")
                    .or_else(|| item_obj.get("filename"))
                    .or_else(|| item_obj.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(key.as_str());

                let title = item_obj
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let sha256 = item_obj
                    .get("sha256")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        item_obj
                            .get("hashes")
                            .and_then(|h| h.get("SHA256"))
                            .and_then(|v| v.as_str())
                    })
                    .map(|s| s.to_string());

                entries.push(ModelCacheEntry {
                    hash: hash.to_string(),
                    name: name.to_string(),
                    title,
                    sha256,
                });
            } else if let Some(str_val) = val.as_str() {
                // Simple { "hash": "model_name" } map
                entries.push(ModelCacheEntry {
                    hash: key.clone(),
                    name: str_val.to_string(),
                    title: None,
                    sha256: if key.len() == 64 {
                        Some(key.clone())
                    } else {
                        None
                    },
                });
            }
        }
    } else if let Some(arr) = root.as_array() {
        for item in arr {
            if let Ok(entry) = serde_json::from_value::<ModelCacheEntry>(item.clone()) {
                entries.push(entry);
            }
        }
    }

    db(&state)?
        .import_model_cache(&entries)
        .map_err(|e| e.to_string())
}

/// Resolve a model name from its short hash or SHA256.
#[tauri::command]
pub fn resolve_model_hash(
    hash: String,
    state: State<'_, AppState>,
) -> Result<Option<ModelCacheEntry>, String> {
    db(&state)?
        .resolve_model_hash(&hash)
        .map_err(|e| e.to_string())
}

/// List all entries in model cache.
#[tauri::command]
pub fn list_model_cache(state: State<'_, AppState>) -> Result<Vec<ModelCacheEntry>, String> {
    db(&state)?.list_model_cache().map_err(|e| e.to_string())
}

// --- File Operations & Drag-and-Drop ---

/// Move files and their sidecars to a target indexed folder, updating database paths.
#[tauri::command]
pub fn move_files(
    file_paths: Vec<String>,
    target_folder_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let target_folder = {
        let database = db(&state)?;
        database
            .list_folders()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.id == target_folder_id)
            .ok_or_else(|| format!("Target folder {target_folder_id} not found"))?
    };

    let target_dir = Path::new(&target_folder.path);
    let mut moved_count = 0;

    let database = db(&state)?;
    for src_str in file_paths {
        let src_path = Path::new(&src_str);
        if !src_path.exists() {
            continue;
        }
        let file_name = match src_path.file_name() {
            Some(name) => name,
            None => continue,
        };
        let dest_path = target_dir.join(file_name);
        if dest_path == src_path {
            continue;
        }

        // Rename main file
        if std::fs::rename(src_path, &dest_path).is_err() {
            // Cross-device fallback: copy then remove
            std::fs::copy(src_path, &dest_path)
                .map_err(|e| format!("Failed to move file to {}: {e}", dest_path.display()))?;
            std::fs::remove_file(src_path).map_err(|e| {
                format!(
                    "Copied to {} but failed to remove source file {}: {e}",
                    dest_path.display(),
                    src_path.display()
                )
            })?;
        }

        // Check and move sibling sidecars (.txt, .json)
        let sidecar_txt = src_path.with_extension("txt");
        if sidecar_txt.exists() {
            let dest_txt = dest_path.with_extension("txt");
            let _ = std::fs::rename(&sidecar_txt, &dest_txt).or_else(|_| {
                std::fs::copy(&sidecar_txt, &dest_txt)
                    .and_then(|_| std::fs::remove_file(&sidecar_txt))
            });
        }

        // Update database record
        let new_path_str = dest_path.to_string_lossy().to_string();
        let _ = database.move_file_record(&src_str, &new_path_str, target_folder_id);
        moved_count += 1;
    }

    Ok(moved_count)
}

/// Copy files and their sidecars to a target indexed folder, inserting new database rows.
#[tauri::command]
pub fn copy_files(
    file_paths: Vec<String>,
    target_folder_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let target_folder = {
        let database = db(&state)?;
        database
            .list_folders()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.id == target_folder_id)
            .ok_or_else(|| format!("Target folder {target_folder_id} not found"))?
    };

    let target_dir = Path::new(&target_folder.path);
    let mut copied_count = 0;

    let database = db(&state)?;
    for src_str in file_paths {
        let src_path = Path::new(&src_str);
        if !src_path.exists() {
            continue;
        }
        let file_name = match src_path.file_name() {
            Some(name) => name,
            None => continue,
        };
        let dest_path = target_dir.join(file_name);
        if dest_path == src_path {
            continue;
        }

        // Copy main file
        std::fs::copy(src_path, &dest_path)
            .map_err(|e| format!("Failed to copy file to {}: {e}", dest_path.display()))?;

        // Copy sidecars if present
        let sidecar_txt = src_path.with_extension("txt");
        if sidecar_txt.exists() {
            let dest_txt = dest_path.with_extension("txt");
            let _ = std::fs::copy(&sidecar_txt, &dest_txt);
        }

        // Copy database record with new path
        if let Ok(Some(mut orig)) = database.get_file_by_path(&src_str) {
            orig.id = None;
            orig.folder_id = target_folder_id;
            orig.path = dest_path.to_string_lossy().to_string();
            let _ = database.upsert_file(&orig);
        }
        copied_count += 1;
    }

    Ok(copied_count)
}

/// Safely move files and sidecars to the system Trash / Recycle Bin and remove from DB.
#[tauri::command]
pub fn trash_files(file_paths: Vec<String>, state: State<'_, AppState>) -> Result<usize, String> {
    let mut trashed_count = 0;
    let database = db(&state)?;

    for path_str in file_paths {
        let path = Path::new(&path_str);
        if path.exists() {
            if let Err(e) = trash::delete(path) {
                // If trash fails (e.g. headless/external), fallback to permanent remove
                std::fs::remove_file(path).map_err(|rem_e| {
                    format!(
                        "Failed to trash or remove {}: trash error: {e}, remove error: {rem_e}",
                        path.display()
                    )
                })?;
            }
            let sidecar_txt = path.with_extension("txt");
            if sidecar_txt.exists() {
                let _ = trash::delete(&sidecar_txt).or_else(|_| std::fs::remove_file(&sidecar_txt));
            }
        }
        let _ = database.delete_file_by_path(&path_str);
        trashed_count += 1;
    }

    Ok(trashed_count)
}

/// Reveal the selected file in the system file manager (Finder / Explorer / Files).
#[tauri::command]
pub fn reveal_in_file_manager(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("File does not exist: {path}"));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {e}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        if path.contains('"') {
            return Err("Path contains invalid characters for Explorer".to_string());
        }
        std::process::Command::new("explorer")
            .arg(format!("/select,\"{path}\""))
            .spawn()
            .map_err(|e| format!("Failed to open Explorer: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent = p.parent().unwrap_or(p);
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {e}"))?;
    }

    Ok(())
}

// --- Database Maintenance ---

/// Run SQLite VACUUM and optimize to compact database and reclaim unused disk pages.
#[tauri::command]
pub fn vacuum_database(state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?.vacuum_database().map_err(|e| e.to_string())
}

/// Backup the current database to a designated file destination via VACUUM INTO.
#[tauri::command]
pub fn backup_database(destination_path: String, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .backup_database(&destination_path)
        .map_err(|e| e.to_string())
}

/// Retrieve database storage and table statistics.
#[tauri::command]
pub fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    db(&state)?.get_database_stats().map_err(|e| e.to_string())
}

/// Restore database from a backup file, verifying integrity and reloading connection.
#[tauri::command]
pub fn restore_database(
    source_path: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri::Manager;
    let src = Path::new(&source_path);
    if !src.exists() {
        return Err(format!("Backup source file does not exist: {source_path}"));
    }

    // Verify backup database is valid and readable
    let _test_db =
        Database::connect(src).map_err(|e| format!("Invalid backup SQLite file: {e}"))?;

    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let active_db_path = data_dir.join("berry.db");

    // Release connection on active database file to avoid Windows lock collision
    {
        let mut guard = state
            .db
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        *guard = Database::connect_in_memory()
            .map_err(|e| format!("Failed to create temporary DB: {e}"))?;
    }

    // Copy backup to active database location
    std::fs::copy(src, &active_db_path)
        .map_err(|e| format!("Failed to copy backup database to active location: {e}"))?;

    // Reopen database connection in AppState
    let new_db = Database::connect(&active_db_path)
        .map_err(|e| format!("Failed to reconnect restored database: {e}"))?;
    let mut guard = state
        .db
        .lock()
        .map_err(|_| "Database lock poisoned".to_string())?;
    *guard = new_db;

    Ok(())
}

/// Open an external URL in the system's default browser.
#[tauri::command]
pub fn open_external_url(url: String, app_handle: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("Refusing to open non-HTTP(S) URL: {url}"));
    }
    app_handle
        .opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {e}"))?;
    Ok(())
}

/// Request a single thumbnail (lazy on-demand generation).
#[tauri::command]
pub async fn get_or_create_thumbnail(
    app_handle: AppHandle,
    file_id: i64,
    file_path: String,
    modified_at: i64,
    max_edge: Option<u32>,
) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let max_edge = max_edge.unwrap_or(384);
    tauri::async_runtime::spawn_blocking(move || {
        berry_scan::ensure_thumbnail(&data_dir, file_id, &file_path, modified_at, max_edge)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Deserialize)]
pub struct BatchThumbnailItem {
    pub file_id: i64,
    pub file_path: String,
    pub modified_at: i64,
}

/// Request background batch thumbnail generation with progress event emission.
#[tauri::command]
pub async fn batch_generate_thumbnails(
    app_handle: AppHandle,
    items: Vec<BatchThumbnailItem>,
    max_edge: Option<u32>,
) -> Result<usize, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let max_edge = max_edge.unwrap_or(384);
    let total = items.len();
    let tuples: Vec<(i64, String, i64)> = items
        .into_iter()
        .map(|i| (i.file_id, i.file_path, i.modified_at))
        .collect();

    let app_clone = app_handle.clone();
    let count = tauri::async_runtime::spawn_blocking(move || {
        berry_scan::batch_generate_thumbnails(
            &data_dir,
            tuples,
            max_edge,
            Some(move |current: usize, total: usize| {
                let _ = app_clone.emit(
                    "thumbnail-progress",
                    berry_scan::ThumbnailProgress {
                        current,
                        total,
                        done: current >= total,
                    },
                );
            }),
        )
    })
    .await
    .map_err(|e| format!("Thumbnail generation task failed: {e}"))?;

    let _ = app_handle.emit(
        "thumbnail-progress",
        berry_scan::ThumbnailProgress {
            current: total,
            total,
            done: true,
        },
    );

    Ok(count)
}

/// Get stats for thumbnail cache on disk.
#[tauri::command]
pub fn get_thumbnail_cache_stats(
    app_handle: AppHandle,
) -> Result<berry_scan::ThumbnailCacheStats, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    berry_scan::get_thumbnail_cache_stats(&data_dir)
}

/// Clear thumbnail cache files from disk.
#[tauri::command]
pub fn clear_thumbnail_cache(app_handle: AppHandle) -> Result<usize, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    berry_scan::clear_thumbnail_cache(&data_dir)
}
