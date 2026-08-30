<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Album,
  AppInfo,
  FileSortField,
  Folder,
  ImageFile,
  LibraryCounts,
  NavTarget,
  ScanProgress,
  SearchCriteria,
  SortDirection,
  Tag,
} from "./types";
import FolderPicker from "./components/FolderPicker.vue";
import FolderList from "./components/FolderList.vue";
import FileList from "./components/FileList.vue";
import VirtualGrid from "./components/VirtualGrid.vue";
import SortBar from "./components/SortBar.vue";
import PreviewPane from "./components/PreviewPane.vue";
import SearchBar from "./components/SearchBar.vue";
import FilterDrawer from "./components/FilterDrawer.vue";
import BatchActionBar from "./components/BatchActionBar.vue";
import AlbumModal from "./components/AlbumModal.vue";
import TagModal from "./components/TagModal.vue";
import { countActiveFilters, criteriaToQuery } from "./utils/search";

const info = ref<AppInfo | null>(null);
const folders = ref<Folder[]>([]);
const libraryCounts = ref<LibraryCounts | null>(null);
const albums = ref<Album[]>([]);
const albumCounts = ref<Record<number, number>>({});
const tags = ref<Tag[]>([]);
const activeTarget = ref<NavTarget>({ type: "all" });
const files = ref<ImageFile[]>([]);
const filesLoading = ref(false);
const searchQuery = ref("");
const filterDrawerOpen = ref(false);
const albumModalOpen = ref(false);
const albumTargetFileIds = ref<number[]>([]);
const tagModalOpen = ref(false);
const tagTargetFileIds = ref<number[]>([]);
const distinctModels = ref<string[]>([]);
const distinctSamplers = ref<string[]>([]);
const activeCriteria = ref<SearchCriteria>({});
const activeFilterCount = computed(() => countActiveFilters(activeCriteria.value));
const selectedFile = ref<ImageFile | null>(null);
const selectedFilePaths = ref<Set<string>>(new Set());
const selectedFilesList = computed(() =>
  files.value.filter((f) => selectedFilePaths.value.has(f.path)),
);
const previewingFile = ref<ImageFile | null>(null);
const viewMode = ref<"grid" | "table">("grid");
const sortField = ref<FileSortField>("modified_at");
const sortDirection = ref<SortDirection>("desc");
const progress = ref<ScanProgress | null>(null);
const error = ref("");

let unlisten: UnlistenFn | null = null;

function handleWindowKeyDown(e: KeyboardEvent) {
  const tag = (document.activeElement?.tagName ?? "").toLowerCase();
  if (tag === "input" || tag === "textarea") return;

  if ((e.ctrlKey || e.metaKey) && (e.key === "a" || e.key === "A")) {
    e.preventDefault();
    onSelectAll();
  } else if (
    e.key === "Escape" &&
    selectedFilePaths.value.size > 0 &&
    !previewingFile.value &&
    !filterDrawerOpen.value
  ) {
    onClearSelection();
  }
}

onMounted(async () => {
  window.addEventListener("keydown", handleWindowKeyDown);
  try {
    info.value = await invoke<AppInfo>("get_app_info");
    await reloadFolders();
    await refreshCounts();
    await reloadFiltersMeta();
    await loadAlbumsAndTags();
    await loadFiles();
  } catch (e) {
    error.value = String(e);
  }

  unlisten = await listen<ScanProgress>("scan-progress", (event) => {
    progress.value = event.payload;
  });
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleWindowKeyDown);
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

async function reloadFiltersMeta() {
  try {
    distinctModels.value = await invoke<string[]>("list_distinct_models");
    distinctSamplers.value = await invoke<string[]>("list_distinct_samplers");
  } catch (e) {
    console.error("Failed to load distinct models/samplers:", e);
  }
}

async function loadAlbumsAndTags() {
  try {
    albums.value = await invoke<Album[]>("list_albums");
    const counts: Record<number, number> = {};
    for (const album of albums.value) {
      counts[album.id] = await invoke<number>("count_album_files", {
        albumId: album.id,
      });
    }
    albumCounts.value = counts;
    tags.value = await invoke<Tag[]>("list_tags");
  } catch (e) {
    console.error("Failed to load albums/tags:", e);
  }
}

function onFolderAdded(folder: Folder) {
  void reloadFolders();
  void refreshCounts();
  void reloadFiltersMeta();
  activeTarget.value = { type: "folder", folder };
  selectedFile.value = null;
  previewingFile.value = null;
  void loadFiles();
}

function onFolderRemoved(folderId: number) {
  folders.value = folders.value.filter((f) => f.id !== folderId);
  void refreshCounts();
  void reloadFiltersMeta();
  if (activeTarget.value.type === "folder" && activeTarget.value.folder.id === folderId) {
    activeTarget.value = { type: "all" };
    selectedFile.value = null;
    previewingFile.value = null;
  }
  void loadFiles();
}

function onSelectNav(target: NavTarget) {
  activeTarget.value = target;
  selectedFile.value = null;
  previewingFile.value = null;
  void loadFiles();
}

const targetTitle = computed(() => {
  switch (activeTarget.value.type) {
    case "all":
      return "All Images";
    case "favorites":
      return "★ Favorites";
    case "nsfw":
      return "🔞 Sensitive (18+)";
    case "folder":
      return activeTarget.value.folder.path.split(/[\\/]/).pop() || activeTarget.value.folder.path;
    case "album":
      return `🗂️ ${activeTarget.value.album.name}`;
    case "tag":
      return `🏷 #${activeTarget.value.tag.name}`;
  }
});

async function onFolderScanned(_folderId: number) {
  await refreshCounts();
  await reloadFiltersMeta();
  await loadAlbumsAndTags();
  await loadFiles();
}

function onFileSelected(file: ImageFile, event?: MouseEvent) {
  selectedFile.value = file;
  if (event?.metaKey || event?.ctrlKey) {
    toggleSelectFile(file);
  } else if (event?.shiftKey && selectedFilesList.value.length > 0) {
    const lastFile = selectedFilesList.value[selectedFilesList.value.length - 1];
    const idx1 = files.value.findIndex((f) => f.path === lastFile.path);
    const idx2 = files.value.findIndex((f) => f.path === file.path);
    if (idx1 !== -1 && idx2 !== -1) {
      const [start, end] = idx1 < idx2 ? [idx1, idx2] : [idx2, idx1];
      for (let i = start; i <= end; i++) {
        selectedFilePaths.value.add(files.value[i].path);
      }
    }
  }
}

function toggleSelectFile(file: ImageFile) {
  if (selectedFilePaths.value.has(file.path)) {
    selectedFilePaths.value.delete(file.path);
  } else {
    selectedFilePaths.value.add(file.path);
  }
}

function onSelectAll() {
  selectedFilePaths.value = new Set(files.value.map((f) => f.path));
}

function onClearSelection() {
  selectedFilePaths.value.clear();
}

function onToggleAll() {
  if (selectedFilePaths.value.size === files.value.length) {
    selectedFilePaths.value.clear();
  } else {
    onSelectAll();
  }
}

async function onBatchRate(rating: number | null) {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length === 0) return;

  try {
    await invoke("set_files_rating", { fileIds: ids, rating });
    const idSet = new Set(ids);
    files.value = files.value.map((f) => {
      if (f.id != null && idSet.has(f.id)) {
        return { ...f, rating: rating ?? undefined };
      }
      return f;
    });
    if (selectedFile.value?.id != null && idSet.has(selectedFile.value.id)) {
      selectedFile.value.rating = rating ?? undefined;
    }
  } catch (e) {
    error.value = String(e);
  }
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

function onOpenAlbumModal(fileIds?: number[]) {
  albumTargetFileIds.value = fileIds ?? [];
  albumModalOpen.value = true;
}

function onBatchAddToAlbum() {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length > 0) {
    onOpenAlbumModal(ids);
  }
}

function onAddedToAlbum(_album: Album) {
  selectedFilePaths.value.clear();
}

function onOpenTagModal(fileIds?: number[]) {
  tagTargetFileIds.value = fileIds ?? [];
  tagModalOpen.value = true;
}

function onBatchTag() {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length > 0) {
    onOpenTagModal(ids);
  }
}

async function onBatchToggleFavorite(isFavorite: boolean) {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length === 0) return;
  try {
    await invoke("set_files_favorite", { fileIds: ids, isFavorite });
    for (const f of selectedFilesList.value) {
      f.is_favorite = isFavorite;
    }
  } catch (err) {
    error.value = String(err);
  }
}

async function onBatchToggleNsfw(isNsfw: boolean) {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length === 0) return;
  try {
    await invoke("set_files_nsfw", { fileIds: ids, isNsfw });
    for (const f of selectedFilesList.value) {
      f.is_nsfw = isNsfw;
    }
  } catch (err) {
    error.value = String(err);
  }
}

function onUpdateFile(file: ImageFile) {
  const idx = files.value.findIndex((f) => f.id === file.id);
  if (idx !== -1) {
    files.value[idx] = { ...file };
  }
  if (selectedFile.value?.id === file.id) {
    selectedFile.value = { ...file };
  }
  if (previewingFile.value?.id === file.id) {
    previewingFile.value = { ...file };
  }
}

async function loadFiles() {
  filesLoading.value = true;
  selectedFilePaths.value.clear();
  try {
    const q = searchQuery.value.trim();
    if (q) {
      const folderId = activeTarget.value.type === "folder" ? activeTarget.value.folder.id : null;
      files.value = await invoke<ImageFile[]>("search_files_by_query", {
        query: q,
        folderId,
        sort: sortField.value,
        direction: sortDirection.value,
      });
    } else {
      const criteria: SearchCriteria = {
        sort: sortField.value,
        direction: sortDirection.value,
      };
      if (activeTarget.value.type === "folder") {
        criteria.folder_id = activeTarget.value.folder.id;
      } else if (activeTarget.value.type === "favorites") {
        criteria.is_favorite = true;
      } else if (activeTarget.value.type === "nsfw") {
        criteria.is_nsfw = true;
      } else if (activeTarget.value.type === "album") {
        criteria.album_id = activeTarget.value.album.id;
      } else if (activeTarget.value.type === "tag") {
        criteria.tag_id = activeTarget.value.tag.id;
      }
      files.value = await invoke<ImageFile[]>("search_files", { criteria });
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
  activeCriteria.value = {};
  void loadFiles();
}

function onApplyFilters(criteria: SearchCriteria) {
  activeCriteria.value = criteria;
  const q = criteriaToQuery(criteria);
  searchQuery.value = q;
  void loadFiles();
}

function onResetFilters() {
  activeCriteria.value = {};
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
      :albums="albums"
      :album-counts="albumCounts"
      :tags="tags"
      :active-target="activeTarget"
      :progress="progress"
      @removed="onFolderRemoved"
      @scanned="onFolderScanned"
      @select-nav="onSelectNav"
      @open-album-modal="() => onOpenAlbumModal()"
      @open-tag-modal="() => onOpenTagModal()"
    />

    <section class="files-view-section">
      <div class="files-view-header">
        <div class="header-left">
          <h2>
            Files
            <span v-if="files.length" class="count-badge">({{ files.length }})</span>
          </h2>
          <span class="folder-badge" :class="{ 'all-badge': activeTarget.type === 'all' }">
            {{ targetTitle }}
          </span>
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
        <button
          type="button"
          class="filter-toggle-btn"
          :class="{ active: filterDrawerOpen || activeFilterCount > 0 }"
          title="Open visual search filters"
          @click="filterDrawerOpen = true"
        >
          <span>⚡ Filters</span>
          <span v-if="activeFilterCount > 0" class="filter-count-badge">
            {{ activeFilterCount }}
          </span>
        </button>
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
          :selected-file-paths="selectedFilePaths"
          :loading="filesLoading"
          @select="onFileSelected"
          @activate="onActivateFile"
          @toggle-select="toggleSelectFile"
        />
        <FileList
          v-else
          :files="files"
          :selected-file="selectedFile"
          :selected-file-paths="selectedFilePaths"
          :loading="filesLoading"
          @select="onFileSelected"
          @activate="onActivateFile"
          @toggle-select="toggleSelectFile"
          @toggle-all="onToggleAll"
        />
      </template>
    </section>

    <PreviewPane
      v-if="previewingFile"
      :file="previewingFile"
      :files="files"
      @close="previewingFile = null"
      @navigate="onPreviewNavigate"
      @rate="onFileRated"
      @update-file="onUpdateFile"
      @open-tag-modal="(id) => onOpenTagModal([id])"
      @open-album-modal="(id) => onOpenAlbumModal([id])"
    />

    <!-- Visual Search Builder / Filter Drawer -->
    <FilterDrawer
      v-model:open="filterDrawerOpen"
      :models="distinctModels"
      :samplers="distinctSamplers"
      :initial-criteria="activeCriteria"
      @apply="onApplyFilters"
      @reset="onResetFilters"
    />

    <!-- Floating Batch Action Bar -->
    <BatchActionBar
      :selected-files="selectedFilesList"
      :total-count="files.length"
      @select-all="onSelectAll"
      @clear-selection="onClearSelection"
      @rate-selected="onBatchRate"
      @add-to-album="onBatchAddToAlbum"
      @tag-selected="onBatchTag"
      @toggle-favorite="onBatchToggleFavorite"
      @toggle-nsfw="onBatchToggleNsfw"
    />

    <!-- Album Management & Assignment Modal -->
    <AlbumModal
      v-model:open="albumModalOpen"
      :file-ids="albumTargetFileIds"
      @added-to-album="onAddedToAlbum"
      @albums-changed="loadAlbumsAndTags"
    />

    <!-- Tag Management & Assignment Modal -->
    <TagModal
      v-model:open="tagModalOpen"
      :file-ids="tagTargetFileIds"
      @tags-changed="loadAlbumsAndTags"
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
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
  margin-bottom: 0.75rem;
}

.filter-toggle-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.45rem 0.85rem;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  background: rgba(128, 128, 128, 0.08);
  color: inherit;
  font: inherit;
  font-size: 0.88em;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
  height: 38px;
  box-sizing: border-box;
}

.filter-toggle-btn:hover {
  background: rgba(128, 128, 128, 0.15);
  border-color: rgba(128, 128, 128, 0.4);
}

.filter-toggle-btn.active {
  background: rgba(47, 111, 237, 0.12);
  border-color: #2f6fed;
  color: #2f6fed;
}

@media (prefers-color-scheme: dark) {
  .filter-toggle-btn.active {
    background: rgba(47, 111, 237, 0.2);
    color: #60a5fa;
  }
}

.filter-count-badge {
  background: #2f6fed;
  color: #fff;
  font-size: 0.75em;
  font-weight: 700;
  border-radius: 999px;
  padding: 0.05rem 0.45rem;
  line-height: 1.2;
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
