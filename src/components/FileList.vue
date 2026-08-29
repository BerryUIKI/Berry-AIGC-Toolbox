<script setup lang="ts">
import type { ImageFile } from "../types";

defineProps<{
  files: ImageFile[];
  loading: boolean;
}>();

/** Strip the Windows `\\?\` verbatim prefix, if present. */
function displayPath(path: string): string {
  return path.replace(/^\\\\\?\\/, "");
}

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleString();
}

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
    <h2>Files</h2>
    <p v-if="loading" class="empty">Loading…</p>
    <p v-else-if="!files.length" class="empty">Select a folder to see its indexed files.</p>

    <div v-else class="scroll">
      <table class="table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Type</th>
            <th>Size</th>
            <th>Modified</th>
            <th>Format</th>
            <th>Prompt</th>
            <th>Size</th>
            <th>Model</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="file in files" :key="file.id ?? file.path">
            <td class="name" :title="displayPath(file.path)">{{ fileName(file.path) }}</td>
            <td>{{ file.container }}</td>
            <td>{{ formatSize(file.size_bytes) }}</td>
            <td class="date">{{ formatDate(file.modified_at) }}</td>
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
</style>
