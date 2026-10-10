import assert from "node:assert/strict";
import test from "node:test";
import { applyBooleanFlagMutation } from "../src/utils/batch-flags.ts";
import { ActionHistory } from "../src/utils/history.ts";
import fs from "node:fs";
import vm from "node:vm";
import ts from "typescript";

function loadHandler(context, name = "refreshBatchFlagMutation") {
  const script = fs.readFileSync("src/App.vue", "utf8").match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile("app.ts", script, ts.ScriptTarget.Latest, true);
  const fn = ast.statements.find(node => ts.isFunctionDeclaration(node) && node.name?.text === name);
  vm.createContext(context);
  vm.runInContext(ts.transpile(fn.getText(ast), { target: ts.ScriptTarget.ESNext }), context);
  return context[name];
}

function refreshContext() {
  const context = {
    galleryContextKey: { value: "favorites:desc" }, libraryRequestVersion: 0,
    selectedFilePaths: { value: new Set(["a", "b"]) }, selectionAnchorPath: { value: "b" },
    selectedFile: { value: { path: "b" } }, files: { value: [{ path: "a" }, { path: "b" }] },
    galleryHasMore: { value: false }, nextGalleryOffset: { value: 0 },
    refreshCounts: async () => {}, loadMoreFiles: async () => assert.fail("unexpected paging"),
  };
  context.loadFiles = async () => {
    context.libraryRequestVersion++;
    context.selectedFilePaths.value = new Set();
    context.selectionAnchorPath.value = null;
    context.files.value = [{ path: "a" }];
  };
  return context;
}

test("production refresh retains surviving selection and removes filtered anchors", async () => {
  const context = refreshContext();
  await loadHandler(context)();
  assert.deepEqual([...context.selectedFilePaths.value], ["a"]);
  assert.equal(context.selectionAnchorPath.value, null);
  assert.equal(context.selectedFile.value, null);
});

test("production refresh refills the prior extent serially and retains a reordered anchor", async () => {
  const context = refreshContext();
  let pages = 0;
  context.galleryHasMore.value = true;
  context.loadMoreFiles = async () => {
    pages++;
    context.nextGalleryOffset.value++;
    context.files.value.push({ path: "b" });
  };
  await loadHandler(context)();
  assert.equal(pages, 1);
  assert.deepEqual([...context.selectedFilePaths.value], ["a", "b"]);
  assert.equal(context.selectionAnchorPath.value, "b");
});

test("production refresh stops on non-advancing pages or navigation without restoring stale selection", async () => {
  const context = refreshContext();
  let pages = 0;
  context.galleryHasMore.value = true;
  context.loadMoreFiles = async () => { pages++; };
  await loadHandler(context)();
  assert.equal(pages, 1);
  const navigated = refreshContext();
  navigated.refreshCounts = async () => {
    navigated.libraryRequestVersion++;
    navigated.galleryContextKey.value = "all:asc";
    navigated.selectedFilePaths.value = new Set(["new-context"]);
  };
  await loadHandler(navigated)();
  assert.deepEqual([...navigated.selectedFilePaths.value], ["new-context"]);
});

test("production flag patch updates details, selected image and lightbox without reverting metadata", () => {
  const cached = new Map();
  const image = { id: 1, path: "a", rating: 5, metadata: { prompt: "new prompt" } };
  const context = { fileDetailsManager: { update: (id, patch) => cached.set(id, patch) },
    files: { value: [image, { id: 2, path: "b" }] }, selectedFile: { value: image }, lightboxFile: { value: image } };
  const patch = loadHandler(context, "patchBatchFlag");
  for (const flag of ["is_favorite", "is_nsfw"]) {
    patch([1], flag, true);
    assert.equal(cached.get(1)[flag], true);
    for (const ref of [context.files.value[0], context.selectedFile.value, context.lightboxFile.value]) {
      assert.equal(ref[flag], true);
      assert.equal(ref.rating, 5);
      assert.equal(ref.metadata.prompt, "new prompt");
    }
    assert.equal(context.files.value[1][flag], undefined);
  }
});

test("favorite/NSFW filter exits and undo/redo refresh through the same completion path", async () => {
  for (const flag of ["favorite", "nsfw"]) {
    const history = new ActionHistory();
    const stored = new Map([[1, true], [2, true]]);
    let visible = [1, 2];
    let refreshes = 0;
    const apply = values => applyBooleanFlagMutation(values,
      async (ids, value) => { for (const id of ids) stored.set(id, value); },
      () => {}, async () => { refreshes++; visible = [...stored].filter(([, value]) => value).map(([id]) => id); });
    await history.execute({ name: flag, execute: () => apply(new Map([[1, false]])), undo: () => apply(new Map([[1, true]])) });
    assert.deepEqual(visible, [2]);
    await history.undo();
    assert.deepEqual(visible, [1, 2]);
    await history.redo();
    assert.deepEqual(visible, [2]);
    assert.equal(refreshes, 3);
  }
});

test("partial group failure patches only confirmed writes and still refreshes authoritative state", async () => {
  const applied = [];
  let refreshes = 0;
  await assert.rejects(applyBooleanFlagMutation(new Map([[1, true], [2, false]]),
    async (ids, value) => { if (!value) throw new Error("IPC rejected"); },
    (ids, value) => applied.push([ids, value]), async () => { refreshes++; }), /IPC rejected/);
  assert.deepEqual(applied, [[[1], true]]);
  assert.equal(refreshes, 1);
});

test("first group rejection still reconciles a potentially partial backend mutation", async () => {
  let refreshes = 0;
  await assert.rejects(applyBooleanFlagMutation(new Map([[1, true]]),
    async () => { throw new Error("partial backend failure"); },
    () => assert.fail("unconfirmed flag must not be patched"), async () => { refreshes++; }));
  assert.equal(refreshes, 1);
});
