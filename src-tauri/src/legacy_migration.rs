//! Migration coordinator, state management, and platform cleanup for Berry -> Omera.
use crate::config_store;
use omera_domain::{
    CleanupItem, LegacyCleanupPreview, LegacyCleanupResult, LegacyMigrationJob,
    LegacyMigrationPreview, LegacyMigrationStatus, MigratedArtifact, MigrationError,
    MigrationReceipt,
};
use omera_storage::legacy_migration as storage_migration;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

/// State holder for active and cached migration plans and operations.
#[derive(Default)]
pub struct MigrationCoordinator {
    pub active_job: Option<LegacyMigrationJob>,
    pub active_plans: HashMap<String, LegacyMigrationPreview>,
    pub active_cleanup_previews: HashMap<String, LegacyCleanupPreview>,
    pub migration_lock: Arc<Mutex<()>>,
}

impl MigrationCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve the overall migration lifecycle status for the application.
    pub fn get_status(&mut self, app: &AppHandle) -> Result<LegacyMigrationStatus, String> {
        let (omera_root, candidates) = resolve_migration_roots(app)?;
        self.get_status_for_roots(&omera_root, &candidates)
    }

    /// Internal method to retrieve migration lifecycle status given specific roots (testable without AppHandle).
    pub fn get_status_for_roots(
        &mut self,
        omera_root: &Path,
        candidates: &[(String, PathBuf)],
    ) -> Result<LegacyMigrationStatus, String> {
        let destination_exists = omera_root.join("omera.db").exists();
        let active_receipt = storage_migration::load_receipt(omera_root).ok().flatten();

        let mut discovered_sources = Vec::new();
        for (identifier, path) in candidates {
            if path.exists() {
                if let Ok(Some(src)) = storage_migration::discover_database_source(path, identifier)
                {
                    discovered_sources.push(src);
                }
            }
        }

        let dest_db_path = omera_root.join("omera.db");
        let destination_is_empty =
            destination_exists && storage_migration::is_database_empty(&dest_db_path);

        let stage = if let Some(ref receipt) = active_receipt {
            if receipt.cleanup_status == "pending" {
                "cleanup_available".to_string()
            } else {
                "migrated".to_string()
            }
        } else if let Some(ref job) = self.active_job {
            job.status.clone()
        } else if discovered_sources.is_empty() {
            if destination_exists && !destination_is_empty {
                "migrated".to_string()
            } else {
                "none".to_string()
            }
        } else if destination_exists && !destination_is_empty {
            "migrated".to_string()
        } else if discovered_sources.len() > 1 {
            "awaiting_source_choice".to_string()
        } else {
            "discovered".to_string()
        };

        let mut available_actions = Vec::new();
        if stage == "cleanup_available" {
            available_actions.push("preview_cleanup".to_string());
            available_actions.push("defer_cleanup".to_string());
        } else if stage == "discovered" || stage == "awaiting_source_choice" {
            available_actions.push("preview_migration".to_string());
        }

        Ok(LegacyMigrationStatus {
            stage,
            discovered_sources,
            destination_exists,
            active_receipt,
            available_actions,
        })
    }

    /// Attempt automatic migration on startup if omera.db does not exist yet
    /// and there is exactly one unambiguous, unlocked legacy source database.
    pub fn auto_migrate_if_unambiguous(&mut self, app: &AppHandle) -> Result<bool, String> {
        let (omera_root, _) = resolve_migration_roots(app)?;
        if omera_root.join("omera.db").exists() {
            return Ok(false);
        }

        let status = self.get_status(app)?;
        if status.discovered_sources.len() == 1 {
            let source = &status.discovered_sources[0];
            if !source.is_locked {
                let preview = self.preview_migration(app, &source.source_id)?;
                if preview.conflicts.is_empty() {
                    let job = self.start_migration(app, &preview.plan_id)?;
                    if job.status == "completed" {
                        eprintln!(
                            "Auto-migrated legacy Berry library from {} to {}",
                            source.database_path,
                            omera_root.join("omera.db").display()
                        );
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Preview a migration plan for a specific discovered source.
    pub fn preview_migration(
        &mut self,
        app: &AppHandle,
        source_id: &str,
    ) -> Result<LegacyMigrationPreview, String> {
        let (omera_root, candidates) = resolve_migration_roots(app)?;
        self.preview_migration_for_roots(&omera_root, &candidates, source_id)
    }

    /// Internal method to preview a migration plan given specific roots (testable without AppHandle).
    pub fn preview_migration_for_roots(
        &mut self,
        omera_root: &Path,
        candidates: &[(String, PathBuf)],
        source_id: &str,
    ) -> Result<LegacyMigrationPreview, String> {
        let mut target_source = None;
        for (identifier, path) in candidates {
            if let Ok(Some(src)) = storage_migration::discover_database_source(path, identifier) {
                if src.source_id == source_id {
                    target_source = Some(src);
                    break;
                }
            }
        }

        let source = target_source.ok_or_else(|| {
            format!("Discovered source '{source_id}' not found among candidate roots")
        })?;

        let mut conflicts = Vec::new();
        let destination_db = omera_root.join("omera.db");
        if destination_db.exists() && !storage_migration::is_database_empty(&destination_db) {
            conflicts.push("Target database omera.db already exists. An explicit import or manual recovery is required.".into());
        }
        if source.is_locked {
            conflicts.push("Source database is active or locked by another process. Please close Berry AI Studio before migrating.".into());
        }

        let exclusions = vec![
            "External linked directories and custom storage vaults are preserved in place and excluded from migration staging.".into(),
            "Snapshot backup archives (*.zip) are preserved in their original location.".into(),
        ];

        let required_space_bytes = source.database_size_bytes * 2 + 1024 * 1024; // DB copy + staging buffer
        let available_space_bytes = get_available_disk_space(omera_root).unwrap_or(u64::MAX);

        if available_space_bytes < required_space_bytes {
            conflicts.push(format!(
                "Insufficient disk space on destination volume: required {} bytes, available {} bytes",
                required_space_bytes, available_space_bytes
            ));
        }

        let plan_id = uuid::Uuid::new_v4().to_string();
        let preview = LegacyMigrationPreview {
            plan_id: plan_id.clone(),
            source,
            destination_root: omera_root.to_string_lossy().to_string(),
            destination_db: destination_db.to_string_lossy().to_string(),
            required_space_bytes,
            available_space_bytes,
            conflicts,
            exclusions,
        };

        self.active_plans.insert(plan_id, preview.clone());
        Ok(preview)
    }

    /// Execute a verified migration plan and record a durable receipt.
    pub fn start_migration(
        &mut self,
        _app: &AppHandle,
        plan_id: &str,
    ) -> Result<LegacyMigrationJob, String> {
        let plan =
            self.active_plans.get(plan_id).cloned().ok_or_else(|| {
                format!("Migration plan '{plan_id}' has expired or was not found")
            })?;

        if !plan.conflicts.is_empty() {
            let conflict_msg = plan.conflicts.join("; ");
            return Err(format!(
                "Cannot proceed with migration due to conflicts: {conflict_msg}"
            ));
        }

        self.execute_migration_plan(&plan)
    }

    /// Internal execution engine for a migration plan.
    pub fn execute_migration_plan(
        &mut self,
        plan: &LegacyMigrationPreview,
    ) -> Result<LegacyMigrationJob, String> {
        let _lock = self
            .migration_lock
            .try_lock()
            .map_err(|_| "Another migration is currently in progress")?;

        let job_id = uuid::Uuid::new_v4().to_string();
        let mut job = LegacyMigrationJob {
            job_id: job_id.clone(),
            plan_id: plan.plan_id.clone(),
            status: "copying".to_string(),
            progress: 0.1,
            current_step: "Staging and validating SQLite database".to_string(),
            error: None,
            receipt: None,
        };
        self.active_job = Some(job.clone());

        let source_root = PathBuf::from(&plan.source.root_path);
        let source_db = PathBuf::from(&plan.source.database_path);
        let destination_root = PathBuf::from(&plan.destination_root);
        let destination_db = PathBuf::from(&plan.destination_db);

        std::fs::create_dir_all(&destination_root).map_err(|e| e.to_string())?;
        let staging_dir = tempfile::tempdir_in(&destination_root).map_err(|e| e.to_string())?;

        // If destination exists but is an empty, unpopulated database created by startup
        // without an active receipt, safely remove the empty stub so the migrated database can publish.
        if destination_db.exists() && storage_migration::is_database_empty(&destination_db) {
            let active_receipt = storage_migration::load_receipt(&destination_root)
                .ok()
                .flatten();
            if active_receipt.is_none() {
                let _ = std::fs::remove_file(&destination_db);
                let _ = std::fs::remove_file(destination_root.join("omera.db-wal"));
                let _ = std::fs::remove_file(destination_root.join("omera.db-shm"));
            }
        }

        // 1. Stage and migrate SQLite database
        let db_size = match storage_migration::migrate_database(
            &source_db,
            &destination_db,
            staging_dir.path(),
        ) {
            Ok(size) => size,
            Err(e) => {
                let err = MigrationError::new(
                    e.code.clone(),
                    e.message_key.clone(),
                    e.retryable,
                    e.context.clone(),
                );
                job.status = "failed".to_string();
                job.error = Some(err);
                self.active_job = Some(job.clone());
                return Err(format!("Database migration failed: {e}"));
            }
        };

        job.progress = 0.5;
        job.status = "validating".to_string();
        job.current_step = "Validating migrated database and configuration".to_string();
        self.active_job = Some(job.clone());

        // Compute SHA-256 integrity hash of published database
        let integrity_hash = compute_file_sha256(&destination_db).unwrap_or_default();

        let mut artifacts = vec![MigratedArtifact {
            category: "database".to_string(),
            source_path: source_db.to_string_lossy().to_string(),
            destination_path: destination_db.to_string_lossy().to_string(),
            status: "success".to_string(),
            size_bytes: db_size,
        }];

        // 2. Migrate configuration if present
        let source_config_path = source_root.join("config.json");
        let dest_config_path = destination_root.join("config.json");
        if source_config_path.exists() {
            let config_size = std::fs::metadata(&source_config_path)
                .map(|m| m.len())
                .unwrap_or(0);

            match config_store::load_readonly(&source_config_path) {
                Ok(legacy_cfg) => {
                    let dest_already_existed = dest_config_path.exists();

                    // Migrate credentials into new Omera keyring
                    let cred_res = config_store::migrate_credentials_to_omera(&legacy_cfg);
                    let save_res = config_store::save_migrated(&dest_config_path, legacy_cfg);

                    let (status, failure_err) = match (cred_res, save_res) {
                        (Err(cred_err), _) => {
                            eprintln!("Error migrating credentials to Omera: {cred_err}");
                            (
                                "failed".to_string(),
                                Some(format!("Credential migration failed: {cred_err}")),
                            )
                        }
                        (_, Err(save_err)) => {
                            eprintln!("Error persisting migrated configuration: {save_err}");
                            (
                                "failed".to_string(),
                                Some(format!("Config persistence failed: {save_err}")),
                            )
                        }
                        (Ok(_), Ok(_)) => {
                            // Verify destination config exists and is parseable
                            match config_store::load_readonly(&dest_config_path) {
                                Ok(_) => {
                                    if dest_already_existed {
                                        ("skipped".to_string(), None)
                                    } else {
                                        ("success".to_string(), None)
                                    }
                                }
                                Err(readback_err) => {
                                    eprintln!("Error validating destination configuration: {readback_err}");
                                    ("failed".to_string(), Some(format!("Config destination verification failed: {readback_err}")))
                                }
                            }
                        }
                    };

                    if let Some(err_msg) = failure_err {
                        job.status = "failed".to_string();
                        job.error = Some(MigrationError::new(
                            "CONFIG_MIGRATION_FAILED",
                            "error.migration.config_failed",
                            true,
                            Some(err_msg),
                        ));
                    }

                    artifacts.push(MigratedArtifact {
                        category: "config".to_string(),
                        source_path: source_config_path.to_string_lossy().to_string(),
                        destination_path: dest_config_path.to_string_lossy().to_string(),
                        status,
                        size_bytes: config_size,
                    });
                }
                Err(e) => {
                    eprintln!(
                        "Warning: could not read legacy config {}: {e}",
                        source_config_path.display()
                    );
                    job.status = "failed".to_string();
                    job.error = Some(MigrationError::new(
                        "CONFIG_MIGRATION_FAILED",
                        "error.migration.config_failed",
                        true,
                        Some(format!("Could not read legacy config: {e}")),
                    ));
                    artifacts.push(MigratedArtifact {
                        category: "config".to_string(),
                        source_path: source_config_path.to_string_lossy().to_string(),
                        destination_path: dest_config_path.to_string_lossy().to_string(),
                        status: "failed".to_string(),
                        size_bytes: config_size,
                    });
                }
            }
        }

        job.progress = 0.8;
        job.status = "activating".to_string();
        job.current_step = "Writing durable migration receipt".to_string();
        self.active_job = Some(job.clone());

        // 3. Write durable migration receipt
        let receipt_id = uuid::Uuid::new_v4().to_string();
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let receipt = MigrationReceipt {
            receipt_id,
            source_id: plan.source.source_id.clone(),
            source_identifier: plan.source.identifier.clone(),
            source_root: source_root.to_string_lossy().to_string(),
            source_schema_version: plan.source.schema_version,
            destination_root: destination_root.to_string_lossy().to_string(),
            destination_db: destination_db.to_string_lossy().to_string(),
            created_at: now_sec,
            artifacts,
            integrity_hash,
            cleanup_status: "pending".to_string(),
        };

        if let Err(e) = storage_migration::save_receipt(&destination_root, &receipt) {
            job.status = "failed".to_string();
            job.error = Some(MigrationError::new(
                "INTEGRITY_FAILED",
                "error.migration.receipt_failed",
                false,
                Some(e.clone()),
            ));
            self.active_job = Some(job.clone());
            return Err(format!("Failed to record durable receipt: {e}"));
        }

        job.progress = 1.0;
        if job.error.is_some() {
            job.status = "failed".to_string();
            job.current_step = "Migration failed during configuration persistence".to_string();
        } else {
            job.status = "completed".to_string();
            job.current_step = "Migration completed successfully".to_string();
        }
        job.receipt = Some(receipt);
        self.active_job = Some(job.clone());

        Ok(job)
    }

    /// Retrieve status of an active or recent migration job.
    pub fn get_job(&self, job_id: &str) -> Result<LegacyMigrationJob, String> {
        self.active_job
            .as_ref()
            .filter(|j| j.job_id == job_id)
            .cloned()
            .ok_or_else(|| format!("Job '{job_id}' not found"))
    }

    /// Preview eligible legacy application-owned files for user-approved cleanup.
    pub fn preview_cleanup(
        &mut self,
        app: &AppHandle,
        receipt_id: &str,
    ) -> Result<LegacyCleanupPreview, String> {
        let (omera_root, _) = resolve_migration_roots(app)?;
        self.preview_cleanup_for_root(&omera_root, receipt_id)
    }

    /// Internal method to preview cleanup given a specific omera_root (testable without AppHandle).
    pub fn preview_cleanup_for_root(
        &mut self,
        omera_root: &Path,
        receipt_id: &str,
    ) -> Result<LegacyCleanupPreview, String> {
        let receipt = storage_migration::load_receipt(omera_root)?
            .filter(|r| r.receipt_id == receipt_id)
            .ok_or_else(|| format!("Receipt '{receipt_id}' not found"))?;

        // 1. Verify destination health
        let dest_db = PathBuf::from(&receipt.destination_db);
        let destination_healthy = storage_migration::verify_destination_health(&dest_db)
            .map_err(|e| format!("Destination check failed: {e}"))?;

        if !destination_healthy {
            return Err("Destination database is unhealthy or missing; cleanup blocked to protect source data.".into());
        }

        // 2. Scan source root and collect eligible app-owned files
        let source_root = PathBuf::from(&receipt.source_root);
        if !source_root.exists() {
            return Err(format!(
                "Source root '{}' no longer exists",
                source_root.display()
            ));
        }

        let mut items = Vec::new();
        let mut total_size_bytes = 0u64;

        if let Ok(entries) = std::fs::read_dir(&source_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Whitelisted application-owned categories
                let (category, eligible, reason) = match name_str.as_ref() {
                    "berry.db" | "berry.db-wal" | "berry.db-shm" => {
                        ("database".to_string(), true, None)
                    }
                    "config.json" | "config.json.bak" => {
                        // Configuration is only eligible for cleanup if the migration receipt
                        // contains a successfully verified or preserved config artifact, AND
                        // the destination configuration file actually exists and is readable.
                        let dest_config = omera_root.join("config.json");
                        let config_artifact_ok = receipt.artifacts.iter().any(|a| {
                            a.category == "config"
                                && (a.status == "success" || a.status == "skipped")
                        });
                        let dest_config_healthy = dest_config.exists()
                            && config_store::load_readonly(&dest_config).is_ok();

                        if config_artifact_ok && dest_config_healthy {
                            ("config".to_string(), true, None)
                        } else {
                            (
                                "config".to_string(),
                                false,
                                Some("Destination configuration not verified or missing".into()),
                            )
                        }
                    }
                    "thumbnails" => ("thumbnails".to_string(), true, None),
                    "models" => {
                        // Model directory is only eligible for cleanup if the destination model
                        // directory exists, is non-empty, and has healthy artifacts matching source.
                        let dest_models = omera_root.join("models");
                        match storage_migration::validate_model_artifact_health(&path, &dest_models)
                        {
                            Ok(true) => ("models".to_string(), true, None),
                            _ => (
                                "models".to_string(),
                                false,
                                Some(
                                    "Destination models not verified, missing or incomplete".into(),
                                ),
                            ),
                        }
                    }
                    "updates" => ("updates".to_string(), true, None),
                    other => {
                        // User assets, external vaults, backups, or unexpected files
                        if other.ends_with(".zip") {
                            (
                                "backup".to_string(),
                                false,
                                Some("Preserved backup archive".into()),
                            )
                        } else {
                            (
                                "user_data".to_string(),
                                false,
                                Some("Protected user directory or asset".into()),
                            )
                        }
                    }
                };

                let size = if path.is_file() {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                } else if path.is_dir() {
                    calculate_folder_size(&path)
                } else {
                    0
                };

                if eligible {
                    total_size_bytes += size;
                }

                items.push(CleanupItem {
                    path: path.to_string_lossy().to_string(),
                    category,
                    size_bytes: size,
                    eligible,
                    reason,
                });
            }
        }

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let preview_id = uuid::Uuid::new_v4().to_string();
        let preview = LegacyCleanupPreview {
            preview_id: preview_id.clone(),
            receipt_id: receipt_id.to_string(),
            expires_at: now_sec + 300, // Valid for 5 minutes
            items,
            total_size_bytes,
            destination_healthy,
        };

        self.active_cleanup_previews
            .insert(preview_id, preview.clone());
        Ok(preview)
    }

    /// Execute user-confirmed cleanup of eligible legacy artifacts using system Trash.
    pub fn confirm_cleanup(
        &mut self,
        app: &AppHandle,
        preview_id: &str,
        confirmed: bool,
    ) -> Result<LegacyCleanupResult, String> {
        let (omera_root, _) = resolve_migration_roots(app)?;
        self.confirm_cleanup_for_root(&omera_root, preview_id, confirmed)
    }

    /// Internal method to execute user-confirmed cleanup given an omera_root (testable without AppHandle).
    pub fn confirm_cleanup_for_root(
        &mut self,
        omera_root: &Path,
        preview_id: &str,
        confirmed: bool,
    ) -> Result<LegacyCleanupResult, String> {
        if !confirmed {
            return Err("Cleanup was not confirmed by user".into());
        }

        let preview = self
            .active_cleanup_previews
            .remove(preview_id)
            .ok_or_else(|| format!("Cleanup preview '{preview_id}' not found or expired"))?;

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now_sec > preview.expires_at {
            return Err("Cleanup preview has expired. Please review and preview again.".into());
        }

        let mut receipt = storage_migration::load_receipt(omera_root)?
            .filter(|r| r.receipt_id == preview.receipt_id)
            .ok_or_else(|| format!("Receipt '{}' not found", preview.receipt_id))?;

        // Re-verify destination database health immediately prior to disposal
        let dest_db = PathBuf::from(&receipt.destination_db);
        if !storage_migration::verify_destination_health(&dest_db).unwrap_or(false) {
            return Err("Destination health verification failed immediately prior to cleanup. Operation aborted.".into());
        }

        let mut cleaned_count = 0;
        let mut failed_count = 0;
        let mut cleaned_bytes = 0u64;
        let mut errors = Vec::new();

        let source_root_canon = PathBuf::from(&receipt.source_root)
            .canonicalize()
            .map_err(|e| e.to_string())?;

        for item in preview.items {
            if !item.eligible {
                continue;
            }

            let path = PathBuf::from(&item.path);
            if !path.exists() {
                continue;
            }

            // Ensure path does not escape the legacy root
            if let Ok(canon) = path.canonicalize() {
                if !canon.starts_with(&source_root_canon) {
                    errors.push(format!(
                        "Rejected path outside source root: {}",
                        path.display()
                    ));
                    failed_count += 1;
                    continue;
                }
            } else {
                continue;
            }

            // Secondary defense: revalidate destination artifact health at execution time
            match item.category.as_str() {
                "config" => {
                    let dest_config = omera_root.join("config.json");
                    let config_artifact_ok = receipt.artifacts.iter().any(|a| {
                        a.category == "config" && (a.status == "success" || a.status == "skipped")
                    });
                    if !config_artifact_ok
                        || !dest_config.exists()
                        || config_store::load_readonly(&dest_config).is_err()
                    {
                        errors.push(format!(
                            "Execution skipped cleanup for config: destination unverified at {}",
                            dest_config.display()
                        ));
                        failed_count += 1;
                        continue;
                    }
                }
                "models" => {
                    let dest_models = omera_root.join("models");
                    if !storage_migration::validate_model_artifact_health(&path, &dest_models)
                        .unwrap_or(false)
                    {
                        errors.push(format!(
                            "Execution skipped cleanup for models: destination unverified or incomplete at {}",
                            dest_models.display()
                        ));
                        failed_count += 1;
                        continue;
                    }
                }
                "database"
                    if !storage_migration::verify_destination_health(&dest_db).unwrap_or(false) =>
                {
                    errors.push(format!(
                        "Execution skipped cleanup for database: destination unhealthy at {}",
                        dest_db.display()
                    ));
                    failed_count += 1;
                    continue;
                }
                _ => {}
            }

            // Move to system Trash. NEVER use permanent deletion!
            match trash::delete(&path) {
                Ok(_) => {
                    cleaned_count += 1;
                    cleaned_bytes += item.size_bytes;
                }
                Err(e) => {
                    failed_count += 1;
                    errors.push(format!("Could not trash {}: {e}", path.display()));
                }
            }
        }

        // Clean up legacy credentials safely
        let dest_config_path = omera_root.join("config.json");
        if dest_config_path.exists() {
            if let Ok(config) = config_store::load(&dest_config_path) {
                let _ = config_store::cleanup_legacy_credentials(&config);
            }
        }

        let status = if failed_count == 0 {
            "cleaned".to_string()
        } else {
            "partially_cleaned".to_string()
        };

        receipt.cleanup_status = status.clone();
        let _ = storage_migration::save_receipt(omera_root, &receipt);

        Ok(LegacyCleanupResult {
            receipt_id: preview.receipt_id,
            cleaned_count,
            failed_count,
            cleaned_bytes,
            errors,
            status,
        })
    }

    /// Persist user decision to defer cleanup and avoid repetitive launch prompts.
    pub fn defer_cleanup(&mut self, app: &AppHandle, receipt_id: &str) -> Result<(), String> {
        let (omera_root, _) = resolve_migration_roots(app)?;
        let mut receipt = storage_migration::load_receipt(&omera_root)?
            .filter(|r| r.receipt_id == receipt_id)
            .ok_or_else(|| format!("Receipt '{receipt_id}' not found"))?;

        receipt.cleanup_status = "deferred".to_string();
        storage_migration::save_receipt(&omera_root, &receipt)
    }
}

/// Resolve candidate legacy roots and target Omera root from the runtime application environment.
pub fn resolve_migration_roots(
    app: &AppHandle,
) -> Result<(PathBuf, Vec<(String, PathBuf)>), String> {
    let current_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let parent = current_data_dir
        .parent()
        .ok_or("Cannot resolve parent of app_data_dir")?;
    let omera_root = parent.join("com.berryuiki.omera");

    let mut candidates = Vec::new();
    let legacy_1 = parent.join("com.berryuiki.berryaistudio");
    if legacy_1 != omera_root {
        candidates.push(("com.berryuiki.berryaistudio".to_string(), legacy_1));
    }
    let legacy_2 = parent.join("com.berryuiki.berryaigctoolbox");
    if legacy_2 != omera_root {
        candidates.push(("com.berryuiki.berryaigctoolbox".to_string(), legacy_2));
    }
    if current_data_dir.join("berry.db").exists()
        && !candidates.iter().any(|(_, p)| p == &current_data_dir)
    {
        candidates.push(("legacy_in_root".to_string(), current_data_dir));
    }

    Ok((omera_root, candidates))
}

fn compute_file_sha256(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    Ok(hex::encode(hasher.finalize()))
}

fn calculate_folder_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() && !meta.is_symlink() {
                    total += calculate_folder_size(&entry.path());
                }
            }
        }
    }
    total
}

#[cfg(unix)]
fn get_available_disk_space(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    let c_path = CString::new(path.to_string_lossy().as_bytes()).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } == 0 {
        Some(stat.f_bavail as u64 * stat.f_frsize as u64)
    } else {
        None
    }
}

#[cfg(windows)]
fn get_available_disk_space(_path: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use omera_domain::DiscoveredSource;

    #[test]
    fn test_coordinator_defaults() {
        let coordinator = MigrationCoordinator::new();
        assert!(coordinator.active_job.is_none());
        assert!(coordinator.active_plans.is_empty());
        assert!(coordinator.active_cleanup_previews.is_empty());
        assert!(coordinator.get_job("non-existent").is_err());
    }

    #[test]
    fn test_compute_file_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").unwrap();
        let hash = compute_file_sha256(&file_path).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_calculate_folder_size() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("a.txt"), b"12345").unwrap();
        std::fs::write(dir.path().join("b.txt"), b"67890").unwrap();
        assert_eq!(calculate_folder_size(dir.path()), 10);
    }

    #[test]
    fn test_migration_preserves_source_config_bytes_identically() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // Ensure keyring works on headless CI runners (e.g. Linux without Secret Service)
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());

        // 1. Create a minimal valid SQLite source database
        let source_db = source_dir.path().join("berry.db");
        let db = omera_storage::Database::connect(&source_db).unwrap();
        drop(db);

        // 2. Create source config with plaintext credentials
        let source_config_path = source_dir.path().join("config.json");
        let mut cfg = crate::commands::AppConfig::default();
        cfg.cloud_backup.webdav_password = Some("plaintext_legacy_password".into());
        let raw_config = serde_json::to_string_pretty(&cfg).unwrap();
        std::fs::write(&source_config_path, raw_config.as_bytes()).unwrap();
        let source_hash_before = compute_file_sha256(&source_config_path).unwrap();

        // 3. Build a migration preview/plan
        let dest_db = dest_dir.path().join("omera.db");
        let preview = LegacyMigrationPreview {
            plan_id: "plan-test-config".to_string(),
            source: DiscoveredSource {
                source_id: "berry-app-data".to_string(),
                identifier: "com.berryuiki.berryaistudio".to_string(),
                root_path: source_dir.path().to_string_lossy().to_string(),
                database_path: source_db.to_string_lossy().to_string(),
                config_path: Some(source_config_path.to_string_lossy().to_string()),
                file_count: 1,
                database_size_bytes: 1024,
                total_size_bytes: 2048,
                schema_version: 1,
                is_locked: false,
            },
            destination_root: dest_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            required_space_bytes: 1024,
            available_space_bytes: 10_000_000,
            conflicts: vec![],
            exclusions: vec![],
        };

        let mut coordinator = MigrationCoordinator::new();
        let job = coordinator
            .execute_migration_plan(&preview)
            .expect("Migration execution should succeed");
        assert_eq!(job.status, "completed");

        // 4. Assert source config.json bytes and hash are 100% identical before and after
        let source_hash_after = compute_file_sha256(&source_config_path).unwrap();
        assert_eq!(
            source_hash_before, source_hash_after,
            "Source config file must NOT be modified or rewritten during migration!"
        );
        let source_content_after = std::fs::read_to_string(&source_config_path).unwrap();
        assert_eq!(source_content_after, raw_config);

        // 5. Destination config was created and credentials are protected
        let dest_config_path = dest_dir.path().join("config.json");
        assert!(dest_config_path.exists());
        let dest_raw = std::fs::read_to_string(&dest_config_path).unwrap();
        // The destination JSON should not contain the plaintext password
        assert!(
            !dest_raw.contains("plaintext_legacy_password"),
            "Destination config must protect migrated credentials"
        );
        let dest_cfg: crate::commands::AppConfig = serde_json::from_str(&dest_raw).unwrap();
        assert!(
            dest_cfg
                .cloud_backup
                .webdav_password
                .as_deref()
                .unwrap_or_default()
                .starts_with("keyring:"),
            "Destination credentials should be saved with keyring reference"
        );
    }

    #[test]
    fn test_migration_failure_preserves_source_config_bytes() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // 1. Create a corrupt source database to cause migration failure in step 1
        let source_db = source_dir.path().join("berry.db");
        std::fs::write(&source_db, b"not-a-valid-sqlite-db").unwrap();

        // 2. Create source config with plaintext credentials
        let source_config_path = source_dir.path().join("config.json");
        let raw_config = r#"{"theme":"light","cloud_backup":{"webdav_password":"secret"}}"#;
        std::fs::write(&source_config_path, raw_config.as_bytes()).unwrap();
        let source_hash_before = compute_file_sha256(&source_config_path).unwrap();

        let dest_db = dest_dir.path().join("omera.db");
        let preview = LegacyMigrationPreview {
            plan_id: "plan-test-failure".to_string(),
            source: DiscoveredSource {
                source_id: "berry-app-data".to_string(),
                identifier: "com.berryuiki.berryaistudio".to_string(),
                root_path: source_dir.path().to_string_lossy().to_string(),
                database_path: source_db.to_string_lossy().to_string(),
                config_path: Some(source_config_path.to_string_lossy().to_string()),
                file_count: 1,
                database_size_bytes: 1024,
                total_size_bytes: 2048,
                schema_version: 1,
                is_locked: false,
            },
            destination_root: dest_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            required_space_bytes: 1024,
            available_space_bytes: 10_000_000,
            conflicts: vec![],
            exclusions: vec![],
        };

        let mut coordinator = MigrationCoordinator::new();
        let res = coordinator.execute_migration_plan(&preview);
        assert!(res.is_err());

        // 3. Source config bytes must still be identical
        let source_hash_after = compute_file_sha256(&source_config_path).unwrap();
        assert_eq!(source_hash_before, source_hash_after);
        assert_eq!(
            std::fs::read_to_string(&source_config_path).unwrap(),
            raw_config
        );
    }

    #[test]
    fn test_malformed_source_config_does_not_mutate_or_delete_source() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // Valid source database
        let source_db = source_dir.path().join("berry.db");
        let db = omera_storage::Database::connect(&source_db).unwrap();
        drop(db);

        // Malformed / unparseable source config
        let source_config_path = source_dir.path().join("config.json");
        let corrupted_content = b"{ unclosed json with sensitive data: 'password' ";
        std::fs::write(&source_config_path, corrupted_content).unwrap();
        let source_hash_before = compute_file_sha256(&source_config_path).unwrap();

        let dest_db = dest_dir.path().join("omera.db");
        let preview = LegacyMigrationPreview {
            plan_id: "plan-test-malformed-config".to_string(),
            source: DiscoveredSource {
                source_id: "berry-app-data".to_string(),
                identifier: "com.berryuiki.berryaistudio".to_string(),
                root_path: source_dir.path().to_string_lossy().to_string(),
                database_path: source_db.to_string_lossy().to_string(),
                config_path: Some(source_config_path.to_string_lossy().to_string()),
                file_count: 1,
                database_size_bytes: 1024,
                total_size_bytes: 2048,
                schema_version: 1,
                is_locked: false,
            },
            destination_root: dest_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            required_space_bytes: 1024,
            available_space_bytes: 10_000_000,
            conflicts: vec![],
            exclusions: vec![],
        };

        let mut coordinator = MigrationCoordinator::new();
        let job = coordinator.execute_migration_plan(&preview).expect(
            "Migration execution should return job with failed status when config is unparseable",
        );
        assert_eq!(job.status, "failed");
        let receipt = job
            .receipt
            .expect("Receipt should still be durably recorded");
        let config_art = receipt
            .artifacts
            .iter()
            .find(|a| a.category == "config")
            .expect("Config artifact should be present in receipt");
        assert_eq!(config_art.status, "failed");

        // The malformed source config file MUST NOT be altered, deleted, or overwritten
        let source_hash_after = compute_file_sha256(&source_config_path).unwrap();
        assert_eq!(source_hash_before, source_hash_after);
        assert_eq!(
            std::fs::read(&source_config_path).unwrap(),
            corrupted_content
        );
    }

    #[test]
    fn test_config_migration_with_nonzero_revision_succeeds_in_absent_destination() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        // Keyring mock for headless CI
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());

        let source_db = source_dir.path().join("berry.db");
        let db = omera_storage::Database::connect(&source_db).unwrap();
        drop(db);

        // Source config with nonzero revision and custom settings
        let source_config_path = source_dir.path().join("config.json");
        let cfg = crate::commands::AppConfig {
            config_revision: 42,
            theme: "dracula".to_string(),
            ..Default::default()
        };
        std::fs::write(
            &source_config_path,
            serde_json::to_string_pretty(&cfg).unwrap(),
        )
        .unwrap();

        let dest_db = dest_dir.path().join("omera.db");
        let preview = LegacyMigrationPreview {
            plan_id: "plan-test-nonzero-rev".to_string(),
            source: DiscoveredSource {
                source_id: "berry-app-data".to_string(),
                identifier: "com.berryuiki.berryaistudio".to_string(),
                root_path: source_dir.path().to_string_lossy().to_string(),
                database_path: source_db.to_string_lossy().to_string(),
                config_path: Some(source_config_path.to_string_lossy().to_string()),
                file_count: 1,
                database_size_bytes: 1024,
                total_size_bytes: 2048,
                schema_version: 1,
                is_locked: false,
            },
            destination_root: dest_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            required_space_bytes: 1024,
            available_space_bytes: 10_000_000,
            conflicts: vec![],
            exclusions: vec![],
        };

        let mut coordinator = MigrationCoordinator::new();
        let job = coordinator
            .execute_migration_plan(&preview)
            .expect("Migration should succeed");
        assert_eq!(job.status, "completed");

        let receipt = job.receipt.expect("Receipt must exist");
        let config_art = receipt
            .artifacts
            .iter()
            .find(|a| a.category == "config")
            .expect("Config artifact present");
        assert_eq!(config_art.status, "success");

        // Verify destination config file exists, has revision 1, and preserved theme
        let dest_config_path = dest_dir.path().join("config.json");
        assert!(dest_config_path.exists());
        let saved_cfg = crate::config_store::load_readonly(&dest_config_path).unwrap();
        assert_eq!(saved_cfg.config_revision, 1);
        assert_eq!(saved_cfg.theme, "dracula");
    }

    #[test]
    fn test_config_migration_skipped_when_destination_exists() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());

        let source_db = source_dir.path().join("berry.db");
        let db = omera_storage::Database::connect(&source_db).unwrap();
        drop(db);

        let source_config_path = source_dir.path().join("config.json");
        let legacy_cfg = crate::commands::AppConfig {
            theme: "legacy_theme".to_string(),
            ..Default::default()
        };
        std::fs::write(
            &source_config_path,
            serde_json::to_string_pretty(&legacy_cfg).unwrap(),
        )
        .unwrap();

        // Destination config already exists
        let dest_config_path = dest_dir.path().join("config.json");
        let dest_cfg = crate::commands::AppConfig {
            theme: "existing_authoritative_theme".to_string(),
            ..Default::default()
        };
        crate::config_store::save(&dest_config_path, dest_cfg).unwrap();

        let dest_db = dest_dir.path().join("omera.db");
        let preview = LegacyMigrationPreview {
            plan_id: "plan-test-skipped".to_string(),
            source: DiscoveredSource {
                source_id: "berry-app-data".to_string(),
                identifier: "com.berryuiki.berryaistudio".to_string(),
                root_path: source_dir.path().to_string_lossy().to_string(),
                database_path: source_db.to_string_lossy().to_string(),
                config_path: Some(source_config_path.to_string_lossy().to_string()),
                file_count: 1,
                database_size_bytes: 1024,
                total_size_bytes: 2048,
                schema_version: 1,
                is_locked: false,
            },
            destination_root: dest_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            required_space_bytes: 1024,
            available_space_bytes: 10_000_000,
            conflicts: vec![],
            exclusions: vec![],
        };

        let mut coordinator = MigrationCoordinator::new();
        let job = coordinator
            .execute_migration_plan(&preview)
            .expect("Migration should succeed");
        assert_eq!(job.status, "completed");

        let receipt = job.receipt.expect("Receipt must exist");
        let config_art = receipt
            .artifacts
            .iter()
            .find(|a| a.category == "config")
            .expect("Config artifact present");
        assert_eq!(config_art.status, "skipped");

        // Destination remains unchanged
        let readback = crate::config_store::load_readonly(&dest_config_path).unwrap();
        assert_eq!(readback.theme, "existing_authoritative_theme");
    }

    #[test]
    fn test_config_migration_persistence_failure_records_failed_artifact_and_blocks_cleanup() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());

        let source_db = source_dir.path().join("berry.db");
        let db = omera_storage::Database::connect(&source_db).unwrap();
        drop(db);

        let source_config_path = source_dir.path().join("config.json");
        let cfg = crate::commands::AppConfig::default();
        std::fs::write(
            &source_config_path,
            serde_json::to_string_pretty(&cfg).unwrap(),
        )
        .unwrap();

        // Make destination config path un-writable by creating a directory where config.json should be,
        // or a read-only collision. In temp dir, creating a directory at dest_config_path causes save_migrated to fail writing a file there.
        let dest_config_path = dest_dir.path().join("config.json");
        std::fs::create_dir_all(&dest_config_path).unwrap();

        let dest_db = dest_dir.path().join("omera.db");
        let preview = LegacyMigrationPreview {
            plan_id: "plan-test-persist-fail".to_string(),
            source: DiscoveredSource {
                source_id: "berry-app-data".to_string(),
                identifier: "com.berryuiki.berryaistudio".to_string(),
                root_path: source_dir.path().to_string_lossy().to_string(),
                database_path: source_db.to_string_lossy().to_string(),
                config_path: Some(source_config_path.to_string_lossy().to_string()),
                file_count: 1,
                database_size_bytes: 1024,
                total_size_bytes: 2048,
                schema_version: 1,
                is_locked: false,
            },
            destination_root: dest_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            required_space_bytes: 1024,
            available_space_bytes: 10_000_000,
            conflicts: vec![],
            exclusions: vec![],
        };

        let mut coordinator = MigrationCoordinator::new();
        let job = coordinator
            .execute_migration_plan(&preview)
            .expect("Migration coordinator returns job");
        assert_eq!(job.status, "failed");
        assert!(job.error.is_some());

        let receipt = job.receipt.expect("Receipt should be saved");
        let config_art = receipt
            .artifacts
            .iter()
            .find(|a| a.category == "config")
            .expect("Config artifact present");
        assert_eq!(config_art.status, "failed");

        // Now test preview_cleanup logic: config.json must NOT be eligible for cleanup!
        // We verify item eligibility for receipt with failed config artifact
        let mut config_eligible = false;
        let mut config_reason = None;
        if let Ok(entries) = std::fs::read_dir(source_dir.path()) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy() == "config.json" {
                    let config_verified = receipt.artifacts.iter().any(|a| {
                        a.category == "config" && (a.status == "success" || a.status == "skipped")
                    });
                    if config_verified {
                        config_eligible = true;
                    } else {
                        config_eligible = false;
                        config_reason =
                            Some("Destination configuration not verified or missing".to_string());
                    }
                }
            }
        }
        assert!(
            !config_eligible,
            "Config must not be eligible for cleanup when artifact status is failed"
        );
        assert_eq!(
            config_reason.as_deref(),
            Some("Destination configuration not verified or missing")
        );
    }

    #[test]
    fn test_empty_destination_does_not_mask_ambiguous_sources() {
        let omera_dir = tempfile::tempdir().unwrap();
        let src1_dir = tempfile::tempdir().unwrap();
        let src2_dir = tempfile::tempdir().unwrap();

        // Create empty omera.db in omera_dir (e.g. created on fresh startup)
        let omera_db = omera_dir.path().join("omera.db");
        let _ = omera_storage::Database::connect(&omera_db).unwrap();

        // Create two valid legacy sources
        let berry1_db = src1_dir.path().join("berry.db");
        let conn1 = omera_storage::rusqlite::Connection::open(&berry1_db).unwrap();
        conn1
            .execute("CREATE TABLE files (id INTEGER PRIMARY KEY);", [])
            .unwrap();
        conn1
            .execute("INSERT INTO files (id) VALUES (1);", [])
            .unwrap();
        drop(conn1);

        let berry2_db = src2_dir.path().join("berry.db");
        let conn2 = omera_storage::rusqlite::Connection::open(&berry2_db).unwrap();
        conn2
            .execute("CREATE TABLE files (id INTEGER PRIMARY KEY);", [])
            .unwrap();
        conn2
            .execute("INSERT INTO files (id) VALUES (1);", [])
            .unwrap();
        drop(conn2);

        let candidates = vec![
            ("src1".to_string(), src1_dir.path().to_path_buf()),
            ("src2".to_string(), src2_dir.path().to_path_buf()),
        ];

        let mut coordinator = MigrationCoordinator::new();
        let status = coordinator
            .get_status_for_roots(omera_dir.path(), &candidates)
            .expect("Status check should succeed");

        // Destination exists, but is empty and 2 sources exist -> awaiting_source_choice
        assert_eq!(status.stage, "awaiting_source_choice");
        assert_eq!(status.discovered_sources.len(), 2);
        assert!(status.destination_exists);
        assert!(status
            .available_actions
            .contains(&"preview_migration".to_string()));
    }

    #[test]
    fn test_empty_destination_does_not_mask_single_source() {
        let omera_dir = tempfile::tempdir().unwrap();
        let src_dir = tempfile::tempdir().unwrap();

        // Create empty omera.db in omera_dir
        let omera_db = omera_dir.path().join("omera.db");
        let _ = omera_storage::Database::connect(&omera_db).unwrap();

        // Create one valid legacy source using Database::connect
        let berry_db = src_dir.path().join("berry.db");
        let source_db_conn = omera_storage::Database::connect(&berry_db).unwrap();
        source_db_conn.add_folder("/legacy/folder").unwrap();
        drop(source_db_conn);

        let candidates = vec![("src".to_string(), src_dir.path().to_path_buf())];

        let mut coordinator = MigrationCoordinator::new();
        let status = coordinator
            .get_status_for_roots(omera_dir.path(), &candidates)
            .expect("Status check should succeed");

        // Destination exists, but is empty and 1 source exists -> discovered
        assert_eq!(status.stage, "discovered");
        assert_eq!(status.discovered_sources.len(), 1);
        assert!(status.destination_exists);
        assert!(status
            .available_actions
            .contains(&"preview_migration".to_string()));

        // Preview migration: empty destination must NOT trigger conflict
        let preview = coordinator
            .preview_migration_for_roots(
                omera_dir.path(),
                &candidates,
                &status.discovered_sources[0].source_id,
            )
            .expect("Preview should succeed");
        assert!(
            preview.conflicts.is_empty(),
            "Conflicts should be empty for empty destination: {:?}",
            preview.conflicts
        );

        // Execute migration into the empty destination
        let job = coordinator
            .execute_migration_plan(&preview)
            .expect("Migration execution should succeed");
        assert_eq!(job.status, "completed");

        // Now omera.db is populated with migrated data (has 1 folder)
        let omera_conn = omera_storage::rusqlite::Connection::open(&omera_db).unwrap();
        let count: i64 = omera_conn
            .query_row("SELECT COUNT(*) FROM folders", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_non_empty_destination_masks_and_reports_conflict() {
        let omera_dir = tempfile::tempdir().unwrap();
        let src_dir = tempfile::tempdir().unwrap();

        // Create omera.db and populate it with a folder/file so it is non-empty
        let omera_db = omera_dir.path().join("omera.db");
        let db = omera_storage::Database::connect(&omera_db).unwrap();
        db.add_folder("/omera/folder").unwrap();

        // Create one legacy source
        let berry_db = src_dir.path().join("berry.db");
        let conn = omera_storage::rusqlite::Connection::open(&berry_db).unwrap();
        conn.execute("CREATE TABLE files (id INTEGER PRIMARY KEY);", [])
            .unwrap();
        conn.execute("INSERT INTO files (id) VALUES (1);", [])
            .unwrap();
        drop(conn);

        let candidates = vec![("src".to_string(), src_dir.path().to_path_buf())];

        let mut coordinator = MigrationCoordinator::new();
        let status = coordinator
            .get_status_for_roots(omera_dir.path(), &candidates)
            .expect("Status check should succeed");

        // Non-empty destination -> migrated
        assert_eq!(status.stage, "migrated");

        // Preview for the source must include conflict about omera.db already existing
        let preview = coordinator
            .preview_migration_for_roots(
                omera_dir.path(),
                &candidates,
                &status.discovered_sources[0].source_id,
            )
            .expect("Preview should succeed");
        assert!(
            preview
                .conflicts
                .iter()
                .any(|c| c.contains("Target database omera.db already exists")),
            "Preview conflicts should mention target database exists: {:?}",
            preview.conflicts
        );
    }

    #[test]
    fn test_cleanup_preview_validates_destination_config_and_models() {
        let omera_dir = tempfile::tempdir().unwrap();
        let src_dir = tempfile::tempdir().unwrap();

        let source_db = src_dir.path().join("berry.db");
        std::fs::write(&source_db, b"sqlite format 3\0test_src_data").unwrap();

        let source_config = src_dir.path().join("config.json");
        std::fs::write(&source_config, b"{\"test\": 123}").unwrap();

        let source_models = src_dir.path().join("models");
        std::fs::create_dir_all(&source_models).unwrap();
        std::fs::write(source_models.join("model1.bin"), b"model_data_bytes_1234").unwrap();

        // Extra files that must never be eligible
        let source_media = src_dir.path().join("vacation.png");
        std::fs::write(&source_media, b"fake_png").unwrap();

        let source_backup = src_dir.path().join("backup.zip");
        std::fs::write(&source_backup, b"fake_zip").unwrap();

        // Save a migration receipt in omera_dir with success status
        let dest_db = omera_dir.path().join("omera.db");
        // Create valid SQLite db at dest_db
        let _ = omera_storage::Database::connect(&dest_db).unwrap();

        let receipt_id = "test-receipt-cleanup-1";
        let receipt = MigrationReceipt {
            receipt_id: receipt_id.into(),
            source_id: "src1".into(),
            source_identifier: "com.berryuiki.berryaistudio".into(),
            source_root: src_dir.path().to_string_lossy().to_string(),
            source_schema_version: 1,
            destination_root: omera_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            artifacts: vec![
                MigratedArtifact {
                    category: "database".into(),
                    source_path: source_db.to_string_lossy().to_string(),
                    destination_path: dest_db.to_string_lossy().to_string(),
                    status: "success".into(),
                    size_bytes: 1024,
                },
                MigratedArtifact {
                    category: "config".into(),
                    source_path: source_config.to_string_lossy().to_string(),
                    destination_path: omera_dir
                        .path()
                        .join("config.json")
                        .to_string_lossy()
                        .to_string(),
                    status: "success".into(),
                    size_bytes: 100,
                },
            ],
            integrity_hash: "hash_clean".into(),
            cleanup_status: "pending".into(),
        };
        storage_migration::save_receipt(omera_dir.path(), &receipt).unwrap();

        let mut coordinator = MigrationCoordinator::new();

        // Case 1: Destination config and destination models are MISSING
        let preview1 = coordinator
            .preview_cleanup_for_root(omera_dir.path(), receipt_id)
            .expect("Preview should succeed");

        let db_item = preview1
            .items
            .iter()
            .find(|i| i.path.ends_with("berry.db"))
            .unwrap();
        assert!(
            db_item.eligible,
            "Database should be eligible when destination is healthy"
        );

        let config_item = preview1
            .items
            .iter()
            .find(|i| i.path.ends_with("config.json"))
            .unwrap();
        assert!(
            !config_item.eligible,
            "Config should NOT be eligible when destination file is missing"
        );
        assert_eq!(
            config_item.reason.as_deref(),
            Some("Destination configuration not verified or missing")
        );

        let models_item = preview1
            .items
            .iter()
            .find(|i| i.path.ends_with("models"))
            .unwrap();
        assert!(
            !models_item.eligible,
            "Models should NOT be eligible when destination models missing"
        );
        assert_eq!(
            models_item.reason.as_deref(),
            Some("Destination models not verified, missing or incomplete")
        );

        let media_item = preview1
            .items
            .iter()
            .find(|i| i.path.ends_with("vacation.png"))
            .unwrap();
        assert!(
            !media_item.eligible,
            "Media file must never be eligible for cleanup"
        );

        let backup_item = preview1
            .items
            .iter()
            .find(|i| i.path.ends_with("backup.zip"))
            .unwrap();
        assert!(
            !backup_item.eligible,
            "Backup zip must never be eligible for cleanup"
        );

        // Case 2: Now add destination config and complete destination models
        let dest_config = omera_dir.path().join("config.json");
        let valid_config_bytes =
            serde_json::to_vec_pretty(&crate::commands::AppConfig::default()).unwrap();
        std::fs::write(&dest_config, &valid_config_bytes).unwrap();

        let dest_models = omera_dir.path().join("models");
        std::fs::create_dir_all(&dest_models).unwrap();
        std::fs::write(dest_models.join("model1.bin"), b"model_data_bytes_1234").unwrap();

        let preview2 = coordinator
            .preview_cleanup_for_root(omera_dir.path(), receipt_id)
            .expect("Preview should succeed");

        let config_item2 = preview2
            .items
            .iter()
            .find(|i| i.path.ends_with("config.json"))
            .unwrap();
        assert!(
            config_item2.eligible,
            "Config should be eligible when destination is healthy"
        );
        assert!(config_item2.reason.is_none());

        let models_item2 = preview2
            .items
            .iter()
            .find(|i| i.path.ends_with("models"))
            .unwrap();
        assert!(
            models_item2.eligible,
            "Models should be eligible when destination models match source"
        );
        assert!(models_item2.reason.is_none());
    }

    #[test]
    fn test_confirm_cleanup_revalidates_destination_artifacts() {
        let omera_dir = tempfile::tempdir().unwrap();
        let src_dir = tempfile::tempdir().unwrap();

        let source_config = src_dir.path().join("config.json");
        std::fs::write(&source_config, b"{\"test\": 123}").unwrap();

        let dest_db = omera_dir.path().join("omera.db");
        let _ = omera_storage::Database::connect(&dest_db).unwrap();

        let dest_config = omera_dir.path().join("config.json");
        let valid_config_bytes =
            serde_json::to_vec_pretty(&crate::commands::AppConfig::default()).unwrap();
        std::fs::write(&dest_config, &valid_config_bytes).unwrap();

        let receipt_id = "test-receipt-confirm-1";
        let receipt = MigrationReceipt {
            receipt_id: receipt_id.into(),
            source_id: "src1".into(),
            source_identifier: "com.berryuiki.berryaistudio".into(),
            source_root: src_dir.path().to_string_lossy().to_string(),
            source_schema_version: 1,
            destination_root: omera_dir.path().to_string_lossy().to_string(),
            destination_db: dest_db.to_string_lossy().to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            artifacts: vec![MigratedArtifact {
                category: "config".into(),
                source_path: source_config.to_string_lossy().to_string(),
                destination_path: dest_config.to_string_lossy().to_string(),
                status: "success".into(),
                size_bytes: 100,
            }],
            integrity_hash: "hash_clean".into(),
            cleanup_status: "pending".into(),
        };
        storage_migration::save_receipt(omera_dir.path(), &receipt).unwrap();

        let mut coordinator = MigrationCoordinator::new();
        let preview = coordinator
            .preview_cleanup_for_root(omera_dir.path(), receipt_id)
            .expect("Preview should succeed");

        let preview_id = preview.preview_id.clone();
        let config_item = preview
            .items
            .iter()
            .find(|i| i.path.ends_with("config.json"))
            .unwrap();
        assert!(config_item.eligible);

        // Simulate destination config becoming corrupted / removed before confirmation!
        std::fs::remove_file(&dest_config).unwrap();

        // Run confirmation
        let result = coordinator
            .confirm_cleanup_for_root(omera_dir.path(), &preview_id, true)
            .expect("Confirmation should run");

        // The config cleanup must have been skipped and an error recorded
        assert_eq!(result.failed_count, 1);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Execution skipped cleanup for config: destination unverified")));
        // Source config must NOT have been trashed
        assert!(
            source_config.exists(),
            "Source config must be preserved when destination validation fails"
        );
    }
}
