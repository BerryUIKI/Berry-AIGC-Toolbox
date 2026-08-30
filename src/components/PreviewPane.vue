<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ImageFile } from "../types";
import {
  assetUrl,
  formatBytes,
  formatDateTime,
  getFileName,
  normalizePath,
} from "../utils/image";

const props = defineProps<{
  file: ImageFile;
  files: ImageFile[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "navigate", file: ImageFile): void;
  (e: "rate", fileId: number, rating: number | null): void;
}>();

const showInspector = ref(true);
const promptCopied = ref(false);
const negativePromptCopied = ref(false);
const showRaw = ref(false);
const ratingSaving = ref(false);

const currentIndex = computed(() =>
  props.files.findIndex((f) => f.path === props.file.path),
);

const hasPrev = computed(() => currentIndex.value > 0);
const hasNext = computed(() => currentIndex.value < props.files.length - 1);

function prev() {
  if (hasPrev.value) {
    emit("navigate", props.files[currentIndex.value - 1]);
  }
}

function next() {
  if (hasNext.value) {
    emit("navigate", props.files[currentIndex.value + 1]);
  }
}

function toggleInspector() {
  showInspector.value = !showInspector.value;
}

async function copyPrompt(text: string | null | undefined, type: "prompt" | "negative") {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    if (type === "prompt") {
      promptCopied.value = true;
      setTimeout(() => {
        promptCopied.value = false;
      }, 1500);
    } else {
      negativePromptCopied.value = true;
      setTimeout(() => {
        negativePromptCopied.value = false;
      }, 1500);
    }
  } catch (err) {
    console.error("Failed to copy text:", err);
  }
}

async function setRating(r: number) {
  if (!props.file.id || ratingSaving.value) return;
  const newRating = props.file.rating === r ? null : r;
  ratingSaving.value = true;
  try {
    await invoke("set_file_rating", {
      fileId: props.file.id,
      rating: newRating,
    });
    emit("rate", props.file.id, newRating);
  } catch (err) {
    console.error("Failed to save rating:", err);
  } finally {
    ratingSaving.value = false;
  }
}

function handleKeyDown(e: KeyboardEvent) {
  // Ignore when typing inside an input/textarea
  const tag = (document.activeElement?.tagName ?? "").toLowerCase();
  if (tag === "input" || tag === "textarea") return;

  if (e.key === "Escape") {
    e.preventDefault();
    emit("close");
  } else if (e.key === "ArrowLeft") {
    e.preventDefault();
    prev();
  } else if (e.key === "ArrowRight") {
    e.preventDefault();
    next();
  } else if (e.key === "i" || e.key === "I") {
    e.preventDefault();
    toggleInspector();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});

// Reset copy state when navigating files
watch(
  () => props.file.path,
  () => {
    promptCopied.value = false;
    negativePromptCopied.value = false;
  },
);
</script>

<template>
  <div class="preview-backdrop" @click.self="emit('close')">
    <div class="preview-dialog">
      <!-- Top header bar -->
      <header class="dialog-header">
        <div class="header-left">
          <span class="file-name" :title="normalizePath(file.path)">
            {{ getFileName(file.path) }}
          </span>
          <span class="file-counter" v-if="files.length > 1">
            {{ currentIndex + 1 }} / {{ files.length }}
          </span>
        </div>

        <div class="header-center">
          <div class="rating-bar" title="Click a star to rate (1–10), click again to clear">
            <button
              v-for="star in 10"
              :key="star"
              type="button"
              class="star-btn"
              :class="{ active: (file.rating ?? 0) >= star }"
              :disabled="ratingSaving"
              @click="setRating(star)"
            >
              ★
            </button>
            <span v-if="file.rating" class="rating-text">{{ file.rating }}/10</span>
          </div>
        </div>

        <div class="header-right">
          <button
            type="button"
            class="header-btn"
            :class="{ active: showInspector }"
            title="Toggle Metadata Inspector (Hotkey: I)"
            @click="toggleInspector"
          >
            ℹ Inspector
          </button>
          <button
            type="button"
            class="header-btn close-btn"
            title="Close (Esc)"
            @click="emit('close')"
          >
            ✕
          </button>
        </div>
      </header>

      <!-- Main body: image viewport + collapsible inspector -->
      <div class="dialog-body">
        <div class="viewport-area">
          <button
            v-if="hasPrev"
            type="button"
            class="nav-btn prev-btn"
            title="Previous (←)"
            @click="prev"
          >
            ‹
          </button>

          <div class="image-container">
            <img
              v-if="file.container !== 'mp4' && file.container !== 'txt'"
              :src="assetUrl(file.path)"
              :alt="getFileName(file.path)"
              class="main-image"
            />
            <div v-else class="non-image-placeholder">
              <span class="placeholder-icon">📄</span>
              <span class="placeholder-text">{{ file.container.toUpperCase() }} File</span>
              <span class="placeholder-path">{{ normalizePath(file.path) }}</span>
            </div>
          </div>

          <button
            v-if="hasNext"
            type="button"
            class="nav-btn next-btn"
            title="Next (→)"
            @click="next"
          >
            ›
          </button>
        </div>

        <!-- Metadata Inspector Sidebar -->
        <aside v-if="showInspector" class="inspector-sidebar">
          <div class="inspector-header">
            <h3>Metadata Inspector</h3>
            <span v-if="file.metadata?.format" class="format-pill">
              {{ file.metadata.format }}
            </span>
          </div>

          <div class="inspector-content">
            <!-- Basic file specs -->
            <div class="spec-grid">
              <div class="spec-item">
                <span class="spec-label">Dimensions</span>
                <span class="spec-value">
                  {{
                    file.metadata?.width && file.metadata?.height
                      ? `${file.metadata.width} × ${file.metadata.height}`
                      : "—"
                  }}
                </span>
              </div>
              <div class="spec-item">
                <span class="spec-label">File Size</span>
                <span class="spec-value">{{ formatBytes(file.size_bytes) }}</span>
              </div>
              <div class="spec-item">
                <span class="spec-label">Format</span>
                <span class="spec-value">{{ file.container.toUpperCase() }}</span>
              </div>
              <div class="spec-item">
                <span class="spec-label">Modified</span>
                <span class="spec-value">{{ formatDateTime(file.modified_at) }}</span>
              </div>
            </div>

            <!-- Prompt section -->
            <div v-if="file.metadata?.prompt" class="meta-section">
              <div class="section-heading">
                <h4>Prompt</h4>
                <button
                  type="button"
                  class="copy-btn"
                  @click="copyPrompt(file.metadata?.prompt, 'prompt')"
                >
                  {{ promptCopied ? "✓ Copied!" : "Copy" }}
                </button>
              </div>
              <div class="text-block prompt-text">
                {{ file.metadata.prompt }}
              </div>
            </div>

            <!-- Negative Prompt section -->
            <div v-if="file.metadata?.negative_prompt" class="meta-section">
              <div class="section-heading">
                <h4>Negative Prompt</h4>
                <button
                  type="button"
                  class="copy-btn"
                  @click="copyPrompt(file.metadata?.negative_prompt, 'negative')"
                >
                  {{ negativePromptCopied ? "✓ Copied!" : "Copy" }}
                </button>
              </div>
              <div class="text-block negative-text">
                {{ file.metadata.negative_prompt }}
              </div>
            </div>

            <!-- Generation parameters -->
            <div v-if="file.metadata" class="meta-section">
              <div class="section-heading">
                <h4>Generation Parameters</h4>
              </div>
              <div class="params-table">
                <div v-if="file.metadata.model_name" class="param-row">
                  <span class="param-key">Model</span>
                  <span class="param-val" :title="file.metadata.model_name">
                    {{ file.metadata.model_name }}
                    <span v-if="file.metadata.model_hash" class="hash-tag">
                      ({{ file.metadata.model_hash }})
                    </span>
                  </span>
                </div>
                <div v-if="file.metadata.sampler" class="param-row">
                  <span class="param-key">Sampler</span>
                  <span class="param-val">{{ file.metadata.sampler }}</span>
                </div>
                <div v-if="file.metadata.steps" class="param-row">
                  <span class="param-key">Steps</span>
                  <span class="param-val">{{ file.metadata.steps }}</span>
                </div>
                <div v-if="file.metadata.cfg_scale" class="param-row">
                  <span class="param-key">CFG Scale</span>
                  <span class="param-val">{{ file.metadata.cfg_scale }}</span>
                </div>
                <div v-if="file.metadata.seed" class="param-row">
                  <span class="param-key">Seed</span>
                  <span class="param-val selectable">{{ file.metadata.seed }}</span>
                </div>
                <div v-if="file.aesthetic_score" class="param-row">
                  <span class="param-key">Aesthetic Score</span>
                  <span class="param-val">{{ file.aesthetic_score }}</span>
                </div>
              </div>
            </div>

            <!-- Raw parameter accordion -->
            <div v-if="file.metadata?.parameters || file.metadata?.raw" class="meta-section">
              <button
                type="button"
                class="raw-toggle-btn"
                @click="showRaw = !showRaw"
              >
                <span>{{ showRaw ? "▼ Hide Raw Metadata" : "▶ Show Raw Metadata" }}</span>
              </button>
              <pre v-if="showRaw" class="raw-pre">{{
                file.metadata.parameters || file.metadata.raw
              }}</pre>
            </div>

            <!-- Path info -->
            <div class="spec-item path-item">
              <span class="spec-label">File Path</span>
              <span class="spec-value path-val" :title="normalizePath(file.path)">
                {{ normalizePath(file.path) }}
              </span>
            </div>
          </div>
        </aside>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preview-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(6px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-dialog {
  width: 96vw;
  height: 94vh;
  background: #1e1e1e;
  color: #eee;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.dialog-header {
  height: 52px;
  padding: 0 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #181818;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
  max-width: 40%;
}

.file-name {
  font-size: 0.95em;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-counter {
  font-size: 0.8em;
  color: #888;
  flex-shrink: 0;
}

.header-center {
  display: flex;
  align-items: center;
}

.rating-bar {
  display: flex;
  align-items: center;
  gap: 2px;
}

.star-btn {
  background: transparent;
  border: none;
  font-size: 1.15rem;
  color: #555;
  cursor: pointer;
  padding: 0 1px;
  line-height: 1;
  transition: color 0.15s ease, transform 0.1s ease;
}

.star-btn:hover {
  transform: scale(1.15);
  color: #fbbf24;
}

.star-btn.active {
  color: #eab308;
}

.rating-text {
  font-size: 0.8em;
  font-weight: 600;
  color: #eab308;
  margin-left: 0.4rem;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #ccc;
  padding: 0.35rem 0.75rem;
  border-radius: 6px;
  font-size: 0.85em;
  cursor: pointer;
  transition: all 0.15s ease;
}

.header-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.header-btn.active {
  background: #2f6fed;
  border-color: #2f6fed;
  color: #fff;
}

.close-btn {
  font-size: 1.1em;
  padding: 0.25rem 0.65rem;
}

.close-btn:hover {
  background: #dc2626;
  border-color: #dc2626;
}

.dialog-body {
  flex: 1;
  display: flex;
  min-height: 0;
  position: relative;
}

.viewport-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  background: #121212;
}

.image-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem;
  box-sizing: border-box;
}

.main-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 4px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  user-select: none;
}

.non-image-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  color: #888;
}

.placeholder-icon {
  font-size: 3rem;
}

.placeholder-text {
  font-size: 1.1em;
  font-weight: 600;
}

.placeholder-path {
  font-size: 0.8em;
  font-family: monospace;
}

.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 44px;
  height: 64px;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  font-size: 2.2rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
  z-index: 10;
}

.nav-btn:hover {
  background: rgba(47, 111, 237, 0.85);
  border-color: #2f6fed;
}

.prev-btn {
  left: 1rem;
}

.next-btn {
  right: 1rem;
}

/* Inspector Sidebar */
.inspector-sidebar {
  width: 360px;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  background: #181818;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

.inspector-header {
  padding: 0.85rem 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.inspector-header h3 {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}

.format-pill {
  padding: 0.15rem 0.55rem;
  border-radius: 999px;
  font-size: 0.72em;
  font-weight: 600;
  background: rgba(47, 111, 237, 0.2);
  color: #60a5fa;
  border: 1px solid rgba(47, 111, 237, 0.4);
}

.inspector-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.spec-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  background: rgba(255, 255, 255, 0.03);
  padding: 0.75rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.spec-item {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.spec-label {
  font-size: 0.72em;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.spec-value {
  font-size: 0.85em;
  font-weight: 500;
  color: #ddd;
}

.meta-section {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-heading h4 {
  margin: 0;
  font-size: 0.8em;
  font-weight: 600;
  color: #aaa;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.copy-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #aaa;
  font-size: 0.72em;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.copy-btn:hover {
  background: #2f6fed;
  color: #fff;
  border-color: #2f6fed;
}

.text-block {
  background: rgba(0, 0, 0, 0.35);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0.65rem 0.75rem;
  font-size: 0.82em;
  line-height: 1.45;
  color: #ddd;
  max-height: 160px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.params-table {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  background: rgba(255, 255, 255, 0.03);
  padding: 0.75rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.param-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.8em;
  gap: 0.5rem;
}

.param-key {
  color: #888;
}

.param-val {
  font-weight: 500;
  color: #ddd;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.hash-tag {
  color: #888;
  font-size: 0.9em;
}

.selectable {
  user-select: all;
  font-family: monospace;
}

.raw-toggle-btn {
  background: transparent;
  border: none;
  color: #888;
  font-size: 0.78em;
  cursor: pointer;
  padding: 0;
  text-align: left;
  transition: color 0.15s ease;
}

.raw-toggle-btn:hover {
  color: #bbb;
}

.raw-pre {
  margin: 0;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0.6rem;
  font-size: 0.72em;
  font-family: monospace;
  max-height: 180px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  color: #aaa;
}

.path-item {
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 0.75rem;
}

.path-val {
  font-family: monospace;
  font-size: 0.75em;
  word-break: break-all;
  color: #888;
}
</style>
