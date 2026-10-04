import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('AutoTagModal and commands support writing tags to prompt metadata', () => {
  const modalPath = path.resolve(process.cwd(), 'src/components/AutoTagModal.vue');
  const modalContent = fs.readFileSync(modalPath, 'utf-8').replace(/\r\n/g, '\n');

  assert.ok(
    modalContent.includes('const writeToPrompt = ref(false);') &&
    modalContent.includes('const appendPrompt = ref(false);'),
    'AutoTagModal must define writeToPrompt and appendPrompt reactive refs'
  );

  assert.ok(
    modalContent.includes('write_to_prompt: writeToPrompt.value') &&
    modalContent.includes('append_prompt: appendPrompt.value'),
    'taggerConfig computed must forward prompt options to backend'
  );

  assert.ok(
    modalContent.includes('t.autoTagModal.writeToPrompt') &&
    modalContent.includes('t.autoTagModal.appendPrompt'),
    'AutoTagModal template must include writeToPrompt and appendPrompt checkboxes'
  );

  const commandsPath = path.resolve(process.cwd(), 'src-tauri/src/commands.rs');
  const commandsContent = fs.readFileSync(commandsPath, 'utf-8').replace(/\r\n/g, '\n');

  assert.ok(
    commandsContent.includes('config.write_to_prompt') &&
    commandsContent.includes('database.update_file_prompt'),
    'commands.rs must call database.update_file_prompt when write_to_prompt is true'
  );
});

test('All 7 locale files include writeToPrompt and appendPrompt strings', () => {
  const locales = ['de', 'en', 'es', 'fr', 'ja', 'zh-CN', 'zh-TW'];
  for (const loc of locales) {
    const locPath = path.resolve(process.cwd(), `src/i18n/locales/${loc}.ts`);
    const content = fs.readFileSync(locPath, 'utf-8');
    assert.ok(content.includes('writeToPrompt:'), `Locale ${loc} missing writeToPrompt`);
    assert.ok(content.includes('appendPrompt:'), `Locale ${loc} missing appendPrompt`);
  }
});
