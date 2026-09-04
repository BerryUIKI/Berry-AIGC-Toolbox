<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { AppInfo } from "../types";
import {
  clearThumbnailCache,
  getThumbnailCacheStats,
  getThumbnailMaxEdge,
  setThumbnailMaxEdge,
  type ThumbnailCacheStats,
} from "../utils/thumbnail";
import { formatBytes } from "../utils/image";
import {
  currentLocaleSetting,
  setLocale,
  SUPPORTED_LOCALES,
  t,
  type LocaleSetting,
} from "../i18n";

const props = defineProps<{
  show: boolean;
  info: AppInfo | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", settings: {
    locale: LocaleSetting;
    autoScan: boolean;
    blurNsfw: boolean;
    showCardBadges: boolean;
    defaultView: "grid" | "table";
    thumbnailMaxEdge: number;
  }): void;
}>();

const activeTab = ref<"general" | "display" | "parsers" | "about">("general");

// Settings state (persisted in localStorage)
const selectedLocale = ref<LocaleSetting>(currentLocaleSetting.value);
const autoScanOnStartup = ref(localStorage.getItem("berry_autoscan") !== "false");
const blurNsfwDefault = ref(localStorage.getItem("berry_blur_nsfw") !== "false");
const showCardBadges = ref(localStorage.getItem("berry_card_badges") !== "false");
const defaultView = ref(localStorage.getItem("berry_default_view") || "grid");
const thumbnailMaxEdge = ref(getThumbnailMaxEdge());

// Cache stats
const cacheStats = ref<ThumbnailCacheStats | null>(null);
const clearingCache = ref(false);
const cacheMessage = ref("");

async function loadCacheStats() {
  try {
    cacheStats.value = await getThumbnailCacheStats();
  } catch (e) {
    console.error("Failed to load thumbnail cache stats:", e);
  }
}

async function handleClearCache() {
  clearingCache.value = true;
  cacheMessage.value = "";
  try {
    const count = await clearThumbnailCache();
    cacheMessage.value = `✓ ${count} ${t.value.settings.thumbnailsCount}`;
    await loadCacheStats();
  } catch (e) {
    cacheMessage.value = `${e}`;
  } finally {
    clearingCache.value = false;
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      selectedLocale.value = currentLocaleSetting.value;
      autoScanOnStartup.value = localStorage.getItem("berry_autoscan") !== "false";
      blurNsfwDefault.value = localStorage.getItem("berry_blur_nsfw") !== "false";
      showCardBadges.value = localStorage.getItem("berry_card_badges") !== "false";
      defaultView.value = localStorage.getItem("berry_default_view") || "grid";
      thumbnailMaxEdge.value = getThumbnailMaxEdge();
      void loadCacheStats();
    }
  },
);

onMounted(() => {
  if (props.show) {
    void loadCacheStats();
  }
});

function saveSettings() {
  setLocale(selectedLocale.value);
  localStorage.setItem("berry_autoscan", String(autoScanOnStartup.value));
  localStorage.setItem("berry_blur_nsfw", String(blurNsfwDefault.value));
  localStorage.setItem("berry_card_badges", String(showCardBadges.value));
  localStorage.setItem("berry_default_view", defaultView.value);
  setThumbnailMaxEdge(thumbnailMaxEdge.value);
  emit("save", {
    locale: selectedLocale.value,
    autoScan: autoScanOnStartup.value,
    blurNsfw: blurNsfwDefault.value,
    showCardBadges: showCardBadges.value,
    defaultView: (defaultView.value === "table" ? "table" : "grid"),
    thumbnailMaxEdge: thumbnailMaxEdge.value,
  });
  emit("close");
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="settings-dialog">
      <!-- Header -->
      <div class="dialog-header">
        <div class="header-left">
          <span class="dialog-icon">⚙️</span>
          <h3 class="dialog-title">{{ t.settings.title }}</h3>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Body: Left Tabs + Right Content -->
      <div class="dialog-body">
        <aside class="settings-tabs">
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'general' }"
            @click="activeTab = 'general'"
          >
            {{ t.settings.tabs.general }}
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'display' }"
            @click="activeTab = 'display'"
          >
            {{ t.settings.tabs.display }}
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'parsers' }"
            @click="activeTab = 'parsers'"
          >
            {{ t.settings.tabs.parsers }}
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'about' }"
            @click="activeTab = 'about'"
          >
            {{ t.settings.tabs.about }}
          </button>
        </aside>

        <section class="settings-content">
          <!-- Tab: General -->
          <div v-if="activeTab === 'general'" class="settings-panel">
            <h4 class="panel-title">{{ t.settings.generalTitle }}</h4>

            <!-- Language Setting -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.language }}</span>
                <span class="row-desc">{{ t.settings.languageDesc }}</span>
              </div>
              <select v-model="selectedLocale" class="select-input">
                <option
                  v-for="loc in SUPPORTED_LOCALES"
                  :key="loc.key"
                  :value="loc.key"
                >
                  {{ loc.label }}
                </option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.defaultView }}</span>
                <span class="row-desc">{{ t.settings.defaultViewDesc }}</span>
              </div>
              <select v-model="defaultView" class="select-input">
                <option value="grid">{{ t.settings.viewGrid }}</option>
                <option value="table">{{ t.settings.viewTable }}</option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.autoScan }}</span>
                <span class="row-desc">{{ t.settings.autoScanDesc }}</span>
              </div>
              <input v-model="autoScanOnStartup" type="checkbox" class="toggle-checkbox" />
            </div>
          </div>

          <!-- Tab: Display & Safety -->
          <div v-if="activeTab === 'display'" class="settings-panel">
            <h4 class="panel-title">{{ t.settings.displayTitle }}</h4>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.blurNsfw }}</span>
                <span class="row-desc">{{ t.settings.blurNsfwDesc }}</span>
              </div>
              <input v-model="blurNsfwDefault" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.showBadges }}</span>
                <span class="row-desc">{{ t.settings.showBadgesDesc }}</span>
              </div>
              <input v-model="showCardBadges" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.thumbResolution }}</span>
                <span class="row-desc">{{ t.settings.thumbResolutionDesc }}</span>
              </div>
              <select v-model.number="thumbnailMaxEdge" class="select-input">
                <option :value="256">{{ t.settings.thumbCompact }}</option>
                <option :value="384">{{ t.settings.thumbStandard }}</option>
                <option :value="448">{{ t.settings.thumbHd }}</option>
                <option :value="512">{{ t.settings.thumbUltra }}</option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.cacheManagement }}</span>
                <span class="row-desc">
                  {{ t.settings.currentUsage }}
                  <strong style="color:#12b5cb;">
                    {{ cacheStats ? `${formatBytes(cacheStats.total_bytes)} (${cacheStats.file_count} ${t.settings.thumbnailsCount})` : t.settings.calculating }}
                  </strong>
                  <span v-if="cacheMessage" style="margin-left: 8px; color: #4ade80;">{{ cacheMessage }}</span>
                </span>
              </div>
              <button
                type="button"
                class="btn secondary"
                :disabled="clearingCache"
                @click="handleClearCache"
              >
                {{ clearingCache ? t.settings.clearing : t.settings.clearCache }}
              </button>
            </div>
          </div>

          <!-- Tab: Parsers -->
          <div v-if="activeTab === 'parsers'" class="settings-panel">
            <h4 class="panel-title">{{ t.settings.parsersTitle }}</h4>
            <p class="panel-subtitle">{{ t.settings.parsersSubtitle }}</p>

            <div class="parser-list">
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">WebUI (AUTOMATIC1111 / SD.Next)</span>
                <span class="parser-desc">PNG tEXt/iTXt (parameters), WebP EXIF</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">ComfyUI</span>
                <span class="parser-desc">Prompt & Workflow JSON Graph</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">NovelAI</span>
                <span class="parser-desc">Comment / Description / Software Signature</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">Fooocus / Fooocus-MRE</span>
                <span class="parser-desc">Fooocus Parameters & Base Model Parser</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">InvokeAI & EasyDiffusion</span>
                <span class="parser-desc">Invoke Metadata & JSON Sidecar</span>
              </div>
            </div>
          </div>

          <!-- Tab: About -->
          <div v-if="activeTab === 'about'" class="settings-panel">
            <h4 class="panel-title">{{ t.settings.aboutTitle }}</h4>

            <div class="about-card">
              <div class="about-logo">
                <img src="../assets/logo.png" alt="Berry Logo" width="48" height="48" class="about-logo-img" />
              </div>
              <div class="about-details">
                <h5 class="about-name">Berry AIGC Toolbox</h5>
                <p class="about-ver">v{{ info?.app_version || '0.1.1' }}</p>
                <p class="about-desc">{{ t.settings.aboutDesc }}</p>
              </div>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.dbPath }}</span>
                <span class="row-desc path-code" :title="info?.database_path">{{ info?.database_path || '—' }}</span>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="dialog-footer">
        <button type="button" class="btn secondary" @click="emit('close')">{{ t.settings.cancel }}</button>
        <button type="button" class="btn primary" @click="saveSettings">{{ t.settings.save }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  user-select: none;
}

.settings-dialog {
  width: 620px;
  max-width: 90vw;
  height: 480px;
  background: #18181c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
  overflow: hidden;
}

.dialog-header {
  height: 46px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dialog-icon {
  font-size: 1rem;
}

.dialog-title {
  margin: 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: #f1f5f9;
}

.close-btn {
  background: transparent;
  border: none;
  color: #71717a;
  cursor: pointer;
  padding: 4px;
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s;
}

.close-btn:hover {
  color: #ffffff;
}

.dialog-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

.settings-tabs {
  width: 170px;
  background: #141417;
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.tab-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  padding: 8px 10px;
  border-radius: 6px;
  font-size: 0.78rem;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: all 0.12s ease;
}

.tab-btn:hover {
  background: rgba(255, 255, 255, 0.04);
  color: #f1f5f9;
}

.tab-btn.active {
  background: rgba(18, 181, 203, 0.18);
  color: #67e8f9;
  font-weight: 600;
}

.settings-content {
  flex: 1;
  padding: 16px 20px;
  overflow-y: auto;
}

.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.panel-title {
  margin: 0;
  font-size: 0.86rem;
  font-weight: 600;
  color: #f8fafc;
}

.panel-subtitle {
  margin: 0;
  font-size: 0.74rem;
  color: #71717a;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px;
  background: #202024;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.row-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.row-label {
  font-size: 0.78rem;
  font-weight: 500;
  color: #f1f5f9;
}

.row-desc {
  font-size: 0.7rem;
  color: #71717a;
}

.path-code {
  font-family: monospace;
  word-break: break-all;
}

.select-input {
  background: #18181c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #e2e8f0;
  border-radius: 5px;
  padding: 4px 8px;
  font-size: 0.75rem;
  outline: none;
}

.toggle-checkbox {
  width: 16px;
  height: 16px;
  accent-color: #12b5cb;
  cursor: pointer;
}

.parser-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.parser-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: #202024;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 0.74rem;
}

.parser-badge {
  font-size: 0.66rem;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
  font-weight: 600;
}

.parser-name {
  font-weight: 500;
  color: #f1f5f9;
}

.parser-desc {
  color: #71717a;
  margin-left: auto;
  font-size: 0.68rem;
}

.about-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px;
  background: #202024;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.about-logo {
  display: flex;
  align-items: center;
  justify-content: center;
}

.about-logo-img {
  display: block;
  object-fit: contain;
}

.about-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.about-name {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 700;
  color: #f8fafc;
}

.about-ver {
  margin: 0;
  font-size: 0.72rem;
  color: #12b5cb;
  font-weight: 500;
}

.about-desc {
  margin: 2px 0 0;
  font-size: 0.72rem;
  color: #94a3b8;
}

.dialog-footer {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: #141417;
}

.btn {
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 0.76rem;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.12s ease;
  border: none;
}

.btn.secondary {
  background: rgba(255, 255, 255, 0.05);
  color: #cbd5e1;
}

.btn.secondary:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
}

.btn.primary {
  background: #12b5cb;
  color: #ffffff;
  font-weight: 500;
}

.btn.primary:hover {
  background: #0e9aa7;
}
</style>
