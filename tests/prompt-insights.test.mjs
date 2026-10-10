import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import test from "node:test";
import ts from "typescript";

function setup(invoke) {
  const source = fs.readFileSync("src/components/PromptStatsModal.vue", "utf8")
    .match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile("insights.ts", source, ts.ScriptTarget.Latest, true);
  const fn = ast.statements.find(node => ts.isFunctionDeclaration(node) && node.name?.text === "loadStats");
  const context = { invoke, loading: { value: false }, error: { value: "" }, stats: { value: { total_analyzed: 999 } } };
  vm.createContext(context);
  vm.runInContext(ts.transpile(fn.getText(ast)), context);
  return context;
}

test("statistics use the production population and populated model/sampler lists", async () => {
  const response = { total_analyzed: 3, top_positive_words: [], top_negative_words: [],
    top_models: [{ keyword: "model A", count: 2 }], top_samplers: [{ keyword: "Euler", count: 2 }] };
  const state = setup(async (command, args) => {
    assert.equal(command, "get_prompt_insights");
    assert.equal(args.limit, 40);
    return response;
  });
  await state.loadStats();
  assert.equal(state.stats.value, response);
  assert.equal(state.error.value, "");
  assert.equal(state.loading.value, false);
});

test("empty statistics remain distinct from failed refresh and stale results are cleared", async () => {
  const empty = { total_analyzed: 0, top_positive_words: [], top_negative_words: [], top_models: [], top_samplers: [] };
  const state = setup(async () => empty);
  await state.loadStats();
  assert.equal(state.stats.value, empty);
  state.invoke = async () => { throw new Error("database unavailable"); };
  await state.loadStats();
  assert.equal(state.stats.value, null);
  assert.match(state.error.value, /database unavailable/);
  assert.equal(state.loading.value, false);
});
