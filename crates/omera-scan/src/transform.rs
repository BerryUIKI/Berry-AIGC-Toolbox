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
    Container, ExtractedMetadata, Folder, ImageFile, ImportTransformRequest,
    LibraryTransformRequest, MetadataFormat, OriginalDisposition, TransformCollisionPolicy,
    TransformFormat, TransformItemReceipt, TransformItemStatus, TransformJobReceipt,
    TransformMetadataPolicy, TransformSpec,
};
use omera_storage::Database;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

static LIBRARY_TRANSFORM_GATE: Mutex<()> = Mutex::new(());

trait LibraryTransformStore {
    fn list_folders(&self) -> Result<Vec<Folder>, String>;
    fn get_file_by_id(&self, id: i64) -> Result<Option<ImageFile>, String>;
    fn publish(&self, id: i64, source: &ImageFile, derivative: &ImageFile) -> Result<(), String>;
}

impl LibraryTransformStore for Database {
    fn list_folders(&self) -> Result<Vec<Folder>, String> {
        Database::list_folders(self).map_err(|e| e.to_string())
    }
    fn get_file_by_id(&self, id: i64) -> Result<Option<ImageFile>, String> {
        Database::get_file_by_id(self, id).map_err(|e| e.to_string())
    }
    fn publish(&self, id: i64, source: &ImageFile, derivative: &ImageFile) -> Result<(), String> {
        if self
            .update_file_transformed_if_current(id, source, derivative)
            .map_err(|e| e.to_string())?
        {
            Ok(())
        } else {
            Err("Source record changed or disappeared during transformation".into())
        }
    }
}

impl LibraryTransformStore for Mutex<Database> {
    fn list_folders(&self) -> Result<Vec<Folder>, String> {
        let guard = self.lock().map_err(|_| "database lock poisoned")?;
        LibraryTransformStore::list_folders(&*guard)
    }
    fn get_file_by_id(&self, id: i64) -> Result<Option<ImageFile>, String> {
        let guard = self.lock().map_err(|_| "database lock poisoned")?;
        LibraryTransformStore::get_file_by_id(&*guard, id)
    }
    fn publish(&self, id: i64, source: &ImageFile, derivative: &ImageFile) -> Result<(), String> {
        let guard = self.lock().map_err(|_| "database lock poisoned")?;
        LibraryTransformStore::publish(&*guard, id, source, derivative)
    }
}

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
    #[error("AVIF is currently export-only; use PNG, JPEG or WebP for managed library media")]
    AvifExportOnly,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Trash operation failed: {0}")]
    TrashFailed(String),
}

/// The shipped image dependency encodes AVIF but cannot decode it for library use.
/// Inspect bytes as well as extensions so renaming an AVIF cannot bypass the gate.
fn validate_managed_codec(
    source: &Path,
    spec: Option<&TransformSpec>,
) -> Result<(), TransformError> {
    if spec.is_some_and(|spec| spec.format == TransformFormat::Avif)
        || source
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("avif"))
    {
        return Err(TransformError::AvifExportOnly);
    }
    use std::io::Read;
    let mut header = [0; 32];
    let length = File::open(source)?.read(&mut header)?;
    if omera_metadata::detect_container(&header[..length]) == Some(Container::Avif) {
        return Err(TransformError::AvifExportOnly);
    }
    Ok(())
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
        TransformFormat::Avif => "avif".to_string(),
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

    validate_managed_codec(source_path, Some(spec))?;
    fs::create_dir_all(staging_dir)?;

    // Extract source metadata for potential preservation
    let source_metadata = {
        let mut file = File::open(source_path)?;
        let mut header = [0u8; 32];
        use std::io::Read;
        let _ = file.read(&mut header);
        omera_metadata::detect_container(&header)
            .and_then(|c| omera_metadata::extract_metadata(c, source_path))
    };

    let mut img = image::open(source_path)
        .map_err(|e| TransformError::DecodeFailed(format!("Failed to open image: {e}")))?;

    // Handle optional percentage scaling (T4)
    if let Some(pct) = spec.scale_percent {
        if pct > 0 && pct != 100 {
            let (w, h) = img.dimensions();
            let new_w = ((w as f64 * pct as f64 / 100.0).round() as u32).max(1);
            let new_h = ((h as f64 * pct as f64 / 100.0).round() as u32).max(1);
            img = img.resize_exact(new_w, new_h, FilterType::Lanczos3);
        }
    }

    // Handle optional maximum bounding edge downscaling
    if let Some(max_edge) = spec.max_edge {
        if max_edge > 0 {
            let (w, h) = img.dimensions();
            if w > max_edge || h > max_edge {
                img = img.resize(max_edge, max_edge, FilterType::Lanczos3);
            }
        }
    }

    // Handle optional pixel-multiple alignment (T4, e.g. 8 or 16 multiples)
    if let Some(mult) = spec.align_multiple {
        if mult > 1 {
            let (w, h) = img.dimensions();
            let aligned_w = ((w / mult) * mult).max(mult);
            let aligned_h = ((h / mult) * mult).max(mult);
            if aligned_w != w || aligned_h != h {
                img = img.resize_exact(aligned_w, aligned_h, FilterType::Lanczos3);
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

    // Determine metadata to preserve based on policy
    let metadata_to_write = match spec.metadata_policy {
        TransformMetadataPolicy::StripAll => None,
        TransformMetadataPolicy::StripAi => {
            // Strip AI-specific fields but keep other metadata
            source_metadata.as_ref().map(|m| {
                let mut stripped = m.clone();
                stripped.prompt = None;
                stripped.negative_prompt = None;
                stripped.parameters = None;
                stripped.raw = None;
                stripped
            })
        }
        TransformMetadataPolicy::KeepSupported => source_metadata.as_ref().cloned(),
    };

    // Initial encode to staged path
    let mut current_quality = spec.quality;
    encode_image_to_path(
        &img,
        &staged_path,
        &ext,
        current_quality,
        metadata_to_write.as_ref(),
    )?;

    // Bounded target-size search (T4)
    if let Some(target_kb) = spec.target_size_kb {
        if target_kb > 0 {
            let target_bytes = (target_kb as u64) * 1024;
            let mut iter = 0;
            while iter < 4 {
                if let Ok(meta) = fs::metadata(&staged_path) {
                    if meta.len() <= target_bytes {
                        break;
                    }
                    iter += 1;
                    let actual_len = meta.len() as f64;
                    let ratio = target_bytes as f64 / actual_len;
                    let cur_q = current_quality.unwrap_or(80);
                    if cur_q > 25 {
                        let new_q = ((cur_q as f64 * ratio.sqrt() * 0.95).round() as u8)
                            .clamp(15, cur_q.saturating_sub(8));
                        current_quality = Some(new_q);
                        let _ = encode_image_to_path(
                            &img,
                            &staged_path,
                            &ext,
                            current_quality,
                            metadata_to_write.as_ref(),
                        );
                    } else {
                        // Quality is already low, downscale dimensions slightly (0.85x)
                        let (w, h) = img.dimensions();
                        if w > 64 && h > 64 {
                            let new_w = ((w as f64 * 0.85).round() as u32).max(32);
                            let new_h = ((h as f64 * 0.85).round() as u32).max(32);
                            img = img.resize_exact(new_w, new_h, FilterType::Lanczos3);
                            let _ = encode_image_to_path(
                                &img,
                                &staged_path,
                                &ext,
                                current_quality,
                                metadata_to_write.as_ref(),
                            );
                        } else {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }

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
/// For PNG files with KeepSupported policy, preserves generation metadata in text chunks.
fn encode_image_to_path(
    img: &DynamicImage,
    target_path: &Path,
    ext: &str,
    quality: Option<u8>,
    metadata: Option<&omera_domain::ExtractedMetadata>,
) -> Result<(), TransformError> {
    match ext {
        "jpg" | "jpeg" => {
            let file = File::create(target_path)?;
            let mut writer = BufWriter::new(file);
            let q = quality.unwrap_or(85).clamp(1, 100);
            let rgb = img.to_rgb8();
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, q);
            encoder
                .encode_image(&rgb)
                .map_err(|e| TransformError::EncodeFailed(e.to_string()))?;
            writer.flush()?;
        }
        "webp" => {
            let file = File::create(target_path)?;
            let mut writer = BufWriter::new(file);
            let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut writer);
            img.write_with_encoder(encoder)
                .map_err(|e| TransformError::EncodeFailed(e.to_string()))?;
            writer.flush()?;
        }
        "png" => {
            // PNG with optional metadata preservation
            encode_png_with_metadata(img, target_path, metadata)?;
        }
        _ => {
            // Default to PNG encoding
            encode_png_with_metadata(img, target_path, metadata)?;
        }
    }

    Ok(())
}

/// Encode a PNG image with optional embedded metadata preservation.
fn encode_png_with_metadata(
    img: &DynamicImage,
    target_path: &Path,
    metadata: Option<&omera_domain::ExtractedMetadata>,
) -> Result<(), TransformError> {
    let file = File::create(target_path)?;
    let mut encoder = png::Encoder::new(file, img.width(), img.height());

    // Configure color type and bit depth based on image
    let color_type = match img.color() {
        image::ColorType::L8 => png::ColorType::Grayscale,
        image::ColorType::La8 => png::ColorType::GrayscaleAlpha,
        image::ColorType::Rgb8 => png::ColorType::Rgb,
        image::ColorType::Rgba8 => png::ColorType::Rgba,
        _ => png::ColorType::Rgba, // Default to RGBA for other types
    };
    encoder.set_color(color_type);
    encoder.set_depth(png::BitDepth::Eight);

    // Add metadata text chunks if present
    if let Some(meta) = metadata {
        // Preserve the original parameters chunk if available (A1111 format)
        if let Some(ref params) = meta.parameters {
            encoder
                .add_text_chunk("parameters".into(), params.clone())
                .map_err(|e| {
                    TransformError::EncodeFailed(format!("Failed to add parameters chunk: {e}"))
                })?;
        }
    }

    let mut writer = encoder
        .write_header()
        .map_err(|e| TransformError::EncodeFailed(format!("Failed to write PNG header: {e}")))?;

    // Write image data
    let buf = match img.color() {
        image::ColorType::L8 => img.to_luma8().into_raw(),
        image::ColorType::La8 => img.to_luma_alpha8().into_raw(),
        image::ColorType::Rgb8 => img.to_rgb8().into_raw(),
        _ => img.to_rgba8().into_raw(),
    };

    writer.write_image_data(&buf).map_err(|e| {
        TransformError::EncodeFailed(format!("Failed to write PNG image data: {e}"))
    })?;

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

/// Validate destination folder exists and has folder_type == "managed".
pub fn validate_managed_destination_folder(
    db: &Database,
    folder_id: i64,
) -> Result<Folder, String> {
    let folders = db.list_folders().map_err(|e| e.to_string())?;
    let folder = folders
        .into_iter()
        .find(|f| f.id == folder_id)
        .ok_or_else(|| format!("Target managed folder {folder_id} not found"))?;

    if folder.folder_type != "managed" {
        return Err(format!(
            "Target folder {} is not a managed vault folder",
            folder_id
        ));
    }

    let dest_dir = Path::new(&folder.path);
    if !dest_dir.is_dir() {
        return Err(format!(
            "Managed destination directory does not exist: {}",
            folder.path
        ));
    }

    Ok(folder)
}

/// Normalize path string to remove Windows verbatim `\\?\` prefix and canonicalize if possible.
pub(crate) fn normalize_path_string(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = canonical.to_string_lossy();
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
}

/// Internal publication result for a single imported file.
pub struct PublishedImportItem {
    pub receipt: TransformItemReceipt,
    pub file_id: Option<i64>,
}

// Reserve the whole media/sidecar stem, including orphan destination sidecars.
fn resolve_import_publication_path(
    dest_dir: &Path,
    stem: &str,
    extension: &str,
    collision: TransformCollisionPolicy,
) -> Result<PathBuf, TransformError> {
    let mut counter = 0;
    loop {
        let name = if counter == 0 {
            stem.to_string()
        } else {
            format!("{stem}_{counter}")
        };
        let candidate = dest_dir.join(format!("{name}.{extension}"));
        if ![
            candidate.clone(),
            candidate.with_extension("txt"),
            candidate.with_extension("json"),
        ]
        .iter()
        .any(|p| p.symlink_metadata().is_ok())
        {
            return Ok(candidate);
        }
        if collision == TransformCollisionPolicy::Skip {
            return Err(TransformError::DestinationExistsSkipped);
        }
        counter += 1;
    }
}

fn copy_import_sidecars(
    source: &Path,
    output: &Path,
    copied: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    for extension in ["txt", "json"] {
        let src = source.with_extension(extension);
        if src == source || !src.is_file() {
            continue;
        }
        let mut reader = File::open(&src)?;
        let dst = output.with_extension(extension);
        let mut writer = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dst)?;
        copied.push(dst); // Compensation owns only files successfully created here.
        std::io::copy(&mut reader, &mut writer)?;
        writer.flush()?;
    }
    Ok(())
}

/// Publish a single file into a managed vault folder with metadata policy,
/// sidecar preservation, and interruption compensation.
pub fn publish_managed_import_item(
    db: &Database,
    src_path_str: &str,
    dest_dir: &Path,
    staging_dir: &Path,
    folder_id: i64,
    transform_spec: Option<&TransformSpec>,
) -> PublishedImportItem {
    publish_managed_import_item_with_sidecars(
        db,
        src_path_str,
        dest_dir,
        staging_dir,
        folder_id,
        transform_spec,
        copy_import_sidecars,
    )
}

fn publish_managed_import_item_with_sidecars(
    db: &Database,
    src_path_str: &str,
    dest_dir: &Path,
    staging_dir: &Path,
    folder_id: i64,
    transform_spec: Option<&TransformSpec>,
    copy_sidecars: impl FnOnce(&Path, &Path, &mut Vec<PathBuf>) -> std::io::Result<()>,
) -> PublishedImportItem {
    let src_path = Path::new(src_path_str);
    if !src_path.is_file() {
        return PublishedImportItem {
            receipt: TransformItemReceipt {
                source_id_or_path: src_path_str.to_string(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some("Source file does not exist or is not a file".to_string()),
                original_action: Some("kept_intact".to_string()),
            },
            file_id: None,
        };
    }

    if let Err(error) = validate_managed_codec(src_path, transform_spec) {
        return PublishedImportItem {
            receipt: TransformItemReceipt {
                source_id_or_path: src_path_str.to_string(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some(error.to_string()),
                original_action: Some("kept_intact".into()),
            },
            file_id: None,
        };
    }

    let supported_exts = ["png", "jpg", "jpeg", "webp", "avif", "mp4", "webm"];
    let raw_ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !supported_exts.contains(&raw_ext.as_str()) {
        return PublishedImportItem {
            receipt: TransformItemReceipt {
                source_id_or_path: src_path_str.to_string(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some(format!("Unsupported media format: {raw_ext}")),
                original_action: Some("kept_intact".to_string()),
            },
            file_id: None,
        };
    }

    let meta = match src_path.metadata() {
        Ok(m) => m,
        Err(e) => {
            return PublishedImportItem {
                receipt: TransformItemReceipt {
                    source_id_or_path: src_path_str.to_string(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Failed,
                    error_code: Some(format!("Failed to read source metadata: {e}")),
                    original_action: Some("kept_intact".to_string()),
                },
                file_id: None,
            };
        }
    };

    if meta.len() == 0 {
        return PublishedImportItem {
            receipt: TransformItemReceipt {
                source_id_or_path: src_path_str.to_string(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some("Source file is empty (0 bytes)".to_string()),
                original_action: Some("kept_intact".to_string()),
            },
            file_id: None,
        };
    }

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let file_mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(now_ts);

    let is_video = raw_ext == "mp4" || raw_ext == "webm";
    let should_transform = transform_spec.is_some_and(TransformSpec::requires_processing);
    if is_video && should_transform {
        return PublishedImportItem {
            receipt: TransformItemReceipt {
                source_id_or_path: src_path_str.to_string(),
                output_id_or_path: None,
                status: TransformItemStatus::Failed,
                error_code: Some(
                    "Video transformation/metadata stripping is unsupported; source preserved"
                        .into(),
                ),
                original_action: Some("kept_intact".into()),
            },
            file_id: None,
        };
    }

    let file_stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let final_ext: String;
    let published_path: PathBuf;
    let mut copied_sidecars = Vec::new();
    let published_new;

    if should_transform {
        let spec = transform_spec.unwrap();
        let target_ext = resolve_extension(src_path, spec.format);
        let staged_res = transform_file_staged(src_path, staging_dir, spec);
        let staged_path = match staged_res {
            Ok(p) => p,
            Err(e) => {
                return PublishedImportItem {
                    receipt: TransformItemReceipt {
                        source_id_or_path: src_path_str.to_string(),
                        output_id_or_path: None,
                        status: TransformItemStatus::Failed,
                        error_code: Some(e.to_string()),
                        original_action: Some("kept_intact".to_string()),
                    },
                    file_id: None,
                };
            }
        };

        let pub_res = resolve_import_publication_path(
            dest_dir,
            file_stem,
            &target_ext,
            spec.collision_policy,
        );
        let pub_path = match pub_res {
            Ok(p) => p,
            Err(TransformError::DestinationExistsSkipped) => {
                let _ = fs::remove_file(&staged_path);
                return PublishedImportItem {
                    receipt: TransformItemReceipt {
                        source_id_or_path: src_path_str.to_string(),
                        output_id_or_path: None,
                        status: TransformItemStatus::Skipped,
                        error_code: Some("destination_exists_skipped".to_string()),
                        original_action: Some("kept_intact".to_string()),
                    },
                    file_id: None,
                };
            }
            Err(e) => {
                let _ = fs::remove_file(&staged_path);
                return PublishedImportItem {
                    receipt: TransformItemReceipt {
                        source_id_or_path: src_path_str.to_string(),
                        output_id_or_path: None,
                        status: TransformItemStatus::Failed,
                        error_code: Some(e.to_string()),
                        original_action: Some("kept_intact".to_string()),
                    },
                    file_id: None,
                };
            }
        };

        if let Err(e) = fs::rename(&staged_path, &pub_path) {
            let _ = fs::remove_file(&staged_path);
            return PublishedImportItem {
                receipt: TransformItemReceipt {
                    source_id_or_path: src_path_str.to_string(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Failed,
                    error_code: Some(format!("Failed to publish staged file to destination: {e}")),
                    original_action: Some("kept_intact".to_string()),
                },
                file_id: None,
            };
        }

        published_new = true;
        published_path = pub_path;
        final_ext = target_ext;
    } else {
        let collision_policy = transform_spec
            .map(|s| s.collision_policy)
            .unwrap_or(TransformCollisionPolicy::Rename);

        let initial_candidate = dest_dir.join(format!("{file_stem}.{raw_ext}"));
        let (pub_path, needs_copy) = if initial_candidate.symlink_metadata().is_ok()
            || initial_candidate
                .with_extension("txt")
                .symlink_metadata()
                .is_ok()
            || initial_candidate
                .with_extension("json")
                .symlink_metadata()
                .is_ok()
        {
            let is_identical =
                crate::pipeline::files_have_identical_content(src_path, &initial_candidate)
                    .unwrap_or(false);
            if is_identical {
                // Content is verified identical: reuse existing published file (#268).
                (initial_candidate, false)
            } else {
                match resolve_import_publication_path(
                    dest_dir,
                    file_stem,
                    &raw_ext,
                    collision_policy,
                ) {
                    Ok(p) => (p, true),
                    Err(TransformError::DestinationExistsSkipped) => {
                        return PublishedImportItem {
                            receipt: TransformItemReceipt {
                                source_id_or_path: src_path_str.to_string(),
                                output_id_or_path: None,
                                status: TransformItemStatus::Skipped,
                                error_code: Some("destination_exists_skipped".to_string()),
                                original_action: Some("kept_intact".to_string()),
                            },
                            file_id: None,
                        };
                    }
                    Err(e) => {
                        return PublishedImportItem {
                            receipt: TransformItemReceipt {
                                source_id_or_path: src_path_str.to_string(),
                                output_id_or_path: None,
                                status: TransformItemStatus::Failed,
                                error_code: Some(e.to_string()),
                                original_action: Some("kept_intact".to_string()),
                            },
                            file_id: None,
                        };
                    }
                }
            }
        } else {
            (initial_candidate, true)
        };

        if needs_copy {
            if let Err(e) = fs::copy(src_path, &pub_path) {
                return PublishedImportItem {
                    receipt: TransformItemReceipt {
                        source_id_or_path: src_path_str.to_string(),
                        output_id_or_path: None,
                        status: TransformItemStatus::Failed,
                        error_code: Some(format!("Failed to copy file to destination: {e}")),
                        original_action: Some("kept_intact".to_string()),
                    },
                    file_id: None,
                };
            }
        }
        published_new = needs_copy;
        published_path = pub_path;
        final_ext = raw_ext;
    }

    // Unknown sidecar schemas can nest private generation fields anywhere.
    let preserve_sidecars = transform_spec
        .map(|s| s.metadata_policy == TransformMetadataPolicy::KeepSupported)
        .unwrap_or(true);
    if published_new && preserve_sidecars {
        if let Err(e) = copy_sidecars(src_path, &published_path, &mut copied_sidecars) {
            let _ = fs::remove_file(&published_path);
            for sidecar in copied_sidecars {
                let _ = fs::remove_file(sidecar);
            }
            return PublishedImportItem {
                receipt: TransformItemReceipt {
                    source_id_or_path: src_path_str.to_string(),
                    output_id_or_path: None,
                    status: TransformItemStatus::Failed,
                    error_code: Some(format!("Sidecar publication failed: {e}")),
                    original_action: Some("kept_intact".to_string()),
                },
                file_id: None,
            };
        }
    }

    let container = match final_ext.as_str() {
        "png" => Container::Png,
        "jpg" | "jpeg" => Container::Jpeg,
        "webp" => Container::WebP,
        "avif" => Container::Avif,
        "mp4" => Container::Mp4,
        "webm" => Container::Webm,
        _ => Container::Png,
    };

    let mut metadata = omera_metadata::extract_metadata(container, &published_path)
        .or_else(|| omera_metadata::extract_metadata(container, src_path));

    if let Some(spec) = transform_spec {
        match spec.metadata_policy {
            TransformMetadataPolicy::StripAll => {
                metadata = None;
            }
            TransformMetadataPolicy::StripAi => {
                if let Some(m) = metadata.as_mut() {
                    m.prompt = None;
                    m.negative_prompt = None;
                    m.parameters = None;
                    m.raw = None;
                }
            }
            TransformMetadataPolicy::KeepSupported => {}
        }
    }

    let is_nsfw = metadata
        .as_ref()
        .map(omera_metadata::detect_nsfw_from_metadata)
        .unwrap_or(false);

    let tgt_str = normalize_path_string(&published_path);
    let final_meta = published_path.metadata().unwrap_or(meta);

    let image_file = omera_domain::ImageFile {
        id: None,
        folder_id,
        path: tgt_str.clone(),
        size_bytes: final_meta.len(),
        modified_at: file_mtime,
        container,
        metadata,
        rating: None,
        aesthetic_score: None,
        is_favorite: false,
        is_nsfw,
        stack_id: None,
        stack_order: 0,
    };

    let file_id = match db.upsert_file(&image_file) {
        Ok(id) => id,
        Err(e) => {
            // Compensation / rollback: clean up newly published file and copied sidecars
            if published_new {
                let _ = fs::remove_file(&published_path);
            }
            for sidecar in copied_sidecars {
                let _ = fs::remove_file(sidecar);
            }

            return PublishedImportItem {
                receipt: TransformItemReceipt {
                    source_id_or_path: src_path_str.to_string(),
                    output_id_or_path: Some(tgt_str),
                    status: TransformItemStatus::Failed,
                    error_code: Some(format!("Database indexing failed: {e}")),
                    original_action: Some("kept_intact".to_string()),
                },
                file_id: None,
            };
        }
    };

    PublishedImportItem {
        receipt: TransformItemReceipt {
            source_id_or_path: src_path_str.to_string(),
            output_id_or_path: Some(tgt_str),
            status: TransformItemStatus::Succeeded,
            error_code: None,
            original_action: Some("kept_intact".to_string()),
        },
        file_id: Some(file_id),
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
    let target_folder = validate_managed_destination_folder(db, request.managed_destination_id)?;
    let dest_dir = PathBuf::from(&target_folder.path);

    let staging_dir = dest_dir.join(".omera_staging");
    let _ = fs::create_dir_all(&staging_dir);

    let job_id = format!(
        "import_tx_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let mut items = Vec::new();
    let mut succeeded = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let total = request.source_paths.len();

    for (index, src_str) in request.source_paths.iter().enumerate() {
        let src_path = Path::new(src_str);
        let src_name = src_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if let Some(ref cb) = progress_callback {
            cb(index + 1, total, src_name);
        }

        let outcome = publish_managed_import_item(
            db,
            src_str,
            &dest_dir,
            &staging_dir,
            target_folder.id,
            Some(&request.spec),
        );

        match outcome.receipt.status {
            TransformItemStatus::Succeeded => succeeded += 1,
            TransformItemStatus::Failed => failed += 1,
            TransformItemStatus::Skipped => skipped += 1,
            TransformItemStatus::Canceled => {}
        }

        items.push(outcome.receipt);
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

/// Batch import external files into a managed folder using the unified publication service.
pub fn import_files_to_managed_folder(
    db: &Database,
    file_paths: &[String],
    target_folder_id: i64,
    transform_spec: Option<&TransformSpec>,
) -> Result<Vec<i64>, String> {
    if file_paths.is_empty() {
        return Ok(Vec::new());
    }

    let target_folder = validate_managed_destination_folder(db, target_folder_id)?;
    let dest_dir = PathBuf::from(&target_folder.path);

    let should_transform = transform_spec.is_some_and(TransformSpec::requires_processing);

    let staging_dir = dest_dir.join(".omera_staging");
    if should_transform {
        let _ = fs::create_dir_all(&staging_dir);
    }

    let mut imported_ids = Vec::new();

    for src_path_str in file_paths {
        let outcome = publish_managed_import_item(
            db,
            src_path_str,
            &dest_dir,
            &staging_dir,
            target_folder.id,
            transform_spec,
        );

        if let Some(file_id) = outcome.file_id {
            imported_ids.push(file_id);
        } else if outcome.receipt.status == TransformItemStatus::Failed {
            if should_transform {
                let _ = fs::remove_dir(&staging_dir);
            }
            return Err(outcome
                .receipt
                .error_code
                .unwrap_or_else(|| format!("Failed to import {src_path_str}")));
        }
    }

    if should_transform {
        let _ = fs::remove_dir(&staging_dir);
    }

    Ok(imported_ids)
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
    execute_library_batch_transform_from_store(db, request, progress_callback)
}

pub fn execute_shared_library_batch_transform<F>(
    db: &Mutex<Database>,
    request: &LibraryTransformRequest,
    progress_callback: Option<F>,
) -> Result<TransformJobReceipt, String>
where
    F: Fn(usize, usize, &str) + Send + Sync,
{
    execute_library_batch_transform_from_store(db, request, progress_callback)
}

fn execute_library_batch_transform_from_store<D: LibraryTransformStore, F>(
    db: &D,
    request: &LibraryTransformRequest,
    progress_callback: Option<F>,
) -> Result<TransformJobReceipt, String>
where
    F: Fn(usize, usize, &str) + Send + Sync,
{
    let _job_guard = LIBRARY_TRANSFORM_GATE
        .lock()
        .map_err(|_| "transform job lock poisoned")?;
    let folders = db.list_folders()?;
    let job_id = format!(
        "lib_tx_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let mut items = Vec::new();
    let mut succeeded = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let total = request.file_ids.len();

    for (index, file_id) in request.file_ids.iter().enumerate() {
        let file_opt = db.get_file_by_id(*file_id)?;
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

        let file_stem = src_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        let ext = resolve_extension(&src_path, request.spec.format);
        // Avoid inheriting orphan sidecars when the derivative is re-imported.
        let final_path_res = resolve_import_publication_path(
            parent_dir,
            file_stem,
            &ext,
            request.spec.collision_policy,
        );
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

        // Update database record with new metadata while keeping user ratings, albums, tags, stacks.
        // Persistence must precede original disposition to protect source integrity.
        let final_meta = fs::metadata(&final_path);
        let size_bytes = final_meta
            .as_ref()
            .map(|m| m.len())
            .unwrap_or(file.size_bytes);
        let modified_at = final_meta
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(file.modified_at);

        let container = Container::from_id(&ext).unwrap_or(file.container);

        let mut derivative = file.clone();
        derivative.path = final_str.clone();
        derivative.container = container;
        derivative.size_bytes = size_bytes;
        derivative.modified_at = modified_at;
        // Staging already fully decoded the output. Read the published header
        // for geometry; generator Size fields describe provenance, not these pixels.
        let (width, height) = match image::image_dimensions(&final_path) {
            Ok((width, height)) if width > 0 && height > 0 => (width, height),
            result => {
                if src_path != final_path {
                    let _ = fs::remove_file(&final_path);
                }
                failed += 1;
                items.push(TransformItemReceipt {
                    source_id_or_path: file.path.clone(),
                    output_id_or_path: Some(final_str),
                    status: TransformItemStatus::Failed,
                    error_code: Some(format!("Failed to verify published dimensions: {result:?}")),
                    original_action: Some("preserved".to_string()),
                });
                continue;
            }
        };
        // Read the published derivative, never restore stale source generation
        // fields from the indexed row when the selected policy strips them.
        let mut metadata = match request.spec.metadata_policy {
            TransformMetadataPolicy::KeepSupported => {
                omera_metadata::extract_metadata(container, &final_path)
            }
            TransformMetadataPolicy::StripAi | TransformMetadataPolicy::StripAll => None,
        }
        .unwrap_or_else(|| ExtractedMetadata {
            format: MetadataFormat::Unspecified,
            ..Default::default()
        });
        metadata.width = Some(width);
        metadata.height = Some(height);
        derivative.metadata = Some(metadata);

        let target_file_id = file.id.unwrap_or(*file_id);
        if let Err(e) = db.publish(target_file_id, &file, &derivative) {
            // Compensation: if DB update fails and published file is separate from source, remove published derivative
            if src_path != final_path {
                let _ = fs::remove_file(&final_path);
            }
            failed += 1;
            items.push(TransformItemReceipt {
                source_id_or_path: file.path.clone(),
                output_id_or_path: Some(final_str),
                status: TransformItemStatus::Failed,
                error_code: Some(format!("Failed to update database record: {e}")),
                original_action: Some("preserved".to_string()),
            });
            continue;
        }

        // Handle original disposition only after destination file and database update succeed
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
                    let file_stem_archive = src_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("archived");
                    let file_ext_archive =
                        src_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    let archive_dest = resolve_publication_path(
                        &archive_dir,
                        file_stem_archive,
                        file_ext_archive,
                        TransformCollisionPolicy::Rename,
                    )
                    .unwrap_or_else(|_| archive_dir.join(src_path.file_name().unwrap_or_default()));

                    if let Ok(()) = fs::rename(&src_path, &archive_dest) {
                        original_action = "archived";
                    } else {
                        original_action = "archive_failed_kept";
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
    #[test]
    fn shared_transform_releases_database_and_rejects_changed_source_rows() {
        use super::*;
        for change_source in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let source = dir.path().join("source.png");
            image::RgbImage::new(16, 8).save(&source).unwrap();
            let original = fs::read(&source).unwrap();
            let db = Database::connect_in_memory().unwrap();
            let folder = db
                .add_folder_with_mode(
                    &dir.path().to_string_lossy(),
                    "managed",
                    None,
                    None,
                    None,
                    true,
                )
                .unwrap();
            let id = db
                .upsert_file(&ImageFile {
                    id: None,
                    folder_id: folder.id,
                    path: source.to_string_lossy().into_owned(),
                    size_bytes: original.len() as u64,
                    modified_at: 1,
                    container: Container::Png,
                    metadata: None,
                    rating: None,
                    aesthetic_score: None,
                    is_favorite: false,
                    is_nsfw: false,
                    stack_id: None,
                    stack_order: 0,
                })
                .unwrap();
            let shared = std::sync::Arc::new(Mutex::new(db));
            let worker_db = shared.clone();
            let request = LibraryTransformRequest {
                file_ids: vec![id],
                spec: TransformSpec {
                    format: TransformFormat::Png,
                    ..Default::default()
                },
                original_disposition: OriginalDisposition::Keep,
            };
            let (started_tx, started_rx) = std::sync::mpsc::channel();
            let release = std::sync::Arc::new(std::sync::Barrier::new(2));
            let worker_release = release.clone();
            let worker = std::thread::spawn(move || {
                execute_shared_library_batch_transform(
                    &worker_db,
                    &request,
                    Some(move |_, _, _: &str| {
                        started_tx.send(()).unwrap();
                        worker_release.wait();
                    }),
                )
                .unwrap()
            });
            started_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            let start = std::time::Instant::now();
            let concurrent = shared.try_lock().map(|db| {
                db.set_file_rating(id, Some(5)).unwrap();
                if change_source {
                    db.connection()
                        .execute(
                            "UPDATE files SET path = 'changed-by-concurrent-move' WHERE id = ?1",
                            [id],
                        )
                        .unwrap();
                }
                db.get_file_by_id(id).unwrap().unwrap()
            });
            eprintln!("Concurrent transform query/write: {:?}", start.elapsed());
            release.wait();
            let receipt = worker.join().unwrap();
            let concurrent = concurrent.unwrap();
            let current = shared.lock().unwrap().get_file_by_id(id).unwrap().unwrap();
            assert_eq!(current.rating, Some(5));
            assert_eq!(fs::read(&source).unwrap(), original);
            if change_source {
                assert_eq!(receipt.failed, 1);
                assert_eq!(receipt.succeeded, 0);
                assert_eq!(current.path, concurrent.path);
                assert!(!Path::new(receipt.items[0].output_id_or_path.as_ref().unwrap()).exists());
            } else {
                assert_eq!(receipt.succeeded, 1);
                assert_eq!(receipt.failed, 0);
                assert_ne!(current.path, concurrent.path);
                assert!(Path::new(&current.path).exists());
            }
        }
    }

    #[test]
    fn sidecar_failure_rolls_back_only_owned_publication() {
        use super::*;
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png");
        image::RgbImage::new(16, 8).save(&source).unwrap();
        fs::write(source.with_extension("txt"), "private prompt").unwrap();
        let vault = dir.path().join("vault");
        fs::create_dir(&vault).unwrap();
        let db = Database::connect_in_memory().unwrap();
        let folder = db
            .add_folder_with_mode(&vault.to_string_lossy(), "managed", None, None, None, true)
            .unwrap();
        let item = publish_managed_import_item_with_sidecars(
            &db,
            &source.to_string_lossy(),
            &vault,
            &dir.path().join("staging"),
            folder.id,
            Some(&TransformSpec {
                format: TransformFormat::Png,
                ..Default::default()
            }),
            |src, output, copied| {
                // Deterministic collision after image publication, before sidecar copy.
                fs::write(output.with_extension("json"), "concurrent owner's data")?;
                fs::write(src.with_extension("json"), "private workflow")?;
                copy_import_sidecars(src, output, copied)
            },
        );
        assert_eq!(item.receipt.status, TransformItemStatus::Failed);
        assert!(item
            .receipt
            .error_code
            .unwrap()
            .contains("Sidecar publication failed"));
        assert!(item.file_id.is_none());
        assert!(!vault.join("source.png").exists());
        assert!(!vault.join("source.txt").exists());
        assert_eq!(
            fs::read_to_string(vault.join("source.json")).unwrap(),
            "concurrent owner's data"
        );
        assert!(source.exists());
        assert_eq!(
            fs::read_to_string(source.with_extension("txt")).unwrap(),
            "private prompt"
        );
    }

    #[test]
    fn failed_image_transform_publishes_no_sidecars() {
        use super::*;
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("broken.png");
        fs::write(&source, "invalid pixels").unwrap();
        fs::write(source.with_extension("txt"), "private prompt").unwrap();
        let vault = dir.path().join("vault");
        fs::create_dir(&vault).unwrap();
        let db = Database::connect_in_memory().unwrap();
        let item = publish_managed_import_item(
            &db,
            &source.to_string_lossy(),
            &vault,
            &dir.path().join("staging"),
            1,
            Some(&TransformSpec {
                format: TransformFormat::Png,
                ..Default::default()
            }),
        );
        assert_eq!(item.receipt.status, TransformItemStatus::Failed);
        assert_eq!(fs::read_dir(vault).unwrap().count(), 0);
        assert_eq!(fs::read(source).unwrap(), b"invalid pixels");
    }

    use super::*;
    use image::{Rgb, RgbImage};
    use omera_domain::ImageFile;
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
        let skip_res =
            resolve_publication_path(dir.path(), "sample", "jpg", TransformCollisionPolicy::Skip);
        assert!(matches!(
            skip_res,
            Err(TransformError::DestinationExistsSkipped)
        ));

        // Rename appends _1
        let rename_res = resolve_publication_path(
            dir.path(),
            "sample",
            "jpg",
            TransformCollisionPolicy::Rename,
        )
        .unwrap();
        assert_eq!(rename_res.file_name().unwrap(), "sample_1.jpg");
    }

    #[test]
    fn test_managed_avif_rejected_and_webp_decode_verified() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("test_input.png");
        create_dummy_png(&src, 80, 80);

        let staging = dir.path().join("staging");

        // AVIF
        let avif_spec = TransformSpec {
            format: TransformFormat::Avif,
            quality: Some(75),
            max_edge: Some(40),
            ..Default::default()
        };
        assert!(matches!(
            transform_file_staged(&src, &staging, &avif_spec),
            Err(TransformError::AvifExportOnly)
        ));
        assert!(!staging.exists());

        // WebP
        let webp_spec = TransformSpec {
            format: TransformFormat::Webp,
            max_edge: Some(60),
            ..Default::default()
        };
        let staged_webp = transform_file_staged(&src, &staging, &webp_spec).unwrap();
        assert!(staged_webp.exists());
        assert_eq!(staged_webp.extension().unwrap(), "webp");
        let verified_webp = image::open(&staged_webp).unwrap();
        assert_eq!(verified_webp.width(), 60);
        assert_eq!(verified_webp.height(), 60);
    }

    #[test]
    fn test_execute_library_batch_transform_managed_and_link() {
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("managed_vault");
        let link_dir = dir.path().join("link_folder");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::create_dir_all(&link_dir).unwrap();

        let db = Database::connect_in_memory().unwrap();
        let m_folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();
        let l_folder = db
            .add_folder_with_mode(&link_dir.to_string_lossy(), "link", None, None, None, true)
            .unwrap();

        let m_img = managed_dir.join("photo1.png");
        create_dummy_png(&m_img, 100, 100);
        let l_img = link_dir.join("photo2.png");
        create_dummy_png(&l_img, 100, 100);

        let m_file = ImageFile {
            id: None,
            folder_id: m_folder.id,
            path: m_img.to_string_lossy().to_string(),
            size_bytes: 500,
            modified_at: 1000,
            container: Container::Png,
            metadata: None,
            rating: Some(4),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let m_id = db.upsert_file(&m_file).unwrap();

        let l_file = ImageFile {
            id: None,
            folder_id: l_folder.id,
            path: l_img.to_string_lossy().to_string(),
            size_bytes: 500,
            modified_at: 1000,
            container: Container::Png,
            metadata: None,
            rating: Some(5),
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let l_id = db.upsert_file(&l_file).unwrap();

        let req = LibraryTransformRequest {
            file_ids: vec![m_id, l_id],
            spec: TransformSpec {
                format: TransformFormat::Webp,
                max_edge: Some(50),
                ..Default::default()
            },
            original_disposition: OriginalDisposition::Keep,
        };

        let receipt =
            execute_library_batch_transform(&db, &req, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(receipt.total, 2);
        assert_eq!(receipt.succeeded, 1);
        assert_eq!(receipt.skipped, 1);
        assert_eq!(receipt.failed, 0);

        // Managed file was transformed and database updated
        let updated_m = db.get_file_by_id(m_id).unwrap().unwrap();
        assert_eq!(updated_m.container, Container::WebP);
        assert!(updated_m.path.ends_with(".webp"));
        assert_eq!(updated_m.rating, Some(4));
        assert!(updated_m.is_favorite);
        // Original was kept
        assert!(m_img.exists());

        // Link folder file was skipped and preserved untouched
        assert!(l_img.exists());
        let unchanged_l = db.get_file_by_id(l_id).unwrap().unwrap();
        assert_eq!(unchanged_l.container, Container::Png);
    }

    #[test]
    fn test_execute_library_batch_transform_archive_disposition() {
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("managed_vault");
        fs::create_dir_all(&managed_dir).unwrap();

        let db = Database::connect_in_memory().unwrap();
        let m_folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();

        let m_img = managed_dir.join("original_art.png");
        create_dummy_png(&m_img, 120, 120);

        let m_file = ImageFile {
            id: None,
            folder_id: m_folder.id,
            path: m_img.to_string_lossy().to_string(),
            size_bytes: 800,
            modified_at: 1000,
            container: Container::Png,
            metadata: None,
            rating: Some(5),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let m_id = db.upsert_file(&m_file).unwrap();

        let req = LibraryTransformRequest {
            file_ids: vec![m_id],
            spec: TransformSpec {
                format: TransformFormat::Jpeg,
                quality: Some(85),
                ..Default::default()
            },
            original_disposition: OriginalDisposition::Archive,
        };

        let receipt =
            execute_library_batch_transform(&db, &req, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(receipt.succeeded, 1);
        assert_eq!(
            receipt.items[0].original_action.as_deref(),
            Some("archived")
        );

        // Original was moved into .omera_archive
        let archive_dir = managed_dir.join(".omera_archive");
        let archived_file = archive_dir.join("original_art.png");
        assert!(archived_file.exists());
        assert!(!m_img.exists());

        // Transformed JPEG exists
        let updated_m = db.get_file_by_id(m_id).unwrap().unwrap();
        assert_eq!(updated_m.container, Container::Jpeg);
        assert!(Path::new(&updated_m.path).exists());
    }

    #[test]
    fn test_transform_file_staged_t4_percentage_and_alignment() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("input.png");
        let staging = dir.path().join("staging");
        create_dummy_png(&src, 100, 100);

        // Test scale_percent: 50% -> 50x50, align_multiple: 16 -> 48x48
        let spec = TransformSpec {
            format: TransformFormat::Png,
            scale_percent: Some(50),
            align_multiple: Some(16),
            ..Default::default()
        };

        let staged = transform_file_staged(&src, &staging, &spec).unwrap();
        assert!(staged.exists());

        let decoded = image::open(&staged).unwrap();
        assert_eq!(decoded.dimensions(), (48, 48));
    }

    #[test]
    fn test_transform_file_staged_t4_target_size_bounded_search() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("photo.png");
        let staging = dir.path().join("staging");
        create_dummy_png(&src, 200, 200);

        // Request target_size_kb: 2
        let spec = TransformSpec {
            format: TransformFormat::Jpeg,
            quality: Some(95),
            target_size_kb: Some(2),
            ..Default::default()
        };

        let staged = transform_file_staged(&src, &staging, &spec).unwrap();
        assert!(staged.exists());
        let meta = fs::metadata(&staged).unwrap();
        assert!(meta.len() > 0);
    }

    #[test]
    fn test_managed_import_rejects_linked_folder_with_zero_side_effects() {
        let dir = tempdir().unwrap();
        let link_dir = dir.path().join("link_folder");
        let external_dir = dir.path().join("external");
        fs::create_dir_all(&link_dir).unwrap();
        fs::create_dir_all(&external_dir).unwrap();

        let db = Database::connect_in_memory().unwrap();
        let link_folder = db
            .add_folder_with_mode(&link_dir.to_string_lossy(), "link", None, None, None, true)
            .unwrap();

        let src_img = external_dir.join("photo.png");
        create_dummy_png(&src_img, 64, 64);

        // 1. Test execute_managed_import_transform rejects linked folder
        let req = ImportTransformRequest {
            managed_destination_id: link_folder.id,
            source_paths: vec![src_img.to_string_lossy().to_string()],
            spec: TransformSpec::default(),
            source_disposition: omera_domain::transform::ImportSourceDisposition::Keep,
        };

        let err = execute_managed_import_transform(&db, &req, None::<fn(usize, usize, &str)>)
            .unwrap_err();
        assert!(
            err.contains("not a managed vault folder"),
            "Error was: {err}"
        );

        // Verify zero filesystem side effects in link folder
        let link_entries: Vec<_> = fs::read_dir(&link_dir).unwrap().collect();
        assert_eq!(
            link_entries.len(),
            0,
            "Linked folder must have no created files"
        );

        // 2. Test import_files_to_managed_folder rejects linked folder
        let err2 = import_files_to_managed_folder(
            &db,
            &[src_img.to_string_lossy().to_string()],
            link_folder.id,
            None,
        )
        .unwrap_err();
        assert!(
            err2.contains("not a managed vault folder"),
            "Error was: {err2}"
        );

        // Verify zero files in database
        let files = db.list_files(link_folder.id).unwrap();
        assert_eq!(files.len(), 0);

        // 3. Test exact issue #260 reproduction: folder created via default db.add_folder()
        let repro_dest = dir.path().join("repro_external");
        fs::create_dir_all(&repro_dest).unwrap();
        let repro_folder = db.add_folder(&repro_dest.to_string_lossy()).unwrap();
        let repro_req = ImportTransformRequest {
            managed_destination_id: repro_folder.id,
            source_paths: vec![src_img.to_string_lossy().to_string()],
            spec: TransformSpec::default(),
            source_disposition: omera_domain::transform::ImportSourceDisposition::Keep,
        };
        let repro_err =
            execute_managed_import_transform(&db, &repro_req, None::<fn(usize, usize, &str)>)
                .unwrap_err();
        assert!(
            repro_err.contains("not a managed vault folder"),
            "Error was: {repro_err}"
        );
        let repro_entries: Vec<_> = fs::read_dir(&repro_dest).unwrap().collect();
        assert_eq!(
            repro_entries.len(),
            0,
            "Repro external folder must have no created files"
        );

        // 4. Test missing/stale destination ID fails cleanly
        let stale_req = ImportTransformRequest {
            managed_destination_id: 999999,
            source_paths: vec![src_img.to_string_lossy().to_string()],
            spec: TransformSpec::default(),
            source_disposition: omera_domain::transform::ImportSourceDisposition::Keep,
        };
        let stale_err =
            execute_managed_import_transform(&db, &stale_req, None::<fn(usize, usize, &str)>)
                .unwrap_err();
        assert!(stale_err.contains("not found"), "Error was: {stale_err}");
        let stale_err2 = import_files_to_managed_folder(
            &db,
            &[src_img.to_string_lossy().to_string()],
            999999,
            None,
        )
        .unwrap_err();
        assert!(stale_err2.contains("not found"), "Error was: {stale_err2}");

        // 5. Test pipeline destination folder is rejected
        let pipeline_dir = dir.path().join("pipeline_folder");
        fs::create_dir_all(&pipeline_dir).unwrap();
        let pipe_folder = db
            .add_folder_with_mode(
                &pipeline_dir.to_string_lossy(),
                "pipeline",
                Some("/unused"),
                Some("copy"),
                None,
                true,
            )
            .unwrap();
        let pipe_req = ImportTransformRequest {
            managed_destination_id: pipe_folder.id,
            source_paths: vec![src_img.to_string_lossy().to_string()],
            spec: TransformSpec::default(),
            source_disposition: omera_domain::transform::ImportSourceDisposition::Keep,
        };
        let pipe_err =
            execute_managed_import_transform(&db, &pipe_req, None::<fn(usize, usize, &str)>)
                .unwrap_err();
        assert!(
            pipe_err.contains("not a managed vault folder"),
            "Error was: {pipe_err}"
        );
    }

    #[test]
    fn test_managed_import_collision_rename_and_skip() {
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("managed_vault");
        let external_dir = dir.path().join("external");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::create_dir_all(&external_dir).unwrap();

        let db = Database::connect_in_memory().unwrap();
        let folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();

        let src1 = external_dir.join("pic.png");
        create_dummy_png(&src1, 50, 50);

        // First import creates pic.png
        let ids1 = import_files_to_managed_folder(
            &db,
            &[src1.to_string_lossy().to_string()],
            folder.id,
            None,
        )
        .unwrap();
        assert_eq!(ids1.len(), 1);
        assert!(managed_dir.join("pic.png").exists());

        // Create different file with same name
        let src2 = external_dir.join("pic2.png");
        create_dummy_png(&src2, 80, 80);
        let src2_renamed = external_dir.join("sub").join("pic.png");
        fs::create_dir_all(src2_renamed.parent().unwrap()).unwrap();
        fs::copy(&src2, &src2_renamed).unwrap();

        // Second import with collision Rename -> pic_1.png
        let rename_spec = TransformSpec {
            collision_policy: TransformCollisionPolicy::Rename,
            ..Default::default()
        };
        let ids2 = import_files_to_managed_folder(
            &db,
            &[src2_renamed.to_string_lossy().to_string()],
            folder.id,
            Some(&rename_spec),
        )
        .unwrap();
        assert_eq!(ids2.len(), 1);
        assert!(managed_dir.join("pic_1.png").exists());

        // Third import with collision Skip -> skips and returns ok without adding new record
        let skip_spec = TransformSpec {
            collision_policy: TransformCollisionPolicy::Skip,
            ..Default::default()
        };
        let ids3 = import_files_to_managed_folder(
            &db,
            &[src2_renamed.to_string_lossy().to_string()],
            folder.id,
            Some(&skip_spec),
        )
        .unwrap();
        assert_eq!(ids3.len(), 0);

        // Source remains kept intact
        assert!(src1.exists());
        assert!(src2_renamed.exists());
    }

    #[test]
    fn test_managed_import_sidecars_and_metadata_policy() {
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("managed_vault");
        let external_dir = dir.path().join("external");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::create_dir_all(&external_dir).unwrap();

        let db = Database::connect_in_memory().unwrap();
        let folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();

        let src_img = external_dir.join("art.png");
        create_dummy_png(&src_img, 60, 60);
        let sidecar_txt = external_dir.join("art.txt");
        fs::write(&sidecar_txt, "prompt: magical forest, rating: safe").unwrap();
        let sidecar_json = external_dir.join("art.json");
        fs::write(&sidecar_json, r#"{"prompt":"magical forest"}"#).unwrap();

        // Test StripAll drops sidecars
        let strip_all_spec = TransformSpec {
            metadata_policy: TransformMetadataPolicy::StripAll,
            ..Default::default()
        };
        let ids = import_files_to_managed_folder(
            &db,
            &[src_img.to_string_lossy().to_string()],
            folder.id,
            Some(&strip_all_spec),
        )
        .unwrap();
        assert_eq!(ids.len(), 1);

        let file = db.get_file_by_id(ids[0]).unwrap().unwrap();
        assert!(file.metadata.is_none());
        assert!(!managed_dir.join("art.txt").exists());
        assert!(!managed_dir.join("art.json").exists());
    }

    #[test]
    fn test_managed_import_db_failure_compensation() {
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("managed_vault");
        let external_dir = dir.path().join("external");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::create_dir_all(&external_dir).unwrap();

        let db = Database::connect_in_memory().unwrap();
        let _folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();

        let src_img = external_dir.join("test_comp.png");
        create_dummy_png(&src_img, 60, 60);
        let sidecar = external_dir.join("test_comp.txt");
        fs::write(&sidecar, "sidecar text").unwrap();

        let staging = managed_dir.join(".omera_staging");
        fs::create_dir_all(&staging).unwrap();

        // Pass invalid folder_id (e.g. 99999) to publish_managed_import_item so db.upsert_file fails due to foreign key constraint
        let item = publish_managed_import_item(
            &db,
            &src_img.to_string_lossy(),
            &managed_dir,
            &staging,
            99999, // invalid foreign key
            None,
        );

        assert_eq!(item.receipt.status, TransformItemStatus::Failed);
        assert!(item.file_id.is_none());
        assert!(item
            .receipt
            .error_code
            .as_ref()
            .unwrap()
            .contains("Database indexing failed"));

        // Compensation: published file and sidecar must NOT exist in managed_dir
        assert!(
            !managed_dir.join("test_comp.png").exists(),
            "Published image must be rolled back on DB failure"
        );
        assert!(
            !managed_dir.join("test_comp.txt").exists(),
            "Sidecar must be rolled back on DB failure"
        );

        // Source file and sidecar remain intact
        assert!(src_img.exists(), "Source image must remain intact");
        assert!(sidecar.exists(), "Source sidecar must remain intact");
    }

    #[test]
    fn test_library_batch_transform_archive_waits_for_db_persistence() {
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("managed_vault");
        fs::create_dir_all(&managed_dir).unwrap();

        let source = managed_dir.join("source.png");
        create_dummy_png(&source, 64, 32);

        let db = Database::connect_in_memory().unwrap();
        let folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                false,
            )
            .unwrap();

        let file = omera_domain::ImageFile {
            id: None,
            folder_id: folder.id,
            path: source.to_string_lossy().into_owned(),
            size_bytes: fs::metadata(&source).unwrap().len(),
            modified_at: 1,
            container: Container::Png,
            metadata: None,
            rating: Some(4),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let id = db.upsert_file(&file).unwrap();

        // Install synthetic trigger to simulate database write failure on UPDATE
        db.connection()
            .execute_batch(
                "CREATE TRIGGER test_fail_update BEFORE UPDATE ON files BEGIN SELECT RAISE(FAIL, 'simulated database write failure'); END;",
            )
            .unwrap();

        let request = LibraryTransformRequest {
            file_ids: vec![id],
            spec: TransformSpec {
                format: TransformFormat::Jpeg,
                ..Default::default()
            },
            original_disposition: OriginalDisposition::Archive,
        };

        let result =
            execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.failed, 1);
        assert_eq!(result.succeeded, 0);

        // Source file MUST still exist in its original place
        assert!(
            source.exists(),
            "Source was archived before failed DB update; indexed source path is missing"
        );

        // Derivative must have been rolled back / cleaned up on DB failure
        let derivative_jpg = managed_dir.join("source.jpg");
        assert!(
            !derivative_jpg.exists(),
            "Unindexed derivative should be cleaned up on DB failure"
        );

        // No files in .omera_archive
        let archive_dir = managed_dir.join(".omera_archive");
        if archive_dir.exists() {
            let archive_entries: Vec<_> = fs::read_dir(&archive_dir).unwrap().collect();
            assert_eq!(
                archive_entries.len(),
                0,
                "Archive folder should not contain original when DB update failed"
            );
        }
    }

    #[test]
    fn test_library_batch_transform_archive_collision_renames() {
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("managed_vault");
        fs::create_dir_all(&managed_dir).unwrap();

        // Create pre-existing file in .omera_archive with same name
        let archive_dir = managed_dir.join(".omera_archive");
        fs::create_dir_all(&archive_dir).unwrap();
        let existing_archived = archive_dir.join("photo.png");
        fs::write(&existing_archived, "previously archived").unwrap();

        let source = managed_dir.join("photo.png");
        create_dummy_png(&source, 64, 32);

        let db = Database::connect_in_memory().unwrap();
        let folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                false,
            )
            .unwrap();

        let file = omera_domain::ImageFile {
            id: None,
            folder_id: folder.id,
            path: source.to_string_lossy().into_owned(),
            size_bytes: fs::metadata(&source).unwrap().len(),
            modified_at: 1,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let id = db.upsert_file(&file).unwrap();

        let request = LibraryTransformRequest {
            file_ids: vec![id],
            spec: TransformSpec {
                format: TransformFormat::Jpeg,
                ..Default::default()
            },
            original_disposition: OriginalDisposition::Archive,
        };

        let result =
            execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed, 0);

        // Pre-existing archived file remains untouched
        assert_eq!(
            fs::read_to_string(&existing_archived).unwrap(),
            "previously archived"
        );

        // Newly archived file renamed to photo_1.png
        let renamed_archived = archive_dir.join("photo_1.png");
        assert!(
            renamed_archived.exists(),
            "Collision in archive folder must be resolved via renaming"
        );
    }

    #[test]
    fn test_managed_import_dedup_content_identity() {
        // Issue #268: Managed import treats equal byte lengths as identical content.
        // Equal-size different content must NOT be falsely deduplicated;
        // Identical content must be verified via byte comparison and return existing identity.
        let dir = tempdir().unwrap();
        let managed_dir = dir.path().join("vault");
        let ext_dir = dir.path().join("external");
        fs::create_dir_all(&managed_dir).unwrap();
        fs::create_dir_all(&ext_dir).unwrap();

        let db = Database::connect_in_memory().unwrap();
        let folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                true,
            )
            .unwrap();

        // 1. Initial import of photo.png
        let src1 = ext_dir.join("photo.png");
        create_dummy_png(&src1, 50, 50);
        let src1_bytes = fs::read(&src1).unwrap();
        let initial_ids = import_files_to_managed_folder(
            &db,
            &[src1.to_string_lossy().to_string()],
            folder.id,
            None,
        )
        .unwrap();
        assert_eq!(initial_ids.len(), 1);
        let photo_file_id = initial_ids[0];
        assert!(managed_dir.join("photo.png").exists());

        // 2. Prepare equal-length DIFFERENT content with same basename
        let src2_dir = ext_dir.join("other_source");
        fs::create_dir_all(&src2_dir).unwrap();
        let src2 = src2_dir.join("photo.png");
        let mut different_bytes = src1_bytes.clone();
        // Modify bytes in the middle while maintaining identical length
        let mid = different_bytes.len() / 2;
        different_bytes[mid] ^= 0xFF;
        different_bytes[mid + 1] ^= 0xAA;
        assert_eq!(src1_bytes.len(), different_bytes.len());
        assert_ne!(src1_bytes, different_bytes);
        fs::write(&src2, &different_bytes).unwrap();

        // 3. Import equal-length different content with Rename policy
        let rename_spec = TransformSpec {
            collision_policy: TransformCollisionPolicy::Rename,
            ..Default::default()
        };
        let rename_ids = import_files_to_managed_folder(
            &db,
            &[src2.to_string_lossy().to_string()],
            folder.id,
            Some(&rename_spec),
        )
        .unwrap();
        assert_eq!(rename_ids.len(), 1);
        assert_ne!(rename_ids[0], photo_file_id);
        // photo.png preserved untouched; photo_1.png created with different bytes
        assert_eq!(fs::read(managed_dir.join("photo.png")).unwrap(), src1_bytes);
        assert_eq!(
            fs::read(managed_dir.join("photo_1.png")).unwrap(),
            different_bytes
        );

        // 4. Import equal-length different content with Skip policy
        let skip_spec = TransformSpec {
            collision_policy: TransformCollisionPolicy::Skip,
            ..Default::default()
        };
        let skip_ids = import_files_to_managed_folder(
            &db,
            &[src2.to_string_lossy().to_string()],
            folder.id,
            Some(&skip_spec),
        )
        .unwrap();
        // Distinct content skipped as a collision: must not return photo_file_id!
        assert_eq!(skip_ids.len(), 0);

        // 5. Import truly IDENTICAL content with same basename
        let src3_dir = ext_dir.join("identical_source");
        fs::create_dir_all(&src3_dir).unwrap();
        let src3 = src3_dir.join("photo.png");
        fs::write(&src3, &src1_bytes).unwrap();

        let identical_ids = import_files_to_managed_folder(
            &db,
            &[src3.to_string_lossy().to_string()],
            folder.id,
            None,
        )
        .unwrap();
        assert_eq!(identical_ids.len(), 1);
        assert_eq!(identical_ids[0], photo_file_id);
        // Original file on destination is still intact
        assert_eq!(fs::read(managed_dir.join("photo.png")).unwrap(), src1_bytes);
    }
}
