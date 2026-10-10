//! Folder scanning and indexing orchestration.
//!
//! `omera-scan` ties the core crates together into a single operation the app
//! shell can call: walk a folder recursively, detect each media file's
//! container, extract metadata (via a pluggable extractor from
//! `omera-metadata`), persist rows through `omera-storage`, and drop rows for
//! files that disappeared from disk. The Tauri shell only wires this up.

pub mod cloud_sync;
pub mod export;
pub mod file_operations;
pub mod html_showcase;
pub mod pipeline;
pub mod scanner;
pub mod thumbnail;
pub mod transform;

pub use export::{
    estimate_export_single_image, execute_batch_export, execute_shared_batch_export,
    format_export_filename, sanitize_filename_part, ProcessedExportItem,
};
pub use html_showcase::{generate_html_showcase, ShowcaseItemMetadata};
pub use pipeline::{
    get_pipeline_cleanup_queue, harvest_pipeline_folder, harvest_pipeline_folder_with_cancellation,
    process_pipeline_cleanups, process_pipeline_cleanups_at, PipelineCleanupItemOutcome,
    PipelineCleanupReport, PipelineCollisionPolicy, PipelineError, PipelineHarvestItemOutcome,
    PipelineHarvestOptions, PipelineHarvestReport,
};
pub use scanner::{ScanError, ScanProgress, ScanStats, Scanner};
pub use thumbnail::{
    batch_generate_thumbnails, clear_thumbnail_cache, ensure_thumbnail, get_thumbnail_cache_stats,
    get_thumbnail_path, get_thumbnail_queue_diagnostics, reset_thumbnail_queue_diagnostics,
    synchronize_thumbnail_manifest, ThumbnailBatchResult, ThumbnailCacheStats, ThumbnailProgress,
    ThumbnailQueueDiagnostics, ThumbnailRequest,
};
pub use transform::{
    execute_library_batch_transform, execute_managed_import_transform,
    execute_shared_library_batch_transform, import_files_to_managed_folder, resolve_extension,
    resolve_publication_path, transform_file_staged, validate_managed_destination_folder,
    TransformError,
};
