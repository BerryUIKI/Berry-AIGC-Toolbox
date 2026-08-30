<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CheckpointModelStat, ModelCacheEntry } from "../types";

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "filter-model", modelName: string): void;
  (e: "filter-hash", modelHash: string): void;
}>();

const models = ref<CheckpointModelStat[]>([]);
const cachedEntries = ref<ModelCacheEntry[]>([]);
const searchQuery = ref("");
const loading = ref(false);
const importPath = ref("");
const isImporting = ref(false);
const message = ref<{ type: "success" | "error"; text: string } | null>(null);

async function loadData() {
  loading.value = true;
  message.value = null;
  try {
    const [fetchedModels, fetchedCache] = await Promise.all([
      invoke<CheckpointModelStat[]>("get_checkpoint_models"),
      invoke<ModelCacheEntry[]>("list_model_cache"),
    ]);
    models.value = fetchedModels;
    cachedEntries.value = fetchedCache;
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    loading.value = false;
  }
}

const cacheMap = computed(() => {
  const map = new Map<string, ModelCacheEntry>();
  for (const entry of cachedEntries.value) {
    map.set(entry.hash.toLowerCase(), entry);
    if (entry.sha256) {
      map.set(entry.sha256.toLowerCase(), entry);
    }
  }
  return map;
});

const filteredModels = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return models.value;
  return models.value.filter((m) => {
    const nameMatch = m.model_name.toLowerCase().includes(q);
    const hashMatch = m.model_hash?.toLowerCase().includes(q) ?? false;
    const titleMatch = m.model_hash
      ? cacheMap.value.get(m.model_hash.toLowerCase())?.title?.toLowerCase().includes(q)
      : false;
    return nameMatch || hashMatch || titleMatch;
  });
});

async function handleImport() {
  if (!importPath.value.trim()) return;
  isImporting.value = true;
  message.value = null;
  try {
    const count = await invoke<number>("import_model_cache_file", {
      path: importPath.value.trim(),
    });
    message.value = {
      type: "success",
      text: `Successfully imported ${count} model mappings!`,
    };
    importPath.value = "";
    await loadData();
  } catch (err: any) {
    message.value = { type: "error", text: `Import failed: ${err}` };
  } finally {
    isImporting.value = false;
  }
}

function handleFilterModel(model: CheckpointModelStat) {
  emit("filter-model", model.model_name);
  emit("close");
}

function handleFilterHash(model: CheckpointModelStat) {
  if (model.model_hash) {
    emit("filter-hash", model.model_hash);
    emit("close");
  }
}

onMounted(() => {
  if (props.show) {
    loadData();
  }
});
</script>

<template>
  <div v-if="show" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-container">
      <div class="modal-header">
        <div class="modal-title-wrap">
          <svg class="modal-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <h2>Checkpoint Models & Hash Cache</h2>
        </div>
        <button class="close-btn" @click="emit('close')" title="Close (Esc)">✕</button>
      </div>

      <div class="modal-body">
        <!-- Import Bar -->
        <div class="import-card">
          <div class="import-label">Import A1111 cache.json / Model Hash Mappings:</div>
          <div class="import-input-row">
            <input
              v-model="importPath"
              type="text"
              class="import-input"
              placeholder="/path/to/cache.json or model_hashes.json"
              @keydown.enter="handleImport"
            />
            <button
              class="import-btn"
              :disabled="!importPath.trim() || isImporting"
              @click="handleImport"
            >
              {{ isImporting ? "Importing..." : "Import" }}
            </button>
          </div>
        </div>

        <div v-if="message" class="status-banner" :class="message.type">
          {{ message.text }}
        </div>

        <!-- Search Bar -->
        <div class="search-bar-row">
          <input
            v-model="searchQuery"
            type="text"
            class="filter-input"
            placeholder="Search models by name, title, or hash..."
          />
          <div class="model-count-tag">
            {{ filteredModels.length }} / {{ models.length }} Models
          </div>
        </div>

        <!-- Models List -->
        <div v-if="loading" class="loading-state">Loading model library...</div>
        <div v-else-if="filteredModels.length === 0" class="empty-state">
          No checkpoint models match the search criteria.
        </div>
        <div v-else class="models-grid">
          <div
            v-for="model in filteredModels"
            :key="model.model_name + (model.model_hash || '')"
            class="model-card"
          >
            <div class="model-info">
              <div class="model-header-row">
                <span class="model-name" :title="model.model_name">
                  {{ model.model_name }}
                </span>
                <span class="badge-count" title="Number of indexed images">
                  {{ model.count }} {{ model.count === 1 ? 'image' : 'images' }}
                </span>
              </div>

              <!-- Cache resolution title if available -->
              <div
                v-if="model.model_hash && cacheMap.get(model.model_hash.toLowerCase())?.title"
                class="model-title"
              >
                {{ cacheMap.get(model.model_hash.toLowerCase())?.title }}
              </div>

              <div class="model-meta-row">
                <span
                  v-if="model.model_hash"
                  class="hash-badge"
                  :title="'Filter by hash: ' + model.model_hash"
                  @click="handleFilterHash(model)"
                >
                  #{{ model.model_hash }}
                </span>
                <span v-else class="no-hash-badge">No Hash</span>
              </div>
            </div>

            <div class="model-actions">
              <button
                class="action-btn"
                title="Filter gallery by this model"
                @click="handleFilterModel(model)"
              >
                Filter
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
}

.modal-container {
  background: var(--bg-surface, #1e1e24);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  width: 90%;
  max-width: 760px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.4);
  color: var(--text-primary, #eee);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, #333);
}

.modal-title-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-icon {
  width: 22px;
  height: 22px;
  color: #3b82f6;
}

.modal-header h2 {
  font-size: 1.15rem;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary, #999);
  font-size: 1.2rem;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
}

.close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.modal-body {
  padding: 18px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.import-card {
  background: var(--bg-card, #25252d);
  border: 1px solid var(--border-color, #3a3a46);
  border-radius: 8px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.import-label {
  font-size: 0.85rem;
  color: var(--text-secondary, #aaa);
}

.import-input-row {
  display: flex;
  gap: 8px;
}

.import-input {
  flex: 1;
  background: var(--bg-input, #16161a);
  border: 1px solid var(--border-color, #3a3a46);
  color: var(--text-primary, #fff);
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 0.85rem;
}

.import-input:focus {
  outline: none;
  border-color: #3b82f6;
}

.import-btn {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.2s;
}

.import-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.status-banner {
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
}

.status-banner.success {
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.4);
  color: #4ade80;
}

.status-banner.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.4);
  color: #f87171;
}

.search-bar-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter-input {
  flex: 1;
  background: var(--bg-input, #16161a);
  border: 1px solid var(--border-color, #3a3a46);
  color: var(--text-primary, #fff);
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.9rem;
}

.filter-input:focus {
  outline: none;
  border-color: #3b82f6;
}

.model-count-tag {
  font-size: 0.85rem;
  color: var(--text-secondary, #999);
  white-space: nowrap;
}

.loading-state,
.empty-state {
  text-align: center;
  padding: 36px 0;
  color: var(--text-secondary, #888);
  font-size: 0.9rem;
}

.models-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 420px;
  overflow-y: auto;
  padding-right: 4px;
}

.model-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-card, #25252d);
  border: 1px solid var(--border-color, #3a3a46);
  border-radius: 8px;
  padding: 10px 14px;
  gap: 12px;
}

.model-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.model-header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.model-name {
  font-weight: 500;
  font-size: 0.95rem;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.badge-count {
  font-size: 0.75rem;
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  color: #aaa;
  white-space: nowrap;
}

.model-title {
  font-size: 0.85rem;
  color: #60a5fa;
}

.model-meta-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hash-badge {
  font-family: monospace;
  font-size: 0.78rem;
  background: rgba(59, 130, 246, 0.15);
  color: #93c5fd;
  border: 1px solid rgba(59, 130, 246, 0.3);
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.hash-badge:hover {
  background: rgba(59, 130, 246, 0.3);
  color: #bfdbfe;
}

.no-hash-badge {
  font-size: 0.75rem;
  color: #777;
}

.action-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--border-color, #444);
  color: var(--text-primary, #ddd);
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: #3b82f6;
  border-color: #3b82f6;
  color: #fff;
}
</style>
