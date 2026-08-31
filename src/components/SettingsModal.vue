<script setup lang="ts">
import { ref } from "vue";
import type { AppInfo } from "../types";

defineProps<{
  show: boolean;
  info: AppInfo | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const activeTab = ref<"general" | "display" | "parsers" | "about">("general");

// Settings state (persisted in localStorage)
const autoScanOnStartup = ref(localStorage.getItem("berry_autoscan") !== "false");
const blurNsfwDefault = ref(localStorage.getItem("berry_blur_nsfw") !== "false");
const showCardBadges = ref(localStorage.getItem("berry_card_badges") !== "false");
const defaultView = ref(localStorage.getItem("berry_default_view") || "grid");

function saveSettings() {
  localStorage.setItem("berry_autoscan", String(autoScanOnStartup.value));
  localStorage.setItem("berry_blur_nsfw", String(blurNsfwDefault.value));
  localStorage.setItem("berry_card_badges", String(showCardBadges.value));
  localStorage.setItem("berry_default_view", defaultView.value);
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
          <h3 class="dialog-title">首选项与设置</h3>
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
            常规 (General)
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'display' }"
            @click="activeTab = 'display'"
          >
            显示与安全 (Display)
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'parsers' }"
            @click="activeTab = 'parsers'"
          >
            解析器支持 (Parsers)
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'about' }"
            @click="activeTab = 'about'"
          >
            关于与存储 (About)
          </button>
        </aside>

        <section class="settings-content">
          <!-- Tab: General -->
          <div v-if="activeTab === 'general'" class="settings-panel">
            <h4 class="panel-title">常规偏好</h4>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">默认画廊视图</span>
                <span class="row-desc">选择启动软件时默认使用的图片展示方式</span>
              </div>
              <select v-model="defaultView" class="select-input">
                <option value="grid">网格瀑布流 (Grid)</option>
                <option value="table">详细列表 (Table)</option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">启动时自动扫描</span>
                <span class="row-desc">启动时自动检查已添加文件夹中的新增/变动图片</span>
              </div>
              <input v-model="autoScanOnStartup" type="checkbox" class="toggle-checkbox" />
            </div>
          </div>

          <!-- Tab: Display & Safety -->
          <div v-if="activeTab === 'display'" class="settings-panel">
            <h4 class="panel-title">显示与安全保护</h4>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">默认遮罩敏感内容 (NSFW)</span>
                <span class="row-desc">自动模糊标记为敏感/成人内容的图像，点击后方可显示</span>
              </div>
              <input v-model="blurNsfwDefault" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">显示卡片角标</span>
                <span class="row-desc">在网格图片卡片上显示格式、尺寸与评分徽章</span>
              </div>
              <input v-model="showCardBadges" type="checkbox" class="toggle-checkbox" />
            </div>
          </div>

          <!-- Tab: Parsers -->
          <div v-if="activeTab === 'parsers'" class="settings-panel">
            <h4 class="panel-title">内置元数据解析引擎</h4>
            <p class="panel-subtitle">Berry AIGC Toolbox 支持以下生成工具的生成参数与工作流无损解析：</p>

            <div class="parser-list">
              <div class="parser-item">
                <span class="parser-badge active">✓ 已启用</span>
                <span class="parser-name">WebUI (AUTOMATIC1111 / SD.Next)</span>
                <span class="parser-desc">PNG tEXt/iTXt (parameters), WebP EXIF</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">✓ 已启用</span>
                <span class="parser-name">ComfyUI</span>
                <span class="parser-desc">Prompt & Workflow JSON 树解析</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">✓ 已启用</span>
                <span class="parser-name">NovelAI</span>
                <span class="parser-desc">Comment / Description / Software 签名解析</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">✓ 已启用</span>
                <span class="parser-name">Fooocus / Fooocus-MRE</span>
                <span class="parser-desc">Fooocus 格式参数与模型解析</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">✓ 已启用</span>
                <span class="parser-name">InvokeAI & EasyDiffusion</span>
                <span class="parser-desc">Invoke Metadata & JSON Sidecar 支持</span>
              </div>
            </div>
          </div>

          <!-- Tab: About -->
          <div v-if="activeTab === 'about'" class="settings-panel">
            <h4 class="panel-title">关于与存储</h4>

            <div class="about-card">
              <div class="about-logo">
                <img src="../assets/logo.png" alt="Berry Logo" width="48" height="48" class="about-logo-img" />
              </div>
              <div class="about-details">
                <h5 class="about-name">Berry AIGC Toolbox</h5>
                <p class="about-ver">版本 v{{ info?.app_version || '0.1.1' }} (Clean-Slate Architecture)</p>
                <p class="about-desc">基于 Tauri 2 + Rust + Vue 3 的高性能本地 AIGC 图像资产管理工作台。</p>
              </div>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">本地 SQLite 数据库路径</span>
                <span class="row-desc path-code" :title="info?.database_path">{{ info?.database_path || '—' }}</span>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="dialog-footer">
        <button type="button" class="btn secondary" @click="emit('close')">取消</button>
        <button type="button" class="btn primary" @click="saveSettings">保存偏好</button>
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
