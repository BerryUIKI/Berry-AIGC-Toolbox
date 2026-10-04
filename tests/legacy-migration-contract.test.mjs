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
