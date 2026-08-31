//! High-performance thumbnail generation with limited concurrency and disk cache management.

use image::ImageReader;
use rayon::prelude::*;
use rayon::ThreadPool;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Dedicated background thread pool with strictly limited concurrency (max 2 threads)
/// to ensure the main UI and WebView are never starved of CPU or disk I/O.
static THUMB_POOL: OnceLock<ThreadPool> = OnceLock::new();

fn get_thumb_pool() -> &'static ThreadPool {
    THUMB_POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .thread_name(|idx| format!("berry-thumb-{idx}"))
            .build()
            .expect("Failed to initialize thumbnail worker thread pool")
    })
}

/// Stats for the thumbnail cache on disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThumbnailCacheStats {
    pub total_bytes: u64,
    pub file_count: usize,
    pub cache_dir: String,
}

/// Thumbnail generation progress event payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThumbnailProgress {
    pub current: usize,
    pub total: usize,
    pub done: bool,
}

/// Compute canonical destination path for a thumbnail.
pub fn get_thumbnail_path(
    cache_dir: &Path,
    file_id: i64,
    modified_at: i64,
    max_edge: u32,
) -> PathBuf {
    let thumb_dir = cache_dir.join("thumbnails");
    thumb_dir.join(format!("{}_{}_{}.webp", file_id, modified_at, max_edge))
}

/// Generate a downscaled thumbnail and save to `dst_path` as WebP or JPEG.
pub fn generate_thumbnail(src_path: &Path, dst_path: &Path, max_edge: u32) -> Result<(), String> {
    if !src_path.exists() {
        return Err(format!(
            "Source image does not exist: {}",
            src_path.display()
        ));
    }

    // Ensure parent directory exists
    if let Some(parent) = dst_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create thumbnail dir: {e}"))?;
    }

    // Open and decode source image
    let reader = ImageReader::open(src_path)
        .map_err(|e| format!("Failed to open image {}: {e}", src_path.display()))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format for {}: {e}", src_path.display()))?;

    let img = reader
        .decode()
        .map_err(|e| format!("Failed to decode image {}: {e}", src_path.display()))?;

    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return Err("Image dimensions are zero".to_string());
    }

    // Calculate downscaled dimensions (respecting aspect ratio)
    let (target_w, target_h) = if w >= h {
        if w > max_edge {
            let scale = max_edge as f32 / w as f32;
            (max_edge, ((h as f32 * scale).round() as u32).max(1))
        } else {
            (w, h)
        }
    } else {
        if h > max_edge {
            let scale = max_edge as f32 / h as f32;
            (((w as f32 * scale).round() as u32).max(1), max_edge)
        } else {
            (w, h)
        }
    };

    // Fast Lanczos3 downscaling
    let thumb = img.thumbnail(target_w, target_h);

    // Save as WebP (or fallback to JPEG if needed)
    thumb
        .save_with_format(dst_path, image::ImageFormat::WebP)
        .or_else(|_| thumb.save_with_format(dst_path, image::ImageFormat::Jpeg))
        .map_err(|e| format!("Failed to encode thumbnail {}: {e}", dst_path.display()))?;

    Ok(())
}

/// Ensure a thumbnail exists on disk for a given file. If not present, generate it.
pub fn ensure_thumbnail(
    cache_dir: &Path,
    file_id: i64,
    file_path: &str,
    modified_at: i64,
    max_edge: u32,
) -> Result<String, String> {
    let dst = get_thumbnail_path(cache_dir, file_id, modified_at, max_edge);
    if dst.exists() {
        return Ok(dst.to_string_lossy().to_string());
    }

    let src = Path::new(file_path);
    generate_thumbnail(src, &dst, max_edge)?;

    Ok(dst.to_string_lossy().to_string())
}

/// Batch generate thumbnails in parallel using limited Rayon worker pool (max 2 threads).
pub fn batch_generate_thumbnails<F>(
    cache_dir: &Path,
    items: Vec<(i64, String, i64)>, // (file_id, file_path, modified_at)
    max_edge: u32,
    progress_callback: Option<F>,
) -> usize
where
    F: Fn(usize, usize) + Send + Sync,
{
    let total = items.len();
    if total == 0 {
        return 0;
    }

    let completed_counter = AtomicUsize::new(0);
    let pool = get_thumb_pool();

    let count = pool.install(|| {
        items
            .into_par_iter()
            .filter_map(|(file_id, file_path, modified_at)| {
                let dst = get_thumbnail_path(cache_dir, file_id, modified_at, max_edge);
                let generated = if dst.exists() {
                    false
                } else {
                    let src = Path::new(&file_path);
                    generate_thumbnail(src, &dst, max_edge).is_ok()
                };

                let current = completed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                if let Some(ref cb) = progress_callback {
                    cb(current, total);
                }

                if generated {
                    Some(1)
                } else {
                    None
                }
            })
            .count()
    });

    count
}

/// Get disk statistics for the thumbnail cache.
pub fn get_thumbnail_cache_stats(cache_dir: &Path) -> Result<ThumbnailCacheStats, String> {
    let thumb_dir = cache_dir.join("thumbnails");
    if !thumb_dir.exists() {
        return Ok(ThumbnailCacheStats {
            total_bytes: 0,
            file_count: 0,
            cache_dir: thumb_dir.to_string_lossy().to_string(),
        });
    }

    let mut total_bytes = 0u64;
    let mut file_count = 0usize;

    if let Ok(entries) = fs::read_dir(&thumb_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_bytes += meta.len();
                    file_count += 1;
                }
            }
        }
    }

    Ok(ThumbnailCacheStats {
        total_bytes,
        file_count,
        cache_dir: thumb_dir.to_string_lossy().to_string(),
    })
}

/// Clear all cached thumbnail files from disk.
pub fn clear_thumbnail_cache(cache_dir: &Path) -> Result<usize, String> {
    let thumb_dir = cache_dir.join("thumbnails");
    if !thumb_dir.exists() {
        return Ok(0);
    }

    let mut removed = 0;
    if let Ok(entries) = fs::read_dir(&thumb_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() && fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }
    }

    Ok(removed)
}
