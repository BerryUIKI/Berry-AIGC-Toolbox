import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('AutoTagModal.vue separates batch and current apply states to prevent identical button labels', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/AutoTagModal.vue');
  const raw = fs.readFileSync(filePath, 'utf-8');
  const content = raw.replace(/\r\n/g, '\n');

  assert.ok(
    content.includes('const isApplyingCurrent = ref(false);') &&
    content.includes('const isApplyingBatch = ref(false);'),
    'AutoTagModal must track separate loading state for single vs batch tag actions'
  );

  assert.ok(
    content.includes('isApplyingBatch\n                ? t.autoTagModal.tagging') ||
    content.includes('isApplyingBatch ? t.autoTagModal.tagging'),
    'Batch apply button must check isApplyingBatch'
  );

  assert.ok(
    content.includes('isApplyingCurrent ? t.autoTagModal.tagging'),
    'Current apply button must check isApplyingCurrent'
  );
});

test('crates/omera-tagger derives parent directory name when stem is generic model', () => {
  const filePath = path.resolve(process.cwd(), 'crates/omera-tagger/src/tagger.rs');
  const raw = fs.readFileSync(filePath, 'utf-8');
  const content = raw.replace(/\r\n/g, '\n');

  assert.ok(
    content.includes('stem.eq_ignore_ascii_case("model")') &&
    content.includes('.parent()'),
    'tagger.rs must inspect parent directory when stem is model'
  );
});

test('App.vue hides BatchActionBar when modals are open', () => {
  const filePath = path.resolve(process.cwd(), 'src/App.vue');
  const raw = fs.readFileSync(filePath, 'utf-8');
  const content = raw.replace(/\r\n/g, '\n');

  assert.ok(
    content.includes('const isAnyModalOpen = computed('),
    'App.vue must define isAnyModalOpen'
  );

  assert.ok(
    content.includes('v-if="selectedFilesList.length > 0 && !isAnyModalOpen"'),
    'BatchActionBar must be hidden when any modal is open'
  );
});
