import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import test from "node:test";
import ts from "typescript";
import { collapseStackMembers, resolveStackHeroPaths } from "../src/utils/stack.ts";

function context(invoke) {
  const source = fs.readFileSync('src/App.vue','utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile('app.ts',source,ts.ScriptTarget.Latest,true);
  const hero = {id:2,path:'/matching.png',stack_id:'s',stack_order:1};
  const state = vm.createContext({ invoke, files:{value:[hero]},
    galleryContextKey:{value:'favorites'},libraryRequestVersion:1,
    expandedStacks:{value:new Set()},pendingStackExpansions:new Set(),allowMultipleStacksOpen:{value:false},
    searchQuery:{value:'prompt:cat'},isSemanticSearch:{value:false},currentPagedCriteria:()=>({is_favorite:true}),
    error:{value:''},stackMap:{value:{s:{count:2,heroId:2}}},selectedFile:{value:null},
    selectedFilePaths:{value:new Set()},selectionAnchorPath:{value:null},collapseStackMembers,resolveStackHeroPaths,
  });
  for (const name of ['onToggleStackExpand','collapseStackLocally']) {
    const fn = ast.statements.find(node=>ts.isFunctionDeclaration(node)&&node.name?.text===name);
    vm.runInContext(ts.transpile(fn.getText(ast),{target:ts.ScriptTarget.ESNext}),state);
  }
  return state;
}

test("production expansion requests scoped members and collapse removes hidden selections", async () => {
  const member = {id:3,path:'/second.png',stack_id:'s',stack_order:2};
  let calls = 0;
  const state = context(async (command,args) => {
    calls++;
    assert.equal(command,'get_filtered_stack_members');
    assert.equal(args.stackId,'s');
    assert.equal(args.query,'prompt:cat');
    assert.equal(args.context.is_favorite,true);
    return [state.files.value[0],member];
  });
  await state.onToggleStackExpand('s');
  assert.equal(state.files.value.length,2);
  assert.equal(state.expandedStacks.value.has('s'),true);
  state.selectedFile.value=member; state.selectedFilePaths.value=new Set(['/matching.png','/second.png']);
  await state.onToggleStackExpand('s');
  assert.equal(calls,1);
  assert.deepEqual([...state.selectedFilePaths.value],['/matching.png']);
  assert.equal(state.selectedFile.value.path,'/matching.png');
  assert.equal(state.selectionAnchorPath.value,'/matching.png');
});

test("production expansion deduplicates requests and rejects responses after navigation or reload", async () => {
  for (const change of ['context','revision']) {
    let resolve, calls=0;
    const state = context(()=>{calls++; return new Promise(done=>{resolve=done;});});
    const pending = state.onToggleStackExpand('s');
    await state.onToggleStackExpand('s');
    assert.equal(calls,1);
    if(change==='context') state.galleryContextKey.value='all'; else state.libraryRequestVersion++;
    state.files.value=[{id:99,path:'/new-context.png'}];
    resolve([{id:3,path:'/late.png',stack_id:'s'}]);
    await pending;
    assert.equal(state.files.value[0].id,99);
    assert.equal(state.expandedStacks.value.size,0);
    assert.equal(state.pendingStackExpansions.size,0);
  }
});
