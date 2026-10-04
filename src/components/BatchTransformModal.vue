<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { t } from "../i18n";
import type {
  Folder,
  ImageFile,
  TransformFormat,
  TransformMetadataPolicy,
  TransformCollisionPolicy,
  TransformSpec,
  OriginalDisposition,
  LibraryTransformRequest,
  TransformJobReceipt,
  TransformProgressEvent,
} from "../types";

const props = defineProps<{
  show: boolean;
  files: ImageFile[];
  folders: Folder[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "completed", receipt: TransformJobReceipt): void;
}>();

// Separate managed files from external linked folder files
const managedFiles = computed(() =>
  props.files.filter((f) => {
    const folder = props.folders.find((fd) => fd.id === f.folder_id);
    return folder && folder.folder_type === "managed";
  })
);

const linkedFiles = computed(() =>
  props.files.filter((f) => {
    const folder = props.folders.find((fd) => fd.id === f.folder_id);
    return !folder || folder.folder_type !== "managed";
  })
);

const format = ref<TransformFormat>("webp");
const quality = ref<number>(85);
const maxEdgePreset = ref<"none" | "1080" | "2048" | "3840" | "custom">("none");
const customMaxEdge = ref<number>(1920);

const metadataPolicy = ref<TransformMetadataPolicy>("keep_supported");
const collisionPolicy = ref<TransformCollisionPolicy>("rename");
const originalDisposition = ref<OriginalDisposition>("keep");

const transforming = ref<boolean>(false);
const progressCurrent = ref<number>(0);
const progressTotal = ref<number>(0);
const progressFile = ref<string>("");

const error = ref<string | null>(null);
const receipt = ref<TransformJobReceipt | null>(null);

let unlistenProgress: UnlistenFn[] = [];

watch(
  () => props.show,
  async (newVal) => {
    if (newVal) {
      error.value = null;
      receipt.value = null;
      transforming.value = false;
      progressCurrent.value = 0;
      progressTotal.value = 0;
      progressFile.value = "";
      originalDisposition.value = "keep";

      if (unlistenProgress.length === 0) {
        try {
          const u1 = await listen<TransformProgressEvent>(
            "omera://transform-progress",
            (e) => {
              progressCurrent.value = e.payload.current;
              progressTotal.value = e.payload.total;
              progressFile.value = e.payload.current_path;
            }
          );
          const u2 = await listen<TransformProgressEvent>(
            "berry://transform-progress",
            (e) => {
              progressCurrent.value = e.payload.current;
              progressTotal.value = e.payload.total;
              progressFile.value = e.payload.current_path;
            }
          );
          unlistenProgress = [u1, u2];
        } catch (err) {
          console.warn("Failed to attach transform progress listeners:", err);
        }
      }
    } else {
      unlistenProgress.forEach((fn) => fn());
      unlistenProgress = [];
    }
  },
  { immediate: true }
);

onUnmounted(() => {
  unlistenProgress.forEach((fn) => fn());
  unlistenProgress = [];
});

const progressPercent = computed(() => {
  if (progressTotal.value === 0) return 0;
  return Math.min(100, Math.round((progressCurrent.value / progressTotal.value) * 100));
});

function getFilename(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

async function handleStartTransform() {
  if (managedFiles.value.length === 0) return;

  transforming.value = true;
  error.value = null;
  receipt.value = null;
  progressCurrent.value = 0;
  progressTotal.value = managedFiles.value.length;
  progressFile.value = "";

  let max_edge: number | null = null;
  if (maxEdgePreset.value === "1080") max_edge = 1920;
  else if (maxEdgePreset.value === "2048") max_edge = 2048;
  else if (maxEdgePreset.value === "3840") max_edge = 3840;
  else if (maxEdgePreset.value === "custom" && customMaxEdge.value > 0) {
    max_edge = customMaxEdge.value;
  }

  const spec: TransformSpec = {
    format: format.value,
    quality: format.value === "jpeg" || format.value === "avif" ? quality.value : null,
    max_edge,
    metadata_policy: metadataPolicy.value,
    collision_policy: collisionPolicy.value,
  };

  const fileIds = managedFiles.value
    .map((f) => f.id)
    .filter((id): id is number => typeof id === "number");

  const request: LibraryTransformRequest = {
    file_ids: fileIds,
    spec,
    original_disposition: originalDisposition.value,
  };

  try {
    const jobReceipt = await invoke<TransformJobReceipt>(
      "transform_library_files_batch",
      { request }
    );
    receipt.value = jobReceipt;
    emit("completed", jobReceipt);
  } catch (err: any) {
    error.value = typeof err === "string" ? err : err?.message || String(err);
  } finally {
    transforming.value = false;
  }
}
</script>

<template>
  <div
    v-if="show"
    class="modal-overlay"
    @click.self="!transforming && emit('close')"
    v-dialog="() => !transforming && emit('close')"
  >
    <div class="modal-dialog">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-titles">
          <h3 class="modal-title">⚡ {{ t.batchTransformModal.title }}</h3>
          <p class="modal-subtitle">
            {{ t.batchTransformModal.subtitle }}
          </p>
        </div>
        <button
          type="button"
          class="close-btn"
          :disabled="transforming"
          @click="emit('close')"
        >
          ✕
        </button>
      </div>

      <!-- Preflight Count Badge & Warning -->
      <div class="preflight-banner">
        <div class="preflight-stat">
          <span class="badge managed-badge">
            📁 {{ t.batchTransformModal.eligibleCount.replace('{count}', String(managedFiles.length)) }}
          </span>
        </div>
        <div v-if="linkedFiles.length > 0" class="preflight-warning">
          <span>
            ℹ️ {{ t.batchTransformModal.externalLinkNotice.replace('{count}', String(linkedFiles.length)) }}
          </span>
        </div>
      </div>

      <!-- Error Alert -->
      <div v-if="error" class="alert-box error">
        <span>⚠️ {{ error }}</span>
      </div>

      <!-- Success / Results Summary Banner -->
      <div v-if="receipt !== null" class="summary-box success">
        <div class="summary-header">
          <span class="summary-icon">✅</span>
          <span class="summary-title">{{ t.batchTransformModal.completedTitle }}</span>
        </div>
        <div class="summary-stats-grid">
          <div class="summary-stat-item">
            <span class="stat-label">{{ t.batchTransformModal.summarySucceeded.replace('{count}', '') }}</span>
            <span class="stat-value text-success">{{ receipt.succeeded }}</span>
          </div>
          <div v-if="receipt.failed > 0" class="summary-stat-item">
            <span class="stat-label">{{ t.batchTransformModal.summaryFailed.replace('{count}', '') }}</span>
            <span class="stat-value text-danger">{{ receipt.failed }}</span>
          </div>
          <div v-if="receipt.skipped > 0" class="summary-stat-item">
            <span class="stat-label">{{ t.batchTransformModal.summarySkipped.replace('{count}', '') }}</span>
            <span class="stat-value text-muted">{{ receipt.skipped }}</span>
          </div>
        </div>
      </div>

      <!-- Modal Body (Settings) -->
      <div v-if="receipt === null" class="modal-body">
        <!-- 1. Output Format & Quality -->
        <div class="section-card">
          <h4 class="section-title">🎨 {{ t.batchTransformModal.transcodeSection }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.batchTransformModal.format }}</label>
              <select
                v-model="format"
                class="form-select"
                :disabled="transforming"
              >
                <option value="webp">{{ t.batchTransformModal.formatWebp }}</option>
                <option value="avif">{{ t.batchTransformModal.formatAvif }}</option>
                <option value="jpeg">{{ t.batchTransformModal.formatJpeg }}</option>
                <option value="png">{{ t.batchTransformModal.formatPng }}</option>
                <option value="original">{{ t.batchTransformModal.formatOriginal }}</option>
              </select>
            </div>

            <!-- Quality slider for lossy codecs (JPEG & AVIF) -->
            <div
              v-if="format === 'jpeg' || format === 'avif'"
              class="form-group"
            >
              <div class="label-with-value">
                <label class="form-label">{{ t.batchTransformModal.quality }}</label>
                <span class="value-badge">{{ quality }}%</span>
              </div>
              <input
                v-model.number="quality"
                type="range"
                min="10"
                max="100"
                step="5"
                class="form-range"
                :disabled="transforming"
              />
            </div>

            <div v-else-if="format === 'webp'" class="form-group">
              <label class="form-label">{{ t.batchTransformModal.quality }}</label>
              <div class="lossless-badge">
                <span>🛡️ {{ t.exportModal.losslessWebpNote }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 2. Resolution Constraint Preset -->
        <div class="section-card">
          <h4 class="section-title">📐 {{ t.batchTransformModal.dimensionSection }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.batchTransformModal.maxEdge }}</label>
              <select
                v-model="maxEdgePreset"
                class="form-select"
                :disabled="transforming"
              >
                <option value="none">{{ t.batchTransformModal.maxEdgeNone }}</option>
                <option value="1080">{{ t.batchTransformModal.maxEdge1080p }}</option>
                <option value="2048">{{ t.batchTransformModal.maxEdge2k }}</option>
                <option value="3840">{{ t.batchTransformModal.maxEdge4k }}</option>
                <option value="custom">{{ t.batchTransformModal.maxEdgeCustom }}</option>
              </select>
            </div>

            <div v-if="maxEdgePreset === 'custom'" class="form-group">
              <label class="form-label">{{ t.batchTransformModal.customEdgePlaceholder }}</label>
              <input
                v-model.number="customMaxEdge"
                type="number"
                min="64"
                max="16384"
                step="64"
                class="form-input"
                :disabled="transforming"
              />
            </div>
          </div>
        </div>

        <!-- 3. Original File Disposition (Safety Selection) -->
        <div class="section-card">
          <h4 class="section-title">📦 {{ t.batchTransformModal.dispositionSection }}</h4>
          <div class="disposition-options">
            <label
              class="disposition-card"
              :class="{ selected: originalDisposition === 'keep' }"
            >
              <input
                v-model="originalDisposition"
                type="radio"
                value="keep"
                class="radio-input"
                :disabled="transforming"
              />
              <div class="disposition-content">
                <span class="disposition-title">🛡️ {{ t.batchTransformModal.dispositionKeep }}</span>
                <p class="disposition-desc">{{ t.batchTransformModal.dispositionKeepDesc }}</p>
              </div>
            </label>

            <label
              class="disposition-card"
              :class="{ selected: originalDisposition === 'archive' }"
            >
              <input
                v-model="originalDisposition"
                type="radio"
                value="archive"
                class="radio-input"
                :disabled="transforming"
              />
              <div class="disposition-content">
                <span class="disposition-title">📦 {{ t.batchTransformModal.dispositionArchive }}</span>
                <p class="disposition-desc">{{ t.batchTransformModal.dispositionArchiveDesc }}</p>
              </div>
            </label>

            <label
              class="disposition-card"
              :class="{ selected: originalDisposition === 'trash' }"
            >
              <input
                v-model="originalDisposition"
                type="radio"
                value="trash"
                class="radio-input"
                :disabled="transforming"
              />
              <div class="disposition-content">
                <span class="disposition-title">🗑 {{ t.batchTransformModal.dispositionTrash }}</span>
                <p class="disposition-desc">{{ t.batchTransformModal.dispositionTrashDesc }}</p>
              </div>
            </label>
          </div>
        </div>

        <!-- 4. Policies (Metadata & Collision) -->
        <div class="section-card">
          <h4 class="section-title">⚙️ {{ t.batchTransformModal.metadataPolicy }} & {{ t.batchTransformModal.collisionPolicy }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.batchTransformModal.metadataPolicy }}</label>
              <select
                v-model="metadataPolicy"
                class="form-select"
                :disabled="transforming"
              >
                <option value="keep_supported">{{ t.batchTransformModal.keepSupported }}</option>
                <option value="strip_ai">{{ t.batchTransformModal.stripAi }}</option>
                <option value="strip_all">{{ t.batchTransformModal.stripAll }}</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label">{{ t.batchTransformModal.collisionPolicy }}</label>
              <select
                v-model="collisionPolicy"
                class="form-select"
                :disabled="transforming"
              >
                <option value="rename">{{ t.batchTransformModal.collisionRename }}</option>
                <option value="skip">{{ t.batchTransformModal.collisionSkip }}</option>
              </select>
            </div>
          </div>

          <!-- Non-destructive safety note -->
          <div class="safety-notice">
            <span>🔒 {{ t.batchTransformModal.safetyGuarantee }}</span>
          </div>
        </div>

        <!-- Progress Bar & Active item -->
        <div v-if="transforming" class="progress-section">
          <div class="progress-info">
            <span class="progress-status">⏳ {{ t.batchTransformModal.transforming }}</span>
            <span class="progress-count">{{ progressCurrent }} / {{ progressTotal }} ({{ progressPercent }}%)</span>
          </div>
          <div class="progress-track">
            <div
              class="progress-fill"
              :style="{ width: `${progressPercent}%` }"
            ></div>
          </div>
          <div v-if="progressFile" class="progress-file-info">
            <span class="current-file-label">{{ getFilename(progressFile) }}</span>
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="modal-footer">
        <div class="footer-left"></div>
        <div class="footer-right">
          <button
            type="button"
            class="btn-secondary"
            :disabled="transforming"
            @click="emit('close')"
          >
            {{ receipt !== null ? t.batchTransformModal.close : t.exportModal.cancel }}
          </button>

          <button
            v-if="receipt === null"
            type="button"
            class="btn-primary"
            :disabled="managedFiles.length === 0 || transforming"
            @click="handleStartTransform"
          >
            {{ transforming ? '⏳ ...' : '⚡ ' + t.batchTransformModal.startTransform }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
  padding: 1.5rem;
}

.modal-dialog {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  width: 100%;
  max-width: 640px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.4);
  color: var(--color-text-primary);
  overflow: hidden;
}

.modal-header {
  padding: 1.15rem 1.5rem;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
}

.header-titles {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.modal-title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.modal-subtitle {
  margin: 0;
  font-size: 0.85rem;
  color: var(--color-text-muted);
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 1.25rem;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 0.2rem 0.5rem;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.close-btn:hover:not(:disabled) {
  color: var(--color-text-primary);
  background: var(--color-bg-tertiary);
}

.preflight-banner {
  padding: 0.75rem 1.5rem;
  background: var(--color-bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.managed-badge {
  background: var(--color-accent-subtle, rgba(99, 102, 241, 0.15));
  color: var(--color-accent, #6366f1);
  padding: 0.2rem 0.6rem;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 500;
}

.preflight-warning {
  font-size: 0.8rem;
  color: var(--color-text-muted);
  line-height: 1.4;
}

.modal-body {
  padding: 1.25rem 1.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.section-card {
  background: var(--color-bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.section-title {
  margin: 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.grid-2-cols {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.85rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.form-label {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--color-text-muted);
}

.label-with-value {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.value-badge {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-accent, #6366f1);
}

.lossless-badge {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  padding: 0.4rem 0.6rem;
  background: var(--color-bg-tertiary);
  border: 1px dashed var(--border-color);
  border-radius: 6px;
}

.form-select,
.form-input {
  background: var(--color-bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 0.5rem 0.65rem;
  color: var(--color-text-primary);
  font-size: 0.85rem;
  outline: none;
  transition: border-color 0.15s ease;
}

.form-select:focus,
.form-input:focus {
  border-color: var(--color-accent, #6366f1);
}

.form-range {
  accent-color: var(--color-accent, #6366f1);
  cursor: pointer;
  height: 20px;
}

.disposition-options {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.disposition-card {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 0.75rem 0.85rem;
  background: var(--color-bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.disposition-card:hover {
  background: var(--color-bg-hover, var(--color-bg-tertiary));
}

.disposition-card.selected {
  border-color: var(--color-accent, #6366f1);
  background: var(--color-accent-subtle, rgba(99, 102, 241, 0.08));
}

.radio-input {
  margin-top: 0.2rem;
  accent-color: var(--color-accent, #6366f1);
  cursor: pointer;
}

.disposition-content {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.disposition-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.disposition-desc {
  margin: 0;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  line-height: 1.35;
}

.safety-notice {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  background: var(--color-bg-tertiary);
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border-left: 3px solid var(--color-accent, #6366f1);
  line-height: 1.35;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
  background: var(--color-bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.8rem;
  font-weight: 500;
}

.progress-track {
  width: 100%;
  height: 8px;
  background: var(--color-bg-tertiary);
  border-radius: 999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--color-accent, #6366f1);
  border-radius: 999px;
  transition: width 0.15s ease-out;
}

.progress-file-info {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.alert-box {
  margin: 0.75rem 1.5rem 0;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  font-size: 0.85rem;
}

.alert-box.error {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

.summary-box {
  margin: 1rem 1.5rem;
  padding: 1rem 1.25rem;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.summary-box.success {
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.25);
  color: var(--color-text-primary);
}

.summary-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.summary-title {
  font-weight: 600;
  font-size: 0.95rem;
}

.summary-stats-grid {
  display: flex;
  gap: 1.5rem;
  padding-top: 0.25rem;
}

.summary-stat-item {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.stat-value {
  font-size: 1.25rem;
  font-weight: 700;
}

.text-success {
  color: #22c55e;
}

.text-danger {
  color: #ef4444;
}

.text-muted {
  color: var(--color-text-muted);
}

.modal-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.footer-right {
  display: flex;
  gap: 0.75rem;
}

.btn-secondary {
  background: var(--color-bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 0.5rem 1rem;
  color: var(--color-text-primary);
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-secondary:hover:not(:disabled) {
  background: var(--color-bg-hover, var(--color-bg-tertiary));
}

.btn-primary {
  background: var(--color-accent, #6366f1);
  border: none;
  border-radius: 6px;
  padding: 0.5rem 1.25rem;
  color: #ffffff;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled,
.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
