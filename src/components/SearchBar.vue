<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { t } from "../i18n";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    loading?: boolean;
    resultCount?: number | null;
  }>(),
  {
    placeholder: "",
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
</script>

<template>
  <div class="search-bar-eagle">
    <div class="search-box">
      <span class="search-icon">
        <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor">
          <path d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0z"/>
        </svg>
      </span>

      <input
        ref="inputRef"
        :value="localQuery"
        type="text"
        class="search-input"
        :placeholder="placeholder || t.search.placeholder"
        @input="onInput"
        @keydown.enter="onEnter"
      />

      <span v-if="loading" class="spinner" :title="t.view.loading">⏳</span>

      <button
        v-if="localQuery"
        type="button"
        class="clear-btn"
        :title="t.search.clearSearch"
        @click="clear"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.search-bar-eagle {
  display: flex;
  align-items: center;
  width: 100%;
  min-width: 120px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #202024;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0 8px;
  height: 30px;
  width: 100%;
  transition: all 0.15s ease;
}

.search-box:focus-within {
  border-color: rgba(168, 85, 247, 0.5);
  box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.15);
  background: #242428;
}

.search-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #71717a;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  outline: none;
  font: inherit;
  font-size: 0.78rem;
  color: #f1f5f9;
}

.search-input::placeholder {
  color: #52525b;
}

.spinner {
  font-size: 0.75rem;
  flex-shrink: 0;
}

.clear-btn {
  background: transparent;
  border: none;
  color: #71717a;
  font-size: 0.75rem;
  cursor: pointer;
  padding: 2px 4px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  flex-shrink: 0;
  transition: all 0.12s ease;
}

.clear-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #f87171;
}
</style>
