<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { t } from "../i18n";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    isSemantic?: boolean;
    placeholder?: string;
    loading?: boolean;
    resultCount?: number | null;
    models?: string[];
    tags?: string[];
  }>(),
  {
    isSemantic: false,
    placeholder: "",
    loading: false,
    resultCount: null,
    models: () => [],
    tags: () => [],
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "update:isSemantic", value: boolean): void;
  (e: "search", value: string): void;
  (e: "clear"): void;
  (e: "open-clip-manager"): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const localQuery = ref(props.modelValue);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const showSuggestions = ref(false);
const activeSuggestionIndex = ref(0);

interface SuggestionItem {
  prefix: string;
  value: string;
  display: string;
  type: "model" | "tag" | "filter";
}

const suggestions = ref<SuggestionItem[]>([]);
const syntaxWarning = ref<string | null>(null);

function checkSyntax(text: string) {
  if (props.isSemantic || !text.trim()) {
    syntaxWarning.value = null;
    return;
  }
  const tokens = text.trim().split(/\s+/);
  for (const tok of tokens) {
    if (tok.includes(":")) {
      const [key, val] = tok.split(":", 2);
      const k = key.toLowerCase();
      if (["steps", "rating", "duration", "fps"].includes(k) && val) {
        const cleanVal = val.replace(/^[<>=]+/, "");
        if (cleanVal && !cleanVal.includes("..") && isNaN(Number(cleanVal))) {
          syntaxWarning.value = `Invalid numeric filter: "${tok}"`;
          return;
        }
      }
      if (["cfg", "aesthetic"].includes(k) && val) {
        const cleanVal = val.replace(/^[<>=]+/, "");
        if (cleanVal && !cleanVal.includes("..") && isNaN(parseFloat(cleanVal))) {
          syntaxWarning.value = `Invalid numeric filter: "${tok}"`;
          return;
        }
      }
    }
  }
  syntaxWarning.value = null;
}

function computeSuggestions(cursorPos: number) {
  if (props.isSemantic) {
    suggestions.value = [];
    showSuggestions.value = false;
    return;
  }
  const textBeforeCursor = localQuery.value.slice(0, cursorPos);
  const words = textBeforeCursor.split(/\s+/);
  const currentWord = words[words.length - 1] || "";

  if (currentWord.startsWith("model:")) {
    const query = currentWord.slice(6).toLowerCase().replace(/^"/, "");
    const matched = (props.models || [])
      .filter((m) => m.toLowerCase().includes(query))
      .slice(0, 8)
      .map((m) => ({
        prefix: "model:",
        value: m.includes(" ") ? `"${m}"` : m,
        display: m,
        type: "model" as const,
      }));
    suggestions.value = matched;
    showSuggestions.value = matched.length > 0;
    activeSuggestionIndex.value = 0;
    return;
  }

  if (currentWord.startsWith("tag:")) {
    const query = currentWord.slice(4).toLowerCase().replace(/^"/, "");
    const matched = (props.tags || [])
      .filter((t) => t.toLowerCase().includes(query))
      .slice(0, 8)
      .map((t) => ({
        prefix: "tag:",
        value: t.includes(" ") ? `"${t}"` : t,
        display: t,
        type: "tag" as const,
      }));
    suggestions.value = matched;
    showSuggestions.value = matched.length > 0;
    activeSuggestionIndex.value = 0;
    return;
  }

  showSuggestions.value = false;
  suggestions.value = [];
}

function applySuggestion(item: SuggestionItem) {
  const input = inputRef.value;
  const cursorPos = input ? (input.selectionStart ?? localQuery.value.length) : localQuery.value.length;
  const textBefore = localQuery.value.slice(0, cursorPos);
  const textAfter = localQuery.value.slice(cursorPos);
  const words = textBefore.split(/\s+/);
  words[words.length - 1] = `${item.prefix}${item.value}`;
  const newQuery = `${words.join(" ")} ${textAfter.trimStart()}`;
  localQuery.value = newQuery;
  showSuggestions.value = false;
  emit("update:modelValue", newQuery);
  emit("search", newQuery.trim());
  if (input) {
    input.focus();
  }
}

watch(
  () => props.modelValue,
  (val) => {
    if (val !== localQuery.value) {
      localQuery.value = val;
      checkSyntax(val);
    }
  },
);

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  localQuery.value = target.value;
  emit("update:modelValue", target.value);
  checkSyntax(target.value);
  computeSuggestions(target.selectionStart ?? target.value.length);

  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  debounceTimer = setTimeout(() => {
    emit("search", localQuery.value.trim());
  }, 250);
}

function onKeyDown(e: KeyboardEvent) {
  if (showSuggestions.value && suggestions.value.length > 0) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      activeSuggestionIndex.value = (activeSuggestionIndex.value + 1) % suggestions.value.length;
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      activeSuggestionIndex.value = (activeSuggestionIndex.value - 1 + suggestions.value.length) % suggestions.value.length;
      return;
    }
    if (e.key === "Enter" || e.key === "Tab") {
      e.preventDefault();
      const selected = suggestions.value[activeSuggestionIndex.value];
      if (selected) {
        applySuggestion(selected);
        return;
      }
    }
    if (e.key === "Escape") {
      showSuggestions.value = false;
      return;
    }
  }

  if (e.key === "Enter") {
    onEnter();
  }
}

function onEnter() {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  showSuggestions.value = false;
  emit("search", localQuery.value.trim());
}

function clear() {
  localQuery.value = "";
  showSuggestions.value = false;
  syntaxWarning.value = null;
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
function toggleMode() {
  const next = !props.isSemantic;
  showSuggestions.value = false;
  emit("update:isSemantic", next);
  emit("search", localQuery.value.trim());
}
</script>

<template>
  <div class="search-bar-eagle">
    <div :class="['search-box', { 'semantic-mode': isSemantic, 'has-warning': !!syntaxWarning }]">
      <!-- Semantic Mode Toggle Button -->
      <button
        type="button"
        class="mode-toggle-btn"
        :class="{ active: isSemantic }"
        :title="isSemantic ? t.search.semanticSearch : t.search.syntaxSearch"
        @click="toggleMode"
      >
        <span class="mode-icon">{{ isSemantic ? '🧠' : '🔍' }}</span>
      </button>

      <input
        ref="inputRef"
        :value="localQuery"
        type="text"
        class="search-input"
        :placeholder="isSemantic ? t.search.semanticPlaceholder : (placeholder || t.search.placeholder)"
        @input="onInput"
        @keydown="onKeyDown"
      />

      <!-- CLIP Manager Trigger Button in Semantic Mode -->
      <button
        v-if="isSemantic"
        type="button"
        class="clip-index-btn"
        :title="t.search.clipManager"
        @click="emit('open-clip-manager')"
      >
        ⚙️
      </button>

      <span v-if="loading" class="spinner" :title="t.view.loading">⏳</span>

      <span v-if="syntaxWarning" class="syntax-warning-badge" :title="syntaxWarning">⚠️</span>

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

    <!-- Autocomplete suggestions dropdown -->
    <ul v-if="showSuggestions && suggestions.length > 0" class="suggestions-list" role="listbox">
      <li
        v-for="(item, idx) in suggestions"
        :key="item.value"
        :class="['suggestion-item', { active: idx === activeSuggestionIndex }]"
        role="option"
        @mousedown.prevent="applySuggestion(item)"
      >
        <span class="suggestion-tag">{{ item.type }}:</span>
        <span class="suggestion-val">{{ item.display }}</span>
      </li>
    </ul>
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
  background: var(--color-bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 0 8px;
  height: 30px;
  width: 100%;
  transition: all 0.15s ease;
}

.search-box:focus-within {
  border-color: rgba(168, 85, 247, 0.5);
  box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.15);
  background: var(--color-bg-tertiary);
}

.search-box.semantic-mode {
  border-color: rgba(139, 92, 246, 0.4);
  background: var(--color-bg-secondary);
}

.search-box.semantic-mode:focus-within {
  border-color: #8b5cf6;
  box-shadow: 0 0 0 2px rgba(139, 92, 246, 0.25);
  background: var(--color-bg-tertiary);
}

.mode-toggle-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.82rem;
  transition: all 0.15s ease;
  line-height: 1;
}

.mode-toggle-btn:hover {
  background: var(--color-bg-hover);
  transform: scale(1.05);
}

.mode-toggle-btn.active {
  background: rgba(139, 92, 246, 0.25);
}

.clip-index-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  color: #a855f7;
  transition: all 0.15s ease;
  line-height: 1;
}

.clip-index-btn:hover {
  background: rgba(168, 85, 247, 0.2);
  transform: scale(1.08);
}

.search-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
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
  color: var(--color-text-primary);
}

.search-input::placeholder {
  color: var(--color-text-muted);
}

.spinner {
  font-size: 0.75rem;
  flex-shrink: 0;
}

.clear-btn {
  background: transparent;
  border: none;
  color: var(--color-text-muted);
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
  background: var(--color-bg-hover);
  color: var(--color-danger);
}

.search-bar-eagle {
  position: relative;
}

.search-box.has-warning {
  border-color: #f59e0b;
}

.syntax-warning-badge {
  font-size: 0.72rem;
  cursor: help;
  user-select: none;
}

.suggestions-list {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  list-style: none;
  padding: 4px 0;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
  max-height: 200px;
  overflow-y: auto;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  font-size: 0.78rem;
  cursor: pointer;
  color: var(--color-text-primary);
  transition: background 0.12s ease;
}

.suggestion-item:hover,
.suggestion-item.active {
  background: var(--color-bg-hover);
}

.suggestion-tag {
  color: #a855f7;
  font-weight: 600;
  font-size: 0.72rem;
}

.suggestion-val {
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
