import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import test from "node:test";
import ts from "typescript";
import { collapseStackMembers } from "../src/utils/stack.ts";

function selectionContext(files) {
  const source = fs.readFileSync("src/App.vue", "utf8").match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile("app.ts", source, ts.ScriptTarget.Latest, true);
  const context = vm.createContext({
    files: { value: files }, selectedFilePaths: { value: new Set() },
    selectedFile: { value: null }, selectionAnchorPath: { value: null },
    invoke: () => assert.fail("selection must not fetch the result population"),
  });
  for (const name of ["onSelectAll", "onClearSelection", "onToggleAll"]) {
    const fn = ast.statements.find(node => ts.isFunctionDeclaration(node) && node.name?.text === name);
    assert.ok(fn, `production ${name}`);
    vm.runInContext(ts.transpile(fn.getText(ast), { target: ts.ScriptTarget.ESNext }), context);
  }
  return context;
}

test("production bulk selection stays bounded to loaded pages until repeated", () => {
  const all = Array.from({ length: 1276 }, (_, id) => ({ id, path: `/${id}.png` }));
  const context = selectionContext(all.slice(0, 400));
  context.onSelectAll();
  assert.equal(context.selectedFilePaths.value.size, 400);
  context.files.value = all.slice(0, 800);
  assert.equal(context.selectedFilePaths.value.size, 400);
  context.onSelectAll();
  assert.equal(context.selectedFilePaths.value.size, 800);
  context.onToggleAll();
  assert.equal(context.selectedFilePaths.value.size, 0);
  context.files.value = all.filter(file => file.id % 100 === 0);
  context.onSelectAll();
  assert.deepEqual([...context.selectedFilePaths.value], context.files.value.map(file => file.path));
  context.onClearSelection();
  assert.equal(context.selectedFilePaths.value.size, 0);
  assert.equal(context.selectedFile.value, null);
  assert.equal(context.selectionAnchorPath.value, null);
});

test("production bulk selection excludes collapsed stack members until expanded", () => {
  const files = [0, 1, 2].map(id => ({ id, path: `/${id}.png`, stack_id: "s", stack_order: id }));
  const context = selectionContext(collapseStackMembers(files, { s: { count: 3, heroId: 0 } }));
  context.onSelectAll();
  assert.deepEqual([...context.selectedFilePaths.value], ["/0.png"]);
  context.files.value = files;
  assert.equal(context.selectedFilePaths.value.size, 1);
  context.onSelectAll();
  assert.deepEqual([...context.selectedFilePaths.value], files.map(file => file.path));
});
