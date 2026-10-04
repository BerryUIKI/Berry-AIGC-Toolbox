import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

test("Commands and Lib declare unload_tagger_model and unload_clip_model", () => {
  const commandsFile = fs.readFileSync(path.resolve("src-tauri/src/commands.rs"), "utf-8");
  const libFile = fs.readFileSync(path.resolve("src-tauri/src/lib.rs"), "utf-8");

  assert.ok(
    commandsFile.includes("pub fn unload_tagger_model("),
    "commands.rs must define unload_tagger_model",
  );
  assert.ok(
    commandsFile.includes("pub fn unload_clip_model("),
    "commands.rs must define unload_clip_model",
  );

  assert.ok(
    libFile.includes("commands::unload_tagger_model"),
    "lib.rs must register unload_tagger_model in invoke_handler",
  );
  assert.ok(
    libFile.includes("commands::unload_clip_model"),
    "lib.rs must register unload_clip_model in invoke_handler",
  );
});

test("Modals contain unload model buttons and handlers", () => {
  const autoTagModal = fs.readFileSync(path.resolve("src/components/AutoTagModal.vue"), "utf-8");
  const clipManagerModal = fs.readFileSync(path.resolve("src/components/ClipManagerModal.vue"), "utf-8");

  assert.ok(
    autoTagModal.includes('invoke("unload_tagger_model")'),
    "AutoTagModal must invoke unload_tagger_model",
  );
  assert.ok(
    autoTagModal.includes('class="unload-btn"'),
    "AutoTagModal must render unload button",
  );

  assert.ok(
    clipManagerModal.includes('invoke("unload_clip_model")'),
    "ClipManagerModal must invoke unload_clip_model",
  );
  assert.ok(
    clipManagerModal.includes('onUnloadModel'),
    "ClipManagerModal must handle onUnloadModel",
  );
});

test("All 7 locales contain unloadModel and modelUnloaded keys", () => {
  const locales = ["en", "zh-CN", "zh-TW", "ja", "de", "fr", "es"];
  for (const locale of locales) {
    const file = fs.readFileSync(path.resolve(`src/i18n/locales/${locale}.ts`), "utf-8");
    assert.ok(
      file.includes("unloadModel:"),
      `locale ${locale} must include unloadModel`,
    );
    assert.ok(
      file.includes("modelUnloaded:"),
      `locale ${locale} must include modelUnloaded`,
    );
  }
});
