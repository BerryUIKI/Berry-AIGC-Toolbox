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
</script>

<template>
  <section class="files">
    <h2>Files</h2>
    <p v-if="loading" class="empty">Loading…</p>
    <p v-else-if="!files.length" class="empty">Select a folder to see its indexed files.</p>

    <table v-else class="table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Type</th>
          <th>Size</th>
          <th>Modified</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="file in files" :key="file.id ?? file.path">
          <td class="name" :title="displayPath(file.path)">{{ fileName(file.path) }}</td>
          <td>{{ file.container }}</td>
          <td>{{ formatSize(file.size_bytes) }}</td>
          <td>{{ formatDate(file.modified_at) }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
.empty {
  color: #888;
  font-size: 0.9em;
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
}

.container {
  white-space: nowrap;
}
</style>
