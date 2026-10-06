import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const repoRoot = path.resolve(process.cwd());

test('README.md does not claim shipped bidirectional sync or ETag streaming guarantees', () => {
  const content = fs.readFileSync(path.join(repoRoot, 'README.md'), 'utf-8');

  // Should NOT claim bidirectional sync as shipped
  assert.doesNotMatch(content, /streaming SHA-256 bidirectional sync/i);
  assert.doesNotMatch(content, /bidirectional mirroring/i);

  // Should describe upload-only / one-way mirroring and note bidirectional sync is planned
  assert.match(content, /Incremental Remote Media Mirroring/i);
  assert.match(content, /bidirectional sync planned/i);
  assert.match(content, /Upload-Only/i);
});

test('Translated README files align on upload-only mirroring and planned bidirectional sync', () => {
  const zhCn = fs.readFileSync(path.join(repoRoot, 'README.zh-CN.md'), 'utf-8');
  assert.doesNotMatch(zhCn, /双向资产同步/);
  assert.doesNotMatch(zhCn, /双向镜像同步/);
  assert.match(zhCn, /单向媒体上传/);
  assert.match(zhCn, /双向同步规划中/);
  assert.match(zhCn, /单向上传/);

  const zhTw = fs.readFileSync(path.join(repoRoot, 'README.zh-TW.md'), 'utf-8');
  assert.doesNotMatch(zhTw, /雙向資產同步/);
  assert.doesNotMatch(zhTw, /雙向鏡像同步/);
  assert.match(zhTw, /單向媒體上傳/);
  assert.match(zhTw, /雙向同步規劃中/);
  assert.match(zhTw, /單向上傳/);

  const ja = fs.readFileSync(path.join(repoRoot, 'README.ja.md'), 'utf-8');
  assert.doesNotMatch(ja, /双方向アセット同期/);
  assert.doesNotMatch(ja, /検知、双方向同期/);
  assert.match(ja, /単方向メディアアップロード/);
  assert.match(ja, /双方向同期は計画中/);
  assert.match(ja, /単方向アップロード/);
});

test('Wiki documentation accurately documents upload-only capabilities and planned status', () => {
  const enWiki = fs.readFileSync(
    path.join(repoRoot, 'docs/wiki/en/05-export-and-collaboration/cloud-backup-and-sync.md'),
    'utf-8'
  );
  assert.match(enWiki, /one-way incremental upload mirroring/i);
  assert.match(enWiki, /Upload-Only Mirroring/i);
  assert.match(enWiki, /Planned Bidirectional Sync/i);
  assert.doesNotMatch(enWiki, /reports files to upload, skip, or delete/i);

  const zhCnWiki = fs.readFileSync(
    path.join(repoRoot, 'docs/wiki/zh-CN/05-export-and-collaboration/cloud-backup-and-sync.md'),
    'utf-8'
  );
  assert.match(zhCnWiki, /单向增量镜像上传/);
  assert.match(zhCnWiki, /仅支持单向上传/);
  assert.match(zhCnWiki, /规划中双向同步/);
});

test('ENGINEERING_HANDOFF.md documents cloud_sync.rs upload-only status for #257', () => {
  const handoff = fs.readFileSync(path.join(repoRoot, 'docs/ENGINEERING_HANDOFF.md'), 'utf-8');
  assert.match(handoff, /cloud_sync\.rs.*upload-oriented/i);
  assert.match(handoff, /#257/);
});
