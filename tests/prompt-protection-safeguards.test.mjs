import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('AppConfig and SettingsModal provide developer settings for prompt overwrite safeguards', () => {
  const configTsPath = path.resolve(process.cwd(), 'src/utils/config.ts');
  const configTsContent = fs.readFileSync(configTsPath, 'utf-8');

  assert.ok(
    configTsContent.includes('allow_override_existing_prompt: boolean;'),
    'AppConfig interface must include allow_override_existing_prompt'
  );
  assert.ok(
    configTsContent.includes('allow_override_existing_prompt: false'),
    'DEFAULT_CONFIG must default allow_override_existing_prompt to false'
  );

  const settingsPath = path.resolve(process.cwd(), 'src/components/SettingsModal.vue');
  const settingsContent = fs.readFileSync(settingsPath, 'utf-8');

  assert.ok(
    settingsContent.includes("activeTab === 'developer'"),
    'SettingsModal must include developer tab'
  );
  assert.ok(
    settingsContent.includes('allowOverrideExistingPrompt'),
    'SettingsModal must bind allowOverrideExistingPrompt setting'
  );
  assert.ok(
    settingsContent.includes('allow_override_existing_prompt: allowOverrideExistingPrompt.value'),
    'SettingsModal must persist allow_override_existing_prompt'
  );

  const commandsPath = path.resolve(process.cwd(), 'src-tauri/src/commands.rs');
  const commandsContent = fs.readFileSync(commandsPath, 'utf-8');

  assert.ok(
    commandsContent.includes('pub allow_override_existing_prompt: bool'),
    'Backend AppConfig must include allow_override_existing_prompt field'
  );
  assert.ok(
    commandsContent.includes('allow_override_existing_prompt: false'),
    'Backend Default for AppConfig must default allow_override_existing_prompt to false'
  );
  assert.ok(
    commandsContent.includes('config.allow_override_existing_prompt') &&
    commandsContent.includes('database.update_file_prompt'),
    'auto_tag_file and batch_auto_tag_files must pass allow_override_existing_prompt to update_file_prompt'
  );
});

test('AutoTagModal enforces prompt protection when existing prompt is non-empty', () => {
  const modalPath = path.resolve(process.cwd(), 'src/components/AutoTagModal.vue');
  const modalContent = fs.readFileSync(modalPath, 'utf-8');

  assert.ok(
    modalContent.includes('allowOverridePrompt?: boolean'),
    'AutoTagModal must accept allowOverridePrompt prop'
  );
  assert.ok(
    modalContent.includes('isPromptWriteLocked'),
    'AutoTagModal must compute isPromptWriteLocked state'
  );
  assert.ok(
    modalContent.includes('t.settings.promptProtected'),
    'AutoTagModal must display prompt protected indicator when locked'
  );
});

test('All 7 locale files define developer and promptProtected translations', () => {
  const locales = ['de', 'en', 'es', 'fr', 'ja', 'zh-CN', 'zh-TW'];
  for (const loc of locales) {
    const locPath = path.resolve(process.cwd(), `src/i18n/locales/${loc}.ts`);
    const content = fs.readFileSync(locPath, 'utf-8');
    assert.ok(content.includes('developer:'), `Locale ${loc} missing developer`);
    assert.ok(content.includes('allowOverrideExistingPrompt:'), `Locale ${loc} missing allowOverrideExistingPrompt`);
    assert.ok(content.includes('promptProtected:'), `Locale ${loc} missing promptProtected`);
  }
});
