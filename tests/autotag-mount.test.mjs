import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

test('AutoTagModal.vue executes loadModelList immediately upon mounting', () => {
  const filePath = path.resolve(process.cwd(), 'src/components/AutoTagModal.vue');
  const raw = fs.readFileSync(filePath, 'utf-8');
  const content = raw.replace(/\r\n/g, '\n');

  // Verify that watch on props.show includes immediate: true
  assert.ok(
    content.includes("watch(\n  () => props.show,\n  (val) => {\n    if (val) {\n      message.value = null;\n      predictions.value = [];\n      void loadModelList();\n    }\n  },\n  { immediate: true },\n);"),
    'AutoTagModal.vue must watch props.show with { immediate: true } so models load on initial mount'
  );
});
