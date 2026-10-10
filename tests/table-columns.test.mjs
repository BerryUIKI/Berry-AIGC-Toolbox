import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import ts from "typescript";
import { normalizeTableColumns } from "../src/utils/table-columns.ts";

test("legacy and malformed Table preferences resolve to independent compact defaults", () => {
  const first = normalizeTableColumns(undefined);
  assert.deepEqual(first.filter(c=>c.visible).map(c=>c.id), ['preview','name','prompt','model']);
  assert.ok(first.filter(c=>c.visible).reduce((sum,c)=>sum+c.width,28)<=460);
  first[0].width=600;
  assert.equal(normalizeTableColumns([])[0].width,44);
  assert.deepEqual(normalizeTableColumns('invalid'),normalizeTableColumns([]));
});

test("Table preference normalization keeps required name, ignores unknown IDs and bounds widths", () => {
  const columns = normalizeTableColumns([{id:'name',visible:false,width:-10},{id:'prompt',visible:true,width:1000},
    {id:'model',visible:false,width:NaN},{id:'unknown',visible:true,width:123},{id:'name',visible:true,width:600}]);
  assert.deepEqual(columns.find(c=>c.id==='name'),{id:'name',visible:true,width:80});
  assert.equal(columns.find(c=>c.id==='prompt').width,640);
  assert.deepEqual(columns.find(c=>c.id==='model'),{id:'model',visible:false,width:100});
  assert.equal(columns.length,9);
  assert.deepEqual(normalizeTableColumns(JSON.parse(JSON.stringify(columns))),columns);
});


test("production column saves retain drafts on failure, guard overlap, and preserve unrelated config", async () => {
  const script = fs.readFileSync('src/App.vue','utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile('app.ts',script,ts.ScriptTarget.Latest,true);
  const fn = ast.statements.find(node=>ts.isFunctionDeclaration(node)&&node.name?.text==='onSaveTableColumns');
  const original = normalizeTableColumns([]);
  const draft = normalizeTableColumns([{id:'prompt',visible:true,width:320}]);
  let resolve, calls=0;
  const state = vm.createContext({normalizeTableColumns,tableColumns:{value:original},tableColumnsSaving:{value:false},tableColumnsError:{value:''},
    invoke:async command=>{ assert.equal(command,'get_app_config'); return {config_revision:7,theme:'violet'}; },
    saveAppConfig:async config=>{calls++; assert.equal(config.config_revision,7); assert.equal(config.theme,'violet'); await new Promise(done=>{resolve=done;}); throw 'conflict';},
  });
  vm.runInContext(ts.transpile(fn.getText(ast),{target:ts.ScriptTarget.ESNext}),state);
  const pending = state.onSaveTableColumns(draft);
  await Promise.resolve();
  await state.onSaveTableColumns(draft);
  assert.equal(calls,1); assert.equal(state.tableColumnsSaving.value,true);
  resolve(); await pending;
  assert.equal(state.tableColumns.value,original);
  assert.equal(state.tableColumnsError.value,'conflict');
  assert.equal(state.tableColumnsSaving.value,false);
  state.saveAppConfig=async config=>{config.config_revision=8;};
  await state.onSaveTableColumns(draft);
  assert.deepEqual(state.tableColumns.value,draft);
  assert.equal(state.tableColumnsError.value,'');
  state.invoke=async()=>{throw 'read failed';};
  state.saveAppConfig=async()=>assert.fail('failed read must not save fallback defaults');
  await state.onSaveTableColumns(original);
  assert.equal(state.tableColumnsError.value,'read failed');
  assert.deepEqual(state.tableColumns.value,draft);
});
