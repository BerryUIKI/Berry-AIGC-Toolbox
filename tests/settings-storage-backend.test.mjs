import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('SettingsModal.vue does not hardcode storage_backend on load or save', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/SettingsModal.vue');
  const raw = fs.readFileSync(filePath, 'utf-8');
  const content = raw.replace(/\r\n/g, '\n');

  // Verify that loadConfig does not hardcode storageBackend.value = "sqlite" unconditionally
  assert.ok(
    content.includes('storageBackend.value =\n      (config.storage_backend as "sqlite" | "mysql" | "postgres") || "sqlite"') ||
    content.includes('storageBackend.value = (config.storage_backend as "sqlite" | "mysql" | "postgres") || "sqlite"'),
    'SettingsModal must load storageBackend from config.storage_backend'
  );

  // Verify that saveSettings persists storageBackend.value instead of hardcoding "sqlite"
  assert.ok(
    content.includes('storage_backend: storageBackend.value,'),
    'SettingsModal must save storageBackend.value into config'
  );

  assert.ok(
    !content.includes('storage_backend: "sqlite",'),
    'SettingsModal must not contain hardcoded storage_backend: "sqlite" inside saveSettings'
  );
});
