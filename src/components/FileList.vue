<script setup lang="ts">
import type { ImageFile } from "../types";
import {
  assetUrl,
  formatBytes,
  formatDateTime,
  getFileName,
  normalizePath,
} from "../utils/image";

defineProps<{
  files: ImageFile[];
  loading: boolean;
  selectedFile?: ImageFile | null;
  selectedFilePaths?: Set<string>;
}>();

const emit = defineEmits<{
  (e: "select", file: ImageFile, event?: MouseEvent): void;
  (e: "activate", file: ImageFile): void;
  (e: "toggleSelect", file: ImageFile): void;
  (e: "toggleAll"): void;
}>();

/** First `max` chars, with an ellipsis when truncated. */
function snippet(text: string | null | undefined, max: number): string {
  if (!text) return "—";
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

/** Display width × height for a file's extracted metadata, or "—". */
function size(meta: ImageFile["metadata"]): string {
  if (!meta?.width || !meta?.height) return "—";
  return `${meta.width} × ${meta.height}`;
}
</script>

<template>
  <section class="files">
    <p v-if="loading" class="empty">Loading…</p>
    <p v-else-if="!files.length" class="empty">Select a folder to see its indexed files.</p>

    <div v-else class="scroll">
      <table class="table">
        <thead>
          <tr>
            <th class="th-checkbox">
              <input
                type="checkbox"
                :checked="files.length > 0 && selectedFilePaths?.size === files.length"
                title="Select/Deselect All"
                @click.stop="emit('toggleAll')"
              />
            </th>
            <th class="th-preview">Preview</th>
            <th>Name</th>
            <th>Type</th>
            <th>Size</th>
            <th>Modified</th>
            <th>Format</th>
            <th>Prompt</th>
            <th>Dimensions</th>
            <th>Model</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="file in files"
            :key="file.id ?? file.path"
            :class="{
              'row-selected': selectedFile?.path === file.path,
              'row-multi-selected': selectedFilePaths?.has(file.path),
            }"
            @click="emit('select', file, $event)"
            @dblclick="emit('activate', file)"
          >
            <td class="td-checkbox">
              <input
                type="checkbox"
                :checked="selectedFilePaths?.has(file.path)"
                @click.stop="emit('toggleSelect', file)"
              />
            </td>
            <td class="preview-cell">
              <img
                v-if="file.container !== 'mp4' && file.container !== 'txt'"
                :src="assetUrl(file.path)"
                :alt="getFileName(file.path)"
                class="thumb"
                loading="lazy"
              />
              <div v-else class="thumb-placeholder">
                {{ file.container.toUpperCase() }}
              </div>
            </td>
            <td class="name" :title="normalizePath(file.path)">{{ getFileName(file.path) }}</td>
            <td>{{ file.container }}</td>
            <td>{{ formatBytes(file.size_bytes) }}</td>
            <td class="date">{{ formatDateTime(file.modified_at) }}</td>
            <td>
              <span v-if="file.metadata" class="format">{{ file.metadata.format }}</span>
              <span v-else class="none">—</span>
            </td>
            <td class="prompt" :title="file.metadata?.prompt ?? ''">
              {{ snippet(file.metadata?.prompt, 80) }}
            </td>
            <td class="nowrap">{{ size(file.metadata) }}</td>
            <td class="model" :title="file.metadata?.model_name ?? ''">
              {{ snippet(file.metadata?.model_name, 32) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.empty {
  color: #888;
  font-size: 0.9em;
}

.scroll {
  overflow-x: auto;
}

.table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9em;
}

.table th,
.table td {
  text-align: left;
  padding: 0.4rem 0.6rem;
  border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  white-space: nowrap;
}

.table th {
  color: #888;
  font-weight: 600;
  font-size: 0.8em;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.th-preview {
  width: 44px;
}

.preview-cell {
  width: 44px;
  padding: 0.25rem 0.4rem !important;
}

.thumb {
  width: 38px;
  height: 38px;
  object-fit: cover;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.05);
  display: block;
}

.thumb-placeholder {
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7em;
  font-weight: 600;
  color: #999;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 4px;
}

.name {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  word-break: break-all;
  max-width: 22rem;
  overflow: hidden;
  text-overflow: ellipsis;
}

.date {
  font-size: 0.85em;
  color: #888;
}

.prompt {
  max-width: 24rem;
  overflow: hidden;
  text-overflow: ellipsis;
}

.model {
  max-width: 16rem;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 0.85em;
}

.format {
  display: inline-block;
  padding: 0.05rem 0.5rem;
  border-radius: 999px;
  font-size: 0.75em;
  font-weight: 600;
  background: rgba(47, 111, 237, 0.15);
  color: #2f6fed;
}

.none {
  color: #999;
}

tbody tr {
  cursor: pointer;
  transition: background-color 0.15s ease;
}

tbody tr:hover {
  background: rgba(0, 0, 0, 0.03);
}

@media (prefers-color-scheme: dark) {
  tbody tr:hover {
    background: rgba(255, 255, 255, 0.04);
  }
}

tbody tr.row-selected {
  background: rgba(47, 111, 237, 0.15) !important;
}

tbody tr.row-multi-selected {
  background: rgba(47, 111, 237, 0.08);
}

.th-checkbox,
.td-checkbox {
  width: 32px;
  text-align: center !important;
  padding: 0.4rem 0.2rem !important;
}

.th-checkbox input,
.td-checkbox input {
  cursor: pointer;
  accent-color: #2f6fed;
}
</style>
