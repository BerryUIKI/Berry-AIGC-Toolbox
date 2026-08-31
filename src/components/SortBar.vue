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
  <div class="sort-bar-eagle">
    <select :value="sortField" class="sort-select" title="排序方式" @change="onFieldChange">
      <option value="modified_at">时间</option>
      <option value="path">名称</option>
      <option value="size_bytes">大小</option>
      <option value="rating">评分</option>
      <option value="aesthetic_score">美学评分</option>
    </select>

    <button
      type="button"
      class="direction-btn"
      :title="sortDirection === 'desc' ? '降序 (高到低)' : '升序 (低到高)'"
      @click="toggleDirection"
    >
      {{ sortDirection === "desc" ? "↓" : "↑" }}
    </button>
  </div>
</template>

<style scoped>
.sort-bar-eagle {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
}

.sort-select {
  font: inherit;
  font-size: 0.76rem;
  height: 28px;
  padding: 0 6px;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: #202024;
  color: #d4d4d8;
  cursor: pointer;
  outline: none;
  transition: all 0.12s ease;
}

.sort-select:hover {
  background: #27272a;
  border-color: rgba(255, 255, 255, 0.15);
}

.sort-select:focus {
  border-color: rgba(168, 85, 247, 0.5);
}

.direction-btn {
  font: inherit;
  font-size: 0.8rem;
  height: 28px;
  width: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: #202024;
  color: #a1a1aa;
  cursor: pointer;
  transition: all 0.12s ease;
}

.direction-btn:hover {
  background: #27272a;
  color: #f4f4f5;
}
</style>
