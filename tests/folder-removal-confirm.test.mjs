import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";

test("Sidebar.vue requires user confirmation before removing an indexed folder", async () => {
  const source = await fs.readFile(new URL("../src/components/Sidebar.vue", import.meta.url), "utf8");

  assert.ok(source.includes("removeFolderConfirm"), "Sidebar.vue must reference removeFolderConfirm");
  assert.ok(source.includes("window.confirm"), "Sidebar.vue must guard removeFolder with window.confirm");
  assert.match(
    source,
    /if\s*\(!window\.confirm\([^)]+\)\)\s*\{\s*return;\s*\}/,
    "Sidebar.vue must abort deletion when user cancels the confirm dialog",
  );
});

test("All locales define removeFolderConfirm in nav section", async () => {
  const locales = ["en", "de", "es", "fr", "ja", "zh-CN", "zh-TW"];

  for (const loc of locales) {
    const content = await fs.readFile(new URL(`../src/i18n/locales/${loc}.ts`, import.meta.url), "utf8");
    assert.ok(
      content.includes("removeFolderConfirm:"),
      `Locale ${loc} must define removeFolderConfirm`,
    );
    assert.ok(
      content.includes("{name}"),
      `Locale ${loc} removeFolderConfirm must contain {name} placeholder`,
    );
  }
});
