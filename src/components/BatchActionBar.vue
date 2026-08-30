<script setup lang="ts">
import { ref } from "vue";
import type { ImageFile } from "../types";

const props = defineProps<{
  selectedFiles: ImageFile[];
  totalCount: number;
}>();

const emit = defineEmits<{
  (e: "selectAll"): void;
  (e: "clearSelection"): void;
  (e: "rateSelected", rating: number | null): void;
}>();

const ratingMenuOpen = ref(false);
const copiedPaths = ref(false);
const copiedPrompts = ref(false);

async function copyPaths() {
  const text = props.selectedFiles.map((f) => f.path).join("\n");
  await navigator.clipboard.writeText(text);
  copiedPaths.value = true;
  setTimeout(() => {
    copiedPaths.value = false;
  }, 1800);
}

async function copyPrompts() {
  const text = props.selectedFiles
    .map((f) => f.metadata?.prompt?.trim())
    .filter(Boolean)
    .join("\n\n---\n\n");
  await navigator.clipboard.writeText(text);
  copiedPrompts.value = true;
  setTimeout(() => {
    copiedPrompts.value = false;
  }, 1800);
}

function onSetRating(rating: number | null) {
  emit("rateSelected", rating);
  ratingMenuOpen.value = false;
}
</script>

<template>
  <div v-if="selectedFiles.length > 0" class="batch-bar-container">
    <div class="batch-bar" role="toolbar" aria-label="Batch Actions">
      <div class="batch-info">
        <span class="batch-badge">
          {{ selectedFiles.length }} of {{ totalCount }} selected
        </span>
        <button
          v-if="selectedFiles.length < totalCount"
          type="button"
          class="btn-text"
          @click="emit('selectAll')"
        >
          Select All
        </button>
        <button
          type="button"
          class="btn-text"
          @click="emit('clearSelection')"
        >
          Deselect
        </button>
      </div>

      <div class="batch-actions">
        <!-- Rating Dropdown -->
        <div class="rating-dropdown-wrapper">
          <button
            type="button"
            class="action-btn"
            :class="{ active: ratingMenuOpen }"
            @click="ratingMenuOpen = !ratingMenuOpen"
          >
            ★ Set Rating
          </button>
          <div v-if="ratingMenuOpen" class="rating-menu">
            <button
              v-for="r in 10"
              :key="r"
              type="button"
              class="rating-opt"
              @click="onSetRating(r)"
            >
              ★ {{ r }}
            </button>
            <div class="menu-divider" />
            <button
              type="button"
              class="rating-opt clear-opt"
              @click="onSetRating(null)"
            >
              Clear Rating
            </button>
          </div>
        </div>

        <!-- Copy Paths -->
        <button
          type="button"
          class="action-btn"
          title="Copy file paths to clipboard"
          @click="copyPaths"
        >
          {{ copiedPaths ? "✓ Paths Copied!" : "📋 Copy Paths" }}
        </button>

        <!-- Copy Prompts -->
        <button
          type="button"
          class="action-btn"
          title="Copy prompts to clipboard"
          @click="copyPrompts"
        >
          {{ copiedPrompts ? "✓ Prompts Copied!" : "💬 Copy Prompts" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.batch-bar-container {
  position: fixed;
  bottom: 1.5rem;
  left: 50%;
  transform: translateX(-50%);
  z-index: 100;
  animation: slideUp 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideUp {
  from {
    transform: translate(-50%, 20px);
    opacity: 0;
  }
  to {
    transform: translate(-50%, 0);
    opacity: 1;
  }
}

.batch-bar {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 0.6rem 1.2rem;
  background: #1e1e1e;
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 999px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

@media (prefers-color-scheme: light) {
  .batch-bar {
    background: #ffffff;
    color: #1a1a1a;
    border: 1px solid rgba(0, 0, 0, 0.12);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  }
}

.batch-info {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.batch-badge {
  font-size: 0.85em;
  font-weight: 600;
  background: #2f6fed;
  color: #fff;
  padding: 0.2rem 0.6rem;
  border-radius: 999px;
  white-space: nowrap;
}

.btn-text {
  background: transparent;
  border: none;
  color: #aaa;
  font: inherit;
  font-size: 0.82em;
  cursor: pointer;
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
  transition: all 0.15s ease;
  white-space: nowrap;
}

@media (prefers-color-scheme: light) {
  .btn-text {
    color: #666;
  }
}

.btn-text:hover {
  color: #2f6fed;
  background: rgba(47, 111, 237, 0.1);
}

.batch-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.action-btn {
  background: rgba(128, 128, 128, 0.2);
  border: 1px solid rgba(128, 128, 128, 0.3);
  color: inherit;
  font: inherit;
  font-size: 0.84em;
  font-weight: 500;
  padding: 0.35rem 0.75rem;
  border-radius: 999px;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.action-btn:hover,
.action-btn.active {
  background: #2f6fed;
  border-color: #2f6fed;
  color: #fff;
}

.rating-dropdown-wrapper {
  position: relative;
}

.rating-menu {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  background: #252525;
  color: #eee;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  padding: 0.4rem;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.3);
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 0.25rem;
  width: 220px;
  z-index: 101;
}

@media (prefers-color-scheme: light) {
  .rating-menu {
    background: #ffffff;
    color: #222;
    border: 1px solid rgba(0, 0, 0, 0.15);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
  }
}

.rating-opt {
  background: transparent;
  border: none;
  color: inherit;
  padding: 0.3rem 0.2rem;
  border-radius: 4px;
  font-size: 0.82em;
  cursor: pointer;
  text-align: center;
  transition: background 0.1s ease;
}

.rating-opt:hover {
  background: rgba(47, 111, 237, 0.25);
}

.menu-divider {
  grid-column: 1 / -1;
  height: 1px;
  background: rgba(128, 128, 128, 0.2);
  margin: 0.2rem 0;
}

.clear-opt {
  grid-column: 1 / -1;
  color: #f87171;
}

.clear-opt:hover {
  background: rgba(239, 68, 68, 0.15);
}
</style>
