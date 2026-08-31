<script setup lang="ts">
import { ref } from "vue";
import { currentLocaleSetting, setLocale, SUPPORTED_LOCALES, type LocaleSetting } from "../i18n";

const open = ref(false);

function select(locale: LocaleSetting) {
  setLocale(locale);
  open.value = false;
}
</script>

<template>
  <div class="lang-selector-wrapper">
    <button
      type="button"
      class="lang-btn"
      :class="{ active: open }"
      title="Change Language"
      @click="open = !open"
    >
      <span class="globe-icon">🌐</span>
      <span class="current-lang-text">{{ SUPPORTED_LOCALES.find(l => l.key === currentLocaleSetting)?.label }}</span>
      <span class="arrow-indicator">▾</span>
    </button>

    <div v-if="open" class="lang-menu" @mouseleave="open = false">
      <button
        v-for="item in SUPPORTED_LOCALES"
        :key="item.key"
        type="button"
        class="lang-item"
        :class="{ active: item.key === currentLocaleSetting }"
        @click="select(item.key)"
      >
        <span class="lang-label">{{ item.label }}</span>
        <span v-if="item.key === currentLocaleSetting" class="check">✓</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.lang-selector-wrapper {
  position: relative;
  display: inline-block;
}

.lang-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: rgba(128, 128, 128, 0.1);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 6px;
  padding: 4px 10px;
  color: inherit;
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.lang-btn:hover,
.lang-btn.active {
  background: rgba(128, 128, 128, 0.2);
  border-color: rgba(128, 128, 128, 0.4);
}

.globe-icon {
  font-size: 0.95rem;
}

.arrow-indicator {
  font-size: 0.75rem;
  opacity: 0.7;
}

.lang-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  background: var(--bg-surface, #222);
  border: 1px solid var(--border-color, #444);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  min-width: 140px;
  z-index: 1500;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 4px 0;
}

.lang-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  background: transparent;
  border: none;
  color: #ddd;
  font-size: 0.85rem;
  text-align: left;
  cursor: pointer;
  transition: background 0.1s ease;
}

.lang-item:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

.lang-item.active {
  color: #60a5fa;
  font-weight: 600;
  background: rgba(96, 165, 250, 0.1);
}

.check {
  font-size: 0.8rem;
  color: #60a5fa;
}
</style>
