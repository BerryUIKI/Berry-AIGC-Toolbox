//! AIGC pipeline harvesting and deferred cleanup services.
//!
//! Provides reusable orchestration for watching local AI output folders (e.g. ComfyUI,
//! SD WebUI, Fooocus), copying harvested media files with sidecars, extracting metadata,
//! upserting library records, and safely managing delayed source cleanups with source
//! and destination revalidation.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use omera_domain::{CleanupQueueItem, Container, ImageFile, TransformCollisionPolicy};
use omera_storage::Database;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors arising during pipeline harvesting or cleanup processing.
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Folder not found: {0}")]
    FolderNotFound(i64),
    #[error("Folder is not an ingestion pipeline: {0}")]
    NotPipelineFolder(i64),
    #[error("Pipeline folder has no source path configured")]
    MissingSourcePath,
    #[error("Pipeline source path does not exist: {0}")]
    SourcePathNotFound(String),
    #[error("Pipeline source path is not a directory: {0}")]
    SourcePathNotDirectory(String),
    #[error("Pipeline destination directory creation failed: {0}")]
    DestinationDirCreate(String),
    #[error("Pipeline source directory read failed: {0}")]
    SourceDirRead(String),
    #[error("Database error: {0}")]
    Database(#[from] omera_storage::DatabaseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Pipeline operation cancelled")]
    Cancelled,
}

/// Collision resolution policy for pipeline harvesting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PipelineCollisionPolicy {
    /// Skip if target file exists with identical file size; otherwise overwrite (legacy behavior).
    #[default]
    SkipIdentical,
    /// Generate a non-colliding filename (e.g. `image_1.png`) if target exists.
    Rename,
    /// Skip copying whenever a target file exists, regardless of size.
    SkipAlways,
    /// Overwrite existing target file unconditionally.
    Overwrite,
}

/// Configuration options for harvesting a pipeline folder.
#[derive(Debug, Clone, Default)]
pub struct PipelineHarvestOptions {
    /// Collision resolution policy. Defaults to `PipelineCollisionPolicy::SkipIdentical`.
    pub collision_policy: PipelineCollisionPolicy,
    /// Debounce threshold in seconds. Defaults to 1 second.
    pub debounce_seconds: Option<u64>,
}

/// Structured per-item outcome of harvesting a pipeline file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineHarvestItemOutcome {
    Harvested {
        source_path: String,
        target_path: String,
        file_id: i64,
        cleanup_enqueued: bool,
    },
    SkippedIdentical {
        source_path: String,
        target_path: String,
    },
    SkippedCollision {
        source_path: String,
        target_path: String,
    },
    SkippedDebounce {
        source_path: String,
        modified_at: i64,
    },
    SkippedEmpty {
        source_path: String,
    },
    SkippedUnsupported {
        source_path: String,
    },
    FailedCopy {
        source_path: String,
        target_path: String,
        error: String,
    },
    FailedDatabase {
        source_path: String,
        target_path: String,
        error: String,
    },
}

/// Summary report of a pipeline folder harvesting run.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PipelineHarvestReport {
    pub harvested_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub cancelled: bool,
    pub items: Vec<PipelineHarvestItemOutcome>,
}

/// Structured per-item outcome of processing a deferred pipeline cleanup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineCleanupItemOutcome {
    Deleted {
        queue_id: i64,
        source_path: String,
        target_file_id: i64,
    },
    SkippedAlreadyRemoved {
        queue_id: i64,
        source_path: String,
    },
    FailedDestinationMissing {
        queue_id: i64,
        source_path: String,
        target_file_id: i64,
        reason: String,
    },
    FailedSourceInvalid {
        queue_id: i64,
        source_path: String,
        reason: String,
    },
    FailedTrash {
        queue_id: i64,
        source_path: String,
        error: String,
    },
}

/// Summary report of deferred pipeline cleanups processed.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PipelineCleanupReport {
    pub deleted_count: u64,
    pub failed_count: u64,
    pub cancelled: bool,
    pub items: Vec<PipelineCleanupItemOutcome>,
}

/// Harvests new media from a configured pipeline folder into Omera.
pub fn harvest_pipeline_folder(
    db: &Database,
    folder_id: i64,
    options: Option<PipelineHarvestOptions>,
) -> Result<PipelineHarvestReport, PipelineError> {
    harvest_pipeline_folder_with_cancellation(db, folder_id, options, None, None)
}

/// Harvests new media from a configured pipeline folder with cancellation and progress reporting.
pub fn harvest_pipeline_folder_with_cancellation(
    db: &Database,
    folder_id: i64,
    options: Option<PipelineHarvestOptions>,
    cancellation_token: Option<&AtomicBool>,
    progress_callback: Option<&dyn Fn(&PipelineHarvestItemOutcome)>,
) -> Result<PipelineHarvestReport, PipelineError> {
    let folder = db
        .find_folder_by_id(folder_id)?
        .ok_or(PipelineError::FolderNotFound(folder_id))?;

    if folder.folder_type != "pipeline" {
        return Err(PipelineError::NotPipelineFolder(folder_id));
    }

    let source_path_str = match &folder.source_path {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return Err(PipelineError::MissingSourcePath),
    };

    let source_dir = Path::new(&source_path_str);
    if !source_dir.exists() {
        return Err(PipelineError::SourcePathNotFound(source_path_str));
    }
    if !source_dir.is_dir() {
        return Err(PipelineError::SourcePathNotDirectory(source_path_str));
    }

    let dest_dir = Path::new(&folder.path);
    std::fs::create_dir_all(dest_dir)
        .map_err(|e| PipelineError::DestinationDirCreate(e.to_string()))?;

    let supported_exts = ["png", "jpg", "jpeg", "webp", "mp4"];
    let mut report = PipelineHarvestReport::default();

    let entries =
        std::fs::read_dir(source_dir).map_err(|e| PipelineError::SourceDirRead(e.to_string()))?;

    let now_ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let debounce_seconds = options
        .as_ref()
        .and_then(|o| o.debounce_seconds)
        .unwrap_or(1) as i64;
    let collision_policy = options
        .as_ref()
        .map(|o| o.collision_policy)
        .unwrap_or_default();

    if let Some(token) = cancellation_token {
        if token.load(Ordering::Relaxed) {
            report.cancelled = true;
            return Ok(report);
        }
    }

    for entry in entries.flatten() {
        if let Some(token) = cancellation_token {
            if token.load(Ordering::Relaxed) {
                report.cancelled = true;
                break;
            }
        }

        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let src_str = file_path.display().to_string();

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !supported_exts.contains(&ext.as_str()) {
            let outcome = PipelineHarvestItemOutcome::SkippedUnsupported {
                source_path: src_str,
            };
            report.skipped_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        let meta = match file_path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.len() == 0 {
            let outcome = PipelineHarvestItemOutcome::SkippedEmpty {
                source_path: src_str,
            };
            report.skipped_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        let file_mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if debounce_seconds > 0 && now_ts.saturating_sub(file_mtime) < debounce_seconds {
            let outcome = PipelineHarvestItemOutcome::SkippedDebounce {
                source_path: src_str,
                modified_at: file_mtime,
            };
            report.skipped_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        let file_name = match file_path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        let target_path = match collision_policy {
            PipelineCollisionPolicy::Rename => {
                let file_stem = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("file");
                match crate::transform::resolve_publication_path(
                    dest_dir,
                    file_stem,
                    &ext,
                    TransformCollisionPolicy::Rename,
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        let outcome = PipelineHarvestItemOutcome::FailedCopy {
                            source_path: src_str,
                            target_path: dest_dir.join(&file_name).display().to_string(),
                            error: format!("Collision resolution failed: {e}"),
                        };
                        report.failed_count += 1;
                        if let Some(cb) = progress_callback {
                            cb(&outcome);
                        }
                        report.items.push(outcome);
                        continue;
                    }
                }
            }
            PipelineCollisionPolicy::SkipAlways => {
                let candidate = dest_dir.join(&file_name);
                if candidate.exists() {
                    let outcome = PipelineHarvestItemOutcome::SkippedCollision {
                        source_path: src_str,
                        target_path: candidate.display().to_string(),
                    };
                    report.skipped_count += 1;
                    if let Some(cb) = progress_callback {
                        cb(&outcome);
                    }
                    report.items.push(outcome);
                    continue;
                }
                candidate
            }
            PipelineCollisionPolicy::SkipIdentical => {
                let candidate = dest_dir.join(&file_name);
                if candidate.exists() {
                    if let Ok(t_meta) = candidate.metadata() {
                        if t_meta.len() == meta.len() {
                            let outcome = PipelineHarvestItemOutcome::SkippedIdentical {
                                source_path: src_str,
                                target_path: candidate.display().to_string(),
                            };
                            report.skipped_count += 1;
                            if let Some(cb) = progress_callback {
                                cb(&outcome);
                            }
                            report.items.push(outcome);
                            continue;
                        }
                    }
                }
                candidate
            }
            PipelineCollisionPolicy::Overwrite => dest_dir.join(&file_name),
        };

        let tgt_str = target_path.display().to_string();

        if let Err(e) = std::fs::copy(&file_path, &target_path) {
            let outcome = PipelineHarvestItemOutcome::FailedCopy {
                source_path: src_str,
                target_path: tgt_str,
                error: format!("Failed to copy file: {e}"),
            };
            report.failed_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        // Sidecar preservation: .txt and .json
        let src_txt = file_path.with_extension("txt");
        let tgt_txt = target_path.with_extension("txt");
        let mut copied_txt = false;
        if src_txt.is_file() && std::fs::copy(&src_txt, &tgt_txt).is_ok() {
            copied_txt = true;
        }

        let src_json = file_path.with_extension("json");
        let tgt_json = target_path.with_extension("json");
        let mut copied_json = false;
        if src_json.is_file() && std::fs::copy(&src_json, &tgt_json).is_ok() {
            copied_json = true;
        }

        let container = match ext.as_str() {
            "png" => Container::Png,
            "jpg" | "jpeg" => Container::Jpeg,
            "webp" => Container::WebP,
            "mp4" => Container::Mp4,
            _ => continue,
        };

        let metadata = omera_metadata::extract_metadata(container, &target_path);

        let image_file = ImageFile {
            id: None,
            folder_id,
            path: tgt_str.clone(),
            size_bytes: meta.len(),
            modified_at: file_mtime,
            container,
            metadata,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let inserted_id = match db.upsert_file(&image_file) {
            Ok(id) => id,
            Err(e) => {
                // Compensation: remove target file and copied sidecars
                let _ = std::fs::remove_file(&target_path);
                if copied_txt {
                    let _ = std::fs::remove_file(&tgt_txt);
                }
                if copied_json {
                    let _ = std::fs::remove_file(&tgt_json);
                }
                let outcome = PipelineHarvestItemOutcome::FailedDatabase {
                    source_path: src_str,
                    target_path: tgt_str,
                    error: format!("Database upsert error: {e}"),
                };
                report.failed_count += 1;
                if let Some(cb) = progress_callback {
                    cb(&outcome);
                }
                report.items.push(outcome);
                continue;
            }
        };

        let mut cleanup_enqueued = false;
        if folder.ingest_action.as_deref() == Some("move") {
            let grace = folder.grace_period_hours.unwrap_or(24);
            if grace <= 0 {
                let _ = trash::delete(&file_path);
                if src_txt.exists() {
                    let _ = trash::delete(&src_txt);
                }
                if src_json.exists() {
                    let _ = trash::delete(&src_json);
                }
            } else if let Err(e) = db.enqueue_cleanup(&src_str, inserted_id, grace) {
                eprintln!("Failed to enqueue cleanup for {src_str}: {e}");
            } else {
                cleanup_enqueued = true;
            }
        }

        let outcome = PipelineHarvestItemOutcome::Harvested {
            source_path: src_str,
            target_path: tgt_str,
            file_id: inserted_id,
            cleanup_enqueued,
        };
        report.harvested_count += 1;
        if let Some(cb) = progress_callback {
            cb(&outcome);
        }
        report.items.push(outcome);
    }

    Ok(report)
}

/// Processes scheduled pipeline cleanups due at the current system time.
pub fn process_pipeline_cleanups(
    db: &Database,
    cancellation_token: Option<&AtomicBool>,
) -> Result<PipelineCleanupReport, PipelineError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    process_pipeline_cleanups_at(db, now, cancellation_token, None)
}

/// Processes scheduled pipeline cleanups due at a specific unix timestamp,
/// enforcing destination and source revalidation before moving any file to trash.
pub fn process_pipeline_cleanups_at(
    db: &Database,
    now_timestamp: i64,
    cancellation_token: Option<&AtomicBool>,
    progress_callback: Option<&dyn Fn(&PipelineCleanupItemOutcome)>,
) -> Result<PipelineCleanupReport, PipelineError> {
    let due_items = db.list_due_cleanups(now_timestamp)?;
    let mut report = PipelineCleanupReport::default();

    if let Some(token) = cancellation_token {
        if token.load(Ordering::Relaxed) {
            report.cancelled = true;
            return Ok(report);
        }
    }

    for item in due_items {
        if let Some(token) = cancellation_token {
            if token.load(Ordering::Relaxed) {
                report.cancelled = true;
                break;
            }
        }

        // 1. Destination revalidation:
        // Must exist in DB
        let dest_file = match db.get_file_by_id(item.target_image_id) {
            Ok(Some(f)) => f,
            Ok(None) => {
                let outcome = PipelineCleanupItemOutcome::FailedDestinationMissing {
                    queue_id: item.id,
                    source_path: item.source_file_path.clone(),
                    target_file_id: item.target_image_id,
                    reason: "Target image not found in database".to_string(),
                };
                let _ = db.update_cleanup_status(item.id, "failed");
                report.failed_count += 1;
                if let Some(cb) = progress_callback {
                    cb(&outcome);
                }
                report.items.push(outcome);
                continue;
            }
            Err(e) => {
                let outcome = PipelineCleanupItemOutcome::FailedDestinationMissing {
                    queue_id: item.id,
                    source_path: item.source_file_path.clone(),
                    target_file_id: item.target_image_id,
                    reason: format!("Database query error: {e}"),
                };
                let _ = db.update_cleanup_status(item.id, "failed");
                report.failed_count += 1;
                if let Some(cb) = progress_callback {
                    cb(&outcome);
                }
                report.items.push(outcome);
                continue;
            }
        };

        // Destination file must exist on disk and be non-empty
        let dest_path = Path::new(&dest_file.path);
        if !dest_path.is_file() {
            let outcome = PipelineCleanupItemOutcome::FailedDestinationMissing {
                queue_id: item.id,
                source_path: item.source_file_path.clone(),
                target_file_id: item.target_image_id,
                reason: format!(
                    "Destination file does not exist on disk: {}",
                    dest_file.path
                ),
            };
            let _ = db.update_cleanup_status(item.id, "failed");
            report.failed_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        if let Ok(meta) = dest_path.metadata() {
            if meta.len() == 0 {
                let outcome = PipelineCleanupItemOutcome::FailedDestinationMissing {
                    queue_id: item.id,
                    source_path: item.source_file_path.clone(),
                    target_file_id: item.target_image_id,
                    reason: "Destination file is empty (0 bytes)".to_string(),
                };
                let _ = db.update_cleanup_status(item.id, "failed");
                report.failed_count += 1;
                if let Some(cb) = progress_callback {
                    cb(&outcome);
                }
                report.items.push(outcome);
                continue;
            }
        }

        // 2. Source revalidation:
        let src_p = Path::new(&item.source_file_path);
        if item.source_file_path.trim().is_empty() {
            let outcome = PipelineCleanupItemOutcome::FailedSourceInvalid {
                queue_id: item.id,
                source_path: item.source_file_path.clone(),
                reason: "Source file path is empty".to_string(),
            };
            let _ = db.update_cleanup_status(item.id, "failed");
            report.failed_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        // Never delete directory / root media paths
        if src_p.is_dir() {
            let outcome = PipelineCleanupItemOutcome::FailedSourceInvalid {
                queue_id: item.id,
                source_path: item.source_file_path.clone(),
                reason: "Source path is a directory; refusing to delete directories".to_string(),
            };
            let _ = db.update_cleanup_status(item.id, "failed");
            report.failed_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        // Refuse if source is identical to destination
        if src_p == dest_path {
            let outcome = PipelineCleanupItemOutcome::FailedSourceInvalid {
                queue_id: item.id,
                source_path: item.source_file_path.clone(),
                reason: "Source path is identical to destination path; refusing to self-delete"
                    .to_string(),
            };
            let _ = db.update_cleanup_status(item.id, "failed");
            report.failed_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        // If source file already does not exist on disk, mark resolved
        if !src_p.exists() {
            let outcome = PipelineCleanupItemOutcome::SkippedAlreadyRemoved {
                queue_id: item.id,
                source_path: item.source_file_path.clone(),
            };
            let _ = db.update_cleanup_status(item.id, "deleted");
            report.deleted_count += 1;
            if let Some(cb) = progress_callback {
                cb(&outcome);
            }
            report.items.push(outcome);
            continue;
        }

        // 3. Move to trash safely:
        match trash::delete(src_p) {
            Ok(()) => {
                let txt = src_p.with_extension("txt");
                if txt.exists() {
                    let _ = trash::delete(&txt);
                }
                let json = src_p.with_extension("json");
                if json.exists() {
                    let _ = trash::delete(&json);
                }
                let _ = db.update_cleanup_status(item.id, "deleted");
                report.deleted_count += 1;
                let outcome = PipelineCleanupItemOutcome::Deleted {
                    queue_id: item.id,
                    source_path: item.source_file_path.clone(),
                    target_file_id: item.target_image_id,
                };
                if let Some(cb) = progress_callback {
                    cb(&outcome);
                }
                report.items.push(outcome);
            }
            Err(e) => {
                let outcome = PipelineCleanupItemOutcome::FailedTrash {
                    queue_id: item.id,
                    source_path: item.source_file_path.clone(),
                    error: e.to_string(),
                };
                let _ = db.update_cleanup_status(item.id, "failed");
                report.failed_count += 1;
                if let Some(cb) = progress_callback {
                    cb(&outcome);
                }
                report.items.push(outcome);
            }
        }
    }

    Ok(report)
}

/// Retrieves the current pipeline cleanup queue items with an optional limit.
pub fn get_pipeline_cleanup_queue(
    db: &Database,
    limit: Option<usize>,
) -> Result<Vec<CleanupQueueItem>, PipelineError> {
    db.get_cleanup_queue(limit.unwrap_or(50))
        .map_err(PipelineError::Database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_dummy_png(path: &Path) {
        let png_bytes: [u8; 67] = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        std::fs::write(path, png_bytes).unwrap();
    }

    #[test]
    fn test_harvest_validation_errors() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        // 1. Folder not found
        let err = harvest_pipeline_folder(&db, 9999, None).unwrap_err();
        assert!(matches!(err, PipelineError::FolderNotFound(9999)));

        // 2. Folder not pipeline mode (e.g. "link")
        let link_folder = db
            .add_folder_with_mode(
                &tmp.path().to_string_lossy(),
                "link",
                None,
                None,
                None,
                false,
            )
            .unwrap();
        let err = harvest_pipeline_folder(&db, link_folder.id, None).unwrap_err();
        assert!(matches!(err, PipelineError::NotPipelineFolder(_)));

        // 3. Pipeline folder with missing source path
        let dest_dir1 = tmp.path().join("dest1");
        std::fs::create_dir_all(&dest_dir1).unwrap();
        let pipe_missing_src = db
            .add_folder_with_mode(
                &dest_dir1.to_string_lossy(),
                "pipeline",
                None,
                Some("copy"),
                None,
                true,
            )
            .unwrap();
        let err = harvest_pipeline_folder(&db, pipe_missing_src.id, None).unwrap_err();
        assert!(matches!(err, PipelineError::MissingSourcePath));

        // 4. Pipeline folder with non-existent source path
        let dest_dir2 = tmp.path().join("dest2");
        std::fs::create_dir_all(&dest_dir2).unwrap();
        let non_existent_src = tmp.path().join("does_not_exist");
        let pipe_non_existent = db
            .add_folder_with_mode(
                &dest_dir2.to_string_lossy(),
                "pipeline",
                Some(&non_existent_src.to_string_lossy()),
                Some("copy"),
                None,
                true,
            )
            .unwrap();
        let err = harvest_pipeline_folder(&db, pipe_non_existent.id, None).unwrap_err();
        assert!(matches!(err, PipelineError::SourcePathNotFound(_)));

        // 5. Pipeline folder where source is a file not a directory
        let dest_dir3 = tmp.path().join("dest3");
        std::fs::create_dir_all(&dest_dir3).unwrap();
        let src_file = tmp.path().join("some_file.png");
        write_dummy_png(&src_file);
        let pipe_file_src = db
            .add_folder_with_mode(
                &dest_dir3.to_string_lossy(),
                "pipeline",
                Some(&src_file.to_string_lossy()),
                Some("copy"),
                None,
                true,
            )
            .unwrap();
        let err = harvest_pipeline_folder(&db, pipe_file_src.id, None).unwrap_err();
        assert!(matches!(err, PipelineError::SourcePathNotDirectory(_)));
    }

    #[test]
    fn test_harvest_synthetic_files_with_sidecars_and_filters() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        let src_dir = tmp.path().join("source");
        let dest_dir = tmp.path().join("dest");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&dest_dir).unwrap();

        // 1. Valid image with sidecars (.txt, .json)
        let img1 = src_dir.join("sample1.png");
        write_dummy_png(&img1);
        std::fs::write(src_dir.join("sample1.txt"), "prompt: a cute kitten").unwrap();
        std::fs::write(src_dir.join("sample1.json"), "{\"workflow\": 1}").unwrap();

        // 2. Another valid image without sidecars
        let img2 = src_dir.join("sample2.jpg");
        std::fs::write(&img2, b"fake jpg content for test").unwrap();

        // 3. Unsupported file
        std::fs::write(src_dir.join("notes.txt"), "plain text file").unwrap();

        // 4. Empty 0-byte file
        std::fs::write(src_dir.join("empty.png"), b"").unwrap();

        let folder = db
            .add_folder_with_mode(
                &dest_dir.to_string_lossy(),
                "pipeline",
                Some(&src_dir.to_string_lossy()),
                Some("copy"),
                None,
                true,
            )
            .unwrap();

        let opts = PipelineHarvestOptions {
            collision_policy: PipelineCollisionPolicy::SkipIdentical,
            debounce_seconds: Some(0),
        };

        let report = harvest_pipeline_folder(&db, folder.id, Some(opts)).unwrap();

        assert_eq!(report.harvested_count, 2);
        assert_eq!(report.skipped_count, 4); // sample1.txt, sample1.json, notes.txt (unsupported) + empty.png (empty)
        assert_eq!(report.failed_count, 0);

        // Verify destination has harvested files and sidecars
        assert!(dest_dir.join("sample1.png").exists());
        assert!(dest_dir.join("sample1.txt").exists());
        assert!(dest_dir.join("sample1.json").exists());
        assert!(dest_dir.join("sample2.jpg").exists());
        assert!(!dest_dir.join("notes.txt").exists());
        assert!(!dest_dir.join("empty.png").exists());

        // Verify sidecar contents preserved
        let txt_content = std::fs::read_to_string(dest_dir.join("sample1.txt")).unwrap();
        assert_eq!(txt_content, "prompt: a cute kitten");
        let json_content = std::fs::read_to_string(dest_dir.join("sample1.json")).unwrap();
        assert_eq!(json_content, "{\"workflow\": 1}");

        // Verify DB records
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files
            .iter()
            .any(|f| f.path == dest_dir.join("sample1.png").to_string_lossy()));
        assert!(files
            .iter()
            .any(|f| f.path == dest_dir.join("sample2.jpg").to_string_lossy()));
    }

    #[test]
    fn test_harvest_debounce() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        let src_dir = tmp.path().join("source");
        let dest_dir = tmp.path().join("dest");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&dest_dir).unwrap();

        let img = src_dir.join("recent.png");
        write_dummy_png(&img);

        let folder = db
            .add_folder_with_mode(
                &dest_dir.to_string_lossy(),
                "pipeline",
                Some(&src_dir.to_string_lossy()),
                Some("copy"),
                None,
                true,
            )
            .unwrap();

        // High debounce threshold (100 seconds)
        let opts = PipelineHarvestOptions {
            collision_policy: PipelineCollisionPolicy::SkipIdentical,
            debounce_seconds: Some(100),
        };

        let report = harvest_pipeline_folder(&db, folder.id, Some(opts)).unwrap();
        assert_eq!(report.harvested_count, 0);
        assert_eq!(report.skipped_count, 1);
        assert!(matches!(
            report.items[0],
            PipelineHarvestItemOutcome::SkippedDebounce { .. }
        ));
        assert!(!dest_dir.join("recent.png").exists());
    }

    #[test]
    fn test_harvest_collision_policies() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        let src_dir = tmp.path().join("source");
        let dest_dir = tmp.path().join("dest");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&dest_dir).unwrap();

        let src_img = src_dir.join("item.png");
        write_dummy_png(&src_img);

        let folder = db
            .add_folder_with_mode(
                &dest_dir.to_string_lossy(),
                "pipeline",
                Some(&src_dir.to_string_lossy()),
                Some("copy"),
                None,
                true,
            )
            .unwrap();

        // 1. Initial harvest
        let opts = PipelineHarvestOptions {
            collision_policy: PipelineCollisionPolicy::SkipIdentical,
            debounce_seconds: Some(0),
        };
        let rep1 = harvest_pipeline_folder(&db, folder.id, Some(opts.clone())).unwrap();
        assert_eq!(rep1.harvested_count, 1);

        // 2. Second harvest with identical file -> SkippedIdentical
        let rep2 = harvest_pipeline_folder(&db, folder.id, Some(opts)).unwrap();
        assert_eq!(rep2.harvested_count, 0);
        assert_eq!(rep2.skipped_count, 1);
        assert!(matches!(
            rep2.items[0],
            PipelineHarvestItemOutcome::SkippedIdentical { .. }
        ));

        // 3. Collision with SkipAlways
        let opts_skip = PipelineHarvestOptions {
            collision_policy: PipelineCollisionPolicy::SkipAlways,
            debounce_seconds: Some(0),
        };
        let rep3 = harvest_pipeline_folder(&db, folder.id, Some(opts_skip)).unwrap();
        assert_eq!(rep3.harvested_count, 0);
        assert_eq!(rep3.skipped_count, 1);
        assert!(matches!(
            rep3.items[0],
            PipelineHarvestItemOutcome::SkippedCollision { .. }
        ));

        // 4. Collision with Rename: change source file content/size so size differs
        std::fs::write(&src_img, b"different longer content for rename test").unwrap();
        let opts_rename = PipelineHarvestOptions {
            collision_policy: PipelineCollisionPolicy::Rename,
            debounce_seconds: Some(0),
        };
        let rep4 = harvest_pipeline_folder(&db, folder.id, Some(opts_rename)).unwrap();
        assert_eq!(rep4.harvested_count, 1);
        assert!(dest_dir.join("item.png").exists());
        assert!(dest_dir.join("item_1.png").exists());
    }

    #[test]
    fn test_harvest_move_immediate_vs_delayed() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        let src_dir = tmp.path().join("source");
        let dest_dir = tmp.path().join("dest");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&dest_dir).unwrap();

        // Folder with move and delayed cleanup (grace = 48 hours)
        let folder = db
            .add_folder_with_mode(
                &dest_dir.to_string_lossy(),
                "pipeline",
                Some(&src_dir.to_string_lossy()),
                Some("move"),
                Some(48),
                true,
            )
            .unwrap();

        let src_img = src_dir.join("delayed.png");
        write_dummy_png(&src_img);

        let opts = PipelineHarvestOptions {
            collision_policy: PipelineCollisionPolicy::SkipIdentical,
            debounce_seconds: Some(0),
        };

        let report = harvest_pipeline_folder(&db, folder.id, Some(opts)).unwrap();
        assert_eq!(report.harvested_count, 1);
        match &report.items[0] {
            PipelineHarvestItemOutcome::Harvested {
                cleanup_enqueued, ..
            } => {
                assert!(*cleanup_enqueued);
            }
            _ => panic!("Expected Harvested outcome"),
        }

        // Verify cleanup queue item was added
        let queue = get_pipeline_cleanup_queue(&db, None).unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].source_file_path, src_img.to_string_lossy());
        assert_eq!(queue[0].status, "pending");
    }

    #[test]
    fn test_cleanup_destination_revalidation() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        let src_file = tmp.path().join("source.png");
        write_dummy_png(&src_file);

        let dest_file = tmp.path().join("dest.png");
        write_dummy_png(&dest_file);

        // Case 1: Target image ID does not exist in DB (e.g. dangling reference)
        let folder0 = db
            .add_folder_with_mode(
                &tmp.path().join("f0").to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();
        let dummy_rec = ImageFile {
            id: None,
            folder_id: folder0.id,
            path: dest_file.to_string_lossy().to_string(),
            size_bytes: 67,
            modified_at: 1000,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let file_id_to_drop = db.upsert_file(&dummy_rec).unwrap();
        let q1 = db
            .enqueue_cleanup(&src_file.to_string_lossy(), file_id_to_drop, 0)
            .unwrap();

        // Delete the file with FK checks temporarily bypassed to test dangling reference revalidation
        db.connection()
            .execute("PRAGMA foreign_keys = OFF;", [])
            .unwrap();
        db.connection()
            .execute("DELETE FROM files WHERE id = ?1;", [file_id_to_drop])
            .unwrap();
        db.connection()
            .execute("PRAGMA foreign_keys = ON;", [])
            .unwrap();

        let rep1 = process_pipeline_cleanups_at(&db, i64::MAX, None, None).unwrap();
        assert_eq!(rep1.deleted_count, 0);
        assert_eq!(rep1.failed_count, 1);
        assert!(src_file.exists()); // Source file NOT deleted!
        let item1 = db.get_cleanup_queue(10).unwrap();
        assert_eq!(item1[0].id, q1);
        assert_eq!(item1[0].status, "failed");

        // Case 2: Target image exists in DB, but destination file missing from disk
        let folder = db
            .add_folder_with_mode(
                &tmp.path().to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();

        let missing_dest_path = tmp.path().join("non_existent_dest.png");
        let img_record = ImageFile {
            id: None,
            folder_id: folder.id,
            path: missing_dest_path.to_string_lossy().to_string(),
            size_bytes: 100,
            modified_at: 1000,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let file_id = db.upsert_file(&img_record).unwrap();

        let _q2 = db
            .enqueue_cleanup(&src_file.to_string_lossy(), file_id, 0)
            .unwrap();
        let rep2 = process_pipeline_cleanups_at(&db, i64::MAX, None, None).unwrap();
        assert_eq!(rep2.deleted_count, 0);
        assert_eq!(rep2.failed_count, 1);
        assert!(src_file.exists()); // Source file still NOT deleted!

        // Case 3: Target image exists in DB, but destination file is 0-bytes
        let empty_dest_path = tmp.path().join("empty_dest.png");
        std::fs::write(&empty_dest_path, b"").unwrap();
        let img_empty_record = ImageFile {
            id: None,
            folder_id: folder.id,
            path: empty_dest_path.to_string_lossy().to_string(),
            size_bytes: 0,
            modified_at: 1000,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let empty_id = db.upsert_file(&img_empty_record).unwrap();

        let _q3 = db
            .enqueue_cleanup(&src_file.to_string_lossy(), empty_id, 0)
            .unwrap();
        let rep3 = process_pipeline_cleanups_at(&db, i64::MAX, None, None).unwrap();
        assert_eq!(rep3.deleted_count, 0);
        assert_eq!(rep3.failed_count, 1);
        assert!(src_file.exists()); // Source file preserved!
    }

    #[test]
    fn test_cleanup_source_revalidation() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        let dest_file = tmp.path().join("dest.png");
        write_dummy_png(&dest_file);

        let folder = db
            .add_folder_with_mode(
                &tmp.path().to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();

        let img_record = ImageFile {
            id: None,
            folder_id: folder.id,
            path: dest_file.to_string_lossy().to_string(),
            size_bytes: 67,
            modified_at: 1000,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let file_id = db.upsert_file(&img_record).unwrap();

        // 1. Source is a directory (never delete user media / directory roots)
        let dir_source = tmp.path().join("media_root");
        std::fs::create_dir_all(&dir_source).unwrap();
        let _q1 = db
            .enqueue_cleanup(&dir_source.to_string_lossy(), file_id, 0)
            .unwrap();
        let rep1 = process_pipeline_cleanups_at(&db, i64::MAX, None, None).unwrap();
        assert_eq!(rep1.failed_count, 1);
        assert!(dir_source.exists()); // Directory is intact!

        // 2. Source is identical to destination
        let _q2 = db
            .enqueue_cleanup(&dest_file.to_string_lossy(), file_id, 0)
            .unwrap();
        let rep2 = process_pipeline_cleanups_at(&db, i64::MAX, None, None).unwrap();
        assert_eq!(rep2.failed_count, 1);
        assert!(dest_file.exists()); // Refused to self-delete!

        // 3. Source file was already removed by user
        let missing_src = tmp.path().join("already_gone.png");
        let _q3 = db
            .enqueue_cleanup(&missing_src.to_string_lossy(), file_id, 0)
            .unwrap();
        let rep3 = process_pipeline_cleanups_at(&db, i64::MAX, None, None).unwrap();
        assert_eq!(rep3.deleted_count, 1);
        assert_eq!(rep3.failed_count, 0);
        assert!(matches!(
            rep3.items[0],
            PipelineCleanupItemOutcome::SkippedAlreadyRemoved { .. }
        ));
    }

    #[test]
    fn test_harvest_and_cleanup_cancellation() {
        let db = Database::connect_in_memory().unwrap();
        let tmp = tempdir().unwrap();

        let src_dir = tmp.path().join("source");
        let dest_dir = tmp.path().join("dest");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&dest_dir).unwrap();

        let folder = db
            .add_folder_with_mode(
                &dest_dir.to_string_lossy(),
                "pipeline",
                Some(&src_dir.to_string_lossy()),
                Some("copy"),
                None,
                true,
            )
            .unwrap();

        let cancel = AtomicBool::new(true);

        let harvest_rep =
            harvest_pipeline_folder_with_cancellation(&db, folder.id, None, Some(&cancel), None)
                .unwrap();
        assert!(harvest_rep.cancelled);

        let cleanup_rep = process_pipeline_cleanups_at(&db, i64::MAX, Some(&cancel), None).unwrap();
        assert!(cleanup_rep.cancelled);
    }
}
