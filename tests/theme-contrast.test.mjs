import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");

test("style.css declares light theme variables and readable select option styles", () => {
  const css = fs.readFileSync(path.join(rootDir, "src", "style.css"), "utf-8");

  // Check light theme variables
  assert.match(css, /:root\[data-theme="light"\]\s*\{[^}]*color-scheme:\s*light;/);
  assert.match(css, /--color-bg-primary:\s*#ffffff;/);
  assert.match(css, /--color-text-primary:\s*#111827;/);

  // Check system light theme media query
  assert.match(css, /@media\s*\(prefers-color-scheme:\s*light\)\s*\{[^}]*:root\[data-theme="system"\]/);

  // Check cyan badge variables for contrast in dark and light themes
  assert.match(css, /--badge-cyan-text:\s*#67e8f9;/);
  assert.match(css, /:root\[data-theme="light"\]\s*\{[^}]*--badge-cyan-text:\s*#155e75;/);

  // Check global select and option styling
  assert.match(css, /select\s*option\s*\{[^}]*color:\s*var\(--color-text-primary\);/);
  assert.match(css, /select\s*option\s*\{[^}]*background-color:\s*var\(--color-bg-primary\);/);

  // Check focus-visible outline for accessibility
  assert.match(css, /:focus-visible\s*\{[^}]*outline:\s*2px\s*solid\s*var\(--color-accent\);/);
});

test("SettingsModal.vue uses theme variables for select-input and url-input", () => {
  const vue = fs.readFileSync(path.join(rootDir, "src", "components", "SettingsModal.vue"), "utf-8");

  // .select-input should use var(--color-text-primary), not hardcoded #e2e8f0
  const selectMatch = vue.match(/\.select-input\s*\{([^}]+)\}/);
  assert.ok(selectMatch, ".select-input rule should exist");
  assert.match(selectMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(selectMatch[1], /color:\s*#e2e8f0/);

  // .url-input should also use var(--color-text-primary)
  const urlMatch = vue.match(/\.url-input\s*\{([^}]+)\}/);
  assert.ok(urlMatch, ".url-input rule should exist");
  assert.match(urlMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(urlMatch[1], /color:\s*#e2e8f0/);

  // .tab-btn.active should use semantic cyan text variable instead of hardcoded #67e8f9
  const tabBtnActiveMatch = vue.match(/\.tab-btn\.active\s*\{([^}]+)\}/);
  assert.ok(tabBtnActiveMatch, ".tab-btn.active rule should exist");
  assert.match(tabBtnActiveMatch[1], /color:\s*var\(--badge-cyan-text/);
  assert.doesNotMatch(tabBtnActiveMatch[1], /color:\s*#67e8f9/);
});

test("SortBar.vue uses theme variables instead of hardcoded dark colors", () => {
  const vue = fs.readFileSync(path.join(rootDir, "src", "components", "SortBar.vue"), "utf-8");

  const sortSelectMatch = vue.match(/\.sort-select\s*\{([^}]+)\}/);
  assert.ok(sortSelectMatch, ".sort-select rule should exist");
  assert.match(sortSelectMatch[1], /background:\s*var\(--color-bg-secondary\)/);
  assert.match(sortSelectMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(sortSelectMatch[1], /background:\s*#202024/);
});

test("TitleBar.vue uses theme variables for brand title, version subtitle, and window controls", () => {
  const vue = fs.readFileSync(path.join(rootDir, "src", "components", "TitleBar.vue"), "utf-8");

  const brandMatch = vue.match(/\.brand-title\s*\{([^}]+)\}/);
  assert.ok(brandMatch, ".brand-title rule should exist");
  assert.match(brandMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(brandMatch[1], /color:\s*#f1f5f9/);

  const subtitleMatch = vue.match(/\.brand-subtitle\s*\{([^}]+)\}/);
  assert.ok(subtitleMatch, ".brand-subtitle rule should exist");
  assert.match(subtitleMatch[1], /color:\s*var\(--badge-cyan-text/);
  assert.doesNotMatch(subtitleMatch[1], /color:\s*#67e8f9/);

  const controlMatch = vue.match(/\.control-btn:hover\s*\{([^}]+)\}/);
  assert.ok(controlMatch, ".control-btn:hover rule should exist");
  assert.match(controlMatch[1], /background:\s*var\(--color-bg-hover\)/);
});

test("Core components and modals use semantic theme variables instead of hardcoded dark colors", () => {
  // MenuBar
  const menuBar = fs.readFileSync(path.join(rootDir, "src", "components", "MenuBar.vue"), "utf-8");
  assert.doesNotMatch(menuBar, /background:\s*#1c1c20/);
  assert.doesNotMatch(menuBar, /color:\s*#67e8f9/);
  assert.match(menuBar, /background:\s*var\(--color-bg-primary\)/);

  // StatusBar
  const statusBar = fs.readFileSync(path.join(rootDir, "src", "components", "StatusBar.vue"), "utf-8");
  assert.doesNotMatch(statusBar, /background:\s*#111114/);
  assert.doesNotMatch(statusBar, /color:\s*#67e8f9/);
  assert.match(statusBar, /background:\s*var\(--color-bg-primary\)/);

  // VirtualGrid
  const virtualGrid = fs.readFileSync(path.join(rootDir, "src", "components", "VirtualGrid.vue"), "utf-8");
  assert.doesNotMatch(virtualGrid, /background:\s*#252525/);
  assert.match(virtualGrid, /background:\s*var\(--color-bg-primary\)/);

  // UpdateModal
  const updateModal = fs.readFileSync(path.join(rootDir, "src", "components", "UpdateModal.vue"), "utf-8");
  assert.doesNotMatch(updateModal, /background:\s*#18181c/);
  assert.match(updateModal, /background:\s*var\(--color-bg-primary\)/);

  // PromptStatsModal
  const promptStats = fs.readFileSync(path.join(rootDir, "src", "components", "PromptStatsModal.vue"), "utf-8");
  assert.doesNotMatch(promptStats, /background:\s*#1e1e1e/);
  assert.match(promptStats, /background:\s*var\(--color-bg-primary\)/);

  // ThumbnailDiagnosticsModal
  const diagModal = fs.readFileSync(path.join(rootDir, "src", "components", "ThumbnailDiagnosticsModal.vue"), "utf-8");
  assert.doesNotMatch(diagModal, /background:\s*#1e1e24/);
  assert.match(diagModal, /background:\s*var\(--color-bg-primary\)/);

  // PreviewPane
  const previewPane = fs.readFileSync(path.join(rootDir, "src", "components", "PreviewPane.vue"), "utf-8");
  assert.doesNotMatch(previewPane, /\.preview-dialog\s*\{[^}]*background:\s*#1e1e1e/);
  assert.match(previewPane, /\.preview-dialog\s*\{[^}]*background:\s*var\(--color-bg-primary\)/);

  // AutoTagModal
  const autoTagModal = fs.readFileSync(path.join(rootDir, "src", "components", "AutoTagModal.vue"), "utf-8");
  assert.match(autoTagModal, /\.section-title\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
  assert.match(autoTagModal, /\.empty-model-desc\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
  assert.match(autoTagModal, /\.source-label\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
  assert.match(autoTagModal, /\.no-tags-prompt\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
});

test("Modal dialogs and dropdowns use semantic text tokens without illegible light-mode fallbacks", () => {
  const css = fs.readFileSync(path.join(rootDir, "src", "style.css"), "utf-8");
  assert.match(css, /--text-primary:\s*var\(--color-text-primary\);/);
  assert.match(css, /--text-secondary:\s*var\(--color-text-secondary\);/);

  const targets = [
    "DatabaseManagerModal.vue",
    "FileOperationModal.vue",
    "LoraManagerModal.vue",
    "ModelManagerModal.vue",
    "ShortcutsHelpModal.vue",
    "LanguageSelector.vue",
  ];

  for (const file of targets) {
    const content = fs.readFileSync(path.join(rootDir, "src", "components", file), "utf-8");
    assert.doesNotMatch(content, /var\(--text-primary,\s*#[0-9a-fA-F]+\)/, `${file} should not have hardcoded dark text-primary fallback`);
    assert.doesNotMatch(content, /var\(--text-secondary,\s*#[0-9a-fA-F]+\)/, `${file} should not have hardcoded dark text-secondary fallback`);
  }
});

test("InspectorPane.vue uses semantic theme tokens for title, tags, and container borders", () => {
  const vue = fs.readFileSync(path.join(rootDir, "src", "components", "InspectorPane.vue"), "utf-8");
  assert.match(vue, /\.inspector-title\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
  assert.match(vue, /\.tag-pill\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
  assert.match(vue, /\.file-name\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
  assert.match(vue, /\.detected-name\s*\{[^}]*color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(vue, /\.inspector-title\s*\{[^}]*color:\s*#f1f5f9/);
  assert.doesNotMatch(vue, /\.tag-pill\s*\{[^}]*color:\s*#f1f5f9/);
});

