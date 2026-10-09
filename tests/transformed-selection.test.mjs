import assert from "node:assert/strict";
import test from "node:test";
import { FileDetailsManager, refreshPublishedSelections } from "../src/utils/file-details.ts";

const source = { id: 1, path: "/source.png", modified_at: 1, rating: 8, is_favorite: true, is_nsfw: false, metadata: { width: 64, height: 32 } };
const output = { ...source, path: "/source_1.png", modified_at: 2, metadata: { format: null, width: 32, height: 16 }, rating: 2 };
const receipt = [{ source_id_or_path: source.path, output_id_or_path: output.path, status: "succeeded" }];

test("successful transform refreshes open details, revision and cache with one deduplicated fetch", async () => {
  const manager = new FileDetailsManager();
  manager.set(source);
  manager.reset(); // Same reset used by the post-transform gallery reload.
  let selections = [{ ...source }, { ...source }];
  let fetches = 0;
  await refreshPublishedSelections(receipt, manager, () => selections, async () => {
    fetches++;
    selections[0].rating = 9; // Edit while IPC is in flight must survive.
    return output;
  }, (previous, published) => {
    selections = selections.map((file) => FileDetailsManager.revisionKey(file) === FileDetailsManager.revisionKey(previous) ? published : file);
  });
  assert.equal(fetches, 1);
  for (const file of selections) {
    assert.equal(file.path, output.path);
    assert.deepEqual([file.metadata.width, file.metadata.height], [32, 16]);
    assert.equal(file.rating, 9);
  }
  assert.equal(manager.get(output).path, output.path);
  assert.equal(manager.get(source), null);
});

test("a failed, skipped or unrelated transform does not fetch selected details", async () => {
  for (const items of [[], [{ ...receipt[0], status: "failed" }], [{ ...receipt[0], status: "skipped" }], [{ ...receipt[0], source_id_or_path: "/other.png" }]]) {
    await refreshPublishedSelections(items, new FileDetailsManager(), () => [source, null], async () => assert.fail("unexpected fetch"), () => assert.fail("unexpected replacement"));
  }
});

test("publication refresh rejects stale generation, selection, path and identity", async () => {
  for (const mutation of ["reset", "navigate", "new-revision", "wrong-path", "wrong-id"]) {
    const manager = new FileDetailsManager();
    let selection = source;
    let resolve;
    const pending = refreshPublishedSelections(receipt, manager, () => [selection], () => new Promise((done) => { resolve = done; }), () => assert.fail(`stale replacement: ${mutation}`));
    let response = output;
    if (mutation === "reset") manager.reset();
    if (mutation === "navigate") selection = { ...source, id: 2 };
    if (mutation === "new-revision") selection = { ...source, modified_at: 10 };
    if (mutation === "wrong-path") response = { ...output, path: "/unexpected.png" };
    if (mutation === "wrong-id") response = { ...output, id: 2 };
    resolve(response);
    await pending;
    assert.equal(manager.size, 0);
  }
});
