import assert from "node:assert/strict";
import fs from "node:fs/promises";
import test from "node:test";

test("Legacy Migration Workflow UI: Modal, entry points, and contracts", async () => {
  const modalContent = await fs.readFile(
    new URL("../src/components/LegacyMigrationModal.vue", import.meta.url),
    "utf8",
  );
  const settingsContent = await fs.readFile(
    new URL("../src/components/SettingsModal.vue", import.meta.url),
    "utf8",
  );
  const onboardingContent = await fs.readFile(
    new URL("../src/components/OnboardingModal.vue", import.meta.url),
    "utf8",
  );
  const menuBarContent = await fs.readFile(
    new URL("../src/components/MenuBar.vue", import.meta.url),
    "utf8",
  );
  const appContent = await fs.readFile(
    new URL("../src/App.vue", import.meta.url),
    "utf8",
  );
  const typesContent = await fs.readFile(
    new URL("../src/types.ts", import.meta.url),
    "utf8",
  );

  // 1. Types definition: TypeScript types match IPC contracts
  assert.match(typesContent, /export interface DiscoveredSource/);
  assert.match(typesContent, /export interface LegacyMigrationStatus/);
  assert.match(typesContent, /export interface LegacyMigrationPreview/);
  assert.match(typesContent, /export interface LegacyMigrationJob/);

  // 2. LegacyMigrationModal integrates all Tauri IPC calls
  assert.match(
    modalContent,
    /invoke<LegacyMigrationStatus>\("get_legacy_migration_status"\)/,
    "Modal must call get_legacy_migration_status",
  );
  assert.match(
    modalContent,
    /invoke<LegacyMigrationPreview>\(\s*"preview_legacy_migration"[\s\S]*?sourceId:\s*selectedSourceId\.value/,
    "Modal must call preview_legacy_migration with sourceId",
  );
  assert.match(
    modalContent,
    /invoke<LegacyMigrationJob>\(\s*"start_legacy_migration"[\s\S]*?planId:\s*activePreview\.value\.plan_id/,
    "Modal must call start_legacy_migration with planId",
  );
  assert.match(
    modalContent,
    /invoke<LegacyMigrationJob>\(\s*"get_legacy_migration_job"[\s\S]*?jobId/,
    "Modal must poll get_legacy_migration_job with jobId",
  );

  // 3. UI access points exist
  // SettingsModal has entry point
  assert.match(
    settingsContent,
    /LegacyMigrationModal/,
    "SettingsModal must import LegacyMigrationModal",
  );
  assert.match(
    settingsContent,
    /showLegacyMigrationModal/,
    "SettingsModal must manage showLegacyMigrationModal state",
  );

  // OnboardingModal detects legacy sources and shows banner
  assert.match(
    onboardingContent,
    /get_legacy_migration_status/,
    "OnboardingModal must query get_legacy_migration_status",
  );
  assert.match(
    onboardingContent,
    /legacyDiscoveredSourcesCount/,
    "OnboardingModal must track legacyDiscoveredSourcesCount",
  );

  // MenuBar has menu entry and emits event
  assert.match(
    menuBarContent,
    /openLegacyMigration/,
    "MenuBar must define and emit openLegacyMigration",
  );

  // App.vue connects MenuBar and handles legacy migration
  assert.match(
    appContent,
    /legacyMigrationModalOpen/,
    "App.vue must manage legacyMigrationModalOpen",
  );
  assert.match(
    appContent,
    /@open-legacy-migration="legacyMigrationModalOpen = true"/,
    "App.vue must listen to @open-legacy-migration",
  );
  assert.match(
    appContent,
    /<LegacyMigrationModal/,
    "App.vue must mount LegacyMigrationModal",
  );
  assert.match(
    appContent,
    /@migrated="onLegacyMigrated"/,
    "App.vue must handle @migrated",
  );

  // 4. Preserves source data notice & safety guarantee
  assert.match(
    modalContent,
    /t\.legacyMigration\.sourcePreservedNotice/,
    "Modal must explicitly display sourcePreservedNotice",
  );

  // 5. Retry workflow on failure
  assert.match(
    modalContent,
    /handleStartMigration/,
    "Modal must allow retry execution on failed jobs",
  );
  assert.match(
    modalContent,
    /handleBackToSources/,
    "Modal must allow backing out to source selection",
  );
});

test("Legacy Migration: All 7 locales provide complete translation keys", async () => {
  const locales = ["en", "zh-CN", "zh-TW", "de", "es", "fr", "ja"];
  const requiredKeys = [
    "title",
    "subtitle",
    "discoveredTitle",
    "discoveredDesc",
    "sourceLabel",
    "sourcePath",
    "dbPath",
    "fileCount",
    "dbSize",
    "totalSize",
    "schemaVersion",
    "isLocked",
    "isUnlocked",
    "previewButton",
    "previewing",
    "previewTitle",
    "destinationTarget",
    "requiredSpace",
    "availableSpace",
    "insufficientSpace",
    "conflictsTitle",
    "exclusionsTitle",
    "noConflicts",
    "startMigration",
    "migrating",
    "stepCopying",
    "stepValidating",
    "stepConfig",
    "stepModels",
    "stepActivating",
    "completeTitle",
    "completeDesc",
    "migratedCount",
    "sourcePreservedNotice",
    "cleanupPromptTitle",
    "cleanupPromptDesc",
    "cleanupPreviewButton",
    "skipCleanupButton",
    "failedTitle",
    "retryButton",
    "backToSources",
    "errorDetails",
    "statusMigrated",
    "openSettingsMigration",
    "close",
  ];

  for (const locale of locales) {
    const file = await fs.readFile(
      new URL(`../src/i18n/locales/${locale}.ts`, import.meta.url),
      "utf8",
    );
    assert.match(file, /legacyMigration:\s*\{/, `Locale ${locale} must define legacyMigration section`);
    for (const key of requiredKeys) {
      assert.match(
        file,
        new RegExp(`${key}:\\s*["'\`]`),
        `Locale ${locale} must define legacyMigration.${key}`,
      );
    }
  }
});
