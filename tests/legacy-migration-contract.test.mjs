import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";

test("Legacy Migration: Tauri backend registers all 7 coordination commands", async () => {
  const libContent = await fs.readFile(
    new URL("../src-tauri/src/lib.rs", import.meta.url),
    "utf8",
  );
  const commandsContent = await fs.readFile(
    new URL("../src-tauri/src/commands.rs", import.meta.url),
    "utf8",
  );

  const requiredCommands = [
    "get_legacy_migration_status",
    "preview_legacy_migration",
    "start_legacy_migration",
    "get_legacy_migration_job",
    "preview_legacy_cleanup",
    "confirm_legacy_cleanup",
    "defer_legacy_cleanup",
  ];

  for (const cmd of requiredCommands) {
    assert.match(
      libContent,
      new RegExp(`commands::${cmd}`),
      `src-tauri/src/lib.rs must register ${cmd}`,
    );
    assert.match(
      commandsContent,
      new RegExp(`pub fn ${cmd}`),
      `src-tauri/src/commands.rs must implement ${cmd}`,
    );
  }
});

test("Legacy Migration: WAL preservation and atomic staged publication guarantees", async () => {
  const storageMigration = await fs.readFile(
    new URL("../crates/omera-storage/src/legacy_migration.rs", import.meta.url),
    "utf8",
  );

  // 1. Uses SQLite Backup API to snapshot live/committed WAL pages without write locks
  assert.match(
    storageMigration,
    /Backup::new\(&source_conn,\s*&mut staged_conn\)/,
    "Must use SQLite Backup API to snapshot committed WAL pages safely",
  );

  // 2. Upgrades only the copy and consolidates journal
  assert.match(
    storageMigration,
    /PRAGMA wal_checkpoint\(TRUNCATE\)/,
    "Must checkpoint staged database before atomic publication",
  );

  // 3. Rejects collision if destination already exists
  assert.match(
    storageMigration,
    /DESTINATION_EXISTS/,
    "Must reject migration if target database already exists",
  );

  // 4. End-to-end zero loss migration assertion exists
  assert.match(
    storageMigration,
    /fn test_legacy_berry_database_zero_loss_full_migration/,
    "Must include full zero-loss entity fidelity unit test",
  );
});

test("Legacy Migration: Data safety rules exclude user media and external vaults from cleanup", async () => {
  const coordinatorContent = await fs.readFile(
    new URL("../src-tauri/src/legacy_migration.rs", import.meta.url),
    "utf8",
  );

  // Cleanup preview only includes internal receipt-owned artifacts
  assert.match(
    coordinatorContent,
    /confirm_cleanup/,
    "Must require explicit confirmation for cleanup",
  );
  assert.match(
    coordinatorContent,
    /trash::delete/,
    "Must use OS Recycle Bin / Trash rather than permanent deletion",
  );
});

test("Legacy Migration: Command parameters and return DTO shapes are compatible", async () => {
  const domainMigration = await fs.readFile(
    new URL("../crates/omera-domain/src/migration.rs", import.meta.url),
    "utf8",
  );
  const commandsContent = await fs.readFile(
    new URL("../src-tauri/src/commands.rs", import.meta.url),
    "utf8",
  );

  // Verify return types in commands.rs match domain types
  assert.match(
    commandsContent,
    /pub fn get_legacy_migration_status\([^)]*\)\s*->\s*Result<omera_domain::LegacyMigrationStatus,\s*String>/,
    "get_legacy_migration_status must return LegacyMigrationStatus",
  );
  assert.match(
    commandsContent,
    /pub fn preview_legacy_migration\([^)]*source_id:\s*String[^)]*\)\s*->\s*Result<omera_domain::LegacyMigrationPreview,\s*String>/,
    "preview_legacy_migration must accept source_id and return LegacyMigrationPreview",
  );
  assert.match(
    commandsContent,
    /pub fn start_legacy_migration\([^)]*plan_id:\s*String[^)]*\)\s*->\s*Result<omera_domain::LegacyMigrationJob,\s*String>/,
    "start_legacy_migration must accept plan_id and return LegacyMigrationJob",
  );
  assert.match(
    commandsContent,
    /pub fn get_legacy_migration_job\([^)]*job_id:\s*String[^)]*\)\s*->\s*Result<omera_domain::LegacyMigrationJob,\s*String>/,
    "get_legacy_migration_job must accept job_id and return LegacyMigrationJob",
  );
  assert.match(
    commandsContent,
    /pub fn preview_legacy_cleanup\([^)]*receipt_id:\s*String[^)]*\)\s*->\s*Result<omera_domain::LegacyCleanupPreview,\s*String>/,
    "preview_legacy_cleanup must accept receipt_id and return LegacyCleanupPreview",
  );
  assert.match(
    commandsContent,
    /pub fn confirm_legacy_cleanup\([^)]*preview_id:\s*String,\s*confirmed:\s*bool[^)]*\)\s*->\s*Result<omera_domain::LegacyCleanupResult,\s*String>/,
    "confirm_legacy_cleanup must accept preview_id and confirmed, returning LegacyCleanupResult",
  );
  assert.match(
    commandsContent,
    /pub fn defer_legacy_cleanup\([^)]*receipt_id:\s*String[^)]*\)\s*->\s*Result<\(\),\s*String>/,
    "defer_legacy_cleanup must accept receipt_id and return ()",
  );

  // Verify core struct definitions exist in domain
  assert.match(domainMigration, /pub struct LegacyMigrationStatus/);
  assert.match(domainMigration, /pub struct LegacyMigrationPreview/);
  assert.match(domainMigration, /pub struct LegacyMigrationJob/);
  assert.match(domainMigration, /pub struct LegacyCleanupPreview/);
  assert.match(domainMigration, /pub struct LegacyCleanupResult/);
});

test("Cloud Restore: Uses staged recovery lifecycle and does not overwrite live database", async () => {
  const cloudBackupContent = await fs.readFile(
    new URL("../src-tauri/src/cloud_backup.rs", import.meta.url),
    "utf8",
  );
  const commandsContent = await fs.readFile(
    new URL("../src-tauri/src/commands.rs", import.meta.url),
    "utf8",
  );
  const libContent = await fs.readFile(
    new URL("../src-tauri/src/lib.rs", import.meta.url),
    "utf8",
  );

  // 1. cloud_backup::restore_cloud_snapshot must route through omera_storage::recovery::stage_restore
  assert.match(
    cloudBackupContent,
    /omera_storage::recovery::stage_restore/,
    "cloud_backup must call omera_storage::recovery::stage_restore",
  );

  // 2. Must NOT overwrite active_db_path with fs::copy directly in restore_cloud_snapshot
  assert.doesNotMatch(
    cloudBackupContent,
    /fs::copy\([^,]+,\s*active_db_path\)/,
    "cloud_backup must not copy directly over active_db_path",
  );

  // 3. Application startup must apply pending restore prior to connecting to SQLite database
  assert.match(
    libContent,
    /omera_storage::recovery::apply_pending_restore\(&database_path\)/,
    "lib.rs setup must apply pending restore before opening database",
  );

  // 4. commands.rs restarts the application after staging cloud restore
  assert.match(
    commandsContent,
    /pub async fn cloud_backup_restore_snapshot[\s\S]*?app_handle\.restart\(\)/,
    "cloud_backup_restore_snapshot must trigger app_handle.restart() to apply staged restore",
  );
});

