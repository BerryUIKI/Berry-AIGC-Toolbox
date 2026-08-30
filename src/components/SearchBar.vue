<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    loading?: boolean;
    resultCount?: number | null;
  }>(),
  {
    placeholder: "Search prompts, models, steps:>=20, rating:>=8... (Press / to focus)",
    loading: false,
    resultCount: null,
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "search", value: string): void;
  (e: "clear"): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const localQuery = ref(props.modelValue);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.modelValue,
  (val) => {
    if (val !== localQuery.value) {
      localQuery.value = val;
    }
  },
);

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  localQuery.value = target.value;
  emit("update:modelValue", target.value);

  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  debounceTimer = setTimeout(() => {
    emit("search", localQuery.value.trim());
  }, 250);
}

function onEnter() {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  emit("search", localQuery.value.trim());
}

function clear() {
  localQuery.value = "";
  emit("update:modelValue", "");
  emit("clear");
  inputRef.value?.focus();
}

// Global hotkey: press '/' or 'Cmd+F' / 'Ctrl+F' to focus search input
function handleGlobalKey(e: KeyboardEvent) {
  const tag = (document.activeElement?.tagName ?? "").toLowerCase();
  if (tag === "input" || tag === "textarea") return;

  if (e.key === "/" || ((e.ctrlKey || e.metaKey) && (e.key === "f" || e.key === "F"))) {
    e.preventDefault();
    inputRef.value?.focus();
    inputRef.value?.select();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleGlobalKey);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKey);
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
});

// Parse active chips for visual tags
interface Chip {
  raw: string;
  label: string;
}

const chips = computed<Chip[]>(() => {
  const q = localQuery.value.trim();
  if (!q) return [];
  const parts: Chip[] = [];

  // Match tokens like key:"value" or key:value
  const regex = /(\b[a-zA-Z_]+:(?:"[^"]*"|[^\s]+))/g;
  let match: RegExpExecArray | null;

  while ((match = regex.exec(q)) !== null) {
    const raw = match[1];
    const colonIdx = raw.indexOf(":");
    const key = raw.slice(0, colonIdx);
    let val = raw.slice(colonIdx + 1);
    if (val.startsWith('"') && val.endsWith('"')) {
      val = val.slice(1, -1);
    }
    parts.push({
      raw,
      label: `${key}: ${val}`,
    });
  }

  return parts;
});

function removeChip(chipRaw: string) {
  let q = localQuery.value;
  q = q.replace(chipRaw, "").replace(/\s{2,}/g, " ").trim();
  localQuery.value = q;
  emit("update:modelValue", q);
  emit("search", q);
}
</script>

<template>
  <div class="search-bar-wrapper">
    <div class="search-box">
      <span class="search-icon">🔍</span>
      <input
        ref="inputRef"
        type="text"
        class="search-input"
        :value="localQuery"
        :placeholder="placeholder"
        @input="onInput"
        @keydown.enter="onEnter"
        @keydown.esc="clear"
      />

      <span v-if="loading" class="spinner">⏳</span>

      <button
        v-if="localQuery"
        type="button"
        class="clear-btn"
        title="Clear search (Esc)"
        @click="clear"
      >
        ✕
      </button>
    </div>

    <!-- Active filter chips -->
    <div v-if="chips.length > 0" class="chips-container">
      <span
        v-for="chip in chips"
        :key="chip.raw"
        class="search-chip"
        :title="`Remove filter ${chip.label}`"
      >
        {{ chip.label }}
        <button
          type="button"
          class="chip-remove-btn"
          @click.stop="removeChip(chip.raw)"
        >
          ×
        </button>
      </span>
    </div>
  </div>
</template>

<style scoped>
.search-bar-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  width: 100%;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: rgba(128, 128, 128, 0.08);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  padding: 0.4rem 0.75rem;
  transition: all 0.15s ease;
}

.search-box:focus-within {
  border-color: #2f6fed;
  box-shadow: 0 0 0 2px rgba(47, 111, 237, 0.2);
  background: rgba(128, 128, 128, 0.12);
}

.search-icon {
  font-size: 0.95em;
  opacity: 0.7;
  user-select: none;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  font: inherit;
  font-size: 0.9em;
  color: inherit;
}

.search-input::placeholder {
  color: #888;
}

.spinner {
  font-size: 0.85em;
  animation: pulse 1s infinite alternate;
}

@keyframes pulse {
  from {
    opacity: 0.4;
  }
  to {
    opacity: 1;
  }
}

.clear-btn {
  background: transparent;
  border: none;
  color: #888;
  font-size: 0.9em;
  cursor: pointer;
  padding: 0 0.2rem;
  line-height: 1;
  transition: color 0.15s ease;
}

.clear-btn:hover {
  color: #d33;
}

.chips-container {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.search-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  font-size: 0.75em;
  font-weight: 500;
  padding: 0.15rem 0.5rem;
  border-radius: 999px;
  background: rgba(47, 111, 237, 0.12);
  color: #2f6fed;
  border: 1px solid rgba(47, 111, 237, 0.25);
}

@media (prefers-color-scheme: dark) {
  .search-chip {
    background: rgba(47, 111, 237, 0.2);
    color: #60a5fa;
  }
}

.chip-remove-btn {
  background: transparent;
  border: none;
  font-size: 1em;
  line-height: 1;
  color: inherit;
  cursor: pointer;
  padding: 0;
  opacity: 0.7;
}

.chip-remove-btn:hover {
  opacity: 1;
}
</style>
