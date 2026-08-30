<script setup lang="ts">
import { ref, watch } from "vue";
import type { SearchCriteria } from "../types";

const props = defineProps<{
  open: boolean;
  models: string[];
  samplers: string[];
  initialCriteria?: SearchCriteria;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "apply", criteria: SearchCriteria): void;
  (e: "reset"): void;
}>();

// Form state
const prompt = ref("");
const negativePrompt = ref("");
const selectedModel = ref("");
const selectedSampler = ref("");
const minSteps = ref<number | "">("");
const maxSteps = ref<number | "">("");
const minCfg = ref<number | "">("");
const maxCfg = ref<number | "">("");
const minRating = ref<number | "">("");
const maxRating = ref<number | "">("");
const minAesthetic = ref<number | "">("");
const isFavorite = ref<boolean | null>(null);
const isNsfw = ref<boolean | null>(null);

// Sync form state when drawer opens or initialCriteria changes
watch(
  () => [props.open, props.initialCriteria],
  () => {
    if (props.open) {
      const c = props.initialCriteria ?? {};
      prompt.value = c.prompt ?? "";
      negativePrompt.value = c.negative_prompt ?? "";
      selectedModel.value = c.model_name ?? "";
      selectedSampler.value = c.sampler ?? "";
      minSteps.value = c.min_steps ?? "";
      maxSteps.value = c.max_steps ?? "";
      minCfg.value = c.min_cfg ?? "";
      maxCfg.value = c.max_cfg ?? "";
      minRating.value = c.min_rating ?? "";
      maxRating.value = c.max_rating ?? "";
      minAesthetic.value = c.min_aesthetic ?? "";
      isFavorite.value = c.is_favorite ?? null;
      isNsfw.value = c.is_nsfw ?? null;
    }
  },
  { immediate: true },
);

function close() {
  emit("update:open", false);
}

function handleBackdrop(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    close();
  }
}

function apply() {
  const criteria: SearchCriteria = {
    prompt: prompt.value.trim() || null,
    negative_prompt: negativePrompt.value.trim() || null,
    model_name: selectedModel.value || null,
    sampler: selectedSampler.value || null,
    min_steps: minSteps.value !== "" ? Number(minSteps.value) : null,
    max_steps: maxSteps.value !== "" ? Number(maxSteps.value) : null,
    min_cfg: minCfg.value !== "" ? Number(minCfg.value) : null,
    max_cfg: maxCfg.value !== "" ? Number(maxCfg.value) : null,
    min_rating: minRating.value !== "" ? Number(minRating.value) : null,
    max_rating: maxRating.value !== "" ? Number(maxRating.value) : null,
    min_aesthetic: minAesthetic.value !== "" ? Number(minAesthetic.value) : null,
    is_favorite: isFavorite.value,
    is_nsfw: isNsfw.value,
  };
  emit("apply", criteria);
  close();
}

function reset() {
  prompt.value = "";
  negativePrompt.value = "";
  selectedModel.value = "";
  selectedSampler.value = "";
  minSteps.value = "";
  maxSteps.value = "";
  minCfg.value = "";
  maxCfg.value = "";
  minRating.value = "";
  maxRating.value = "";
  minAesthetic.value = "";
  isFavorite.value = null;
  isNsfw.value = null;
  emit("reset");
  close();
}
</script>

<template>
  <div v-if="open" class="drawer-backdrop" @click="handleBackdrop">
    <div class="drawer-panel" role="dialog" aria-modal="true">
      <header class="drawer-header">
        <div class="header-title">
          <span class="header-icon">⚙</span>
          <h3>Search Filters</h3>
        </div>
        <button type="button" class="close-btn" title="Close (Esc)" @click="close">
          ✕
        </button>
      </header>

      <div class="drawer-body">
        <!-- Model Checkpoint -->
        <div class="filter-group">
          <label class="group-label">Model Checkpoint</label>
          <select v-model="selectedModel" class="form-select">
            <option value="">Any model</option>
            <option v-for="m in models" :key="m" :value="m">
              {{ m }}
            </option>
          </select>
        </div>

        <!-- Sampler -->
        <div class="filter-group">
          <label class="group-label">Sampler</label>
          <select v-model="selectedSampler" class="form-select">
            <option value="">Any sampler</option>
            <option v-for="s in samplers" :key="s" :value="s">
              {{ s }}
            </option>
          </select>
        </div>

        <!-- Rating Range -->
        <div class="filter-group">
          <label class="group-label">User Rating (1–10)</label>
          <div class="range-inputs">
            <div class="input-col">
              <span class="sub-label">Min</span>
              <select v-model="minRating" class="form-select small-select">
                <option value="">Any</option>
                <option v-for="r in 10" :key="r" :value="r">★ {{ r }}+</option>
              </select>
            </div>
            <span class="range-sep">to</span>
            <div class="input-col">
              <span class="sub-label">Max</span>
              <select v-model="maxRating" class="form-select small-select">
                <option value="">Any</option>
                <option v-for="r in 10" :key="r" :value="r">★ {{ r }}</option>
              </select>
            </div>
          </div>
        </div>

        <!-- Steps Range -->
        <div class="filter-group">
          <label class="group-label">Sampling Steps</label>
          <div class="range-inputs">
            <input
              v-model.number="minSteps"
              type="number"
              min="1"
              max="200"
              placeholder="Min"
              class="form-input range-field"
            />
            <span class="range-sep">—</span>
            <input
              v-model.number="maxSteps"
              type="number"
              min="1"
              max="200"
              placeholder="Max"
              class="form-input range-field"
            />
          </div>
          <div class="quick-chips">
            <button type="button" class="quick-chip" @click="minSteps = 20">20+</button>
            <button type="button" class="quick-chip" @click="minSteps = 30">30+</button>
            <button type="button" class="quick-chip" @click="minSteps = 50">50+</button>
          </div>
        </div>

        <!-- CFG Scale -->
        <div class="filter-group">
          <label class="group-label">CFG Scale</label>
          <div class="range-inputs">
            <input
              v-model.number="minCfg"
              type="number"
              step="0.5"
              min="1"
              max="30"
              placeholder="Min"
              class="form-input range-field"
            />
            <span class="range-sep">—</span>
            <input
              v-model.number="maxCfg"
              type="number"
              step="0.5"
              min="1"
              max="30"
              placeholder="Max"
              class="form-input range-field"
            />
          </div>
        </div>

        <!-- Aesthetic Score -->
        <div class="filter-group">
          <label class="group-label">Min Aesthetic Score</label>
          <input
            v-model.number="minAesthetic"
            type="number"
            step="0.1"
            min="0"
            max="10"
            placeholder="e.g. 7.0"
            class="form-input"
          />
        </div>

        <!-- Favorites Filter -->
        <div class="filter-group">
          <label class="group-label">Favorites</label>
          <div class="segmented-control">
            <button
              type="button"
              class="seg-btn"
              :class="{ active: isFavorite === null }"
              @click="isFavorite = null"
            >
              All
            </button>
            <button
              type="button"
              class="seg-btn"
              :class="{ active: isFavorite === true }"
              @click="isFavorite = true"
            >
              ★ Favorites
            </button>
          </div>
        </div>

        <!-- Content Sensitivity / NSFW Filter -->
        <div class="filter-group">
          <label class="group-label">Content Sensitivity</label>
          <div class="segmented-control">
            <button
              type="button"
              class="seg-btn"
              :class="{ active: isNsfw === null }"
              @click="isNsfw = null"
            >
              All
            </button>
            <button
              type="button"
              class="seg-btn"
              :class="{ active: isNsfw === false }"
              @click="isNsfw = false"
            >
              🛡 SFW
            </button>
            <button
              type="button"
              class="seg-btn"
              :class="{ active: isNsfw === true }"
              @click="isNsfw = true"
            >
              🔞 NSFW
            </button>
          </div>
        </div>

        <!-- Prompt Substring -->
        <div class="filter-group">
          <label class="group-label">Prompt Contains</label>
          <input
            v-model="prompt"
            type="text"
            placeholder="e.g. masterpiece, cinematic..."
            class="form-input"
          />
        </div>

        <!-- Negative Prompt Substring -->
        <div class="filter-group">
          <label class="group-label">Negative Prompt Contains</label>
          <input
            v-model="negativePrompt"
            type="text"
            placeholder="e.g. blurry, low quality..."
            class="form-input"
          />
        </div>
      </div>

      <footer class="drawer-footer">
        <button type="button" class="btn-reset" @click="reset">
          Reset All
        </button>
        <button type="button" class="btn-apply" @click="apply">
          Apply Filters
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(2px);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
  animation: fadeIn 0.15s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.drawer-panel {
  width: 360px;
  max-width: 90vw;
  height: 100%;
  background: #ffffff;
  color: #222222;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  animation: slideIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}

@media (prefers-color-scheme: dark) {
  .drawer-panel {
    background: #1e1e1e;
    color: #e5e5e5;
    border-left: 1px solid #333;
  }
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid rgba(128, 128, 128, 0.2);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-icon {
  font-size: 1.1em;
}

.header-title h3 {
  margin: 0;
  font-size: 1.1em;
  font-weight: 600;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 1.1em;
  color: #888;
  cursor: pointer;
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
}

.close-btn:hover {
  color: inherit;
  background: rgba(128, 128, 128, 0.15);
}

.drawer-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.group-label {
  font-size: 0.82em;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  opacity: 0.75;
}

.form-select,
.form-input {
  width: 100%;
  padding: 0.45rem 0.65rem;
  border: 1px solid rgba(128, 128, 128, 0.3);
  border-radius: 6px;
  background: rgba(128, 128, 128, 0.06);
  color: inherit;
  font: inherit;
  font-size: 0.9em;
  outline: none;
  box-sizing: border-box;
}

.form-select:focus,
.form-input:focus {
  border-color: #2f6fed;
  background: rgba(128, 128, 128, 0.12);
}

.range-inputs {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.input-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.sub-label {
  font-size: 0.75em;
  opacity: 0.6;
}

.range-sep {
  font-size: 0.85em;
  opacity: 0.6;
  padding-top: 0.8rem;
}

.range-field {
  flex: 1;
}

.quick-chips {
  display: flex;
  gap: 0.4rem;
  margin-top: 0.3rem;
}

.quick-chip {
  background: rgba(128, 128, 128, 0.1);
  border: 1px solid rgba(128, 128, 128, 0.2);
  color: inherit;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  font-size: 0.78em;
  cursor: pointer;
  transition: all 0.15s ease;
}

.quick-chip:hover {
  background: #2f6fed;
  color: #fff;
  border-color: #2f6fed;
}

.drawer-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-top: 1px solid rgba(128, 128, 128, 0.2);
  background: rgba(128, 128, 128, 0.04);
}

.btn-reset {
  background: transparent;
  border: 1px solid rgba(128, 128, 128, 0.3);
  color: inherit;
  padding: 0.45rem 0.9rem;
  border-radius: 6px;
  font-size: 0.85em;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-reset:hover {
  background: rgba(128, 128, 128, 0.15);
}

.btn-apply {
  background: #2f6fed;
  color: #fff;
  border: none;
  padding: 0.45rem 1.2rem;
  border-radius: 6px;
  font-size: 0.85em;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.btn-apply:hover {
  opacity: 0.9;
}

.segmented-control {
  display: flex;
  background: rgba(128, 128, 128, 0.12);
  border-radius: 6px;
  padding: 0.2rem;
  gap: 0.2rem;
}

.seg-btn {
  flex: 1;
  background: transparent;
  border: none;
  color: #888;
  font: inherit;
  font-size: 0.8em;
  font-weight: 500;
  padding: 0.35rem 0.6rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
  text-align: center;
}

.seg-btn:hover {
  color: inherit;
}

.seg-btn.active {
  background: #2f6fed;
  color: #fff;
  font-weight: 600;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}
</style>
