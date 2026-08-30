<script setup lang="ts">
import type { FileSortField, SortDirection } from "../types";

const props = defineProps<{
  sortField: FileSortField;
  sortDirection: SortDirection;
}>();

const emit = defineEmits<{
  (e: "update:sortField", value: FileSortField): void;
  (e: "update:sortDirection", value: SortDirection): void;
  (e: "change"): void;
}>();

function onFieldChange(e: Event) {
  const target = e.target as HTMLSelectElement;
  emit("update:sortField", target.value as FileSortField);
  emit("change");
}

function toggleDirection() {
  const next = props.sortDirection === "desc" ? "asc" : "desc";
  emit("update:sortDirection", next);
  emit("change");
}
</script>

<template>
  <div class="sort-bar">
    <label class="sort-label">
      <span class="label-text">Sort:</span>
      <select :value="sortField" class="sort-select" @change="onFieldChange">
        <option value="modified_at">Date</option>
        <option value="path">Name</option>
        <option value="size_bytes">Size</option>
        <option value="rating">Rating</option>
        <option value="aesthetic_score">Aesthetic</option>
      </select>
    </label>

    <button
      type="button"
      class="direction-btn"
      :title="sortDirection === 'desc' ? 'Descending (High to Low)' : 'Ascending (Low to High)'"
      @click="toggleDirection"
    >
      {{ sortDirection === "desc" ? "↓ Desc" : "↑ Asc" }}
    </button>
  </div>
</template>

<style scoped>
.sort-bar {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}

.sort-label {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.85em;
  color: #666;
}

@media (prefers-color-scheme: dark) {
  .sort-label {
    color: #aaa;
  }
}

.sort-select {
  font: inherit;
  font-size: 0.85em;
  padding: 0.3rem 0.6rem;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.06);
  color: inherit;
  cursor: pointer;
  outline: none;
}

.sort-select:focus {
  border-color: #2f6fed;
}

.direction-btn {
  font: inherit;
  font-size: 0.85em;
  padding: 0.3rem 0.65rem;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.06);
  color: inherit;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  transition: all 0.15s ease;
}

.direction-btn:hover {
  background: rgba(128, 128, 128, 0.12);
  border-color: rgba(128, 128, 128, 0.4);
}
</style>
