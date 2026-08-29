<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppInfo, Folder, ImageFile, ScanProgress } from "./types";
import FolderPicker from "./components/FolderPicker.vue";
import FolderList from "./components/FolderList.vue";
import FileList from "./components/FileList.vue";

const info = ref<AppInfo | null>(null);
const folders = ref<Folder[]>([]);
const files = ref<ImageFile[]>([]);
const filesLoading = ref(false);
const selectedFolder = ref<Folder | null>(null);
const progress = ref<ScanProgress | null>(null);
const error = ref("");

let unlisten: UnlistenFn | null = null;

onMounted(async () => {
  try {
    info.value = await invoke<AppInfo>("get_app_info");
    await reloadFolders();
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

function onFolderAdded(folder: Folder) {
  void reloadFolders();
  selectedFolder.value = folder;
  void loadFiles(folder);
}

function onFolderRemoved(folderId: number) {
  folders.value = folders.value.filter((f) => f.id !== folderId);
  if (selectedFolder.value?.id === folderId) {
    selectedFolder.value = null;
    files.value = [];
  }
}

async function onFolderSelected(folder: Folder) {
  selectedFolder.value = folder;
  await loadFiles(folder);
}

async function onFolderScanned(folderId: number) {
  // Refresh the file list if the scan finished on the currently selected folder.
  if (selectedFolder.value?.id === folderId) {
    await loadFiles(selectedFolder.value);
  }
}

async function loadFiles(folder: Folder) {
  filesLoading.value = true;
  try {
    files.value = await invoke<ImageFile[]>("list_files", { folderId: folder.id });
  } catch (e) {
    error.value = String(e);
  } finally {
    filesLoading.value = false;
  }
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
      :progress="progress"
      @removed="onFolderRemoved"
      @selected="onFolderSelected"
      @scanned="onFolderScanned"
    />

    <FileList :files="files" :loading="filesLoading" />
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
  max-width: 56rem;
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
</style>
