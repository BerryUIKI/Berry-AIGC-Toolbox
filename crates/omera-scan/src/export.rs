use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use image::ImageReader;
use omera_domain::{
    ExportEstimateResult, ExportFormat, ExportOptions, ExportProgressEvent, ExportSidecar,
    ExportSummary, ImageFile, MetadataPrivacyMode,
};
use omera_storage::Database;
use rayon::prelude::*;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::html_showcase::{generate_html_showcase, ShowcaseItemMetadata};

/// Encode a PNG image with optional embedded metadata preservation.
/// This function mirrors the implementation in transform.rs to ensure consistent
/// metadata handling across export and transformation paths.
fn encode_png_with_metadata(
    img: &image::DynamicImage,
    buffer: &mut Vec<u8>,
    metadata: Option<&omera_domain::ExtractedMetadata>,
) -> Result<(), String> {
    let mut encoder = png::Encoder::new(Cursor::new(buffer), img.width(), img.height());

    // Configure color type and bit depth based on image
    let color_type = match img.color() {
        image::ColorType::L8 => png::ColorType::Grayscale,
        image::ColorType::La8 => png::ColorType::GrayscaleAlpha,
        image::ColorType::Rgb8 => png::ColorType::Rgb,
        image::ColorType::Rgba8 => png::ColorType::Rgba,
        _ => png::ColorType::Rgba,
    };
    encoder.set_color(color_type);
    encoder.set_depth(png::BitDepth::Eight);

    // Add metadata text chunks if present
    if let Some(meta) = metadata {
        // Preserve the original parameters chunk if available (A1111 format)
        if let Some(ref params) = meta.parameters {
            encoder
                .add_text_chunk("parameters".into(), params.clone())
                .map_err(|e| format!("Failed to add parameters chunk: {e}"))?;
        }
    }

    let mut writer = encoder
        .write_header()
        .map_err(|e| format!("Failed to write PNG header: {e}"))?;

    // Write image data
    let buf = match img.color() {
        image::ColorType::L8 => img.to_luma8().into_raw(),
        image::ColorType::La8 => img.to_luma_alpha8().into_raw(),
        image::ColorType::Rgb8 => img.to_rgb8().into_raw(),
        _ => img.to_rgba8().into_raw(),
    };

    writer
        .write_image_data(&buf)
        .map_err(|e| format!("Failed to write PNG image data: {e}"))?;

    Ok(())
}

/// Sanitize filename by removing invalid OS characters and trimming.
pub fn sanitize_filename_part(part: &str) -> String {
    let sanitized: String = part
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = sanitized.trim().trim_matches('.');
    if trimmed.is_empty() {
        "export".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Format an output filename based on a user-supplied template string.
pub fn format_export_filename(
    template: &str,
    file: &ImageFile,
    target_ext: &str,
    index: usize,
) -> String {
    let original_stem = Path::new(&file.path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let file_id_str = file.id.map(|id| id.to_string()).unwrap_or_default();
    let rating_str = file
        .rating
        .map(|r| r.to_string())
        .unwrap_or_else(|| "0".to_string());

    let date_str = if file.modified_at > 0 {
        let secs = file.modified_at;
        let days = secs / 86400;
        let mut year = 1970;
        let mut remaining_days = days;
        loop {
            let days_in_year = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                366
            } else {
                365
            };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_per_month = [
            31,
            if is_leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let mut month = 1;
        for &m_days in &days_per_month {
            if remaining_days < m_days {
                break;
            }
            remaining_days -= m_days;
            month += 1;
        }
        let day = remaining_days + 1;
        format!("{year:04}{month:02}{day:02}")
    } else {
        "20260101".to_string()
    };

    let model_str = file
        .metadata
        .as_ref()
        .and_then(|m| m.model_name.as_deref())
        .unwrap_or("unknown_model");

    let seed_str = file
        .metadata
        .as_ref()
        .and_then(|m| m.seed.as_deref())
        .unwrap_or("0");

    let mut result = template.to_string();
    if result.is_empty() {
        result = "{name}".to_string();
    }

    result = result.replace("{name}", original_stem);
    result = result.replace("{filename}", original_stem);
    result = result.replace("{id}", &file_id_str);
    result = result.replace("{index}", &(index + 1).to_string());
    result = result.replace("{date}", &date_str);
    result = result.replace("{rating}", &rating_str);
    result = result.replace("{model}", model_str);
    result = result.replace("{seed}", seed_str);

    let clean_stem = sanitize_filename_part(&result);
    format!("{clean_stem}.{target_ext}")
}

/// In-memory representation of a successfully processed export item.
pub struct ProcessedExportItem {
    pub base_filename: String,
    pub image_filename: String,
    pub image_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub sidecar: Option<(String, Vec<u8>)>,
    pub showcase_item: Option<ShowcaseItemMetadata>,
}

/// Process a single image file according to export format, downscaling, and privacy rules.
pub fn process_single_image(
    file: &ImageFile,
    options: &ExportOptions,
    index: usize,
) -> Result<ProcessedExportItem, String> {
    let src_path = Path::new(&file.path);
    if !src_path.exists() {
        return Err(format!("Source image does not exist: {}", file.path));
    }

    let target_ext = match options.format {
        ExportFormat::Original => src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase(),
        ExportFormat::Webp => "webp".to_string(),
        ExportFormat::Jpeg => "jpg".to_string(),
        ExportFormat::Png => "png".to_string(),
        ExportFormat::Avif => "avif".to_string(),
    };

    let image_filename =
        format_export_filename(&options.filename_template, file, &target_ext, index);
    let base_stem = Path::new(&image_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    // Fast-path: keep original format, no resizing, no privacy stripping
    let is_fast_pass = options.format == ExportFormat::Original
        && options.privacy == MetadataPrivacyMode::KeepAll
        && options.max_edge.is_none();

    // Extract source metadata from actual file bytes when needed for preservation
    let source_metadata = if options.privacy == MetadataPrivacyMode::KeepAll
        && (options.format == ExportFormat::Png
            || (options.format == ExportFormat::Original && target_ext == "png"))
    {
        let mut file_handle = File::open(src_path)
            .map_err(|e| format!("Failed to open {} for metadata extraction: {e}", file.path))?;
        let mut header = [0u8; 32];
        let _ = file_handle.read(&mut header);
        omera_metadata::detect_container(&header)
            .and_then(|c| omera_metadata::extract_metadata(c, src_path))
    } else {
        None
    };

    let (image_bytes, final_width, final_height) = if is_fast_pass {
        let bytes = fs::read(src_path).map_err(|e| format!("Failed to read {}: {e}", file.path))?;
        let w = file.metadata.as_ref().and_then(|m| m.width).unwrap_or(0);
        let h = file.metadata.as_ref().and_then(|m| m.height).unwrap_or(0);
        (bytes, w, h)
    } else {
        // Decode image
        let reader = ImageReader::open(src_path)
            .map_err(|e| format!("Failed to open {}: {e}", file.path))?
            .with_guessed_format()
            .map_err(|e| format!("Failed to guess format for {}: {e}", file.path))?;

        let mut img = reader
            .decode()
            .map_err(|e| format!("Failed to decode {}: {e}", file.path))?;

        // Downscale if max_edge specified
        if let Some(max_edge) = options.max_edge {
            let (w, h) = (img.width(), img.height());
            if w > max_edge || h > max_edge {
                img = img.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3);
            }
        }

        let (w, h) = (img.width(), img.height());

        // Encode to target format, preserving supported metadata for PNG with KeepAll
        let mut buffer = Vec::new();
        match options.format {
            ExportFormat::Webp => {
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut buffer);
                img.write_with_encoder(encoder)
                    .map_err(|e| format!("Failed to encode WebP: {e}"))?;
            }
            ExportFormat::Jpeg => {
                let rgb = img.to_rgb8();
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut buffer,
                    options.quality.clamp(1, 100),
                );
                encoder
                    .encode_image(&rgb)
                    .map_err(|e| format!("Failed to encode JPEG: {e}"))?;
            }
            ExportFormat::Avif => {
                let q = options.quality.clamp(1, 100);
                let speed: u8 = 6;
                let encoder =
                    image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut buffer, speed, q);
                img.write_with_encoder(encoder)
                    .map_err(|e| format!("Failed to encode AVIF: {e}"))?;
            }
            ExportFormat::Png => {
                // Use metadata-preserving encoder for PNG
                encode_png_with_metadata(&img, &mut buffer, source_metadata.as_ref())?;
            }
            ExportFormat::Original => {
                if target_ext == "jpg" || target_ext == "jpeg" {
                    let rgb = img.to_rgb8();
                    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                        &mut buffer,
                        options.quality.clamp(1, 100),
                    );
                    encoder
                        .encode_image(&rgb)
                        .map_err(|e| format!("Failed to encode JPEG: {e}"))?;
                } else if target_ext == "webp" {
                    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut buffer);
                    img.write_with_encoder(encoder)
                        .map_err(|e| format!("Failed to encode WebP: {e}"))?;
                } else if target_ext == "avif" {
                    let q = options.quality.clamp(1, 100);
                    let speed: u8 = 6;
                    let encoder = image::codecs::avif::AvifEncoder::new_with_speed_quality(
                        &mut buffer,
                        speed,
                        q,
                    );
                    img.write_with_encoder(encoder)
                        .map_err(|e| format!("Failed to encode AVIF: {e}"))?;
                } else {
                    // Original format is PNG - use metadata-preserving encoder
                    encode_png_with_metadata(&img, &mut buffer, source_metadata.as_ref())?;
                }
            }
        }
        (buffer, w, h)
    };

    // Generate optional sidecar adhering strictly to the privacy policy
    let sidecar = match options.sidecar {
        ExportSidecar::None => None,
        ExportSidecar::TextPrompt => {
            // TextPrompt sidecar must respect prompt-removal policies
            match options.privacy {
                MetadataPrivacyMode::StripAll
                | MetadataPrivacyMode::StripPromptOnly
                | MetadataPrivacyMode::StripAllAiMetadata => None,
                MetadataPrivacyMode::KeepAll => {
                    let prompt_text = file
                        .metadata
                        .as_ref()
                        .and_then(|m| m.prompt.as_deref())
                        .or_else(|| file.metadata.as_ref().and_then(|m| m.raw.as_deref()))
                        .unwrap_or_default()
                        .to_string();
                    if prompt_text.is_empty() {
                        None
                    } else {
                        let sidecar_filename = format!("{base_stem}.txt");
                        Some((sidecar_filename, prompt_text.into_bytes()))
                    }
                }
            }
        }
        ExportSidecar::JsonMetadata => {
            let json_str = match options.privacy {
                MetadataPrivacyMode::StripAll => None,
                MetadataPrivacyMode::StripAllAiMetadata => {
                    let mut m = file.metadata.clone().unwrap_or_default();
                    m.prompt = None;
                    m.negative_prompt = None;
                    m.parameters = None;
                    m.raw = None;
                    m.model_name = None;
                    m.model_hash = None;
                    m.sampler = None;
                    m.seed = None;
                    m.cfg_scale = None;
                    m.steps = None;
                    serde_json::to_string_pretty(&m).ok()
                }
                MetadataPrivacyMode::StripPromptOnly => {
                    let mut m = file.metadata.clone().unwrap_or_default();
                    m.prompt = None;
                    m.negative_prompt = None;
                    m.parameters = None;
                    m.raw = None;
                    serde_json::to_string_pretty(&m).ok()
                }
                MetadataPrivacyMode::KeepAll => file
                    .metadata
                    .as_ref()
                    .and_then(|m| serde_json::to_string_pretty(m).ok()),
            };
            json_str.map(|s| {
                let sidecar_filename = format!("{base_stem}.json");
                (sidecar_filename, s.into_bytes())
            })
        }
    };

    let showcase_item = if options.export_html_showcase {
        let (prompt, negative_prompt, model, sampler, seed, cfg_scale, steps) =
            match options.privacy {
                MetadataPrivacyMode::KeepAll => (
                    file.metadata.as_ref().and_then(|m| m.prompt.clone()),
                    file.metadata
                        .as_ref()
                        .and_then(|m| m.negative_prompt.clone()),
                    file.metadata.as_ref().and_then(|m| m.model_name.clone()),
                    file.metadata.as_ref().and_then(|m| m.sampler.clone()),
                    file.metadata.as_ref().and_then(|m| m.seed.clone()),
                    file.metadata.as_ref().and_then(|m| m.cfg_scale),
                    file.metadata.as_ref().and_then(|m| m.steps),
                ),
                MetadataPrivacyMode::StripPromptOnly => (
                    None,
                    None,
                    file.metadata.as_ref().and_then(|m| m.model_name.clone()),
                    file.metadata.as_ref().and_then(|m| m.sampler.clone()),
                    file.metadata.as_ref().and_then(|m| m.seed.clone()),
                    file.metadata.as_ref().and_then(|m| m.cfg_scale),
                    file.metadata.as_ref().and_then(|m| m.steps),
                ),
                MetadataPrivacyMode::StripAllAiMetadata | MetadataPrivacyMode::StripAll => {
                    (None, None, None, None, None, None, None)
                }
            };

        Some(ShowcaseItemMetadata {
            filename: image_filename.clone(),
            width: final_width,
            height: final_height,
            prompt,
            negative_prompt,
            model,
            sampler,
            seed,
            cfg_scale,
            steps,
            rating: file.rating,
        })
    } else {
        None
    };

    Ok(ProcessedExportItem {
        base_filename: base_stem,
        image_filename,
        image_bytes,
        width: final_width,
        height: final_height,
        sidecar,
        showcase_item,
    })
}

/// Compute size estimation and dimensions for an image without writing to disk.
pub fn estimate_export_single_image(
    file: &ImageFile,
    options: &ExportOptions,
) -> Result<ExportEstimateResult, String> {
    let src_path = Path::new(&file.path);
    if !src_path.exists() {
        return Err(format!("Source image does not exist: {}", file.path));
    }

    let original_bytes = match fs::metadata(src_path) {
        Ok(m) => m.len(),
        Err(_) => file.size_bytes,
    };

    let processed = process_single_image(file, options, 0)?;
    let estimated_bytes = processed.image_bytes.len() as u64;

    let (original_width, original_height) = (
        file.metadata.as_ref().and_then(|m| m.width).unwrap_or(0),
        file.metadata.as_ref().and_then(|m| m.height).unwrap_or(0),
    );

    let (output_width, output_height) = (processed.width, processed.height);

    let savings_percent = if original_bytes > 0 {
        ((original_bytes as f64 - estimated_bytes as f64) / original_bytes as f64) * 100.0
    } else {
        0.0
    };

    let format_str = match options.format {
        ExportFormat::Original => "original".to_string(),
        ExportFormat::Webp => "webp".to_string(),
        ExportFormat::Jpeg => "jpeg".to_string(),
        ExportFormat::Png => "png".to_string(),
        ExportFormat::Avif => "avif".to_string(),
    };

    Ok(ExportEstimateResult {
        original_bytes,
        estimated_bytes,
        original_width,
        original_height,
        output_width,
        output_height,
        format: format_str,
        savings_percent,
    })
}

struct ExportInputs {
    files: Vec<ImageFile>,
    errors: Vec<String>,
}

fn prepare_export_inputs(db: &Database, options: &ExportOptions) -> ExportInputs {
    let mut errors = Vec::new();
    let mut files = Vec::with_capacity(options.file_ids.len());
    for &id in &options.file_ids {
        match db.get_file_by_id(id) {
            Ok(Some(file)) => files.push(file),
            Ok(None) => errors.push(format!("File id {id} not found in database")),
            Err(e) => errors.push(format!("Database error querying file id {id}: {e}")),
        }
    }

    ExportInputs { files, errors }
}

/// Export from a shared connection without holding its guard during media/output work.
pub fn execute_shared_batch_export<P>(
    db: &Mutex<Database>,
    options: &ExportOptions,
    progress_callback: P,
) -> Result<ExportSummary, String>
where
    P: Fn(ExportProgressEvent) + Send + Sync + 'static,
{
    let start_time = Instant::now();
    let inputs = {
        let guard = db.lock().map_err(|_| "database lock poisoned")?;
        prepare_export_inputs(&guard, options)
    };
    Ok(execute_export_inputs(
        inputs,
        options,
        progress_callback,
        start_time,
    ))
}

/// Export using an exclusively owned connection (non-application callers).
pub fn execute_batch_export<P>(
    db: &Database,
    options: &ExportOptions,
    progress_callback: P,
) -> ExportSummary
where
    P: Fn(ExportProgressEvent) + Send + Sync + 'static,
{
    let start_time = Instant::now();
    execute_export_inputs(
        prepare_export_inputs(db, options),
        options,
        progress_callback,
        start_time,
    )
}

fn execute_export_inputs<P>(
    inputs: ExportInputs,
    options: &ExportOptions,
    progress_callback: P,
    start_time: Instant,
) -> ExportSummary
where
    P: Fn(ExportProgressEvent) + Send + Sync + 'static,
{
    let ExportInputs { files, mut errors } = inputs;
    let total = options.file_ids.len();
    let mut total_exported = 0;
    let mut total_bytes_written = 0u64;

    let dest_path = Path::new(&options.destination_path);
    let mut zip_writer = if options.as_zip {
        if let Some(parent) = dest_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::File::create(dest_path) {
            Ok(file) => Some(ZipWriter::new(file)),
            Err(e) => {
                errors.push(format!("Failed to create output zip file: {e}"));
                return ExportSummary {
                    success: false,
                    total_exported: 0,
                    total_failed: total,
                    total_bytes_written: 0,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    output_path: options.destination_path.clone(),
                    errors,
                };
            }
        }
    } else {
        if let Err(e) = fs::create_dir_all(dest_path) {
            errors.push(format!("Failed to create destination directory: {e}"));
            return ExportSummary {
                success: false,
                total_exported: 0,
                total_failed: total,
                total_bytes_written: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
                output_path: options.destination_path.clone(),
                errors,
            };
        }
        None
    };

    let zip_options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let processed_counter = AtomicUsize::new(0);
    let mut showcase_items = Vec::new();
    let mut used_filenames: HashSet<String> = HashSet::new();

    // Process files in bounded chunks of 16 to keep memory usage strictly bounded
    for (chunk_idx, chunk) in files.chunks(16).enumerate() {
        let chunk_offset = chunk_idx * 16;
        let results: Vec<Result<ProcessedExportItem, String>> = chunk
            .par_iter()
            .enumerate()
            .map(|(i, file)| process_single_image(file, options, chunk_offset + i))
            .collect();

        for res in results {
            let current = processed_counter.fetch_add(1, Ordering::Relaxed) + 1;
            match res {
                Ok(item) => {
                    let image_len = item.image_bytes.len() as u64;

                    // Resolve collisions by appending numeric suffixes
                    let mut final_image_filename = item.image_filename.clone();
                    let stem = Path::new(&item.image_filename)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("image")
                        .to_string();
                    let ext = Path::new(&item.image_filename)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string();

                    let mut collision_count = 1;
                    while used_filenames.contains(&final_image_filename)
                        || (!options.as_zip && dest_path.join(&final_image_filename).exists())
                    {
                        final_image_filename = if ext.is_empty() {
                            format!("{stem}_{collision_count}")
                        } else {
                            format!("{stem}_{collision_count}.{ext}")
                        };
                        collision_count += 1;
                    }
                    used_filenames.insert(final_image_filename.clone());

                    // Resolve sidecar filename if present
                    let final_sidecar = item.sidecar.map(|(sc_orig, sc_bytes)| {
                        let sc_ext = Path::new(&sc_orig)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("txt");
                        let new_sc_stem = Path::new(&final_image_filename)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(&stem);
                        let mut sc_name = format!("{new_sc_stem}.{sc_ext}");
                        let mut sc_count = 1;
                        while used_filenames.contains(&sc_name)
                            || (!options.as_zip && dest_path.join(&sc_name).exists())
                        {
                            sc_name = format!("{new_sc_stem}_{sc_count}.{sc_ext}");
                            sc_count += 1;
                        }
                        used_filenames.insert(sc_name.clone());
                        (sc_name, sc_bytes)
                    });

                    let mut write_success = false;

                    if let Some(ref mut zip) = zip_writer {
                        use std::io::Write;
                        if let Err(e) = zip.start_file(&final_image_filename, zip_options) {
                            errors
                                .push(format!("Failed to add {final_image_filename} to zip: {e}"));
                        } else if let Err(e) = zip.write_all(&item.image_bytes) {
                            errors.push(format!(
                                "Failed to write {final_image_filename} to zip: {e}"
                            ));
                        } else {
                            total_bytes_written += image_len;
                            write_success = true;

                            if let Some((ref sc_name, ref sc_bytes)) = final_sidecar {
                                if zip.start_file(sc_name, zip_options).is_ok()
                                    && zip.write_all(sc_bytes).is_ok()
                                {
                                    total_bytes_written += sc_bytes.len() as u64;
                                }
                            }
                        }
                    } else {
                        let out_image_path = dest_path.join(&final_image_filename);
                        if let Err(e) = fs::write(&out_image_path, &item.image_bytes) {
                            errors
                                .push(format!("Failed to write {}: {e}", out_image_path.display()));
                        } else {
                            total_bytes_written += image_len;
                            write_success = true;

                            if let Some((ref sc_name, ref sc_bytes)) = final_sidecar {
                                let out_sidecar_path = dest_path.join(sc_name);
                                if fs::write(&out_sidecar_path, sc_bytes).is_ok() {
                                    total_bytes_written += sc_bytes.len() as u64;
                                }
                            }
                        }
                    }

                    if write_success {
                        total_exported += 1;
                        if let Some(mut showcase) = item.showcase_item {
                            showcase.filename = final_image_filename.clone();
                            showcase_items.push(showcase);
                        }
                    }

                    progress_callback(ExportProgressEvent {
                        current,
                        total,
                        current_filename: final_image_filename,
                    });
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }
    }

    if options.export_html_showcase && !showcase_items.is_empty() {
        let title = options
            .html_title
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Omera Showcase");
        let html_content = generate_html_showcase(title, &showcase_items);
        let html_bytes = html_content.as_bytes();
        let html_len = html_bytes.len() as u64;

        if let Some(ref mut zip) = zip_writer {
            use std::io::Write;
            let _ = zip.start_file("index.html", zip_options);
            let _ = zip.write_all(html_bytes);
            total_bytes_written += html_len;
        } else {
            let out_html_path = dest_path.join("index.html");
            if let Err(e) = fs::write(&out_html_path, html_bytes) {
                errors.push(format!("Failed to write index.html: {e}"));
            } else {
                total_bytes_written += html_len;
            }
        }
    }

    if let Some(zip) = zip_writer {
        if let Err(e) = zip.finish() {
            errors.push(format!("Failed to finalize zip archive: {e}"));
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let total_failed = total.saturating_sub(total_exported);

    ExportSummary {
        success: errors.is_empty(),
        total_exported,
        total_failed,
        total_bytes_written,
        duration_ms,
        output_path: options.destination_path.clone(),
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_export_allows_concurrent_reads_and_retains_missing_id_outcomes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png");
        image::DynamicImage::new_rgb8(16, 16).save(&source).unwrap();
        let original = fs::read(&source).unwrap();
        let database = Database::connect_in_memory().unwrap();
        let folder = database.add_folder(&dir.path().to_string_lossy()).unwrap();
        database
            .upsert_file(&ImageFile {
                id: None,
                folder_id: folder.id,
                path: source.to_string_lossy().into_owned(),
                container: omera_domain::Container::Png,
                size_bytes: original.len() as u64,
                modified_at: 1,
                metadata: None,
                rating: None,
                aesthetic_score: None,
                is_favorite: false,
                is_nsfw: false,
                stack_id: None,
                stack_order: 0,
            })
            .unwrap();
        let id = database.list_files(folder.id).unwrap()[0].id.unwrap();
        let database = std::sync::Arc::new(Mutex::new(database));
        let worker_db = database.clone();
        let options = ExportOptions {
            file_ids: vec![-1, id],
            format: ExportFormat::Png,
            quality: 85,
            privacy: MetadataPrivacyMode::KeepAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".into(),
            destination_path: dir.path().join("output").to_string_lossy().into_owned(),
            as_zip: false,
            max_edge: None,
            export_html_showcase: false,
            html_title: None,
        };
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let release = std::sync::Arc::new(std::sync::Barrier::new(2));
        let worker_release = release.clone();
        let worker = std::thread::spawn(move || {
            execute_shared_batch_export(&worker_db, &options, move |_| {
                started_tx.send(()).unwrap();
                worker_release.wait();
            })
            .unwrap()
        });
        started_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let start = Instant::now();
        let query = database.try_lock().map(|db| db.get_file_by_id(id).unwrap());
        eprintln!("Concurrent batch-export query: {:?}", start.elapsed());
        release.wait();
        let summary = worker.join().unwrap();
        assert!(
            query.unwrap().is_some(),
            "output work must release the application guard"
        );
        assert_eq!(summary.total_exported, 1);
        assert_eq!(summary.total_failed, 1);
        assert!(!summary.success);
        assert!(summary
            .errors
            .iter()
            .any(|error| error.contains("File id -1")));
        assert!(dir.path().join("output/source.png").exists());
        assert_eq!(fs::read(source).unwrap(), original);
    }
    use image::{Rgb, RgbImage};
    use omera_domain::{Container, ExtractedMetadata, MetadataFormat};

    #[test]
    fn test_format_export_filename_substitution() {
        let file = ImageFile {
            id: Some(42),
            folder_id: 1,
            path: "C:\\AI_Images\\hero_cyberpunk.png".to_string(),
            container: Container::Png,
            size_bytes: 1024,
            modified_at: 1726000000,
            metadata: None,
            rating: Some(5),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let formatted = format_export_filename("{date}_{name}_r{rating}_{id}", &file, "webp", 0);
        assert!(formatted.starts_with("2024"));
        assert!(formatted.contains("hero_cyberpunk"));
        assert!(formatted.contains("r5_42"));
        assert!(formatted.ends_with(".webp"));
    }

    #[test]
    fn test_sanitize_filename_part() {
        let dirty = "my:bad/file*name?<test>|";
        let clean = sanitize_filename_part(dirty);
        assert_eq!(clean, "my_bad_file_name__test__");
    }

    #[test]
    fn test_process_single_image_webp_downscale_and_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("sample.png");

        // Create a test 200x100 RGB image
        let mut img = RgbImage::new(200, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 128, 0]);
        }
        img.save(&src_img_path).unwrap();

        let metadata = ExtractedMetadata {
            format: MetadataFormat::A1111,
            parameters: Some("beautiful sunset, masterpiece".to_string()),
            raw: None,
            prompt: Some("beautiful sunset, masterpiece".to_string()),
            negative_prompt: Some("low quality, blurry".to_string()),
            width: Some(200),
            height: Some(100),
            seed: Some("123456789".to_string()),
            steps: Some(30),
            cfg_scale: Some(7.5),
            sampler: Some("Euler a".to_string()),
            model_name: Some("dreamshaper_v8".to_string()),
            model_hash: Some("abcdef12".to_string()),
            duration_seconds: None,
            fps: None,
            video_codec: None,
        };

        let file = ImageFile {
            id: Some(1),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: fs::metadata(&src_img_path).unwrap().len(),
            modified_at: 1726000000,
            metadata: Some(metadata),
            rating: Some(4),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![1],
            format: ExportFormat::Webp,
            quality: 80,
            privacy: MetadataPrivacyMode::StripAllAiMetadata,
            sidecar: ExportSidecar::TextPrompt,
            filename_template: "{model}_{id}_{name}".to_string(),
            destination_path: dir.path().join("output.zip").to_string_lossy().to_string(),
            as_zip: true,
            max_edge: Some(100),
            export_html_showcase: true,
            html_title: Some("My Test Showcase".to_string()),
        };

        let processed = process_single_image(&file, &options, 0).unwrap();
        assert_eq!(processed.image_filename, "dreamshaper_v8_1_sample.webp");
        assert!(!processed.image_bytes.is_empty());

        // Verify decoded dimensions were downscaled to max_edge = 100
        let decoded = ImageReader::new(Cursor::new(&processed.image_bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 50);

        // Verify sidecar - StripAllAiMetadata must not leak prompt
        assert!(
            processed.sidecar.is_none(),
            "StripAllAiMetadata must not produce a prompt-bearing sidecar"
        );

        // Verify showcase item
        assert!(processed.showcase_item.is_some());
        let showcase = processed.showcase_item.unwrap();
        assert_eq!(showcase.filename, "dreamshaper_v8_1_sample.webp");
        assert_eq!(showcase.width, 100);
        assert_eq!(showcase.height, 50);
        // Privacy was StripAllAiMetadata so prompt is stripped
        assert!(showcase.prompt.is_none());
    }

    #[test]
    fn test_process_single_image_original_jpeg_downscale_preserves_jpeg() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("photo.jpg");

        let mut img = RgbImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([100, 150, 200]);
        }
        img.save(&src_img_path).unwrap();

        let file = ImageFile {
            id: Some(10),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Jpeg,
            size_bytes: fs::metadata(&src_img_path).unwrap().len(),
            modified_at: 1726000000,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![10],
            format: ExportFormat::Original,
            quality: 85,
            privacy: MetadataPrivacyMode::StripAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".to_string(),
            destination_path: dir.path().join("out").to_string_lossy().to_string(),
            as_zip: false,
            max_edge: Some(50),
            export_html_showcase: false,
            html_title: None,
        };

        let processed = process_single_image(&file, &options, 0).unwrap();
        assert_eq!(processed.image_filename, "photo.jpg");
        // Verify output is valid JPEG (SOI marker 0xFF 0xD8) and NOT PNG
        assert_eq!(&processed.image_bytes[0..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn test_process_single_image_avif_and_estimate() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("hero.png");

        let mut img = RgbImage::new(120, 120);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([50, 120, 200]);
        }
        img.save(&src_img_path).unwrap();

        let file = ImageFile {
            id: Some(42),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: fs::metadata(&src_img_path).unwrap().len(),
            modified_at: 1726000000,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![42],
            format: ExportFormat::Avif,
            quality: 70,
            privacy: MetadataPrivacyMode::KeepAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".to_string(),
            destination_path: dir.path().join("out").to_string_lossy().to_string(),
            as_zip: false,
            max_edge: Some(60),
            export_html_showcase: false,
            html_title: None,
        };

        let processed = process_single_image(&file, &options, 0).unwrap();
        assert_eq!(processed.image_filename, "hero.avif");
        assert_eq!(processed.width, 60);
        assert_eq!(processed.height, 60);

        // Verify estimate function
        let estimate = estimate_export_single_image(&file, &options).unwrap();
        assert_eq!(estimate.format, "avif");
        assert_eq!(estimate.output_width, 60);
        assert_eq!(estimate.output_height, 60);
        assert!(estimate.estimated_bytes > 0);
    }

    #[test]
    fn test_keepall_png_export_preserves_embedded_metadata_with_resize() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("source_with_prompt.png");

        // Create a PNG with embedded A1111 parameters using the png crate
        let mut img_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(Cursor::new(&mut img_data), 200, 100);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .add_text_chunk(
                    "parameters".into(),
                    "masterpiece, beautiful landscape\nNegative prompt: low quality\nSteps: 30, Sampler: Euler a, CFG scale: 7.5, Seed: 123456789, Size: 200x100, Model: dreamshaper_v8".to_string(),
                )
                .unwrap();
            let mut writer = encoder.write_header().unwrap();
            let rgb_data = vec![128u8; 200 * 100 * 3];
            writer.write_image_data(&rgb_data).unwrap();
        }
        fs::write(&src_img_path, &img_data).unwrap();

        let file = ImageFile {
            id: Some(1),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: img_data.len() as u64,
            modified_at: 1726000000,
            metadata: None, // Intentionally None to test extraction from bytes
            rating: Some(4),
            aesthetic_score: None,
            is_favorite: true,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![1],
            format: ExportFormat::Png,
            quality: 85,
            privacy: MetadataPrivacyMode::KeepAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".to_string(),
            destination_path: dir.path().to_string_lossy().to_string(),
            as_zip: false,
            max_edge: Some(100), // Force resize to trigger re-encoding
            export_html_showcase: false,
            html_title: None,
        };

        let processed = process_single_image(&file, &options, 0).unwrap();
        assert_eq!(processed.width, 100);
        assert_eq!(processed.height, 50);

        // Re-read the exported bytes and verify metadata is preserved
        let exported_container = omera_metadata::detect_container(
            &processed.image_bytes[..32.min(processed.image_bytes.len())],
        )
        .expect("Should detect PNG container");
        assert_eq!(exported_container, Container::Png);

        // Write to temp file for metadata extraction
        let exported_path = dir.path().join("exported_output.png");
        fs::write(&exported_path, &processed.image_bytes).unwrap();

        let extracted = omera_metadata::extract_metadata(exported_container, &exported_path)
            .expect("Should extract metadata from exported PNG");

        assert!(
            extracted.prompt.is_some(),
            "Exported PNG should preserve prompt metadata"
        );
        assert!(extracted.prompt.unwrap().contains("masterpiece"));
        assert!(extracted.parameters.is_some());
        assert!(extracted.parameters.unwrap().contains("Steps: 30"));
    }

    #[test]
    fn test_keepall_original_png_with_resize_preserves_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("original.png");

        // Create source PNG with parameters
        let mut img_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(Cursor::new(&mut img_data), 150, 150);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .add_text_chunk(
                    "parameters".into(),
                    "cyberpunk cityscape\nNegative prompt: blurry\nSteps: 25, Seed: 999, Model: sdxl_base".to_string(),
                )
                .unwrap();
            let mut writer = encoder.write_header().unwrap();
            let rgb_data = vec![64u8; 150 * 150 * 3];
            writer.write_image_data(&rgb_data).unwrap();
        }
        fs::write(&src_img_path, &img_data).unwrap();

        let file = ImageFile {
            id: Some(2),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: img_data.len() as u64,
            modified_at: 1726000000,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![2],
            format: ExportFormat::Original, // Keep original format (PNG)
            quality: 85,
            privacy: MetadataPrivacyMode::KeepAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".to_string(),
            destination_path: dir.path().to_string_lossy().to_string(),
            as_zip: false,
            max_edge: Some(80), // Resize forces re-encoding
            export_html_showcase: false,
            html_title: None,
        };

        let processed = process_single_image(&file, &options, 0).unwrap();
        assert_eq!(processed.width, 80);

        // Verify metadata preservation
        let exported_path = dir.path().join("exported_original.png");
        fs::write(&exported_path, &processed.image_bytes).unwrap();

        let extracted = omera_metadata::extract_metadata(Container::Png, &exported_path)
            .expect("Original PNG export should preserve metadata");

        assert!(extracted.prompt.unwrap().contains("cyberpunk"));
        assert!(extracted.negative_prompt.unwrap().contains("blurry"));
    }

    #[test]
    fn test_byte_exact_passthrough_with_keepall_no_resize() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("passthrough.png");

        // Create a specific PNG with metadata
        let mut img_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(Cursor::new(&mut img_data), 64, 64);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .add_text_chunk(
                    "parameters".into(),
                    "test prompt for passthrough".to_string(),
                )
                .unwrap();
            let mut writer = encoder.write_header().unwrap();
            let rgb_data = vec![200u8; 64 * 64 * 3];
            writer.write_image_data(&rgb_data).unwrap();
        }
        fs::write(&src_img_path, &img_data).unwrap();

        let file = ImageFile {
            id: Some(3),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: img_data.len() as u64,
            modified_at: 1726000000,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![3],
            format: ExportFormat::Original,
            quality: 85,
            privacy: MetadataPrivacyMode::KeepAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".to_string(),
            destination_path: dir.path().to_string_lossy().to_string(),
            as_zip: false,
            max_edge: None, // No resize - should trigger fast path
            export_html_showcase: false,
            html_title: None,
        };

        let processed = process_single_image(&file, &options, 0).unwrap();

        // Fast path should produce byte-exact copy
        assert_eq!(
            processed.image_bytes.len(),
            img_data.len(),
            "Byte-exact pass-through should preserve original size"
        );
        assert_eq!(
            processed.image_bytes, img_data,
            "Original + KeepAll + no resize must be byte-exact"
        );
    }

    #[test]
    fn test_absent_metadata_export_does_not_fail() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("no_metadata.png");

        // Plain PNG with no metadata chunks
        let mut img = RgbImage::new(50, 50);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 0, 0]);
        }
        img.save(&src_img_path).unwrap();

        let file = ImageFile {
            id: Some(4),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: fs::metadata(&src_img_path).unwrap().len(),
            modified_at: 1726000000,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let options = ExportOptions {
            file_ids: vec![4],
            format: ExportFormat::Png,
            quality: 85,
            privacy: MetadataPrivacyMode::KeepAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".to_string(),
            destination_path: dir.path().to_string_lossy().to_string(),
            as_zip: false,
            max_edge: Some(40),
            export_html_showcase: false,
            html_title: None,
        };

        let processed = process_single_image(&file, &options, 0).unwrap();
        assert_eq!(processed.width, 40);
        assert!(!processed.image_bytes.is_empty());
    }

    #[test]
    fn test_privacy_policy_sidecar_combinations() {
        let dir = tempfile::tempdir().unwrap();
        let src_img_path = dir.path().join("test.png");

        let mut img = RgbImage::new(64, 64);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([100, 150, 200]);
        }
        img.save(&src_img_path).unwrap();

        let metadata = ExtractedMetadata {
            format: MetadataFormat::A1111,
            parameters: Some("Steps: 20, CFG: 7".to_string()),
            raw: Some("raw metadata string".to_string()),
            prompt: Some("confidential prompt".to_string()),
            negative_prompt: Some("private negative".to_string()),
            width: Some(64),
            height: Some(64),
            seed: Some("12345".to_string()),
            steps: Some(20),
            cfg_scale: Some(7.0),
            sampler: Some("Euler".to_string()),
            model_name: Some("test-model".to_string()),
            model_hash: Some("abc123".to_string()),
            duration_seconds: None,
            fps: None,
            video_codec: None,
        };

        let file = ImageFile {
            id: Some(1),
            folder_id: 1,
            path: src_img_path.to_string_lossy().to_string(),
            container: Container::Png,
            size_bytes: fs::metadata(&src_img_path).unwrap().len(),
            modified_at: 1726000000,
            metadata: Some(metadata),
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        // Test all combinations of privacy modes with TextPrompt sidecar
        for (privacy, should_have_prompt_sidecar) in [
            (MetadataPrivacyMode::KeepAll, true),
            (MetadataPrivacyMode::StripPromptOnly, false),
            (MetadataPrivacyMode::StripAllAiMetadata, false),
            (MetadataPrivacyMode::StripAll, false),
        ] {
            let options = ExportOptions {
                file_ids: vec![1],
                format: ExportFormat::Png,
                quality: 85,
                privacy,
                sidecar: ExportSidecar::TextPrompt,
                filename_template: "{name}".to_string(),
                destination_path: dir.path().to_string_lossy().to_string(),
                as_zip: false,
                max_edge: None,
                export_html_showcase: false,
                html_title: None,
            };

            let processed = process_single_image(&file, &options, 0).unwrap();

            if should_have_prompt_sidecar {
                assert!(
                    processed.sidecar.is_some(),
                    "Privacy {privacy:?} + TextPrompt should produce sidecar"
                );
                let (_, sidecar_bytes) = processed.sidecar.unwrap();
                let sidecar_text = String::from_utf8(sidecar_bytes).unwrap();
                assert!(
                    sidecar_text.contains("confidential"),
                    "KeepAll should preserve prompt in sidecar"
                );
            } else {
                assert!(
                    processed.sidecar.is_none(),
                    "Privacy {privacy:?} + TextPrompt must not leak prompt sidecar"
                );
            }
        }

        // Test JsonMetadata sidecar with prompt-stripping policies
        for (privacy, should_have_prompt_fields) in [
            (MetadataPrivacyMode::KeepAll, true),
            (MetadataPrivacyMode::StripPromptOnly, false),
            (MetadataPrivacyMode::StripAllAiMetadata, false),
            (MetadataPrivacyMode::StripAll, false),
        ] {
            let options = ExportOptions {
                file_ids: vec![1],
                format: ExportFormat::Png,
                quality: 85,
                privacy,
                sidecar: ExportSidecar::JsonMetadata,
                filename_template: "{name}".to_string(),
                destination_path: dir.path().to_string_lossy().to_string(),
                as_zip: false,
                max_edge: None,
                export_html_showcase: false,
                html_title: None,
            };

            let processed = process_single_image(&file, &options, 0).unwrap();

            if privacy == MetadataPrivacyMode::StripAll {
                assert!(
                    processed.sidecar.is_none(),
                    "StripAll + JsonMetadata should produce no sidecar"
                );
            } else if should_have_prompt_fields {
                assert!(
                    processed.sidecar.is_some(),
                    "KeepAll + JsonMetadata should produce sidecar"
                );
                let (_, sidecar_bytes) = processed.sidecar.unwrap();
                let sidecar_text = String::from_utf8(sidecar_bytes).unwrap();
                assert!(
                    sidecar_text.contains("confidential"),
                    "KeepAll JSON should contain prompt"
                );
            } else {
                // StripPromptOnly or StripAllAiMetadata with JSON sidecar
                assert!(
                    processed.sidecar.is_some(),
                    "{privacy:?} + JsonMetadata should produce sanitized sidecar"
                );
                let (_, sidecar_bytes) = processed.sidecar.unwrap();
                let sidecar_text = String::from_utf8(sidecar_bytes).unwrap();
                assert!(
                    !sidecar_text.contains("confidential"),
                    "{privacy:?} JSON must not contain prompt"
                );
            }
        }

        // Test showcase output respects privacy
        for (privacy, should_have_prompt_in_showcase) in [
            (MetadataPrivacyMode::KeepAll, true),
            (MetadataPrivacyMode::StripPromptOnly, false),
            (MetadataPrivacyMode::StripAllAiMetadata, false),
            (MetadataPrivacyMode::StripAll, false),
        ] {
            let options = ExportOptions {
                file_ids: vec![1],
                format: ExportFormat::Png,
                quality: 85,
                privacy,
                sidecar: ExportSidecar::None,
                filename_template: "{name}".to_string(),
                destination_path: dir.path().to_string_lossy().to_string(),
                as_zip: false,
                max_edge: None,
                export_html_showcase: true,
                html_title: Some("Test Showcase".to_string()),
            };

            let processed = process_single_image(&file, &options, 0).unwrap();
            let showcase = processed.showcase_item.unwrap();

            if should_have_prompt_in_showcase {
                assert!(
                    showcase.prompt.is_some(),
                    "KeepAll showcase should contain prompt"
                );
                assert!(showcase.prompt.unwrap().contains("confidential"));
            } else {
                assert!(
                    showcase.prompt.is_none(),
                    "{privacy:?} showcase must not contain prompt"
                );
            }
        }
    }
}
