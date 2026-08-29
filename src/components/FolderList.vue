<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Folder, ScanProgress, ScanStats } from "../types";

const props = defineProps<{
  folders: Folder[];
  /** Latest `scan-progress` event, forwarded from the App shell listener. */
  progress: ScanProgress | null;
}>();

const emit = defineEmits<{
  removed: [folderId: number];
  selected: [folder: Folder];
  scanned: [folderId: number];
}>();

const selectedId = ref<number | null>(null);
/** The folder currently being scanned, and whether it is a full scan or a
 * metadata rebuild. Only one scan runs at a time. */
const running = ref<{ id: number; action: "scan" | "rebuild" } | null>(null);
const stats = ref<ScanStats | null>(null);
const error = ref("");

/** Strip the Windows `\\?\` verbatim prefix, if present. */
function displayPath(path: string): string {
  return path.replace(/^\\\\\?\\/, "");
}

function isBusy(id: number): boolean {
  return running.value?.id === id;
}

async function scan(folder: Folder, action: "scan" | "rebuild" = "scan") {
  error.value = "";
  stats.value = null;
  running.value = { id: folder.id, action };
  try {
    stats.value = await invoke<ScanStats>(
      action === "rebuild" ? "rebuild_metadata" : "scan_folder",
      { folderId: folder.id },
    );
    emit("scanned", folder.id);
  } catch (e) {
    error.value = String(e);
  } finally {
    running.value = null;
  }
}

async function remove(folder: Folder) {
  error.value = "";
  try {
    await invoke("remove_folder", { folderId: folder.id });
    if (selectedId.value === folder.id) selectedId.value = null;
    emit("removed", folder.id);
  } catch (e) {
    error.value = String(e);
  }
}

function select(folder: Folder) {
  selectedId.value = folder.id;
  emit("selected", folder);
}

const progressPercent = computed(() => {
  if (!props.progress || props.progress.found === 0) return 0;
  return Math.round((props.progress.scanned / props.progress.found) * 100);
});
</script>

<template>
  <section class="folders">
    <h2>Folders</h2>
    <p v-if="!folders.length" class="empty">No folders added yet.</p>

    <ul class="list">
      <li
        v-for="folder in folders"
        :key="folder.id"
        :class="['row', { active: folder.id === selectedId }]"
        @click="select(folder)"
      >
        <div class="path">
          <span class="name">{{ displayPath(folder.path) }}</span>
          <button
            class="ghost scan"
            :disabled="isBusy(folder.id)"
            @click.stop="scan(folder, 'scan')"
          >
            {{ running?.id === folder.id && running?.action === "scan" ? "Scanning…" : "Scan" }}
          </button>
          <button
            class="ghost rebuild"
            :disabled="isBusy(folder.id)"
            @click.stop="scan(folder, 'rebuild')"
          >
            {{ running?.id === folder.id && running?.action === "rebuild" ? "Rebuilding…" : "Rebuild" }}
          </button>
          <button class="ghost remove" @click.stop="remove(folder)">×</button>
        </div>

        <div v-if="isBusy(folder.id) && props.progress && props.progress.found > 0" class="bar">
          <div class="fill" :style="{ width: progressPercent + '%' }"></div>
          <span class="bar-label">
            {{ props.progress.scanned }} / {{ props.progress.found }}
            <template v-if="props.progress.current"> · {{ props.progress.current }}</template>
          </span>
        </div>

        <p v-if="folder.id === running?.id && stats" class="stats">
          +{{ stats.added }} added · {{ stats.updated }} updated ·
          {{ stats.unchanged }} unchanged · {{ stats.removed }} removed ·
          {{ stats.failed }} failed · {{ stats.duration_ms }} ms
        </p>
      </li>
    </ul>

    <p v-if="error" class="error">{{ error }}</p>
  </section>
</template>

<style scoped>
.folders {
  margin-bottom: 2rem;
}

.empty {
  color: #888;
  font-size: 0.9em;
}

.list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.row {
  padding: 0.5rem 0.75rem;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  margin-bottom: 0.5rem;
  cursor: pointer;
}

.row.active {
  border-color: #2f6fed;
  background: rgba(47, 111, 237, 0.08);
}

.path {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.name {
  flex: 1;
  word-break: break-all;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 0.9em;
}

button.ghost {
  font: inherit;
  font-size: 0.85em;
  border: 1px solid rgba(128, 128, 128, 0.35);
  background: transparent;
  border-radius: 6px;
  padding: 0.15rem 0.6rem;
  cursor: pointer;
}

button.ghost:disabled {
  opacity: 0.6;
  cursor: default;
}

button.remove {
  color: #d33;
}

button.rebuild {
  color: #2f6fed;
}

.bar {
  position: relative;
  margin-top: 0.5rem;
  height: 1.1rem;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.15);
}

.fill {
  height: 100%;
  background: #2f6fed;
  transition: width 0.15s ease-out;
}

.bar-label {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  padding: 0 0.5rem;
  font-size: 0.75em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stats {
  margin: 0.4rem 0 0;
  font-size: 0.8em;
  color: #666;
}

.error {
  color: #d33;
  font-size: 0.85em;
}
</style>
