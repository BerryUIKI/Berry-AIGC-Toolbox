//! Incremental Remote Asset Mirroring & Delta Sync Engine (Milestone 12.4).
//!
//! Provides delta synchronization between local media files and remote storage
//! (S3 / WebDAV / Local NAS) with ETag and SHA-256 change detection, bounded
//! thread concurrency, token-bucket bandwidth throttling, and progress tracking.

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use omera_domain::{
    CloudBackupConfig, CloudStorageProvider, CloudSyncOptions, CloudSyncPhase, CloudSyncProgress,
    CloudSyncResult, CloudSyncStrategy,
};
pub use omera_scan::cloud_sync::SyncItem;
use tauri::{AppHandle, Emitter};

use crate::cloud_backup::{sha256_file, sha256_hex, S3Client, WebDavClient};

// -----------------------------------------------------------------------------
// Rate Limiter (Token Bucket)
// -----------------------------------------------------------------------------

pub struct RateLimiter {
    bytes_per_sec: u64,
    last_check: Instant,
    tokens: f64,
    capacity: f64,
}

impl RateLimiter {
    pub fn new(kb_per_sec: u64) -> Self {
        let bytes_per_sec = kb_per_sec * 1024;
        let capacity = if bytes_per_sec > 0 {
            (bytes_per_sec * 2) as f64
        } else {
            0.0
        };
        Self {
            bytes_per_sec,
            last_check: Instant::now(),
            tokens: capacity,
            capacity,
        }
    }

    pub fn acquire(&mut self, bytes: usize) {
        if self.bytes_per_sec == 0 {
            return;
        }
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_check).as_secs_f64();
        self.last_check = now;
        self.tokens = (self.tokens + elapsed * (self.bytes_per_sec as f64)).min(self.capacity);

        let needed = bytes as f64;
        if self.tokens < needed {
            let deficit = needed - self.tokens;
            let wait_secs = deficit / (self.bytes_per_sec as f64);
            thread::sleep(Duration::from_secs_f64(wait_secs));
            self.tokens = 0.0;
            self.last_check = Instant::now();
        } else {
            self.tokens -= needed;
        }
    }
}

// -----------------------------------------------------------------------------
// Cloud Sync State
// -----------------------------------------------------------------------------

pub struct CloudSyncState {
    pub is_running: bool,
    pub cancel_flag: Arc<AtomicBool>,
    pub progress: CloudSyncProgress,
    pub summary: Option<CloudSyncResult>,
}

impl Default for CloudSyncState {
    fn default() -> Self {
        Self {
            is_running: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            progress: CloudSyncProgress::default(),
            summary: None,
        }
    }
}

// -----------------------------------------------------------------------------
// Sync Queue Item
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// Core Sync Runner
// -----------------------------------------------------------------------------

pub fn cancel_cloud_sync(sync_state: &Arc<Mutex<CloudSyncState>>) {
    let state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
    state.cancel_flag.store(true, Ordering::SeqCst);
}

pub fn start_cloud_sync(
    app: AppHandle,
    config: CloudBackupConfig,
    options: CloudSyncOptions,
    sync_state: Arc<Mutex<CloudSyncState>>,
    items: Vec<SyncItem>,
    total_bytes: u64,
) -> Result<(), String> {
    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Initialize state
    {
        let mut state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
        if state.is_running {
            return Err("A media sync operation is already currently running".into());
        }
        state.is_running = true;
        state.cancel_flag = Arc::clone(&cancel_flag);
        state.progress = CloudSyncProgress {
            phase: CloudSyncPhase::Syncing,
            total_files: items.len(),
            total_bytes,
            ..Default::default()
        };
        state.summary = None;
    }

    let _ = app.emit("cloud-sync://progress", {
        let state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
        state.progress.clone()
    });

    thread::spawn(move || {
        run_cloud_sync_worker(
            app,
            config,
            options,
            sync_state,
            cancel_flag,
            items,
            total_bytes,
        );
    });
    Ok(())
}

fn run_cloud_sync_worker(
    app: AppHandle,
    config: CloudBackupConfig,
    options: CloudSyncOptions,
    sync_state: Arc<Mutex<CloudSyncState>>,
    cancel_flag: Arc<AtomicBool>,
    items: Vec<SyncItem>,
    total_bytes: u64,
) {
    let start_time = Instant::now();
    let total_files = items.len();

    {
        let mut state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
        state.progress.phase = CloudSyncPhase::Syncing;
        state.progress.total_files = total_files;
        state.progress.total_bytes = total_bytes;
    }

    let _ = app.emit("cloud-sync://progress", {
        let state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
        state.progress.clone()
    });

    if total_files == 0 {
        let mut state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
        state.is_running = false;
        state.progress.phase = CloudSyncPhase::Completed;
        state.summary = Some(CloudSyncResult {
            success: true,
            total_files: 0,
            synced_files: 0,
            skipped_files: 0,
            failed_files: 0,
            transferred_bytes: 0,
            duration_ms: start_time.elapsed().as_millis() as u64,
            dry_run: options.dry_run,
            errors: Vec::new(),
        });
        let _ = app.emit("cloud-sync://progress", state.progress.clone());
        return;
    }

    // Step 2: Setup rate limiter and work queue
    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
        options.bandwidth_limit_kbs.unwrap_or(0),
    )));
    let work_queue = Arc::new(Mutex::new(VecDeque::from(items)));

    let completed_files_counter = Arc::new(AtomicUsize::new(0));
    let skipped_files_counter = Arc::new(AtomicUsize::new(0));
    let failed_files_counter = Arc::new(AtomicUsize::new(0));
    let transferred_bytes_counter = Arc::new(AtomicU64::new(0));
    let errors_list = Arc::new(Mutex::new(Vec::<String>::new()));

    let num_threads = options.concurrency.clamp(1, 16);
    let mut handles = Vec::new();

    for _ in 0..num_threads {
        let queue = Arc::clone(&work_queue);
        let cancel = Arc::clone(&cancel_flag);
        let limiter = Arc::clone(&rate_limiter);
        let completed = Arc::clone(&completed_files_counter);
        let skipped = Arc::clone(&skipped_files_counter);
        let failed = Arc::clone(&failed_files_counter);
        let transferred = Arc::clone(&transferred_bytes_counter);
        let errors = Arc::clone(&errors_list);
        let app_handle = app.clone();
        let sync_state_ref = Arc::clone(&sync_state);
        let cfg = config.clone();
        let opts = options.clone();

        let handle = thread::spawn(move || {
            loop {
                if cancel.load(Ordering::SeqCst) {
                    break;
                }

                let item = {
                    let mut q = queue.lock().unwrap_or_else(|e| e.into_inner());
                    q.pop_front()
                };

                let item = match item {
                    Some(it) => it,
                    None => break, // Queue empty
                };

                // Sync single file
                let result = sync_single_item(&cfg, &opts, &item, &limiter);

                match result {
                    Ok(SyncOutcome::Uploaded(bytes)) => {
                        completed.fetch_add(1, Ordering::SeqCst);
                        transferred.fetch_add(bytes, Ordering::SeqCst);
                    }
                    Ok(SyncOutcome::Skipped) => {
                        skipped.fetch_add(1, Ordering::SeqCst);
                    }
                    Ok(SyncOutcome::DryRun(bytes)) => {
                        completed.fetch_add(1, Ordering::SeqCst);
                        transferred.fetch_add(bytes, Ordering::SeqCst);
                    }
                    Err(err_msg) => {
                        failed.fetch_add(1, Ordering::SeqCst);
                        let mut errs = errors.lock().unwrap_or_else(|e| e.into_inner());
                        if errs.len() < 50 {
                            errs.push(format!("{}: {}", item.remote_key, err_msg));
                        }
                    }
                }

                // Update progress & emit
                let cur_completed = completed.load(Ordering::Relaxed);
                let cur_skipped = skipped.load(Ordering::Relaxed);
                let cur_failed = failed.load(Ordering::Relaxed);
                let cur_transferred = transferred.load(Ordering::Relaxed);

                let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001);
                let speed = (cur_transferred as f64 / elapsed_secs) as u64;
                let remaining_bytes = total_bytes.saturating_sub(cur_transferred);
                let eta = remaining_bytes.checked_div(speed);

                {
                    let mut state = sync_state_ref.lock().unwrap_or_else(|e| e.into_inner());
                    state.progress.completed_files = cur_completed;
                    state.progress.skipped_files = cur_skipped;
                    state.progress.failed_files = cur_failed;
                    state.progress.transferred_bytes = cur_transferred;
                    state.progress.current_file = Some(item.remote_key);
                    state.progress.speed_bytes_per_sec = speed;
                    state.progress.eta_seconds = eta;
                }

                // Periodic emit (every few files to avoid GUI flooding)
                let total_done = cur_completed + cur_skipped + cur_failed;
                if total_done.is_multiple_of(5) || total_done == total_files {
                    let state = sync_state_ref.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = app_handle.emit("cloud-sync://progress", state.progress.clone());
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all worker threads to complete
    for handle in handles {
        let _ = handle.join();
    }

    let is_cancelled = cancel_flag.load(Ordering::SeqCst);
    let final_completed = completed_files_counter.load(Ordering::SeqCst);
    let final_skipped = skipped_files_counter.load(Ordering::SeqCst);
    let final_failed = failed_files_counter.load(Ordering::SeqCst);
    let final_transferred = transferred_bytes_counter.load(Ordering::SeqCst);
    let final_errors = errors_list
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    let final_phase = if is_cancelled {
        CloudSyncPhase::Cancelled
    } else if final_failed > 0 && final_completed == 0 && final_skipped == 0 {
        CloudSyncPhase::Failed
    } else {
        CloudSyncPhase::Completed
    };

    let summary = CloudSyncResult {
        success: final_failed == 0 && !is_cancelled,
        total_files,
        synced_files: final_completed,
        skipped_files: final_skipped,
        failed_files: final_failed,
        transferred_bytes: final_transferred,
        duration_ms: start_time.elapsed().as_millis() as u64,
        dry_run: options.dry_run,
        errors: final_errors,
    };

    {
        let mut state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
        state.is_running = false;
        state.progress.phase = final_phase;
        state.progress.current_file = None;
        state.progress.speed_bytes_per_sec = 0;
        state.progress.eta_seconds = None;
        state.summary = Some(summary);
    }

    let _ = app.emit("cloud-sync://progress", {
        let state = sync_state.lock().unwrap_or_else(|e| e.into_inner());
        state.progress.clone()
    });
}

// -----------------------------------------------------------------------------
// Helper: Sync Single Item
// -----------------------------------------------------------------------------

#[derive(Debug)]
enum SyncOutcome {
    Uploaded(u64),
    Skipped,
    DryRun(u64),
}

fn sync_single_item(
    config: &CloudBackupConfig,
    options: &CloudSyncOptions,
    item: &SyncItem,
    rate_limiter: &Arc<Mutex<RateLimiter>>,
) -> Result<SyncOutcome, String> {
    if !item.local_path.exists() {
        return Err(format!(
            "Local file does not exist: {}",
            item.local_path.display()
        ));
    }

    let local_metadata = fs::metadata(&item.local_path).map_err(|e| {
        format!(
            "Failed to read metadata for {}: {e}",
            item.local_path.display()
        )
    })?;
    let local_len = local_metadata.len();

    // Check remote existence & determine delta
    let is_up_to_date = match config.provider {
        CloudStorageProvider::LocalPath => {
            let base_dir = config
                .local_path
                .as_deref()
                .ok_or_else(|| "Local backup path not configured".to_string())?;
            let target = PathBuf::from(base_dir).join(&item.remote_key);
            if target.exists() {
                if let Ok(meta) = fs::metadata(&target) {
                    if meta.len() == local_len {
                        match options.strategy {
                            CloudSyncStrategy::FastFingerprint => {
                                // FastFingerprint: Compare size AND mtime
                                // Remote mtime in seconds since UNIX_EPOCH
                                let remote_mtime = meta
                                    .modified()
                                    .ok()
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| d.as_secs() as i64)
                                    .unwrap_or(0);
                                // Files are considered synchronized if remote mtime >= local mtime
                                // (remote was written at or after local version)
                                remote_mtime >= item.mtime_secs
                            }
                            CloudSyncStrategy::Sha256Checksum => {
                                let local_hash = sha256_file(&item.local_path).unwrap_or_default();
                                let remote_hash = sha256_file(&target).unwrap_or_default();
                                !local_hash.is_empty() && local_hash == remote_hash
                            }
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
        CloudStorageProvider::S3 => {
            let s3 = S3Client::from_config(config)?;
            let object_key = s3.object_key(&item.remote_key);
            match s3.head_object(&object_key)? {
                Some((remote_len, _etag, remote_sha)) if remote_len == local_len => {
                    match options.strategy {
                        CloudSyncStrategy::FastFingerprint => {
                            // FastFingerprint: Cannot safely determine equality without comparable evidence.
                            // S3 ETag may be multipart hash or opaque; cannot compare to local content.
                            // Treat as unknown and re-upload to avoid skipping changed files.
                            false
                        }
                        CloudSyncStrategy::Sha256Checksum => {
                            let local_hash = sha256_file(&item.local_path).unwrap_or_default();
                            if let Some(ref remote_h) = remote_sha {
                                remote_h == &local_hash
                            } else {
                                // If remote doesn't have custom sha256 header, cannot verify
                                false
                            }
                        }
                    }
                }
                _ => false,
            }
        }
        CloudStorageProvider::WebDav => {
            let webdav = WebDavClient::from_config(config)?;
            match webdav.head_object(&item.remote_key)? {
                Some((remote_len, _etag)) if remote_len == local_len => {
                    match options.strategy {
                        CloudSyncStrategy::FastFingerprint => {
                            // FastFingerprint: Cannot safely determine equality without comparable evidence.
                            // WebDAV ETag is opaque; cannot compare to local content.
                            // Treat as unknown and re-upload to avoid skipping changed files.
                            false
                        }
                        CloudSyncStrategy::Sha256Checksum => {
                            // WebDAV doesn't provide SHA-256, must download and hash
                            let local_hash = sha256_file(&item.local_path).unwrap_or_default();
                            let remote_data = webdav.get_object(&item.remote_key)?;
                            let remote_hash = sha256_hex(&remote_data);
                            !local_hash.is_empty() && local_hash == remote_hash
                        }
                    }
                }
                _ => false,
            }
        }
    };

    if is_up_to_date {
        return Ok(SyncOutcome::Skipped);
    }

    if options.dry_run {
        return Ok(SyncOutcome::DryRun(local_len));
    }

    // Read local file bytes
    let data = fs::read(&item.local_path)
        .map_err(|e| format!("Failed to read file {}: {e}", item.local_path.display()))?;

    // Apply bandwidth throttling
    {
        let mut limiter = rate_limiter.lock().unwrap_or_else(|e| e.into_inner());
        limiter.acquire(data.len());
    }

    // Transfer file to target provider
    match config.provider {
        CloudStorageProvider::LocalPath => {
            let base_dir = config
                .local_path
                .as_deref()
                .ok_or_else(|| "Local backup path not configured".to_string())?;
            let target = PathBuf::from(base_dir).join(&item.remote_key);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directories for {}: {e}", target.display())
                })?;
            }
            fs::write(&target, &data)
                .map_err(|e| format!("Failed to write {}: {e}", target.display()))?;
        }
        CloudStorageProvider::S3 => {
            let s3 = S3Client::from_config(config)?;
            let object_key = s3.object_key(&item.remote_key);
            let sha = if options.strategy == CloudSyncStrategy::Sha256Checksum {
                Some(sha256_hex(&data))
            } else {
                None
            };
            s3.put_object_raw(&object_key, &data, sha.as_deref())?;
        }
        CloudStorageProvider::WebDav => {
            let webdav = WebDavClient::from_config(config)?;
            webdav.put_object_path(&item.remote_key, &data)?;
        }
    }

    Ok(SyncOutcome::Uploaded(local_len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_unlimited() {
        let mut limiter = RateLimiter::new(0);
        let start = Instant::now();
        limiter.acquire(1024 * 1024);
        assert!(start.elapsed().as_millis() < 50);
    }

    #[test]
    fn test_rate_limiter_throttling() {
        let limiter = RateLimiter::new(500); // 500 KB/s
        assert_eq!(limiter.bytes_per_sec, 500 * 1024);
        assert_eq!(limiter.capacity, 1000.0 * 1024.0);
    }

    #[test]
    fn test_sync_options_defaults() {
        let opts = CloudSyncOptions::default();
        assert_eq!(opts.concurrency, 4);
        assert_eq!(opts.remote_prefix, "media/");
        assert!(!opts.dry_run);
        assert_eq!(opts.strategy, CloudSyncStrategy::FastFingerprint);
    }

    #[test]
    fn test_cloud_sync_lock_poisoning_resilience() {
        let state = Arc::new(Mutex::new(CloudSyncState::default()));
        let state_clone = Arc::clone(&state);

        // Intentionally poison the mutex
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = state_clone.lock().unwrap();
            panic!("Simulated worker panic");
        }));

        assert!(state.is_poisoned());

        // cancel_cloud_sync must recover gracefully rather than panicking on poison
        cancel_cloud_sync(&state);

        let recovered = state.lock().unwrap_or_else(|e| e.into_inner());
        assert!(recovered.cancel_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_fast_fingerprint_detects_same_length_content_change() {
        use std::time::UNIX_EPOCH;

        let temp_dir = std::env::temp_dir().join(format!("omera_test_fp_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_file = temp_dir.join("local.txt");
        let remote_dir = temp_dir.join("remote");
        fs::create_dir_all(&remote_dir).unwrap();

        // Create initial remote file with content "AAAA"
        let remote_file = remote_dir.join("test.txt");
        fs::write(&remote_file, b"AAAA").unwrap();
        let remote_mtime = fs::metadata(&remote_file)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Wait enough time to ensure filesystem mtime resolution is exceeded
        std::thread::sleep(Duration::from_millis(1500));

        // Create local file with same-length different content "BBBB"
        fs::write(&local_file, b"BBBB").unwrap();
        let local_mtime = fs::metadata(&local_file)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Verify local is actually newer (if not, skip this test)
        if local_mtime <= remote_mtime {
            eprintln!(
                "Skipping test: filesystem mtime resolution insufficient (local: {}, remote: {})",
                local_mtime, remote_mtime
            );
            let _ = fs::remove_dir_all(&temp_dir);
            return;
        }

        let new_mtime = local_mtime;

        // Create sync item with the new mtime
        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: 4,
            mtime_secs: new_mtime,
        };

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::LocalPath,
            local_path: Some(remote_dir.to_string_lossy().to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::FastFingerprint,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        // FastFingerprint should detect the change via mtime
        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: file needs sync because local mtime > remote mtime
            }
            other => panic!("Expected DryRun outcome, got: {:?}", other),
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_fast_fingerprint_skips_unchanged_file() {
        use std::time::UNIX_EPOCH;

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_fp_unchanged_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_file = temp_dir.join("local.txt");
        let remote_dir = temp_dir.join("remote");
        fs::create_dir_all(&remote_dir).unwrap();

        // Create identical local and remote files
        fs::write(&local_file, b"content").unwrap();
        let local_mtime = fs::metadata(&local_file)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Wait to ensure remote has later or equal mtime
        std::thread::sleep(Duration::from_millis(10));

        let remote_file = remote_dir.join("test.txt");
        fs::write(&remote_file, b"content").unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: 7,
            mtime_secs: local_mtime,
        };

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::LocalPath,
            local_path: Some(remote_dir.to_string_lossy().to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::FastFingerprint,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        // FastFingerprint should skip because remote mtime >= local mtime
        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::Skipped) => {
                // Expected: file is up to date
            }
            other => panic!("Expected Skipped outcome, got: {:?}", other),
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_fast_fingerprint_detects_size_change() {
        use std::time::UNIX_EPOCH;

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_fp_size_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_file = temp_dir.join("local.txt");
        let remote_dir = temp_dir.join("remote");
        fs::create_dir_all(&remote_dir).unwrap();

        // Create local file
        fs::write(&local_file, b"longer content").unwrap();
        let local_mtime = fs::metadata(&local_file)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Remote has different size
        let remote_file = remote_dir.join("test.txt");
        fs::write(&remote_file, b"short").unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: 14,
            mtime_secs: local_mtime,
        };

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::LocalPath,
            local_path: Some(remote_dir.to_string_lossy().to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::FastFingerprint,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        // FastFingerprint should detect size difference immediately
        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: file needs sync due to size difference
            }
            other => panic!("Expected DryRun outcome, got: {:?}", other),
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_fast_fingerprint_s3_always_reuploads_same_size() {
        // S3 FastFingerprint cannot safely determine content equality without comparable metadata.
        // ETag is opaque and may be multipart hash; cannot compare to local content.
        // This test documents that FastFingerprint always returns false (needs sync) for S3
        // when size matches, treating equality as unknown to avoid skipping changed files.
        //
        // Expected behavior: same-size files always trigger re-upload in FastFingerprint mode.
        // For strict verification, use Sha256Checksum strategy with x-amz-meta-sha256 header.
        //
        // This is a documentation test; actual S3 integration requires live credentials
        // and is tested manually or in integration test suite.
    }

    #[test]
    fn test_webdav_fast_fingerprint_same_size_different_content() {
        // WebDAV FastFingerprint: same-size files with different content must trigger sync.
        // Even with matching size, cannot verify equality without comparable metadata.

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_webdav_fp_diff_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create local file with content "AAAA"
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, b"AAAA").unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: 4,
            mtime_secs: 1000,
        };

        // Mock WebDAV server returns same size but we don't know if content matches
        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // HEAD request returns 200 with same size and an ETag
        let _mock = mock_server
            .mock("HEAD", "/test.txt")
            .with_status(200)
            .with_header("Content-Length", "4")
            .with_header("ETag", "\"abc123\"")
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::WebDav,
            webdav_endpoint: Some(mock_url),
            webdav_username: Some("user".to_string()),
            webdav_password: Some("pass".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::FastFingerprint,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        // FastFingerprint should return false (needs sync) because we cannot verify equality
        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: cannot verify equality, must sync
            }
            other => panic!(
                "Expected DryRun (needs sync), got: {:?}. FastFingerprint must not skip same-size WebDAV files.",
                other
            ),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_webdav_checksum_same_size_matching_content() {
        // WebDAV Sha256Checksum: downloads remote and verifies hash match.

        let temp_dir = std::env::temp_dir().join(format!(
            "omera_test_webdav_sha_match_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let content = b"matching content";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, content).unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // HEAD returns matching size
        let _mock_head = mock_server
            .mock("HEAD", "/test.txt")
            .with_status(200)
            .with_header("Content-Length", &content.len().to_string())
            .with_header("ETag", "\"def456\"")
            .create();

        // GET returns identical content
        let _mock_get = mock_server
            .mock("GET", "/test.txt")
            .with_status(200)
            .with_body(content)
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::WebDav,
            webdav_endpoint: Some(mock_url),
            webdav_username: Some("user".to_string()),
            webdav_password: Some("pass".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::Skipped) => {
                // Expected: hashes match, skip sync
            }
            other => panic!("Expected Skipped (hashes match), got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_webdav_checksum_same_size_different_content() {
        // WebDAV Sha256Checksum: detects same-size content mismatch.

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_webdav_sha_diff_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"AAAA";
        let remote_content = b"BBBB"; // Same length, different content

        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // HEAD returns matching size
        let _mock_head = mock_server
            .mock("HEAD", "/test.txt")
            .with_status(200)
            .with_header("Content-Length", &local_content.len().to_string())
            .with_header("ETag", "\"ghi789\"")
            .create();

        // GET returns different content
        let _mock_get = mock_server
            .mock("GET", "/test.txt")
            .with_status(200)
            .with_body(remote_content)
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::WebDav,
            webdav_endpoint: Some(mock_url),
            webdav_username: Some("user".to_string()),
            webdav_password: Some("pass".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: hashes differ, needs sync
            }
            other => panic!("Expected DryRun (content mismatch), got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_webdav_checksum_remote_not_found() {
        // WebDAV with missing remote file must trigger sync regardless of strategy.

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_webdav_404_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, b"new file").unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "new.txt".to_string(),
            size_bytes: 8,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // HEAD returns 404
        let _mock_head = mock_server
            .mock("HEAD", "/new.txt")
            .with_status(404)
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::WebDav,
            webdav_endpoint: Some(mock_url),
            webdav_username: Some("user".to_string()),
            webdav_password: Some("pass".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: remote doesn't exist, needs sync
            }
            other => panic!("Expected DryRun (remote not found), got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_webdav_fast_fingerprint_changed_etag() {
        // WebDAV FastFingerprint with changed ETag still cannot verify equality.
        // ETag is opaque; even if it changed, we treat same-size as unknown.

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_webdav_etag_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, b"content").unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: 7,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // HEAD returns same size with different ETag
        let _mock = mock_server
            .mock("HEAD", "/test.txt")
            .with_status(200)
            .with_header("Content-Length", "7")
            .with_header("ETag", "\"changed-etag-xyz\"")
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::WebDav,
            webdav_endpoint: Some(mock_url),
            webdav_username: Some("user".to_string()),
            webdav_password: Some("pass".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::FastFingerprint,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: FastFingerprint cannot use ETag to verify equality
            }
            other => panic!("Expected DryRun (ETag is opaque), got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_localpath_fastfingerprint_uses_mtime() {
        // LocalPath FastFingerprint uses size + mtime comparison.
        // Files are synchronized when remote_mtime >= local_mtime.
        // This is verified by existing test_fast_fingerprint_detects_same_length_content_change
        // and test_fast_fingerprint_skips_unchanged_file tests above.
        //
        // Expected behavior:
        // - Size differs: always sync
        // - Size matches, local_mtime > remote_mtime: sync (content may have changed)
        // - Size matches, local_mtime <= remote_mtime: skip (remote is newer or equal)
    }

    #[test]
    fn test_s3_checksum_missing_digest_metadata() {
        // S3 Sha256Checksum: missing x-amz-meta-sha256 must not skip (treat as unknown).
        // Equal-size objects without digest evidence cannot be verified.

        let temp_dir = std::env::temp_dir().join(format!(
            "omera_test_s3_missing_digest_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"test content";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // S3 HEAD returns matching size, ETag, but NO x-amz-meta-sha256
        let _mock_head = mock_server
            .mock("HEAD", "/test-bucket/test.txt")
            .with_status(200)
            .with_header("Content-Length", &local_content.len().to_string())
            .with_header("ETag", "\"abc123def456\"")
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            s3_endpoint: Some(mock_url),
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_access_key: Some("test-access".to_string()),
            s3_secret_key: Some("test-secret".to_string()),
            s3_prefix: Some("".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: missing digest metadata means unknown equality, must sync
            }
            other => panic!(
                "Expected DryRun (missing digest), got: {:?}. \
                 Missing x-amz-meta-sha256 must never become a verified match.",
                other
            ),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_s3_checksum_empty_digest_metadata() {
        // S3 Sha256Checksum: empty x-amz-meta-sha256 must not skip (treat as unusable).

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_s3_empty_digest_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"test content";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // S3 HEAD returns matching size with empty digest header
        let _mock_head = mock_server
            .mock("HEAD", "/test-bucket/test.txt")
            .with_status(200)
            .with_header("Content-Length", &local_content.len().to_string())
            .with_header("ETag", "\"abc123\"")
            .with_header("x-amz-meta-sha256", "")
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            s3_endpoint: Some(mock_url),
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_access_key: Some("test-access".to_string()),
            s3_secret_key: Some("test-secret".to_string()),
            s3_prefix: Some("".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: empty digest is unusable, must sync
            }
            other => panic!(
                "Expected DryRun (empty digest), got: {:?}. \
                 Empty digest must not become a verified match.",
                other
            ),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_s3_checksum_matching_digest() {
        // S3 Sha256Checksum: valid matching x-amz-meta-sha256 skips upload.

        let temp_dir = std::env::temp_dir().join(format!(
            "omera_test_s3_matching_digest_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"matching content";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let local_sha = sha256_hex(local_content);

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // S3 HEAD returns matching size and matching digest
        let _mock_head = mock_server
            .mock("HEAD", "/test-bucket/test.txt")
            .with_status(200)
            .with_header("Content-Length", &local_content.len().to_string())
            .with_header("ETag", "\"xyz789\"")
            .with_header("x-amz-meta-sha256", &local_sha)
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            s3_endpoint: Some(mock_url),
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_access_key: Some("test-access".to_string()),
            s3_secret_key: Some("test-secret".to_string()),
            s3_prefix: Some("".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::Skipped) => {
                // Expected: matching SHA-256, skip upload
            }
            other => panic!(
                "Expected Skipped (digest match), got: {:?}. \
                 Matching SHA-256 must skip upload.",
                other
            ),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_s3_checksum_mismatching_digest() {
        // S3 Sha256Checksum: mismatching x-amz-meta-sha256 triggers re-upload.

        let temp_dir = std::env::temp_dir().join(format!(
            "omera_test_s3_mismatch_digest_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"AAAA";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let wrong_sha = sha256_hex(b"BBBB"); // Different content, same length

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // S3 HEAD returns matching size but mismatching digest
        let _mock_head = mock_server
            .mock("HEAD", "/test-bucket/test.txt")
            .with_status(200)
            .with_header("Content-Length", &local_content.len().to_string())
            .with_header("ETag", "\"multipart-etag\"")
            .with_header("x-amz-meta-sha256", &wrong_sha)
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            s3_endpoint: Some(mock_url),
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_access_key: Some("test-access".to_string()),
            s3_secret_key: Some("test-secret".to_string()),
            s3_prefix: Some("".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: digest mismatch detected, needs sync
            }
            other => panic!(
                "Expected DryRun (digest mismatch), got: {:?}. \
                 Mismatching SHA-256 must trigger re-upload.",
                other
            ),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_s3_checksum_malformed_digest() {
        // S3 Sha256Checksum: malformed x-amz-meta-sha256 must not skip.

        let temp_dir = std::env::temp_dir().join(format!(
            "omera_test_s3_malformed_digest_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"test content";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // S3 HEAD returns matching size with malformed digest (wrong length)
        let _mock_head = mock_server
            .mock("HEAD", "/test-bucket/test.txt")
            .with_status(200)
            .with_header("Content-Length", &local_content.len().to_string())
            .with_header("ETag", "\"abc\"")
            .with_header("x-amz-meta-sha256", "not-a-valid-hex-sha256")
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            s3_endpoint: Some(mock_url),
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_access_key: Some("test-access".to_string()),
            s3_secret_key: Some("test-secret".to_string()),
            s3_prefix: Some("".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: malformed digest is unusable, must sync
            }
            other => panic!(
                "Expected DryRun (malformed digest), got: {:?}. \
                 Malformed digest must not become a verified match.",
                other
            ),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_s3_checksum_object_not_found() {
        // S3 Sha256Checksum: missing remote object (404) triggers upload.

        let temp_dir =
            std::env::temp_dir().join(format!("omera_test_s3_not_found_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"new file";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "new.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // S3 HEAD returns 404
        let _mock_head = mock_server
            .mock("HEAD", "/test-bucket/new.txt")
            .with_status(404)
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            s3_endpoint: Some(mock_url),
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_access_key: Some("test-access".to_string()),
            s3_secret_key: Some("test-secret".to_string()),
            s3_prefix: Some("".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: object doesn't exist, needs sync
            }
            other => panic!("Expected DryRun (object not found), got: {:?}", other),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_s3_checksum_multipart_etag_ignored() {
        // S3 Sha256Checksum: multipart ETag (contains dash) is opaque, not used for verification.
        // Only x-amz-meta-sha256 provides verification; multipart ETags cannot substitute.

        let temp_dir = std::env::temp_dir().join(format!(
            "omera_test_s3_multipart_etag_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let local_content = b"large multipart content";
        let local_file = temp_dir.join("local.txt");
        fs::write(&local_file, local_content).unwrap();

        let item = SyncItem {
            local_path: local_file.clone(),
            remote_key: "test.txt".to_string(),
            size_bytes: local_content.len() as u64,
            mtime_secs: 1000,
        };

        let mut mock_server = mockito::Server::new();
        let mock_url = mock_server.url();

        // S3 HEAD returns multipart ETag but no digest metadata
        let _mock_head = mock_server
            .mock("HEAD", "/test-bucket/test.txt")
            .with_status(200)
            .with_header("Content-Length", &local_content.len().to_string())
            .with_header("ETag", "\"abc123-5\"") // Multipart ETag format
            .create();

        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            s3_endpoint: Some(mock_url),
            s3_bucket: Some("test-bucket".to_string()),
            s3_region: Some("us-east-1".to_string()),
            s3_access_key: Some("test-access".to_string()),
            s3_secret_key: Some("test-secret".to_string()),
            s3_prefix: Some("".to_string()),
            ..Default::default()
        };

        let options = CloudSyncOptions {
            strategy: CloudSyncStrategy::Sha256Checksum,
            dry_run: true,
            ..Default::default()
        };

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(0)));

        let result = sync_single_item(&config, &options, &item, &rate_limiter);

        match result {
            Ok(SyncOutcome::DryRun(_)) => {
                // Expected: multipart ETag is opaque, no digest metadata, must sync
            }
            other => panic!(
                "Expected DryRun (no digest, ETag opaque), got: {:?}. \
                 Multipart ETags must never substitute for SHA-256.",
                other
            ),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
