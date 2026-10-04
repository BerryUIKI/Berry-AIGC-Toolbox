<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import type {
  Folder,
  Album,
  TransformFormat,
  TransformMetadataPolicy,
  TransformCollisionPolicy,
  TransformSpec,
} from "../types";

const props = defineProps<{
  show: boolean;
  filePaths: string[];
  initialFolderId?: number | null;
  initialAlbumId?: number | null;
  folders: Folder[];
  albums: Album[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "imported", fileIds: number[]): void;
}>();

const managedFolders = computed(() =>
  props.folders.filter((f) => f.folder_type === "managed")
);

const targetFolderId = ref<number | null>(null);
const targetAlbumId = ref<number | null>(null);

const selectedPreset = ref<string>("custom");
const format = ref<TransformFormat>("original");
const quality = ref<number>(85);
const maxEdgePreset = ref<"none" | "1080" | "2048" | "3840" | "custom">("none");
const customMaxEdge = ref<number>(1920);
const scalePercent = ref<number>(100);
const alignMultiple = ref<number>(0);
const targetSizeKb = ref<number | null>(null);

function applyPreset(presetId: string) {
  selectedPreset.value = presetId;
  if (presetId === "web_fast") {
    format.value = "webp";
    quality.value = 80;
    maxEdgePreset.value = "1080";
    scalePercent.value = 100;
    alignMultiple.value = 0;
    targetSizeKb.value = null;
    metadataPolicy.value = "strip_ai";
  } else if (presetId === "archive_avif") {
    format.value = "avif";
    quality.value = 65;
    maxEdgePreset.value = "none";
    scalePercent.value = 100;
    alignMultiple.value = 8;
    targetSizeKb.value = null;
    metadataPolicy.value = "strip_ai";
  } else if (presetId === "target_1mb") {
    format.value = "webp";
    quality.value = 85;
    maxEdgePreset.value = "none";
    scalePercent.value = 100;
    alignMultiple.value = 0;
    targetSizeKb.value = 1024;
    metadataPolicy.value = "keep_supported";
  } else if (presetId === "lossless_webp") {
    format.value = "webp";
    quality.value = 85;
    maxEdgePreset.value = "none";
    scalePercent.value = 100;
    alignMultiple.value = 0;
    targetSizeKb.value = null;
    metadataPolicy.value = "keep_supported";
  }
}

const metadataPolicy = ref<TransformMetadataPolicy>("keep_supported");
const collisionPolicy = ref<TransformCollisionPolicy>("rename");

const importing = ref<boolean>(false);
const error = ref<string | null>(null);
const successCount = ref<number | null>(null);

watch(
  () => props.show,
  (newVal) => {
    if (newVal) {
      error.value = null;
      successCount.value = null;
      importing.value = false;

      // Initialize destination vault
      if (props.initialFolderId && managedFolders.value.some((f) => f.id === props.initialFolderId)) {
        targetFolderId.value = props.initialFolderId;
      } else if (managedFolders.value.length > 0) {
        targetFolderId.value = managedFolders.value[0].id;
      } else {
        targetFolderId.value = null;
      }

      // Initialize destination album
      if (props.initialAlbumId && props.albums.some((a) => a.id === props.initialAlbumId)) {
        targetAlbumId.value = props.initialAlbumId;
      } else {
        targetAlbumId.value = null;
      }
    }
  },
  { immediate: true }
);

function getFolderName(path: string): string {
  const parts = path.replace(/[\\/]+$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

async function handleStartImport() {
  if (!targetFolderId.value) {
    error.value = t.value.importModal.selectManagedVault;
    return;
  }
  if (props.filePaths.length === 0) return;

  importing.value = true;
  error.value = null;
  successCount.value = null;

  let max_edge: number | null = null;
  if (maxEdgePreset.value === "1080") max_edge = 1920;
  else if (maxEdgePreset.value === "2048") max_edge = 2048;
  else if (maxEdgePreset.value === "3840") max_edge = 3840;
  else if (maxEdgePreset.value === "custom" && customMaxEdge.value > 0) {
    max_edge = customMaxEdge.value;
  }

  const transformSpec: TransformSpec = {
    format: format.value,
    quality: format.value === "jpeg" || format.value === "avif" ? quality.value : null,
    max_edge,
    scale_percent: scalePercent.value !== 100 ? scalePercent.value : null,
    align_multiple: alignMultiple.value > 1 ? alignMultiple.value : null,
    target_size_kb: targetSizeKb.value && targetSizeKb.value > 0 ? targetSizeKb.value : null,
    metadata_policy: metadataPolicy.value,
    collision_policy: collisionPolicy.value,
  };

  try {
    const importedIds = await invoke<number[]>("import_files_to_managed_vault", {
      filePaths: props.filePaths,
      targetFolderId: targetFolderId.value,
      targetAlbumId: targetAlbumId.value || null,
      transformSpec,
    });
    successCount.value = importedIds.length;
    emit("imported", importedIds);
  } catch (err: any) {
    error.value = typeof err === "string" ? err : err?.message || String(err);
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="modal-dialog">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-titles">
          <h3 class="modal-title">📥 {{ t.importModal.title }}</h3>
          <p class="modal-subtitle">
            {{ t.importModal.itemsCount.replace('{count}', String(filePaths.length)) }}
          </p>
        </div>
        <button type="button" class="close-btn" :disabled="importing" @click="emit('close')">✕</button>
      </div>

      <!-- Error Alert -->
      <div v-if="error" class="alert-box error">
        <span>⚠️ {{ error }}</span>
      </div>

      <!-- Success Banner -->
      <div v-if="successCount !== null" class="summary-box success">
        <div class="summary-header">
          <span class="summary-icon">✅</span>
          <span class="summary-text">
            {{ t.importModal.importSuccess.replace('{count}', String(successCount)) }}
          </span>
        </div>
      </div>

      <!-- Modal Body -->
      <div class="modal-body">
        <!-- 0. Quick Preset Selector (T4) -->
        <div class="section-card">
          <h4 class="section-title">⚡ {{ t.importModal.presetsTitle }}</h4>
          <div class="form-group">
            <select
              :value="selectedPreset"
              @change="applyPreset(($event.target as HTMLSelectElement).value)"
              class="form-select"
              :disabled="importing || successCount !== null"
            >
              <option value="custom">{{ t.importModal.presetCustom }}</option>
              <option value="web_fast">{{ t.importModal.presetWebFast }}</option>
              <option value="archive_avif">{{ t.importModal.presetArchiveAvif }}</option>
              <option value="target_1mb">{{ t.importModal.presetTarget1mb }}</option>
              <option value="lossless_webp">{{ t.importModal.presetLosslessWebp }}</option>
            </select>
          </div>
        </div>

        <!-- 1. Destination Managed Vault & Album -->
        <div class="section-card">
          <h4 class="section-title">📦 {{ t.importModal.targetVault }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.importModal.targetVault }}</label>
              <select v-model="targetFolderId" class="form-select" :disabled="importing || successCount !== null">
                <option v-if="managedFolders.length === 0" :value="null" disabled>
                  {{ t.importModal.selectManagedVault }}
                </option>
                <option v-for="folder in managedFolders" :key="folder.id" :value="folder.id">
                  📦 {{ getFolderName(folder.path) }}
                </option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label">{{ t.importModal.targetAlbum }}</label>
              <select v-model="targetAlbumId" class="form-select" :disabled="importing || successCount !== null">
                <option :value="null">{{ t.importModal.noAlbum }}</option>
                <option v-for="album in albums" :key="album.id" :value="album.id">
                  🗂️ {{ album.name }}
                </option>
              </select>
            </div>
          </div>
        </div>

        <!-- 2. Transcoding & Compression -->
        <div class="section-card">
          <h4 class="section-title">🎨 {{ t.importModal.transcodeSection }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.importModal.format }}</label>
              <select v-model="format" class="form-select" :disabled="importing || successCount !== null">
                <option value="original">{{ t.importModal.formatOriginal }}</option>
                <option value="webp">{{ t.importModal.formatWebp }}</option>
                <option value="avif">{{ t.importModal.formatAvif }}</option>
                <option value="jpeg">{{ t.importModal.formatJpeg }}</option>
                <option value="png">{{ t.importModal.formatPng }}</option>
              </select>
            </div>

            <div v-if="format === 'jpeg' || format === 'avif'" class="form-group">
              <div class="label-with-value">
                <label class="form-label">{{ t.importModal.quality }}</label>
                <span class="value-badge">{{ quality }}%</span>
              </div>
              <input
                v-model.number="quality"
                type="range"
                min="10"
                max="100"
                step="5"
                class="form-range"
                :disabled="importing || successCount !== null"
              />
            </div>
            <div v-else-if="format === 'webp'" class="form-group">
              <label class="form-label">{{ t.importModal.quality }}</label>
              <div class="lossless-badge">
                <span>🛡️ {{ t.exportModal.losslessWebpNote }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 3. Resolution Constraint & Scaling (T4) -->
        <div class="section-card">
          <h4 class="section-title">📐 {{ t.importModal.dimensionSection }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.importModal.maxEdge }}</label>
              <select v-model="maxEdgePreset" class="form-select" :disabled="importing || successCount !== null">
                <option value="none">{{ t.importModal.maxEdgeNone }}</option>
                <option value="1080">{{ t.importModal.maxEdge1080p }}</option>
                <option value="2048">{{ t.importModal.maxEdge2k }}</option>
                <option value="3840">{{ t.importModal.maxEdge4k }}</option>
                <option value="custom">{{ t.importModal.maxEdgeCustom }}</option>
              </select>
            </div>

            <div v-if="maxEdgePreset === 'custom'" class="form-group">
              <label class="form-label">Custom Max Edge (px)</label>
              <input
                v-model.number="customMaxEdge"
                type="number"
                min="64"
                max="16384"
                step="64"
                class="form-input"
                :disabled="importing || successCount !== null"
              />
            </div>
          </div>

          <!-- Percentage scaling & pixel alignment (T4) -->
          <div class="grid-2-cols" style="margin-top: 10px;">
            <div class="form-group">
              <label class="form-label">{{ t.importModal.scalePercent }}</label>
              <select
                v-model.number="scalePercent"
                class="form-select"
                :disabled="importing || successCount !== null"
              >
                <option :value="100">{{ t.importModal.scale100 }}</option>
                <option :value="75">{{ t.importModal.scale75 }}</option>
                <option :value="50">{{ t.importModal.scale50 }}</option>
                <option :value="25">{{ t.importModal.scale25 }}</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label">{{ t.importModal.alignMultiple }}</label>
              <select
                v-model.number="alignMultiple"
                class="form-select"
                :disabled="importing || successCount !== null"
              >
                <option :value="0">{{ t.importModal.alignNone }}</option>
                <option :value="8">{{ t.importModal.align8 }}</option>
                <option :value="16">{{ t.importModal.align16 }}</option>
              </select>
            </div>
          </div>
        </div>

        <!-- 4. Target File Size Limit (T4) -->
        <div class="section-card">
          <h4 class="section-title">🎯 {{ t.importModal.targetSize }}</h4>
          <div class="form-group">
            <input
              v-model.number="targetSizeKb"
              type="number"
              min="50"
              max="50000"
              step="50"
              class="form-input"
              :placeholder="t.importModal.targetSizePlaceholder"
              :disabled="importing || successCount !== null"
            />
            <p class="form-hint" style="margin-top: 6px; font-size: 0.8rem; color: var(--color-text-secondary);">
              ℹ️ {{ t.importModal.targetSizeHint }}
            </p>
          </div>
        </div>

        <!-- 4. Policies (Metadata & Collision) -->
        <div class="section-card">
          <h4 class="section-title">🛡️ {{ t.importModal.metadataPolicy }} & {{ t.importModal.collisionPolicy }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.importModal.metadataPolicy }}</label>
              <select v-model="metadataPolicy" class="form-select" :disabled="importing || successCount !== null">
                <option value="keep_supported">{{ t.importModal.keepSupported }}</option>
                <option value="strip_ai">{{ t.importModal.stripAi }}</option>
                <option value="strip_all">{{ t.importModal.stripAll }}</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label">{{ t.importModal.collisionPolicy }}</label>
              <select v-model="collisionPolicy" class="form-select" :disabled="importing || successCount !== null">
                <option value="rename">{{ t.importModal.collisionRename }}</option>
                <option value="skip">{{ t.importModal.collisionSkip }}</option>
              </select>
            </div>
          </div>

          <!-- Source Safety Guarantee Note -->
          <div class="safety-notice">
            <span>🔒 {{ t.importModal.sourceSafetyNotice }}</span>
          </div>
        </div>

        <!-- Progress Spinner / State -->
        <div v-if="importing" class="progress-section">
          <div class="progress-info">
            <span class="progress-status">⏳ {{ t.importModal.importing }}</span>
          </div>
          <div class="progress-track indeterminate">
            <div class="progress-fill-indeterminate"></div>
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
            :disabled="importing"
            @click="emit('close')"
          >
            {{ successCount !== null ? t.exportModal.close : t.exportModal.cancel }}
          </button>

          <button
            v-if="successCount === null"
            type="button"
            class="btn-primary"
            :disabled="!targetFolderId || importing || filePaths.length === 0"
            @click="handleStartImport"
          >
            {{ importing ? '⏳ ...' : '🚀 ' + t.importModal.startImport }}
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
  max-width: 620px;
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
}

.modal-title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.modal-subtitle {
  margin: 0.3rem 0 0 0;
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.close-btn {
  background: none;
  border: none;
  color: var(--color-text-secondary);
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
}
.close-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.alert-box {
  margin: 1rem 1.5rem 0 1.5rem;
  padding: 0.75rem 1rem;
  border-radius: 6px;
  font-size: 0.85rem;
}
.alert-box.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.35);
  color: #fca5a5;
}

.summary-box {
  margin: 1rem 1.5rem 0 1.5rem;
  padding: 0.85rem 1rem;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}
.summary-box.success {
  background: rgba(52, 211, 153, 0.12);
  border: 1px solid rgba(52, 211, 153, 0.35);
}

.summary-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 0.88rem;
  font-weight: 500;
  color: #6ee7b7;
}

.modal-body {
  padding: 1.25rem 1.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1.15rem;
}

.section-card {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 8px;
  padding: 1rem 1.15rem;
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.section-title {
  margin: 0;
  font-size: 0.92rem;
  font-weight: 600;
  color: #f1f5f9;
}

.grid-2-cols {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.label-with-value {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.form-label {
  font-size: 0.82rem;
  font-weight: 500;
  color: #cbd5e1;
}

.value-badge {
  font-size: 0.78rem;
  color: #818cf8;
  font-weight: 600;
}

.form-select,
.form-input {
  background: #121317;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 0.5rem 0.7rem;
  color: #f8fafc;
  font-size: 0.85rem;
  outline: none;
  transition: border-color 0.2s ease;
}
.form-select:focus,
.form-input:focus {
  border-color: #6366f1;
}

.form-range {
  accent-color: #6366f1;
  height: 6px;
  cursor: pointer;
}

.lossless-badge {
  display: flex;
  align-items: center;
  height: 36px;
  color: #818cf8;
  font-size: 0.82rem;
  font-weight: 500;
}

.safety-notice {
  background: rgba(99, 102, 241, 0.06);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 6px;
  padding: 0.6rem 0.8rem;
  font-size: 0.76rem;
  color: #c7d2fe;
  line-height: 1.4;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  background: rgba(99, 102, 241, 0.06);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 6px;
  padding: 0.75rem 0.9rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  font-weight: 500;
  color: #c7d2fe;
}

.progress-track.indeterminate {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
  position: relative;
}

.progress-fill-indeterminate {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  background: #6366f1;
  width: 40%;
  animation: indeterminate 1.5s infinite ease-in-out;
}

@keyframes indeterminate {
  0% {
    left: -40%;
  }
  100% {
    left: 100%;
  }
}

.modal-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-right {
  display: flex;
  gap: 0.75rem;
}

.btn-secondary {
  background: var(--color-bg-hover);
  border: 1px solid var(--border-color);
  color: var(--color-text-secondary);
  border-radius: 6px;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  cursor: pointer;
}
.btn-secondary:hover:not(:disabled) {
  background: var(--color-bg-active);
  color: var(--color-text-primary);
}

.btn-primary {
  background: #4f46e5;
  border: 1px solid #6366f1;
  color: #ffffff;
  border-radius: 6px;
  padding: 0.5rem 1.15rem;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease;
}
.btn-primary:hover:not(:disabled) {
  background: #4338ca;
}
.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
