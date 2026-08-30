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
import PromptStatsModal from "./components/PromptStatsModal.vue";
import ModelManagerModal from "./components/ModelManagerModal.vue";
import FileOperationModal from "./components/FileOperationModal.vue";
import DatabaseManagerModal from "./components/DatabaseManagerModal.vue";
import ShortcutsHelpModal from "./components/ShortcutsHelpModal.vue";
import LanguageSelector from "./components/LanguageSelector.vue";
import { t } from "./i18n";
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
const promptStatsModalOpen = ref(false);
const modelManagerModalOpen = ref(false);
const dbManagerModalOpen = ref(false);
const shortcutsHelpModalOpen = ref(false);
const fileOpModalOpen = ref(false);
const fileOpMode = ref<"move" | "copy" | "trash">("move");
const fileOpTargetFiles = ref<ImageFile[]>([]);
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
  if (tag === "input" || tag === "textarea") {
    if (e.key === "Escape") {
      (document.activeElement as HTMLElement)?.blur();
    }
    return;
  }

  // Help Modal: ?
  if (e.key === "?" || (e.shiftKey && e.key === "/")) {
    e.preventDefault();
    shortcutsHelpModalOpen.value = !shortcutsHelpModalOpen.value;
    return;
  }

  // Focus Search Bar: / or Cmd+F / Ctrl+F
  if (e.key === "/" || ((e.ctrlKey || e.metaKey) && (e.key === "f" || e.key === "F"))) {
    e.preventDefault();
    const searchInput = document.querySelector<HTMLInputElement>(".search-bar input");
    searchInput?.focus();
    searchInput?.select();
    return;
  }

  // Select All: Cmd+A / Ctrl+A
  if ((e.ctrlKey || e.metaKey) && (e.key === "a" || e.key === "A")) {
    e.preventDefault();
    onSelectAll();
    return;
  }

  // Escape: Close modals or clear selection
  if (e.key === "Escape") {
    if (shortcutsHelpModalOpen.value) {
      shortcutsHelpModalOpen.value = false;
      return;
    }
    if (dbManagerModalOpen.value) {
      dbManagerModalOpen.value = false;
      return;
    }
    if (modelManagerModalOpen.value) {
      modelManagerModalOpen.value = false;
      return;
    }
    if (promptStatsModalOpen.value) {
      promptStatsModalOpen.value = false;
      return;
    }
    if (filterDrawerOpen.value) {
      filterDrawerOpen.value = false;
      return;
    }
    if (albumModalOpen.value) {
      albumModalOpen.value = false;
      return;
    }
    if (tagModalOpen.value) {
      tagModalOpen.value = false;
      return;
    }
    if (fileOpModalOpen.value) {
      fileOpModalOpen.value = false;
      return;
    }
    if (selectedFilePaths.value.size > 0) {
      onClearSelection();
      return;
    }
  }

  // Open Preview / Inspector: Space or Enter
  if (e.key === " " || e.key === "Enter") {
    if (!previewingFile.value && (selectedFile.value || selectedFilesList.value.length > 0)) {
      e.preventDefault();
      previewingFile.value = selectedFile.value || selectedFilesList.value[0];
      return;
    }
  }

  // Star Ratings: 1 - 5 (or 0 to clear)
  if (["0", "1", "2", "3", "4", "5"].includes(e.key)) {
    const targetFile =
      selectedFile.value || (selectedFilesList.value.length > 0 ? selectedFilesList.value[0] : null);
    if (targetFile && targetFile.id != null) {
      e.preventDefault();
      const rating = e.key === "0" ? null : parseInt(e.key, 10);
      if (selectedFilesList.value.length > 1) {
        void onBatchRate(rating);
      } else {
        void invoke("set_file_rating", { fileId: targetFile.id, rating });
        onFileRated(targetFile.id, rating);
      }
      return;
    }
  }

  // Favorite toggle: F
  if (e.key === "f" || e.key === "F") {
    const targetFile =
      selectedFile.value || (selectedFilesList.value.length > 0 ? selectedFilesList.value[0] : null);
    if (targetFile) {
      e.preventDefault();
      if (selectedFilesList.value.length > 1) {
        const anyUnfav = selectedFilesList.value.some((f) => !f.is_favorite);
        void onBatchToggleFavorite(anyUnfav);
      } else {
        void onBatchToggleFavorite(!targetFile.is_favorite);
      }
      return;
    }
  }

  // Delete / Trash: Delete or Backspace
  if (e.key === "Delete" || e.key === "Backspace") {
    if (selectedFilesList.value.length > 0 || selectedFile.value) {
      e.preventDefault();
      if (selectedFilesList.value.length === 0 && selectedFile.value) {
        selectedFilePaths.value.add(selectedFile.value.path);
      }
      onBatchTrash();
      return;
    }
  }

  // Arrow Keys Navigation when not in preview
  if (!previewingFile.value && files.value.length > 0) {
    const currentIdx = selectedFile.value
      ? files.value.findIndex((f) => f.path === selectedFile.value?.path)
      : -1;

    if (e.key === "ArrowRight" || e.key === "ArrowDown") {
      e.preventDefault();
      const nextIdx = currentIdx < files.value.length - 1 ? currentIdx + 1 : 0;
      selectedFile.value = files.value[nextIdx];
    } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
      e.preventDefault();
      const prevIdx = currentIdx > 0 ? currentIdx - 1 : files.value.length - 1;
      selectedFile.value = files.value[prevIdx];
    }
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

function onBatchMove() {
  fileOpTargetFiles.value = [...selectedFilesList.value];
  fileOpMode.value = "move";
  fileOpModalOpen.value = true;
}

function onBatchCopy() {
  fileOpTargetFiles.value = [...selectedFilesList.value];
  fileOpMode.value = "copy";
  fileOpModalOpen.value = true;
}

function onBatchTrash() {
  fileOpTargetFiles.value = [...selectedFilesList.value];
  fileOpMode.value = "trash";
  fileOpModalOpen.value = true;
}

async function onFileOpCompleted() {
  selectedFilePaths.value.clear();
  await refreshCounts();
  await loadFiles();
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

function onApplyStatsSearch(query: string) {
  searchQuery.value = query;
  activeCriteria.value = {};
  void loadFiles();
}

function onFilterByModel(modelName: string) {
  activeCriteria.value = { ...activeCriteria.value, model_name: modelName };
  searchQuery.value = criteriaToQuery(activeCriteria.value);
  void loadFiles();
}

function onFilterByHash(modelHash: string) {
  activeCriteria.value = { ...activeCriteria.value, model_hash: modelHash };
  searchQuery.value = criteriaToQuery(activeCriteria.value);
  void loadFiles();
}
async function onDropMoveFiles(payload: { filePaths: string[]; folderId: number }) {
  try {
    await invoke("move_files", {
      filePaths: payload.filePaths,
      targetFolderId: payload.folderId,
    });
    await onFileOpCompleted();
  } catch (err) {
    error.value = String(err);
  }
}

async function onDropAddFilesToAlbum(payload: { fileIds: number[]; albumId: number }) {
  try {
    await invoke("add_files_to_album", {
      albumId: payload.albumId,
      fileIds: payload.fileIds,
    });
    await loadAlbumsAndTags();
  } catch (err) {
    error.value = String(err);
  }
}

async function onDropTagFiles(payload: { fileIds: number[]; tagId: number }) {
  try {
    await invoke("tag_files", {
      tagId: payload.tagId,
      fileIds: payload.fileIds,
    });
    await loadAlbumsAndTags();
    await loadFiles();
  } catch (err) {
    error.value = String(err);
  }
}

async function onDatabaseChanged() {
  await reloadFolders();
  await refreshCounts();
  await reloadFiltersMeta();
  await loadAlbumsAndTags();
  await loadFiles();
}
</script>

<template>
  <main class="shell">
    <header class="header">
      <div class="header-main-row">
        <div class="header-titles">
          <h1>{{ t.app.title }}</h1>
          <p class="tagline">{{ t.app.tagline }}</p>
        </div>
        <LanguageSelector />
      </div>
      <p v-if="info" class="meta">
        {{ t.app.version }}{{ info.app_version }} · {{ t.app.schema }}{{ info.schema_version }}
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
      @open-prompt-stats="promptStatsModalOpen = true"
      @move-files-to-folder="onDropMoveFiles"
      @add-files-to-album="onDropAddFilesToAlbum"
      @tag-files="onDropTagFiles"
    />

    <section class="files-view-section">
      <div class="files-view-header">
        <div class="header-left">
          <h2>
            {{ t.view.files }}
            <span v-if="files.length" class="count-badge">({{ files.length }})</span>
          </h2>
          <span class="folder-badge" :class="{ 'all-badge': activeTarget.type === 'all' }">
            {{ targetTitle }}
          </span>
          <span v-if="selectedFile" class="selection-pill" :title="selectedFile.path">
            {{ selectedFile.path.split(/[\\/]/).pop() }}
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
              :title="t.view.grid"
            >
              ⊞ {{ t.view.grid }}
            </button>
            <button
              type="button"
              class="toggle-btn"
              :class="{ active: viewMode === 'table' }"
              @click="viewMode = 'table'"
              :title="t.view.table"
            >
              ☰ {{ t.view.table }}
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
          :title="t.search.filters"
          @click="filterDrawerOpen = true"
        >
          <span>⚡ {{ t.search.filters }}</span>
          <span v-if="activeFilterCount > 0" class="filter-count-badge">
            {{ activeFilterCount }}
          </span>
        </button>
        <button
          type="button"
          class="stats-toggle-btn"
          :title="t.search.insights"
          @click="promptStatsModalOpen = true"
        >
          <span>📊 {{ t.search.insights }}</span>
        </button>
        <button
          type="button"
          class="models-toggle-btn"
          :title="t.search.models"
          @click="modelManagerModalOpen = true"
        >
          <span>🧠 {{ t.search.models }}</span>
        </button>
        <button
          type="button"
          class="models-toggle-btn"
          :title="t.search.database"
          @click="dbManagerModalOpen = true"
        >
          <span>🗄️ {{ t.search.database }}</span>
        </button>
        <button
          type="button"
          class="models-toggle-btn"
          :title="t.search.shortcuts"
          @click="shortcutsHelpModalOpen = true"
        >
          <span>⌨️ {{ t.search.shortcuts }}</span>
        </button>
      </div>

      <div v-if="searchQuery.trim()" class="search-status-bar">
        <span>
          {{ t.search.found }} <strong>{{ files.length }}</strong> {{ files.length === 1 ? t.search.image : t.search.images }}
        </span>
        <button type="button" class="reset-search-btn" @click="onClearSearch">
          ✕ {{ t.search.clearSearch }}
        </button>
      </div>

      <div
        v-if="files.length === 0 && searchQuery.trim() && !filesLoading"
        class="search-empty-state"
      >
        <span class="empty-icon">🔍</span>
        <p class="empty-title">{{ t.search.noMatchTitle }}</p>
        <p class="empty-hint">
          {{ t.search.noMatchHint }}
        </p>
        <button type="button" class="clear-filters-btn" @click="onClearSearch">
          {{ t.search.clearSearch }}
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
      @move-selected="onBatchMove"
      @copy-selected="onBatchCopy"
      @trash-selected="onBatchTrash"
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

    <!-- Prompt & Metadata Insights Modal -->
    <PromptStatsModal
      v-model:open="promptStatsModalOpen"
      @apply-search="onApplyStatsSearch"
    />

    <!-- Checkpoint Models & Hash Cache Modal -->
    <ModelManagerModal
      :show="modelManagerModalOpen"
      @close="modelManagerModalOpen = false"
      @filter-model="onFilterByModel"
      @filter-hash="onFilterByHash"
    />

    <!-- File Operations Modal (Move / Copy / Trash) -->
    <FileOperationModal
      :open="fileOpModalOpen"
      :mode="fileOpMode"
      :files="fileOpTargetFiles"
      :folders="folders"
      @close="fileOpModalOpen = false"
      @completed="onFileOpCompleted"
    />

    <!-- Database & Storage Maintenance Modal -->
    <DatabaseManagerModal
      :show="dbManagerModalOpen"
      @close="dbManagerModalOpen = false"
      @database-changed="onDatabaseChanged"
    />

    <!-- Keyboard Shortcuts Guide Modal -->
    <ShortcutsHelpModal
      :show="shortcutsHelpModalOpen"
      @close="shortcutsHelpModalOpen = false"
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

.header-main-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.header-titles {
  flex: 1;
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

.stats-toggle-btn,
.models-toggle-btn {
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

.stats-toggle-btn:hover,
.models-toggle-btn:hover {
  background: rgba(128, 128, 128, 0.15);
  border-color: rgba(128, 128, 128, 0.4);
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
