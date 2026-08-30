<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppInfo,
  FileSortField,
  Folder,
  ImageFile,
  LibraryCounts,
  ScanProgress,
  SortDirection,
} from "./types";
import FolderPicker from "./components/FolderPicker.vue";
import FolderList from "./components/FolderList.vue";
import FileList from "./components/FileList.vue";
import VirtualGrid from "./components/VirtualGrid.vue";
import SortBar from "./components/SortBar.vue";
import PreviewPane from "./components/PreviewPane.vue";
import SearchBar from "./components/SearchBar.vue";

const info = ref<AppInfo | null>(null);
const folders = ref<Folder[]>([]);
const libraryCounts = ref<LibraryCounts | null>(null);
const files = ref<ImageFile[]>([]);
const filesLoading = ref(false);
const searchQuery = ref("");
const selectedFolder = ref<Folder | null>(null);
const selectedFile = ref<ImageFile | null>(null);
const previewingFile = ref<ImageFile | null>(null);
const viewMode = ref<"grid" | "table">("grid");
const sortField = ref<FileSortField>("modified_at");
const sortDirection = ref<SortDirection>("desc");
const progress = ref<ScanProgress | null>(null);
const error = ref("");

let unlisten: UnlistenFn | null = null;

onMounted(async () => {
  try {
    info.value = await invoke<AppInfo>("get_app_info");
    await reloadFolders();
    await refreshCounts();
    await loadFiles();
  } catch (e) {
    error.value = String(e);
  }

  unlisten = await listen<ScanProgress>("scan-progress", (event) => {
    progress.value = event.payload;
  });
});

onUnmounted(() => {
  unlisten?.();
});

async function reloadFolders() {
  folders.value = await invoke<Folder[]>("list_folders");
}

async function refreshCounts() {
  try {
    libraryCounts.value = await invoke<LibraryCounts>("get_library_counts");
  } catch (e) {
    console.error("Failed to fetch library counts:", e);
  }
}

function onFolderAdded(folder: Folder) {
  void reloadFolders();
  void refreshCounts();
  selectedFolder.value = folder;
  selectedFile.value = null;
  previewingFile.value = null;
  void loadFiles();
}

function onFolderRemoved(folderId: number) {
  folders.value = folders.value.filter((f) => f.id !== folderId);
  void refreshCounts();
  if (selectedFolder.value?.id === folderId) {
    selectedFolder.value = null;
    selectedFile.value = null;
    previewingFile.value = null;
  }
  void loadFiles();
}

async function onFolderSelected(folder: Folder | null) {
  selectedFolder.value = folder;
  selectedFile.value = null;
  previewingFile.value = null;
  await loadFiles();
}

async function onFolderScanned(_folderId: number) {
  await refreshCounts();
  await loadFiles();
}

function onFileSelected(file: ImageFile) {
  selectedFile.value = file;
}

function onActivateFile(file: ImageFile) {
  selectedFile.value = file;
  previewingFile.value = file;
}

function onPreviewNavigate(file: ImageFile) {
  selectedFile.value = file;
  previewingFile.value = file;
}

function onFileRated(fileId: number, rating: number | null) {
  const idx = files.value.findIndex((f) => f.id === fileId);
  if (idx !== -1) {
    files.value[idx].rating = rating ?? undefined;
  }
  if (selectedFile.value?.id === fileId) {
    selectedFile.value.rating = rating ?? undefined;
  }
  if (previewingFile.value?.id === fileId) {
    previewingFile.value.rating = rating ?? undefined;
  }
}

async function loadFiles() {
  filesLoading.value = true;
  try {
    const q = searchQuery.value.trim();
    if (q) {
      files.value = await invoke<ImageFile[]>("search_files_by_query", {
        query: q,
        folderId: selectedFolder.value?.id ?? null,
        sort: sortField.value,
        direction: sortDirection.value,
      });
    } else {
      files.value = await invoke<ImageFile[]>("query_files", {
        folderId: selectedFolder.value?.id ?? null,
        sort: sortField.value,
        direction: sortDirection.value,
      });
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    filesLoading.value = false;
  }
}

function onSearch(query: string) {
  searchQuery.value = query;
  void loadFiles();
}

function onClearSearch() {
  searchQuery.value = "";
  void loadFiles();
}
</script>

<template>
  <main class="shell">
    <header class="header">
      <h1>Berry AIGC Toolbox</h1>
      <p class="tagline">Metadata indexer and viewer for AI-generated images</p>
      <p v-if="info" class="meta">
        v{{ info.app_version }} · schema v{{ info.schema_version }}
        <span class="db-path" :title="info.database_path">{{ info.database_path }}</span>
      </p>
      <p v-else-if="error" class="error">{{ error }}</p>
    </header>

    <FolderPicker @added="onFolderAdded" />

    <FolderList
      :folders="folders"
      :counts="libraryCounts"
      :selected-id="selectedFolder?.id ?? null"
      :progress="progress"
      @removed="onFolderRemoved"
      @selected="onFolderSelected"
      @scanned="onFolderScanned"
    />

    <section class="files-view-section">
      <div class="files-view-header">
        <div class="header-left">
          <h2>
            Files
            <span v-if="files.length" class="count-badge">({{ files.length }})</span>
          </h2>
          <span v-if="selectedFolder" class="folder-badge" :title="selectedFolder.path">
            {{ selectedFolder.path.split(/[\\/]/).pop() || selectedFolder.path }}
          </span>
          <span v-else class="folder-badge all-badge">All Images</span>
          <span v-if="selectedFile" class="selection-pill" :title="selectedFile.path">
            Selected: {{ selectedFile.path.split(/[\\/]/).pop() }}
            <button
              type="button"
              class="preview-trigger-btn"
              title="Open Preview (Enter / Space)"
              @click="onActivateFile(selectedFile)"
            >
              👁
            </button>
          </span>
        </div>

        <div class="toolbar-actions">
          <SortBar
            v-model:sort-field="sortField"
            v-model:sort-direction="sortDirection"
            @change="loadFiles"
          />

          <div class="view-mode-toggle">
            <button
              type="button"
              class="toggle-btn"
              :class="{ active: viewMode === 'grid' }"
              @click="viewMode = 'grid'"
              title="Grid View"
            >
              ⊞ Grid
            </button>
            <button
              type="button"
              class="toggle-btn"
              :class="{ active: viewMode === 'table' }"
              @click="viewMode = 'table'"
              title="Table View"
            >
              ☰ Table
            </button>
          </div>
        </div>
      </div>

      <div class="search-toolbar">
        <SearchBar
          v-model="searchQuery"
          :loading="filesLoading"
          :result-count="searchQuery.trim() ? files.length : null"
          @search="onSearch"
          @clear="onClearSearch"
        />
      </div>

      <div v-if="searchQuery.trim()" class="search-status-bar">
        <span>
          Found <strong>{{ files.length }}</strong> matching {{ files.length === 1 ? "image" : "images" }}
        </span>
        <button type="button" class="reset-search-btn" @click="onClearSearch">
          ✕ Clear Search
        </button>
      </div>

      <div
        v-if="files.length === 0 && searchQuery.trim() && !filesLoading"
        class="search-empty-state"
      >
        <span class="empty-icon">🔍</span>
        <p class="empty-title">No images match your search</p>
        <p class="empty-hint">
          Try adjusting keywords or parameter filters like <code>prompt:cat</code>,
          <code>model:sdxl</code>, or <code>steps:&gt;=20</code>.
        </p>
        <button type="button" class="clear-filters-btn" @click="onClearSearch">
          Clear Search
        </button>
      </div>

      <template v-else>
        <VirtualGrid
          v-if="viewMode === 'grid'"
          :files="files"
          :selected-file="selectedFile"
          :loading="filesLoading"
          @select="onFileSelected"
          @activate="onActivateFile"
        />
        <FileList
          v-else
          :files="files"
          :selected-file="selectedFile"
          :loading="filesLoading"
          @select="onFileSelected"
          @activate="onActivateFile"
        />
      </template>
    </section>

    <!-- Full-screen Preview Modal with Inspector -->
    <PreviewPane
      v-if="previewingFile"
      :file="previewingFile"
      :files="files"
      @close="previewingFile = null"
      @navigate="onPreviewNavigate"
      @rate="onFileRated"
    />
  </main>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: #0f0f0f;
  background-color: #f6f6f6;
  font-synthesis: none;
  -webkit-font-smoothing: antialiased;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
}

.shell {
  max-width: 68rem;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

.header {
  margin-bottom: 1.5rem;
}

h1 {
  margin: 0;
  font-size: 1.5rem;
}

.tagline {
  color: #888;
  margin: 0.25rem 0 0;
}

.meta {
  color: #888;
  font-size: 0.8em;
  margin: 0.5rem 0 0;
}

.db-path {
  display: block;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  word-break: break-all;
  margin-top: 0.15rem;
}

.error {
  color: #d33;
  font-size: 0.85em;
}

.files-view-section {
  margin-top: 2rem;
}

.files-view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.header-left h2 {
  margin: 0;
  font-size: 1.25rem;
  display: flex;
  align-items: center;
  gap: 0.35rem;
}

.count-badge {
  font-size: 0.8em;
  font-weight: 500;
  color: #888;
}

.folder-badge {
  padding: 0.15rem 0.6rem;
  border-radius: 999px;
  font-size: 0.8em;
  font-weight: 500;
  background: rgba(47, 111, 237, 0.12);
  color: #2f6fed;
}

.all-badge {
  background: rgba(16, 185, 129, 0.12);
  color: #059669;
}

@media (prefers-color-scheme: dark) {
  .all-badge {
    color: #34d399;
  }
}

.selection-pill {
  padding: 0.15rem 0.6rem;
  border-radius: 6px;
  font-size: 0.78em;
  background: rgba(0, 0, 0, 0.05);
  color: #666;
  max-width: 20rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

@media (prefers-color-scheme: dark) {
  .selection-pill {
    background: rgba(255, 255, 255, 0.08);
    color: #aaa;
  }
}

.preview-trigger-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0 0.15rem;
  font-size: 1em;
  opacity: 0.7;
  transition: opacity 0.15s ease, transform 0.1s ease;
}

.preview-trigger-btn:hover {
  opacity: 1;
  transform: scale(1.15);
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.view-mode-toggle {
  display: inline-flex;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 6px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.06);
}

.toggle-btn {
  border: none;
  background: transparent;
  padding: 0.35rem 0.75rem;
  font-size: 0.85em;
  cursor: pointer;
  color: #666;
  transition: all 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .toggle-btn {
    color: #aaa;
  }
}

.toggle-btn:hover {
  color: inherit;
}

.toggle-btn.active {
  background: #2f6fed;
  color: #fff;
  font-weight: 600;
}

.search-toolbar {
  margin-bottom: 0.75rem;
}

.search-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.4rem 0.75rem;
  margin-bottom: 0.75rem;
  background: rgba(47, 111, 237, 0.08);
  border-left: 3px solid #2f6fed;
  border-radius: 4px;
  font-size: 0.85em;
  color: #333;
}

@media (prefers-color-scheme: dark) {
  .search-status-bar {
    background: rgba(47, 111, 237, 0.15);
    color: #ddd;
  }
}

.reset-search-btn {
  background: transparent;
  border: none;
  color: #2f6fed;
  font-size: 0.9em;
  font-weight: 500;
  cursor: pointer;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.reset-search-btn:hover {
  background: rgba(47, 111, 237, 0.15);
}

.search-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3.5rem 1rem;
  text-align: center;
  border: 1px dashed rgba(128, 128, 128, 0.25);
  border-radius: 10px;
  background: rgba(128, 128, 128, 0.03);
}

.empty-icon {
  font-size: 2.5em;
  margin-bottom: 0.5rem;
  opacity: 0.6;
}

.empty-title {
  font-size: 1.1em;
  font-weight: 600;
  margin: 0 0 0.4rem;
}

.empty-hint {
  font-size: 0.85em;
  color: #888;
  max-width: 440px;
  margin: 0 0 1rem;
  line-height: 1.4;
}

.empty-hint code {
  background: rgba(128, 128, 128, 0.12);
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  font-family: monospace;
}

.clear-filters-btn {
  background: #2f6fed;
  color: #fff;
  border: none;
  padding: 0.4rem 1rem;
  border-radius: 6px;
  font-size: 0.85em;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.clear-filters-btn:hover {
  opacity: 0.9;
}
</style>
