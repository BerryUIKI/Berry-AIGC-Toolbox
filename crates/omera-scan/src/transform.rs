//! Verified image transformation engine for managed vault imports and batch library optimization.
//!
//! Complies with IMAGE_TRANSFORM_PLAN.md packages T2 (Import Transform) and T3 (Library Batch Transform).
//! Guarantees:
//! - Staged outputs are verified and decoded before publication.
//! - Non-destructive: originals are preserved by default.
//! - Read-only external link folders are never silently modified.
//! - No silent fallback to permanent delete if system trash fails.
//! - Database records preserve user curation (tags, albums, ratings, stacks, flags).

use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, DynamicImage, GenericImageView};
use omera_domain::{
    Container, ImportTransformRequest, LibraryTransformRequest, OriginalDisposition,
    TransformCollisionPolicy, TransformFormat, TransformItemReceipt, TransformItemStatus,
    TransformJobReceipt, TransformSpec,
};
use omera_storage::Database;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

/// Error types occurring during image transformation.
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("Source file does not exist: {0}")]
    SourceNotFound(String),
    #[error("Failed to decode source image: {0}")]
    DecodeFailed(String),
    #[error("Failed to encode derivative: {0}")]
    EncodeFailed(String),
    #[error("Derivative verification failed: {0}")]
    VerificationFailed(String),
    #[error("Destination already exists and collision policy is skip")]
    DestinationExistsSkipped,
    #[error("External linked folder is read-only for managed transformations")]
    ExternalFolderReadOnly,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Trash operation failed: {0}")]
    TrashFailed(String),
}

/// Helper to determine the target file extension for a given transform format and source path.
pub fn resolve_extension(source_path: &Path, format: TransformFormat) -> String {
    match format {
        TransformFormat::Original => source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase(),
        TransformFormat::Jpeg => "jpg".to_string(),
        TransformFormat::Webp => "webp".to_string(),
        TransformFormat::Png => "png".to_string(),
    }
}

/// Decode, optionally downscale, encode to staged path, and verify the staged image.
pub fn transform_file_staged(
    source_path: &Path,
    staging_dir: &Path,
    spec: &TransformSpec,
) -> Result<PathBuf, TransformError> {
    if !source_path.exists() {
        return Err(TransformError::SourceNotFound(
            source_path.to_string_lossy().into_owned(),
        ));
    }

    fs::create_dir_all(staging_dir)?;

    let mut img = image::open(source_path)
        .map_err(|e| TransformError::DecodeFailed(format!("Failed to open image: {e}")))?;

    // Handle optional maximum bounding edge downscaling
    if let Some(max_edge) = spec.max_edge {
        if max_edge > 0 {
            let (w, h) = img.dimensions();
            if w > max_edge || h > max_edge {
                img = img.resize(max_edge, max_edge, FilterType::Lanczos3);
            }
        }
    }

    let ext = resolve_extension(source_path, spec.format);
    let staged_file_name = format!(
        "staged_{}_{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        ext
    );
    let staged_path = staging_dir.join(staged_file_name);

    // Encode to staged path
    encode_image_to_path(&img, &staged_path, &ext, spec.quality)?;

    // Verification gate: verify file exists, has non-zero size, and decodes properly
    let metadata = fs::metadata(&staged_path)?;
    if metadata.len() == 0 {
        let _ = fs::remove_file(&staged_path);
        return Err(TransformError::VerificationFailed(
            "Staged derivative has 0 bytes".to_string(),
        ));
    }

    let verified_img = image::open(&staged_path).map_err(|e| {
        let _ = fs::remove_file(&staged_path);
        TransformError::VerificationFailed(format!("Failed to re-decode staged derivative: {e}"))
    })?;

    if verified_img.width() == 0 || verified_img.height() == 0 {
        let _ = fs::remove_file(&staged_path);
        return Err(TransformError::VerificationFailed(
            "Staged derivative has invalid dimensions".to_string(),
        ));
    }

    Ok(staged_path)
}

/// Encode dynamic image to a destination path based on extension and optional quality.
fn encode_image_to_path(
    img: &DynamicImage,
    target_path: &Path,
    ext: &str,
    quality: Option<u8>,
) -> Result<(), TransformError> {
    let file = File::create(target_path)?;
    let mut writer = BufWriter::new(file);

    match ext {
        "jpg" | "jpeg" => {
            let q = quality.unwrap_or(85).clamp(1, 100);
            let rgb = img.to_rgb8();
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, q);
            encoder
                .encode_image(&rgb)
                .map_err(|e| TransformError::EncodeFailed(e.to_string()))?;
        }
        "webp" => {
            img.write_to(&mut writer, image::ImageFormat::WebP)
                .map_err(|e| TransformError::EncodeFailed(e.to_string()))?;
        }
        "png" => {
            img.write_to(&mut writer, image::ImageFormat::Png)
                .map_err(|e| TransformError::EncodeFailed(e.to_string()))?;
        }
        _ => {
            // Default to PNG encoding
            img.write_to(&mut writer, image::ImageFormat::Png)
                .map_err(|e| TransformError::EncodeFailed(e.to_string()))?;
        }
    }

    writer.flush()?;
    Ok(())
}

/// Determine a unique publication path based on collision policy.
pub fn resolve_publication_path(
    dest_dir: &Path,
    file_stem: &str,
    extension: &str,
    collision: TransformCollisionPolicy,
) -> Result<PathBuf, TransformError> {
    let mut candidate = dest_dir.join(format!("{file_stem}.{extension}"));
    if !candidate.exists() {
        return Ok(candidate);
    }

    match collision {
        TransformCollisionPolicy::Skip => Err(TransformError::DestinationExistsSkipped),
        TransformCollisionPolicy::Rename => {
            let mut counter = 1;
            while candidate.exists() {
                candidate = dest_dir.join(format!("{file_stem}_{counter}.{extension}"));
                counter += 1;
            }
            Ok(candidate)
        }
    }
}

/// Execute managed import transformation (Image Transform Plan T2).
pub fn execute_managed_import_transform<F>(
    db: &Database,
    request: &ImportTransformRequest,
    progress_callback: Option<F>,
) -> Result<TransformJobReceipt, String>
where
    F: Fn(usize, usize, &str) + Send + Sync,
{
    let target_folder = db
        .list_folders()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|f| f.id == request.managed_destination_id)
        .ok_or_else(|| format!("Target managed folder {} not found", request.managed_destination_id))?;

    let dest_dir = PathBuf::from(&target_folder.path);
    if !dest_dir.is_dir() {
        return Err(format!("Managed destination directory does not exist: {}", target_folder.path));
    }

    let staging_dir = dest_dir.join(".omera_staging");
    let _ = fs::create_dir_all(&staging_dir);

    let job_id = format!("import_tx_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
    let mut items = Vec::new();
    let mut succeeded = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let total = request.source_paths.len();

    for (index, src_str) in request.source_paths.iter().enumerate() {
        let src_path = Path::new(src_str);
        let src_name = src_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        if let Some(ref cb) = progress_callback {
            cb(index + 1, total, src_name);
        }

        let file_stem = src_path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
        let ext = resolve_extension(src_path, request.spec.format);

        // Stage and verify
        let staged_res = transform_file_staged(src_path, &staging_dir, &request.spec);
        let staged_path = match staged_res {
            Ok(p) => p,
            Err(e) => {
                failed += 1;
                items.push(TransformItemReceipt {
                    source_id_or_path: src_str.clone(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Failed,
                    error_code: Some(e.to_string()),
                    original_action: Some("kept_intact".to_string()),
                });
                continue;
            }
        };

        // Resolve publication path
        let final_path_res = resolve_publication_path(&dest_dir, file_stem, &ext, request.spec.collision_policy);
        let final_path = match final_path_res {
            Ok(p) => p,
            Err(TransformError::DestinationExistsSkipped) => {
                let _ = fs::remove_file(&staged_path);
                skipped += 1;
                items.push(TransformItemReceipt {
                    source_id_or_path: src_str.clone(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Skipped,
                    error_code: Some("destination_exists_skipped".to_string()),
                    original_action: Some("kept_intact".to_string()),
                });
                continue;
            }
            Err(e) => {
                let _ = fs::remove_file(&staged_path);
                failed += 1;
                items.push(TransformItemReceipt {
                    source_id_or_path: src_str.clone(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Failed,
                    error_code: Some(e.to_string()),
                    original_action: Some("kept_intact".to_string()),
                });
                continue;
            }
        };

        // Atomically publish
        if let Err(e) = fs::rename(&staged_path, &final_path) {
            let _ = fs::remove_file(&staged_path);
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: src_str.clone(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some(format!("Failed to publish staged file: {e}")),
                original_action: Some("kept_intact".to_string()),
            });
            continue;
        }

        // Register published file in database
        let final_str = final_path.to_string_lossy().into_owned();
        let final_meta = fs::metadata(&final_path);
        let size_bytes = final_meta.as_ref().map(|m| m.len() as i64).unwrap_or(0);
        let modified_at = final_meta
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let container = Container::from_path(&final_path).unwrap_or(Container::Png);

        // Try extracting metadata from final path or source path
        let extracted = omera_metadata::extract_metadata(&final_path)
            .or_else(|| omera_metadata::extract_metadata(src_path));

        let is_nsfw = extracted
            .as_ref()
            .map(omera_metadata::detect_nsfw_from_metadata)
            .unwrap_or(false);

        let mut image_file = omera_domain::ImageFile {
            id: None,
            folder_id: target_folder.id,
            path: final_str.clone(),
            container,
            size_bytes,
            modified_at,
            metadata: extracted,
            indexed_at: chrono::Utc::now().to_rfc3339(),
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw,
            stack_id: None,
            stack_order: 0,
            version: 1,
        };

        if let Err(e) = db.upsert_file(&image_file) {
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: src_str.clone(),
                output_id_or_path: Some(final_str),
                status: TransformItemStatus::Failed,
                error_code: Some(format!("Database indexing failed: {e}")),
                original_action: Some("kept_intact".to_string()),
            });
            continue;
        }

        succeeded += 1;
        items.push(TransformItemReceipt {
            source_id_or_path: src_str.clone(),
            output_id_or_path: Some(final_str),
            status: TransformItemStatus::Succeeded,
            error_code: None,
            original_action: Some("kept_intact".to_string()),
        });
    }

    // Clean up empty staging dir
    let _ = fs::remove_dir(&staging_dir);

    Ok(TransformJobReceipt {
        job_id,
        phase: "completed".to_string(),
        total,
        succeeded,
        failed,
        skipped,
        canceled: 0,
        items,
    })
}

/// Execute batch library transformation on existing managed assets (Image Transform Plan T3).
pub fn execute_library_batch_transform<F>(
    db: &Database,
    request: &LibraryTransformRequest,
    progress_callback: Option<F>,
) -> Result<TransformJobReceipt, String>
where
    F: Fn(usize, usize, &str) + Send + Sync,
{
    let folders = db.list_folders().map_err(|e| e.to_string())?;
    let job_id = format!("lib_tx_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
    let mut items = Vec::new();
    let mut succeeded = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let total = request.file_ids.len();

    for (index, file_id) in request.file_ids.iter().enumerate() {
        let file_opt = db.get_file_by_id(*file_id).map_err(|e| e.to_string())?;
        let Some(file) = file_opt else {
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: file_id.to_string(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some("file_not_found_in_database".to_string()),
                original_action: None,
            });
            continue;
        };

        if let Some(ref cb) = progress_callback {
            cb(index + 1, total, &file.path);
        }

        // Verify that folder is a managed vault, not an external reference folder
        let folder = folders.iter().find(|f| f.id == file.folder_id);
        let Some(folder) = folder else {
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: file.path.clone(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some("folder_not_found".to_string()),
                original_action: None,
            });
            continue;
        };

        if folder.folder_type == "link" {
            // External folders are read-only: never mutate or replace them
            skipped += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: file.path.clone(),
                output_id_or_path: None,
                status: TransformItemStatus::Skipped,
                error_code: Some("external_folder_read_only".to_string()),
                original_action: Some("preserved".to_string()),
            });
            continue;
        }

        let src_path = PathBuf::from(&file.path);
        if !src_path.exists() {
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: file.path.clone(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some("source_missing_on_disk".to_string()),
                original_action: None,
            });
            continue;
        }

        let parent_dir = src_path.parent().unwrap_or_else(|| Path::new("."));
        let staging_dir = parent_dir.join(".omera_staging");
        let _ = fs::create_dir_all(&staging_dir);

        // Stage and verify derivative
        let staged_res = transform_file_staged(&src_path, &staging_dir, &request.spec);
        let staged_path = match staged_res {
            Ok(p) => p,
            Err(e) => {
                failed += 1;
                items.push(TransformItemReceipt {
                    source_id_or_path: file.path.clone(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Failed,
                    error_code: Some(e.to_string()),
                    original_action: Some("preserved".to_string()),
                });
                continue;
            }
        };

        let file_stem = src_path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
        let ext = resolve_extension(&src_path, request.spec.format);
        let final_path_res = resolve_publication_path(parent_dir, file_stem, &ext, request.spec.collision_policy);
        let final_path = match final_path_res {
            Ok(p) => p,
            Err(TransformError::DestinationExistsSkipped) => {
                let _ = fs::remove_file(&staged_path);
                skipped += 1;
                items.push(TransformItemReceipt {
                    source_id_or_path: file.path.clone(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Skipped,
                    error_code: Some("destination_exists_skipped".to_string()),
                    original_action: Some("preserved".to_string()),
                });
                continue;
            }
            Err(e) => {
                let _ = fs::remove_file(&staged_path);
                failed += 1;
                items.push(TransformItemReceipt {
                    source_id_or_path: file.path.clone(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Failed,
                    error_code: Some(e.to_string()),
                    original_action: Some("preserved".to_string()),
                });
                continue;
            }
        };

        // Atomically publish
        if let Err(e) = fs::rename(&staged_path, &final_path) {
            let _ = fs::remove_file(&staged_path);
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: file.path.clone(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some(format!("Failed to rename staged derivative: {e}")),
                original_action: Some("preserved".to_string()),
            });
            continue;
        }

        let final_str = final_path.to_string_lossy().into_owned();

        // Handle original disposition
        let mut original_action = "kept";
        let is_same_path = src_path == final_path;

        if !is_same_path {
            match request.original_disposition {
                OriginalDisposition::Keep => {
                    original_action = "kept";
                }
                OriginalDisposition::Archive => {
                    let archive_dir = Path::new(&folder.path).join(".omera_archive");
                    let _ = fs::create_dir_all(&archive_dir);
                    let archive_dest = archive_dir.join(src_path.file_name().unwrap_or_default());
                    if let Ok(()) = fs::rename(&src_path, &archive_dest) {
                        original_action = "archived";
                    }
                }
                OriginalDisposition::Trash => {
                    if let Ok(()) = trash::delete(&src_path) {
                        original_action = "trashed";
                    } else {
                        original_action = "trash_failed_kept";
                    }
                }
            }
        }

        // Update database record with new metadata while keeping user ratings, albums, tags, stacks
        let final_meta = fs::metadata(&final_path);
        let size_bytes = final_meta.as_ref().map(|m| m.len() as i64).unwrap_or(file.size_bytes);
        let modified_at = final_meta
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(file.modified_at);

        let container = Container::from_path(&final_path).unwrap_or(file.container);

        let mut updated_file = file.clone();
        updated_file.path = final_str.clone();
        updated_file.container = container;
        updated_file.size_bytes = size_bytes;
        updated_file.modified_at = modified_at;
        updated_file.version += 1;

        if let Err(e) = db.upsert_file(&updated_file) {
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: file.path.clone(),
                output_id_or_path: Some(final_str),
                status: TransformItemStatus::Failed,
                error_code: Some(format!("Failed to update database record: {e}")),
                original_action: Some(original_action.to_string()),
            });
            continue;
        }

        succeeded += 1;
        items.push(TransformItemReceipt {
            source_id_or_path: file.path.clone(),
            output_id_or_path: Some(final_str),
            status: TransformItemStatus::Succeeded,
            error_code: None,
            original_action: Some(original_action.to_string()),
        });

        let _ = fs::remove_dir(&staging_dir);
    }

    Ok(TransformJobReceipt {
        job_id,
        phase: "completed".to_string(),
        total,
        succeeded,
        failed,
        skipped,
        canceled: 0,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};
    use tempfile::tempdir;

    fn create_dummy_png(path: &Path, width: u32, height: u32) {
        let mut img = RgbImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([100, 150, 200]);
        }
        img.save(path).unwrap();
    }

    #[test]
    fn test_transform_file_staged_and_verify() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("test_input.png");
        create_dummy_png(&src, 100, 100);

        let staging = dir.path().join("staging");
        let spec = TransformSpec {
            format: TransformFormat::Jpeg,
            quality: Some(80),
            max_edge: Some(50),
            ..Default::default()
        };

        let staged = transform_file_staged(&src, &staging, &spec).unwrap();
        assert!(staged.exists());
        assert_eq!(staged.extension().unwrap(), "jpg");

        let verified = image::open(&staged).unwrap();
        assert_eq!(verified.width(), 50);
        assert_eq!(verified.height(), 50);
    }

    #[test]
    fn test_collision_resolution_rename_and_skip() {
        let dir = tempdir().unwrap();
        let existing = dir.path().join("sample.jpg");
        File::create(&existing).unwrap();

        // Skip returns error
        let skip_res = resolve_publication_path(dir.path(), "sample", "jpg", TransformCollisionPolicy::Skip);
        assert!(matches!(skip_res, Err(TransformError::DestinationExistsSkipped)));

        // Rename appends _1
        let rename_res = resolve_publication_path(dir.path(), "sample", "jpg", TransformCollisionPolicy::Rename).unwrap();
        assert_eq!(rename_res.file_name().unwrap(), "sample_1.jpg");
    }
}
