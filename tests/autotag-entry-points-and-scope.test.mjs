import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import path from "node:path";

test("MenuBar.vue exposes dedicated entry points for auto-tagging", () => {
  const menuBarPath = path.resolve(process.cwd(), "src/components/MenuBar.vue");
  const menuBarContent = fs.readFileSync(menuBarPath, "utf-8");

  assert.ok(
    menuBarContent.includes("batchAutoTag: [];"),
    "MenuBar must define batchAutoTag emit"
  );
  assert.ok(
    menuBarContent.includes("openAutoTagger: [];"),
    "MenuBar must define openAutoTagger emit"
  );
  assert.ok(
    menuBarContent.includes("t.menu.batchAutoTag"),
    "MenuBar Edit menu must include batchAutoTag entry"
  );
  assert.ok(
    menuBarContent.includes("t.menu.autoTagger"),
    "MenuBar Tools menu must include autoTagger entry"
  );
});

test("Sidebar.vue exposes auto-tagger tool entry point in footer", () => {
  const sidebarPath = path.resolve(process.cwd(), "src/components/Sidebar.vue");
  const sidebarContent = fs.readFileSync(sidebarPath, "utf-8");

  assert.ok(
    sidebarContent.includes("openAutoTagger: [];"),
    "Sidebar must define openAutoTagger emit"
  );
  assert.ok(
    sidebarContent.includes("t.search.autoTagger"),
    "Sidebar footer must include auto-tagger button"
  );
});

test("AutoTagModal.vue implements batch scope filtering and confirmation guard", () => {
  const modalPath = path.resolve(process.cwd(), "src/components/AutoTagModal.vue");
  const modalContent = fs.readFileSync(modalPath, "utf-8");

  assert.ok(
    modalContent.includes("batchScope"),
    "AutoTagModal must define batchScope reactive ref"
  );
  assert.ok(
    modalContent.includes("effectiveBatchFileIds"),
    "AutoTagModal must compute effectiveBatchFileIds"
  );
  assert.ok(
    modalContent.includes("window.confirm(confirmMsg)"),
    "AutoTagModal must confirm before applying large batch jobs"
  );
  assert.ok(
    modalContent.includes("t.autoTagModal.scopeLabel"),
    "AutoTagModal template must include scope selection controls"
  );
});

test("All 7 locale files define batch scope and menu autoTag keys", () => {
  const locales = ["de", "en", "es", "fr", "ja", "zh-CN", "zh-TW"];
  for (const loc of locales) {
    const locPath = path.resolve(process.cwd(), `src/i18n/locales/${loc}.ts`);
    const content = fs.readFileSync(locPath, "utf-8");
    assert.ok(content.includes("batchAutoTag:"), `Locale ${loc} missing batchAutoTag`);
    assert.ok(content.includes("autoTagger:"), `Locale ${loc} missing autoTagger`);
    assert.ok(content.includes("scopeLabel:"), `Locale ${loc} missing scopeLabel`);
    assert.ok(content.includes("confirmBatchTitle:"), `Locale ${loc} missing confirmBatchTitle`);
  }
});
