<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import { formatBytes } from "../utils/image";
import type {
  LegacyMigrationStatus,
  LegacyMigrationPreview,
  LegacyMigrationJob,
  DiscoveredSource,
} from "../types";

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "migrated"): void;
}>();

const loadingStatus = ref(false);
const statusError = ref<string | null>(null);
const migrationStatus = ref<LegacyMigrationStatus | null>(null);

const selectedSourceId = ref<string>("");
const previewing = ref(false);
const previewError = ref<string | null>(null);
const activePreview = ref<LegacyMigrationPreview | null>(null);

const migrating = ref(false);
const activeJob = ref<LegacyMigrationJob | null>(null);
const migrationError = ref<string | null>(null);

let pollTimer: ReturnType<typeof setInterval> | null = null;

const sources = computed<DiscoveredSource[]>(() => {
  return migrationStatus.value?.discovered_sources || [];
});

const selectedSource = computed<DiscoveredSource | undefined>(() => {
  return sources.value.find((s) => s.source_id === selectedSourceId.value);
});

const isInsufficientSpace = computed<boolean>(() => {
  if (!activePreview.value) return false;
  return activePreview.value.available_space_bytes < activePreview.value.required_space_bytes;
});

watch(
  () => props.show,
  (val) => {
    if (val) {
      resetState();
      void loadStatus();
    } else {
      stopPolling();
    }
  },
  { immediate: true }
);

function resetState() {
  statusError.value = null;
  previewError.value = null;
  migrationError.value = null;
  activePreview.value = null;
  activeJob.value = null;
  migrating.value = false;
  previewing.value = false;
  stopPolling();
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

async function loadStatus() {
  loadingStatus.value = true;
  statusError.value = null;
  try {
    const status = await invoke<LegacyMigrationStatus>("get_legacy_migration_status");
    migrationStatus.value = status;
    if (status.discovered_sources.length > 0 && !selectedSourceId.value) {
      selectedSourceId.value = status.discovered_sources[0].source_id;
    }
  } catch (err: any) {
    statusError.value = typeof err === "string" ? err : err?.message || String(err);
  } finally {
    loadingStatus.value = false;
  }
}

async function handlePreview() {
  if (!selectedSourceId.value) return;
  previewing.value = true;
  previewError.value = null;
  activePreview.value = null;
  try {
    const preview = await invoke<LegacyMigrationPreview>("preview_legacy_migration", {
      sourceId: selectedSourceId.value,
    });
    activePreview.value = preview;
  } catch (err: any) {
    previewError.value = typeof err === "string" ? err : err?.message || String(err);
  } finally {
    previewing.value = false;
  }
}

async function handleStartMigration() {
  if (!activePreview.value) return;
  migrating.value = true;
  migrationError.value = null;
  try {
    const job = await invoke<LegacyMigrationJob>("start_legacy_migration", {
      planId: activePreview.value.plan_id,
    });
    activeJob.value = job;
    if (job.status === "completed") {
      migrating.value = false;
      emit("migrated");
    } else if (job.status === "failed") {
      migrating.value = false;
      migrationError.value = job.error?.message_key || "Migration failed";
    } else {
      startJobPolling(job.job_id);
    }
  } catch (err: any) {
    migrating.value = false;
    migrationError.value = typeof err === "string" ? err : err?.message || String(err);
  }
}

function startJobPolling(jobId: string) {
  stopPolling();
  pollTimer = setInterval(async () => {
    try {
      const job = await invoke<LegacyMigrationJob>("get_legacy_migration_job", { jobId });
      activeJob.value = job;
      if (job.status === "completed") {
        stopPolling();
        migrating.value = false;
        emit("migrated");
      } else if (job.status === "failed") {
        stopPolling();
        migrating.value = false;
        migrationError.value = job.error?.message_key || "Migration encountered an error";
      }
    } catch (err: any) {
      stopPolling();
      migrating.value = false;
      migrationError.value = typeof err === "string" ? err : err?.message || String(err);
    }
  }, 500);
}

function handleBackToSources() {
  activePreview.value = null;
  previewError.value = null;
  migrationError.value = null;
  activeJob.value = null;
  migrating.value = false;
  void loadStatus();
}

function handleClose() {
  stopPolling();
  emit("close");
}
</script>

<template>
  <div v-if="show" class="modal-backdrop" @click.self="handleClose">
    <div
      class="modal-dialog legacy-migration-dialog"
      role="dialog"
      aria-modal="true"
      :aria-label="t.legacyMigration.title"
    >
      <!-- Header -->
      <div class="modal-header">
        <div class="header-titles">
          <h3>📦 {{ t.legacyMigration.title }}</h3>
          <p class="subtitle">{{ t.legacyMigration.subtitle }}</p>
        </div>
        <button
          type="button"
          class="btn-close"
          :aria-label="t.review.close"
          @click="handleClose"
        >
          ✕
        </button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        <!-- Error banner -->
        <div v-if="statusError" class="alert-box error" role="alert">
          <span>⚠️ {{ statusError }}</span>
          <button type="button" class="btn-retry-sm" @click="loadStatus">
            {{ t.review.retry }}
          </button>
        </div>

        <!-- 1. Source Discovery & Selection Stage -->
        <div v-if="!activePreview && !activeJob && !migrating" class="stage-pane">
          <div v-if="loadingStatus" class="loading-state">
            <span class="spinner">⏳</span>
            <p>{{ t.legacyMigration.previewing }}</p>
          </div>

          <div v-else-if="sources.length === 0" class="empty-state">
            <span class="empty-icon">📂</span>
            <h4>{{ t.legacyMigration.discoveredTitle }}</h4>
            <p>{{ t.nav.noFolders }}</p>
          </div>

          <div v-else class="sources-list-container">
            <h4 class="section-title">{{ t.legacyMigration.discoveredTitle }}</h4>
            <p class="section-desc">{{ t.legacyMigration.discoveredDesc }}</p>

            <div class="source-cards" role="radiogroup" :aria-label="t.legacyMigration.discoveredTitle">
              <label
                v-for="src in sources"
                :key="src.source_id"
                class="source-card"
                :class="{ selected: selectedSourceId === src.source_id, locked: src.is_locked }"
              >
                <div class="card-radio">
                  <input
                    v-model="selectedSourceId"
                    type="radio"
                    name="legacy-source"
                    :value="src.source_id"
                    :disabled="src.is_locked"
                  />
                </div>
                <div class="card-info">
                  <div class="card-header-row">
                    <span class="card-title">{{ src.identifier }}</span>
                    <span
                      class="card-lock-badge"
                      :class="src.is_locked ? 'badge-locked' : 'badge-unlocked'"
                    >
                      {{ src.is_locked ? t.legacyMigration.isLocked : t.legacyMigration.isUnlocked }}
                    </span>
                  </div>
                  <div class="card-meta-grid">
                    <div class="meta-item">
                      <span class="meta-label">{{ t.legacyMigration.fileCount }}:</span>
                      <strong class="meta-val">{{ src.file_count }}</strong>
                    </div>
                    <div class="meta-item">
                      <span class="meta-label">{{ t.legacyMigration.dbSize }}:</span>
                      <span class="meta-val">{{ formatBytes(src.database_size_bytes) }}</span>
                    </div>
                    <div class="meta-item">
                      <span class="meta-label">{{ t.legacyMigration.totalSize }}:</span>
                      <span class="meta-val">{{ formatBytes(src.total_size_bytes) }}</span>
                    </div>
                    <div class="meta-item">
                      <span class="meta-label">{{ t.legacyMigration.schemaVersion }}:</span>
                      <span class="meta-val">v{{ src.schema_version }}</span>
                    </div>
                  </div>
                  <div class="card-path">
                    <code>{{ src.root_path }}</code>
                  </div>
                </div>
              </label>
            </div>

            <div v-if="previewError" class="alert-box error" role="alert">
              <span>⚠️ {{ previewError }}</span>
            </div>
          </div>
        </div>

        <!-- 2. Migration Plan Preview Stage -->
        <div v-else-if="activePreview && !activeJob && !migrating" class="stage-pane">
          <div class="plan-header">
            <h4>📋 {{ t.legacyMigration.previewTitle }}</h4>
            <span class="plan-source-badge">{{ activePreview.source.identifier }}</span>
          </div>

          <div class="plan-summary-grid">
            <div class="summary-card">
              <span class="summary-label">{{ t.legacyMigration.destinationTarget }}</span>
              <code class="summary-val">{{ activePreview.destination_db }}</code>
            </div>
            <div class="summary-card">
              <span class="summary-label">{{ t.legacyMigration.requiredSpace }}</span>
              <strong class="summary-val">{{ formatBytes(activePreview.required_space_bytes) }}</strong>
            </div>
            <div class="summary-card" :class="{ 'warning-bg': isInsufficientSpace }">
              <span class="summary-label">{{ t.legacyMigration.availableSpace }}</span>
              <strong class="summary-val" :style="{ color: isInsufficientSpace ? '#f87171' : 'inherit' }">
                {{ formatBytes(activePreview.available_space_bytes) }}
              </strong>
            </div>
          </div>

          <!-- Space Warning -->
          <div v-if="isInsufficientSpace" class="alert-box warning" role="alert">
            ⚠️ {{ t.legacyMigration.insufficientSpace }}
          </div>

          <!-- Conflicts List -->
          <div v-if="activePreview.conflicts.length > 0" class="conflict-section">
            <h5 class="conflict-title">⚠️ {{ t.legacyMigration.conflictsTitle }}</h5>
            <ul class="conflict-list">
              <li v-for="(conflict, idx) in activePreview.conflicts" :key="idx" class="conflict-item">
                {{ conflict }}
              </li>
            </ul>
          </div>
          <div v-else class="alert-box success">
            ✓ {{ t.legacyMigration.noConflicts }}
          </div>

          <!-- Exclusions List -->
          <div v-if="activePreview.exclusions.length > 0" class="exclusion-section">
            <h5 class="exclusion-title">ℹ️ {{ t.legacyMigration.exclusionsTitle }}</h5>
            <ul class="exclusion-list">
              <li v-for="(ex, idx) in activePreview.exclusions" :key="idx" class="exclusion-item">
                {{ ex }}
              </li>
            </ul>
          </div>

          <div v-if="migrationError" class="alert-box error" role="alert">
            <span>⚠️ {{ migrationError }}</span>
          </div>
        </div>

        <!-- 3. Active Migration Progress Stage -->
        <div v-else-if="migrating" class="stage-pane progress-pane">
          <div class="progress-hero">
            <span class="spinner-large">⚡</span>
            <h4>{{ t.legacyMigration.migrating }}</h4>
            <p class="progress-step-text">
              {{ activeJob?.current_step || t.legacyMigration.stepCopying }}
            </p>
          </div>

          <div class="progress-bar-container">
            <div
              class="progress-bar-fill"
              :style="{ width: `${Math.round((activeJob?.progress || 0.1) * 100)}%` }"
            ></div>
          </div>
          <span class="progress-percentage">
            {{ Math.round((activeJob?.progress || 0.1) * 100) }}%
          </span>

          <p class="source-preservation-notice">
            🛡️ {{ t.legacyMigration.sourcePreservedNotice }}
          </p>
        </div>

        <!-- 4. Migration Complete Stage -->
        <div v-else-if="activeJob && activeJob.status === 'completed'" class="stage-pane complete-pane">
          <div class="complete-hero">🎉</div>
          <h4>{{ t.legacyMigration.completeTitle }}</h4>
          <p class="complete-desc">{{ t.legacyMigration.completeDesc }}</p>

          <div class="receipt-card">
            <div class="receipt-row">
              <span>{{ t.legacyMigration.sourceLabel }}:</span>
              <strong>{{ activeJob.receipt?.source_identifier }}</strong>
            </div>
            <div class="receipt-row">
              <span>{{ t.legacyMigration.migratedCount.replace('{count}', String(activeJob.receipt?.artifacts.length || 0)) }}</span>
            </div>
            <div class="receipt-row source-safe">
              <span>🛡️ {{ t.legacyMigration.sourcePreservedNotice }}</span>
            </div>
          </div>
        </div>

        <!-- 5. Migration Failed / Error Stage with Retry -->
        <div v-else-if="activeJob && activeJob.status === 'failed'" class="stage-pane failed-pane">
          <div class="failed-hero">❌</div>
          <h4>{{ t.legacyMigration.failedTitle }}</h4>
          <p class="failed-desc">{{ migrationError || activeJob.error?.message_key }}</p>

          <div v-if="activeJob.error?.context" class="error-context-box">
            <span class="context-label">{{ t.legacyMigration.errorDetails }}:</span>
            <code>{{ activeJob.error.context }}</code>
          </div>

          <p class="source-safe-text">
            🛡️ {{ t.legacyMigration.sourcePreservedNotice }}
          </p>
        </div>
      </div>

      <!-- Footer Controls -->
      <div class="modal-footer">
        <button
          type="button"
          class="btn-secondary"
          :disabled="migrating"
          @click="handleClose"
        >
          {{ activeJob?.status === 'completed' ? t.legacyMigration.close : t.onboarding.skip }}
        </button>

        <div class="footer-actions">
          <!-- Step 1: Preview action -->
          <button
            v-if="!activePreview && !activeJob && !migrating"
            type="button"
            class="btn-primary"
            :disabled="!selectedSourceId || selectedSource?.is_locked || previewing || loadingStatus"
            @click="handlePreview"
          >
            {{ previewing ? t.legacyMigration.previewing : t.legacyMigration.previewButton }}
          </button>

          <!-- Step 2: Back & Start Migration -->
          <template v-else-if="activePreview && !activeJob && !migrating">
            <button
              type="button"
              class="btn-secondary"
              @click="handleBackToSources"
            >
              {{ t.legacyMigration.backToSources }}
            </button>
            <button
              type="button"
              class="btn-primary"
              :disabled="isInsufficientSpace"
              @click="handleStartMigration"
            >
              🚀 {{ t.legacyMigration.startMigration }}
            </button>
          </template>

          <!-- Step 4/5: Retry & Done -->
          <template v-else-if="activeJob?.status === 'failed'">
            <button
              type="button"
              class="btn-secondary"
              @click="handleBackToSources"
            >
              {{ t.legacyMigration.backToSources }}
            </button>
            <button
              type="button"
              class="btn-primary"
              @click="handleStartMigration"
            >
              🔄 {{ t.legacyMigration.retryButton }}
            </button>
          </template>

          <button
            v-else-if="activeJob?.status === 'completed'"
            type="button"
            class="btn-primary"
            @click="handleClose"
          >
            ✓ {{ t.legacyMigration.close }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
  z-index: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}

.legacy-migration-dialog {
  background: var(--color-bg-primary, #1e1e2e);
  color: var(--color-text-primary, #cdd6f4);
  border: 1px solid var(--border-color, #313244);
  border-radius: 12px;
  width: 100%;
  max-width: 680px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.4);
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color, #313244);
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.header-titles h3 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-text-primary, #cdd6f4);
}

.header-titles .subtitle {
  margin: 4px 0 0 0;
  font-size: 0.85rem;
  color: var(--color-text-secondary, #a6adc8);
}

.btn-close {
  background: transparent;
  border: none;
  font-size: 1.1rem;
  color: var(--color-text-secondary, #a6adc8);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.btn-close:hover {
  background: var(--color-bg-secondary, #181825);
  color: var(--color-text-primary, #cdd6f4);
}

.modal-body {
  padding: 20px 24px;
  overflow-y: auto;
  flex: 1;
}

.stage-pane {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-title {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--color-text-primary, #cdd6f4);
}

.section-desc {
  margin: 4px 0 12px 0;
  font-size: 0.85rem;
  color: var(--color-text-secondary, #a6adc8);
}

.source-cards {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.source-card {
  display: flex;
  gap: 14px;
  padding: 14px 16px;
  background: var(--color-bg-secondary, #181825);
  border: 1px solid var(--border-color, #313244);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.source-card:hover:not(.locked) {
  border-color: var(--color-accent, #89b4fa);
}

.source-card.selected {
  border-color: var(--color-accent, #89b4fa);
  background: rgba(137, 180, 250, 0.08);
}

.source-card.locked {
  opacity: 0.6;
  cursor: not-allowed;
}

.card-radio {
  margin-top: 2px;
}

.card-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.card-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-title {
  font-weight: 600;
  font-size: 0.95rem;
  color: var(--color-text-primary, #cdd6f4);
}

.card-lock-badge {
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 500;
}

.badge-unlocked {
  background: rgba(166, 227, 161, 0.15);
  color: #a6e3a1;
}

.badge-locked {
  background: rgba(243, 139, 168, 0.15);
  color: #f38ba8;
}

.card-meta-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
  gap: 8px;
  font-size: 0.82rem;
}

.meta-item {
  display: flex;
  gap: 4px;
  color: var(--color-text-secondary, #a6adc8);
}

.meta-val {
  color: var(--color-text-primary, #cdd6f4);
}

.card-path {
  font-size: 0.78rem;
  color: var(--color-text-secondary, #a6adc8);
  word-break: break-all;
}

.plan-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.plan-header h4 {
  margin: 0;
  font-size: 1.1rem;
}

.plan-source-badge {
  background: rgba(137, 180, 250, 0.15);
  color: #89b4fa;
  padding: 3px 10px;
  border-radius: 12px;
  font-size: 0.8rem;
  font-weight: 500;
}

.plan-summary-grid {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr;
  gap: 12px;
}

.summary-card {
  padding: 12px;
  background: var(--color-bg-secondary, #181825);
  border: 1px solid var(--border-color, #313244);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.summary-card.warning-bg {
  border-color: #f38ba8;
  background: rgba(243, 139, 168, 0.08);
}

.summary-label {
  font-size: 0.78rem;
  color: var(--color-text-secondary, #a6adc8);
}

.summary-val {
  font-size: 0.92rem;
  word-break: break-all;
}

.alert-box {
  padding: 12px 14px;
  border-radius: 8px;
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.alert-box.error {
  background: rgba(243, 139, 168, 0.12);
  border: 1px solid #f38ba8;
  color: #f38ba8;
}

.alert-box.warning {
  background: rgba(249, 226, 175, 0.12);
  border: 1px solid #f9e2af;
  color: #f9e2af;
}

.alert-box.success {
  background: rgba(166, 227, 161, 0.12);
  border: 1px solid #a6e3a1;
  color: #a6e3a1;
}

.conflict-section,
.exclusion-section {
  padding: 12px 14px;
  background: var(--color-bg-secondary, #181825);
  border: 1px solid var(--border-color, #313244);
  border-radius: 8px;
}

.conflict-title,
.exclusion-title {
  margin: 0 0 8px 0;
  font-size: 0.9rem;
}

.conflict-list,
.exclusion-list {
  margin: 0;
  padding-left: 20px;
  font-size: 0.82rem;
  color: var(--color-text-secondary, #a6adc8);
}

.progress-pane,
.complete-pane,
.failed-pane {
  align-items: center;
  text-align: center;
  padding: 24px 0;
}

.spinner-large {
  font-size: 2.2rem;
  display: inline-block;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.95); opacity: 0.8; }
  50% { transform: scale(1.05); opacity: 1; }
  100% { transform: scale(0.95); opacity: 0.8; }
}

.progress-bar-container {
  width: 100%;
  max-width: 440px;
  height: 8px;
  background: var(--color-bg-secondary, #181825);
  border-radius: 4px;
  overflow: hidden;
  margin: 12px 0 4px 0;
}

.progress-bar-fill {
  height: 100%;
  background: var(--color-accent, #89b4fa);
  transition: width 0.3s ease;
}

.progress-percentage {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-accent, #89b4fa);
}

.source-preservation-notice {
  font-size: 0.82rem;
  color: var(--color-text-secondary, #a6adc8);
  margin-top: 16px;
}

.complete-hero,
.failed-hero {
  font-size: 3rem;
  margin-bottom: 8px;
}

.receipt-card {
  width: 100%;
  max-width: 460px;
  padding: 16px;
  background: var(--color-bg-secondary, #181825);
  border: 1px solid var(--border-color, #313244);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 12px;
  text-align: left;
  font-size: 0.85rem;
}

.receipt-row {
  display: flex;
  justify-content: space-between;
}

.source-safe {
  color: #a6e3a1;
  font-weight: 500;
  margin-top: 4px;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid var(--border-color, #313244);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-actions {
  display: flex;
  gap: 10px;
}

.btn-primary {
  background: var(--color-accent, #89b4fa);
  color: #11111b;
  border: none;
  font-weight: 600;
  padding: 8px 18px;
  border-radius: 6px;
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--color-bg-secondary, #181825);
  color: var(--color-text-primary, #cdd6f4);
  border: 1px solid var(--border-color, #313244);
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover:not(:disabled) {
  background: var(--border-color, #313244);
}
</style>
