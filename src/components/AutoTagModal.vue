<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type {
  ImageFile,
  TaggerConfig,
  TaggerModelSummary,
  TagPrediction,
  BatchTagProgress,
  TaggerDownloadProgress,
} from "../types";
import { BatchAutoTagController } from "../utils/batch-tagger";

const props = defineProps<{
  show: boolean;
  selectedFile: ImageFile | null;
  selectedFileCount: number;
  selectedFileIds: number[];
  allowOverridePrompt?: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "tags-applied"): void;
}>();

const models = ref<TaggerModelSummary[]>([]);
const selectedModelPath = ref<string>("");
const loadedModel = ref<{ name: string; model_path: string } | null>(null);

const generalThreshold = ref<number>(35);
const characterThreshold = ref<number>(85);
const includeRating = ref<boolean>(false);
const maxTags = ref<number>(50);
const writeToPrompt = ref(false);
const appendPrompt = ref(false);

const hasExistingPrompt = computed(() => {
  const p = props.selectedFile?.metadata?.prompt;
  return Boolean(p && p.trim().length > 0);
});

const isPromptWriteLocked = computed(() => {
  return hasExistingPrompt.value && !props.allowOverridePrompt;
});

const isDetecting = ref(false);
const isApplyingCurrent = ref(false);
const isApplyingBatch = ref(false);
const isApplying = computed(() => isApplyingCurrent.value || isApplyingBatch.value);
const batchController = new BatchAutoTagController();
const batchProgress = ref<BatchTagProgress | null>(null);
const isDownloading = ref(false);
const downloadProgress = ref<TaggerDownloadProgress | null>(null);
const downloadSource = ref<"auto" | "modelscope" | "hf-mirror" | "huggingface">("auto");
let unlistenDownload: UnlistenFn | null = null;
const predictions = ref<TagPrediction[]>([]);
const message = ref<{ type: "success" | "error" | "info" | "warning"; text: string } | null>(null);

const taggerConfig = computed<TaggerConfig>(() => ({
  general_threshold: generalThreshold.value / 100,
  character_threshold: characterThreshold.value / 100,
  include_rating: includeRating.value,
  max_tags: maxTags.value,
  write_to_prompt: writeToPrompt.value && !isPromptWriteLocked.value,
  append_prompt: appendPrompt.value,
  allow_override_existing_prompt: Boolean(props.allowOverridePrompt),
}));

async function loadModelList() {
  try {
    const list = await invoke<TaggerModelSummary[]>("list_tagger_models");
    models.value = list;
    const currentlyLoaded = await invoke<{ name: string; model_path: string } | null>(
      "get_loaded_tagger_model",
    );
    loadedModel.value = currentlyLoaded;

    if (currentlyLoaded) {
      selectedModelPath.value = currentlyLoaded.model_path;
    } else if (list.length > 0) {
      selectedModelPath.value = list[0].model_path;
      // Auto-load first model if none active
      await onSelectModel(list[0]);
    }
  } catch (err: any) {
    console.error("Failed to list tagger models:", err);
  }
}

async function onSelectModel(modelSummary: TaggerModelSummary) {
  try {
    message.value = null;
    isDetecting.value = true;
    const info = await invoke<{ name: string; model_path: string }>("load_tagger_model", {
      modelPath: modelSummary.model_path,
      tagsPath: modelSummary.tags_path,
    });
    loadedModel.value = info;
    selectedModelPath.value = info.model_path;
    // Refresh models list to update is_loaded flags
    models.value = await invoke<TaggerModelSummary[]>("list_tagger_models");
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isDetecting.value = false;
  }
}

async function browseModelFolder() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: t.value.autoTagModal.browseFolder,
    });

    if (typeof selected === "string") {
      // Find model.onnx and selected_tags.csv inside selected directory
      const modelPath = `${selected}/model.onnx`.replace(/\\/g, "/");
      const tagsPath = `${selected}/selected_tags.csv`.replace(/\\/g, "/");

      message.value = null;
      isDetecting.value = true;
      const info = await invoke<{ name: string; model_path: string }>("load_tagger_model", {
        modelPath,
        tagsPath,
      });
      loadedModel.value = info;
      selectedModelPath.value = info.model_path;
      models.value = await invoke<TaggerModelSummary[]>("list_tagger_models");
      message.value = { type: "success", text: `${t.value.autoTagModal.modelLoaded}: ${info.name}` };
    }
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isDetecting.value = false;
  }
}

async function detectTags() {
  if (!props.selectedFile?.id) return;
  if (!loadedModel.value) {
    message.value = { type: "error", text: t.value.autoTagModal.noModels };
    return;
  }

  isDetecting.value = true;
  message.value = null;
  try {
    const results = await invoke<TagPrediction[]>("auto_tag_file", {
      fileId: props.selectedFile.id,
      config: taggerConfig.value,
      applyTags: false,
    });
    predictions.value = results;
    if (results.length === 0) {
      message.value = { type: "success", text: t.value.autoTagModal.noTagsDetected };
    }
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isDetecting.value = false;
  }
}

async function applyToCurrent() {
  if (!props.selectedFile?.id) return;
  isApplyingCurrent.value = true;
  message.value = null;
  try {
    const results = await invoke<TagPrediction[]>("auto_tag_file", {
      fileId: props.selectedFile.id,
      config: taggerConfig.value,
      applyTags: true,
    });
    predictions.value = results;
    message.value = {
      type: "success",
      text: `${t.value.autoTagModal.success} (${results.length} tags)`,
    };
    emit("tags-applied");
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isApplyingCurrent.value = false;
  }
}

async function applyToBatch() {
  if (props.selectedFileIds.length === 0) return;
  isApplyingBatch.value = true;
  batchProgress.value = null;
  message.value = null;
  try {
    const result = await batchController.start({
      fileIds: props.selectedFileIds,
      config: taggerConfig.value,
      onProgress: (p) => {
        batchProgress.value = p;
      },
    });

    if (batchController.status === "completed") {
      const failed = batchController.progress?.failed_files ?? 0;
      if (failed > 0) {
        message.value = {
          type: "warning",
          text: `${t.value.autoTagModal.success} (${result.processed_files} ok, ${failed} failed, ${result.tags_added} tags)`,
        };
      } else {
        message.value = {
          type: "success",
          text: `${t.value.autoTagModal.success} (${result.processed_files} files, ${result.tags_added} tags)`,
        };
      }
      emit("tags-applied");
    } else if (batchController.status === "canceled") {
      message.value = {
        type: "info",
        text: t.value.autoTagModal.batchCanceled,
      };
      if (result.processed_files > 0) {
        emit("tags-applied");
      }
    }
  } catch (err: any) {
    if (batchController.status === "canceled") {
      message.value = {
        type: "info",
        text: t.value.autoTagModal.batchCanceled,
      };
    } else {
      message.value = { type: "error", text: String(err) };
    }
  } finally {
    isApplyingBatch.value = false;
  }
}

async function cancelBatchAutoTag() {
  if (isApplyingBatch.value) {
    await batchController.cancel();
  }
}

function formatBytes(bytes: number): string {
  if (bytes <= 0 || !Number.isFinite(bytes)) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

const displayPercent = computed(() => {
  if (!downloadProgress.value) return 0;
  const p = downloadProgress.value;
  if (p.phase === "complete") return 100;
  return Math.min(99, Math.floor(p.percent));
});

function getDownloadStatusText(): string {
  if (!downloadProgress.value) return t.value.autoTagModal.downloadShort;
  const p = downloadProgress.value;
  if (p.phase === "downloading_tags") {
    return t.value.autoTagModal.downloadingTags || `Downloading selected_tags.csv (1/2)`;
  }
  if (p.phase === "downloading_model") {
    return t.value.autoTagModal.downloadingModel || `Downloading model.onnx (2/2)`;
  }
  if (p.phase === "complete") {
    return t.value.autoTagModal.downloadFinishing || `Download complete, loading model...`;
  }
  return p.current_file || t.value.autoTagModal.downloadShort;
}

async function startDownloadModel() {
  if (isDownloading.value) return;
  isDownloading.value = true;
  downloadProgress.value = null;
  message.value = null;

  try {
    const summary = await invoke<TaggerModelSummary>("download_tagger_model", {
      options: {
        model_id: "wd-v1-4-convnext-tagger-v2",
        mirror: downloadSource.value,
      },
      source: downloadSource.value,
    });
    message.value = {
      type: "success",
      text: t.value.autoTagModal.downloadSuccess,
    };
    await loadModelList();
    if (summary) {
      await onSelectModel(summary);
    }
  } catch (err: any) {
    const errStr = String(err);
    if (errStr.includes("cancelled") || errStr.includes("canceled")) {
      message.value = {
        type: "error",
        text: t.value.autoTagModal.downloadCanceled,
      };
    } else {
      message.value = {
        type: "error",
        text: `${t.value.autoTagModal.downloadFailed}: ${errStr}`,
      };
    }
  } finally {
    isDownloading.value = false;
    downloadProgress.value = null;
  }
}

async function cancelDownload() {
  try {
    await invoke("cancel_tagger_download");
    isDownloading.value = false;
    downloadProgress.value = null;
    message.value = {
      type: "error",
      text: t.value.autoTagModal.downloadCanceled,
    };
  } catch (err: any) {
    console.error("Failed to cancel tagger download:", err);
  }
}

function getFileName(path: string) {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
}

function getTagCategoryClass(cat: string | number) {
  if (cat === "Character" || cat === 4) return "tag-badge-character";
  if (cat === "Rating" || cat === 9) return "tag-badge-rating";
  return "tag-badge-general";
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.show) {
    emit("close");
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      message.value = null;
      predictions.value = [];
      void loadModelList();
    }
  },
  { immediate: true },
);

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  try {
    unlistenDownload = await listen<TaggerDownloadProgress>(
      "tagger-download-progress",
      (event) => {
        downloadProgress.value = event.payload;
      },
    );
  } catch (err) {
    console.error("Failed to listen to tagger-download-progress:", err);
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  if (unlistenDownload) {
    unlistenDownload();
    unlistenDownload = null;
  }
});
</script>

<template>
  <div v-if="show" class="modal-backdrop" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="modal-dialog">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-left">
          <span class="header-icon">🏷️</span>
          <h2 class="modal-title">{{ t.autoTagModal.title }}</h2>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        <!-- Model Selector & Status -->
        <!-- Downloading Progress Card -->
        <div v-if="isDownloading" class="section-box download-progress-box">
          <div class="download-header-row">
            <div class="download-title-group">
              <span class="download-icon">⚡</span>
              <span class="download-filename">
                {{ getDownloadStatusText() }}
              </span>
            </div>
            <span class="download-pct">
              {{ displayPercent }}%
            </span>
          </div>

          <!-- Progress track -->
          <div class="progress-track">
            <div
              class="progress-fill"
              :style="{ width: `${Math.max(2, Math.min(100, displayPercent))}%` }"
            ></div>
          </div>

          <!-- Progress metrics and cancel button -->
          <div class="download-footer-row">
            <div class="download-metrics">
              <span v-if="downloadProgress && downloadProgress.total_bytes > 0">
                {{ formatBytes(downloadProgress.downloaded_bytes) }} /
                {{ formatBytes(Math.max(downloadProgress.downloaded_bytes, downloadProgress.total_bytes)) }}
              </span>
              <span v-if="downloadProgress && downloadProgress.speed_bytes_per_sec > 0" class="download-speed">
                ({{ formatBytes(downloadProgress.speed_bytes_per_sec) }}/s)
              </span>
            </div>
            <button
              type="button"
              class="cancel-download-btn"
              @click="cancelDownload"
            >
              {{ t.autoTagModal.cancelDownload }}
            </button>
          </div>
        </div>

        <!-- Empty Model Guide Card (When no model installed and not downloading) -->
        <div v-else-if="models.length === 0" class="section-box empty-model-box">
          <div class="empty-model-icon">🤖</div>
          <h3 class="empty-model-title">{{ t.autoTagModal.noModelTitle }}</h3>
          <p class="empty-model-desc">{{ t.autoTagModal.noModelDesc }}</p>

          <div class="empty-model-actions">
            <!-- Download Node Selector -->
            <div class="source-select-group">
              <label class="source-label">{{ t.autoTagModal.downloadSource }}:</label>
              <select v-model="downloadSource" class="source-select">
                <option value="auto">{{ t.autoTagModal.sourceAuto }}</option>
                <option value="modelscope">{{ t.autoTagModal.sourceModelScope }}</option>
                <option value="hf-mirror">{{ t.autoTagModal.sourceMirror }}</option>
                <option value="huggingface">{{ t.autoTagModal.sourceOfficial }}</option>
              </select>
            </div>

            <div class="empty-buttons-row">
              <button
                type="button"
                class="btn-primary auto-download-btn"
                @click="startDownloadModel"
              >
                ⚡ {{ t.autoTagModal.downloadModel }}
              </button>
              <button
                type="button"
                class="browse-btn"
                @click="browseModelFolder"
              >
                📁 {{ t.autoTagModal.browseFolder }}
              </button>
            </div>
          </div>
        </div>

        <!-- Normal Model Selector & Status (When models exist and not downloading) -->
        <div v-else class="section-box">
          <div class="section-title-row">
            <span class="section-title">{{ t.autoTagModal.modelLabel }}</span>
            <span v-if="loadedModel" class="model-status-pill online">
              ● {{ t.autoTagModal.modelLoaded }}: {{ loadedModel.name }}
            </span>
            <span v-else class="model-status-pill offline">
              ○ Not Loaded
            </span>
          </div>

          <div class="model-select-row">
            <select
              v-model="selectedModelPath"
              class="model-dropdown"
              @change="() => {
                const target = models.find(m => m.model_path === selectedModelPath);
                if (target) onSelectModel(target);
              }"
            >
              <option v-for="m in models" :key="m.model_path" :value="m.model_path">
                {{ m.name }} {{ m.is_loaded ? `(★ ${t.autoTagModal.modelLoaded})` : '' }}
              </option>
            </select>
            <button
              type="button"
              class="browse-btn"
              :title="t.autoTagModal.browseFolder"
              @click="browseModelFolder"
            >
              📁 {{ t.autoTagModal.browseFolder }}
            </button>
            <button
              type="button"
              class="download-action-btn"
              :title="t.autoTagModal.downloadModel"
              @click="startDownloadModel"
            >
              ⚡ {{ t.autoTagModal.downloadShort }}
            </button>
          </div>
        </div>

        <!-- Threshold & Configuration Sliders -->
        <div class="section-box">
          <div class="controls-grid">
            <!-- General Tag Threshold -->
            <div class="control-item">
              <div class="control-header">
                <label for="general-slider">{{ t.autoTagModal.generalThreshold }}</label>
                <span class="val-badge">≥ {{ generalThreshold }}%</span>
              </div>
              <input
                id="general-slider"
                type="range"
                min="10"
                max="90"
                step="5"
                v-model.number="generalThreshold"
                class="range-slider"
              />
            </div>

            <!-- Character Tag Threshold -->
            <div class="control-item">
              <div class="control-header">
                <label for="character-slider">{{ t.autoTagModal.characterThreshold }}</label>
                <span class="val-badge green">≥ {{ characterThreshold }}%</span>
              </div>
              <input
                id="character-slider"
                type="range"
                min="50"
                max="95"
                step="5"
                v-model.number="characterThreshold"
                class="range-slider"
              />
            </div>

            <!-- Content Rating Checkbox -->
            <div class="control-item-row">
              <label class="checkbox-label">
                <input type="checkbox" v-model="includeRating" />
                <span>{{ t.autoTagModal.includeRating }}</span>
              </label>

              <!-- Max Tags -->
              <div class="max-tags-group">
                <label>{{ t.autoTagModal.maxTags }}:</label>
                <select v-model.number="maxTags" class="small-select">
                  <option :value="30">30</option>
                  <option :value="50">50</option>
                  <option :value="80">80</option>
                  <option :value="150">150</option>
                </select>
              </div>
            </div>

            <!-- Prompt Metadata Options -->
            <div class="control-item-row prompt-opts-row">
              <label class="checkbox-label" :class="{ disabled: isPromptWriteLocked }">
                <input
                  type="checkbox"
                  v-model="writeToPrompt"
                  :disabled="isPromptWriteLocked"
                />
                <span>{{ t.autoTagModal.writeToPrompt }}</span>
              </label>
              <label v-if="writeToPrompt && !isPromptWriteLocked" class="checkbox-label append-label">
                <input type="checkbox" v-model="appendPrompt" />
                <span>{{ t.autoTagModal.appendPrompt }}</span>
              </label>
              <span v-if="isPromptWriteLocked" class="locked-hint" :title="t.settings.promptProtected">
                🔒 {{ t.settings.promptProtected }}
              </span>
            </div>
          </div>
        </div>

        <!-- Batch Tagging Progress Bar Card -->
        <div v-if="isApplyingBatch && batchProgress" class="section-box batch-progress-box">
          <div class="progress-header">
            <div class="progress-title-row">
              <span class="progress-phase">{{ t.autoTagModal.batchProgressTitle }}</span>
              <span class="progress-ratio">
                {{
                  t.autoTagModal.batchProgressRatio
                    .replace('{current}', String(batchProgress.current))
                    .replace('{total}', String(batchProgress.total))
                    .replace('{percent}', String(Math.floor(batchProgress.percent)))
                }}
              </span>
            </div>
            <button
              type="button"
              class="cancel-batch-btn"
              @click="cancelBatchAutoTag"
            >
              {{ t.autoTagModal.cancelBatch }}
            </button>
          </div>
          <div class="progress-bar-track">
            <div
              class="progress-bar-fill"
              :style="{ width: `${Math.min(100, Math.max(0, batchProgress.percent))}%` }"
            ></div>
          </div>
          <div class="progress-info-row">
            <span class="file-name-hint">{{ batchProgress.current_file }}</span>
            <span v-if="batchProgress.failed_files > 0" class="failed-badge">
              {{ batchProgress.failed_files }} failed
            </span>
          </div>
        </div>

        <!-- Feedback Messages -->
        <div v-if="message" :class="['message-banner', message.type]">
          <span>{{ message.text }}</span>
        </div>

        <!-- Preview of Detected Tags -->
        <div v-if="selectedFile" class="section-box preview-box">
          <div class="preview-header">
            <div class="preview-file-info">
              <span class="file-name">{{ getFileName(selectedFile.path) }}</span>
              <span v-if="predictions.length > 0" class="count-badge">
                {{ predictions.length }} {{ t.tagsModal.existingTags }}
              </span>
            </div>
            <button
              type="button"
              class="detect-btn"
              :disabled="isDetecting || !loadedModel || isDownloading"
              @click="detectTags"
            >
              {{ isDetecting ? t.autoTagModal.tagging : `🔍 ${t.autoTagModal.detectTags}` }}
            </button>
          </div>

          <div v-if="predictions.length > 0" class="tags-container">
            <div
              v-for="tag in predictions"
              :key="tag.name"
              :class="['tag-pill', getTagCategoryClass(tag.category)]"
            >
              <span class="tag-name">{{ tag.name }}</span>
              <span class="tag-score">{{ Math.round(tag.confidence * 100) }}%</span>
            </div>
          </div>
          <div v-else-if="!isDetecting" class="no-tags-prompt">
            {{ t.autoTagModal.noTagsDetected }}
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="modal-footer">
        <button type="button" class="btn-cancel" @click="emit('close')">
          {{ t.preview.close }}
        </button>

        <div class="footer-actions-right">
          <!-- Batch Apply Button -->
          <button
            v-if="selectedFileCount > 1"
            type="button"
            class="btn-primary batch-btn"
            :disabled="isApplying || isDetecting || !loadedModel || isDownloading"
            @click="applyToBatch"
          >
            {{
              isApplyingBatch
                ? t.autoTagModal.tagging
                : t.autoTagModal.applyToBatch.replace('{count}', String(selectedFileCount))
            }}
          </button>

          <!-- Current File Apply Button -->
          <button
            v-if="selectedFile"
            type="button"
            class="btn-primary"
            :disabled="isApplying || isDetecting || !loadedModel || isDownloading"
            @click="applyToCurrent"
          >
            {{ isApplyingCurrent ? t.autoTagModal.tagging : t.autoTagModal.applyToCurrent }}
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
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1050;
  animation: fadeIn 0.15s ease;
}

.modal-dialog {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  width: 90%;
  max-width: 620px;
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-icon {
  font-size: 1.25rem;
}

.modal-title {
  font-size: 1.05rem;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0;
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--color-text-secondary);
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.15s;
}

.close-btn:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-hover);
}

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-box {
  background: var(--color-bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px 14px;
}

.section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.section-title {
  font-size: 0.8rem;
  font-weight: 700;
  color: var(--color-text-primary);
  opacity: 0.92;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.model-status-pill {
  font-size: 0.72rem;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
}

.model-status-pill.online {
  background: rgba(16, 185, 129, 0.12);
  color: #047857;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

:root[data-theme="dark"] .model-status-pill.online {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
}

.model-status-pill.offline {
  background: rgba(239, 68, 68, 0.12);
  color: #b91c1c;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

:root[data-theme="dark"] .model-status-pill.offline {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

.model-select-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.model-dropdown {
  flex: 1;
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 0.82rem;
  outline: none;
  transition: border-color 0.15s ease;
}

.model-dropdown:focus {
  border-color: #6366f1;
}

.browse-btn {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.browse-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--border-color-strong, var(--border-color));
}

.download-action-btn {
  background: rgba(99, 102, 241, 0.12);
  border: 1px solid rgba(99, 102, 241, 0.3);
  color: #4338ca;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}

:root[data-theme="dark"] .download-action-btn {
  background: rgba(99, 102, 241, 0.2);
  border-color: rgba(99, 102, 241, 0.4);
  color: #c7d2fe;
}

.download-action-btn:hover {
  background: #6366f1;
  color: #ffffff;
  border-color: #6366f1;
}

/* Empty Model Box */
.empty-model-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 20px 18px;
  background: var(--color-bg-secondary);
  border: 1px dashed var(--border-color);
  border-radius: 8px;
}

.empty-model-icon {
  font-size: 2rem;
  margin-bottom: 6px;
}

.empty-model-title {
  font-size: 0.92rem;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0 0 6px 0;
}

.empty-model-desc {
  font-size: 0.82rem;
  color: var(--color-text-primary);
  opacity: 0.88;
  line-height: 1.5;
  max-width: 480px;
  margin: 0 0 14px 0;
}

.empty-model-actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.source-select-group {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.78rem;
}

.source-label {
  color: var(--color-text-primary);
  opacity: 0.92;
  font-weight: 600;
}

.source-select {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 0.78rem;
  outline: none;
}

.empty-buttons-row {
  display: flex;
  gap: 10px;
  justify-content: center;
  flex-wrap: wrap;
}

.auto-download-btn {
  background: #6366f1;
  color: #fff;
  border: none;
  font-weight: 600;
  padding: 7px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  gap: 6px;
}

.auto-download-btn:hover {
  background: #4f46e5;
  transform: translateY(-1px);
}

/* Download Progress Box */
.download-progress-box {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px 16px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.download-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.download-title-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.download-icon {
  font-size: 1rem;
}

.download-filename {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.download-pct {
  font-size: 0.85rem;
  font-weight: 700;
  color: #6366f1;
}

.progress-track {
  width: 100%;
  height: 6px;
  background: var(--border-color);
  border-radius: 999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1, #8b5cf6);
  border-radius: 999px;
  transition: width 0.2s ease;
}

.download-footer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.download-metrics {
  font-size: 0.78rem;
  color: var(--color-text-primary);
  opacity: 0.88;
  display: flex;
  gap: 6px;
}

.download-speed {
  color: #059669;
  font-weight: 600;
}

:root[data-theme="dark"] .download-speed {
  color: #34d399;
}

.cancel-download-btn {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  opacity: 0.9;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.cancel-download-btn:hover {
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.4);
  color: #ef4444;
  opacity: 1;
}

.batch-progress-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: rgba(99, 102, 241, 0.04);
  border-color: rgba(99, 102, 241, 0.25);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-phase {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.progress-ratio {
  font-size: 0.78rem;
  font-weight: 600;
  color: #6366f1;
}

.cancel-batch-btn {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  opacity: 0.9;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.cancel-batch-btn:hover {
  background: rgba(239, 68, 68, 0.12);
  border-color: rgba(239, 68, 68, 0.4);
  color: #ef4444;
  opacity: 1;
}

.progress-bar-track {
  width: 100%;
  height: 6px;
  background: var(--border-color);
  border-radius: 999px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #6366f1, #8b5cf6);
  border-radius: 999px;
  transition: width 0.2s ease;
}

.progress-info-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.file-name-hint {
  max-width: 80%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.failed-badge {
  color: #ef4444;
  font-weight: 600;
}

.controls-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.control-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.control-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.val-badge {
  font-size: 0.72rem;
  font-weight: 700;
  color: #1d4ed8;
  background: rgba(37, 99, 235, 0.12);
  padding: 2px 7px;
  border-radius: 4px;
}

:root[data-theme="dark"] .val-badge {
  color: #93c5fd;
  background: rgba(96, 165, 250, 0.18);
}

.val-badge.green {
  color: #047857;
  background: rgba(5, 150, 105, 0.12);
}

:root[data-theme="dark"] .val-badge.green {
  color: #6ee7b7;
  background: rgba(52, 211, 153, 0.18);
}

.range-slider {
  width: 100%;
  accent-color: #6366f1;
  height: 4px;
  cursor: pointer;
}

.control-item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 4px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--color-text-primary);
  cursor: pointer;
}

.max-tags-group {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.small-select {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 0.78rem;
  outline: none;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.preview-file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  overflow: hidden;
}

.file-name {
  font-size: 0.82rem;
  font-weight: 700;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 250px;
}

.count-badge {
  font-size: 0.72rem;
  font-weight: 600;
  color: #4338ca;
  background: rgba(99, 102, 241, 0.12);
  padding: 2px 7px;
  border-radius: 4px;
  white-space: nowrap;
}

:root[data-theme="dark"] .count-badge {
  color: #c7d2fe;
  background: rgba(99, 102, 241, 0.25);
}

.detect-btn {
  background: rgba(99, 102, 241, 0.12);
  border: 1px solid rgba(99, 102, 241, 0.35);
  color: #4338ca;
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.detect-btn:hover:not(:disabled) {
  background: #6366f1;
  color: #ffffff;
  border-color: #6366f1;
}

:root[data-theme="dark"] .detect-btn {
  background: rgba(99, 102, 241, 0.2);
  border-color: rgba(99, 102, 241, 0.4);
  color: #c7d2fe;
}

:root[data-theme="dark"] .detect-btn:hover:not(:disabled) {
  background: #6366f1;
  color: #ffffff;
}

.detect-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-height: 160px;
  overflow-y: auto;
  padding-right: 4px;
}

.tag-pill {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 0.72rem;
  border: 1px solid transparent;
}

.tag-badge-general {
  background: #eef2ff;
  border-color: #c7d2fe;
  color: #3730a3;
}

:root[data-theme="dark"] .tag-badge-general {
  background: rgba(99, 102, 241, 0.18);
  border-color: rgba(99, 102, 241, 0.35);
  color: #e0e7ff;
}

.tag-badge-character {
  background: #ecfdf5;
  border-color: #a7f3d0;
  color: #065f46;
}

:root[data-theme="dark"] .tag-badge-character {
  background: rgba(16, 185, 129, 0.18);
  border-color: rgba(16, 185, 129, 0.35);
  color: #a7f3d0;
}

.tag-badge-rating {
  background: #faf5ff;
  border-color: #e9d5ff;
  color: #6b21a8;
}

:root[data-theme="dark"] .tag-badge-rating {
  background: rgba(168, 85, 247, 0.18);
  border-color: rgba(168, 85, 247, 0.35);
  color: #f3e8ff;
}

.tag-score {
  font-weight: 700;
  opacity: 0.8;
}

.no-tags-prompt {
  font-size: 0.82rem;
  color: var(--color-text-primary);
  opacity: 0.85;
  text-align: center;
  padding: 14px 0;
}

.model-dropdown option,
.source-select option,
.small-select option {
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

.message-banner {
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 0.78rem;
  font-weight: 600;
}

.message-banner.success {
  background: rgba(16, 185, 129, 0.12);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #047857;
}

:root[data-theme="dark"] .message-banner.success {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
}

.message-banner.error {
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #b91c1c;
}

:root[data-theme="dark"] .message-banner.error {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
}

.footer-actions-right {
  display: flex;
  gap: 8px;
}

.btn-cancel {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--color-text-secondary);
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-cancel:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-hover);
}

.btn-primary {
  background: #6366f1;
  border: none;
  color: #fff;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary:hover:not(:disabled) {
  background: #4f46e5;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.batch-btn {
  background: #8b5cf6;
}

.batch-btn:hover:not(:disabled) {
  background: #7c3aed;
}
</style>
