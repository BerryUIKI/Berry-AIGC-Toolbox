import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { startCloudSyncWithConfirmation } from "../src/utils/cloud-sync-namespace.ts";

const preview = JSON.parse(readFileSync(new URL("./fixtures/cloud-sync-namespace.json", import.meta.url)));
const config = { provider: "local_path", local_path: "/backup" };

test("confirmed layout uses the nested Rust wire DTO and snapshots caller state", async () => {
  const options = { folder_ids: [12], remote_prefix: "media/", namespace_manifest_id: "old" };
  const currentConfig = { ...config };
  const calls = [];
  const started = await startCloudSyncWithConfirmation(currentConfig, options, (mapping) => {
    assert.deepEqual(mapping, preview);
    assert.equal(mapping.manifest.roots[0].legacy_ambiguous, true);
    options.folder_ids.push(99);
    options.remote_prefix = "changed";
    currentConfig.local_path = "/changed";
    return true;
  }, async (command, args) => {
    calls.push({ command, args });
    return command === "cloud_sync_preview_namespace" ? preview : undefined;
  });
  assert.equal(started, true);
  assert.deepEqual(calls, [
    { command: "cloud_sync_preview_namespace", args: { options: {
      folder_ids: [12], remote_prefix: "media/", namespace_manifest_id: "old",
    } } },
    { command: "cloud_sync_start", args: { config, options: {
      folder_ids: [12], remote_prefix: "media/", namespace_manifest_id: preview.manifest_id,
    } } },
  ]);
});

test("declining the layout never starts a transfer", async () => {
  const calls = [];
  assert.equal(await startCloudSyncWithConfirmation(config, {}, () => false, async (command) => {
    calls.push(command);
    return preview;
  }), false);
  assert.deepEqual(calls, ["cloud_sync_preview_namespace"]);
});

test("preview errors never confirm or start", async () => {
  await assert.rejects(startCloudSyncWithConfirmation(config, {}, () => {
    assert.fail("cannot confirm a failed preview");
  }, async (command) => {
    assert.equal(command, "cloud_sync_preview_namespace");
    throw new Error("manifest cannot be saved");
  }), /manifest cannot be saved/);
});

test("stale acknowledgement errors propagate without retry", async () => {
  const calls = [];
  await assert.rejects(startCloudSyncWithConfirmation(config, {}, () => true, async (command) => {
    calls.push(command);
    if (command === "cloud_sync_preview_namespace") return preview;
    throw new Error("Preview and confirm the current cloud folder layout");
  }), /Preview and confirm/);
  assert.deepEqual(calls, ["cloud_sync_preview_namespace", "cloud_sync_start"]);
});
