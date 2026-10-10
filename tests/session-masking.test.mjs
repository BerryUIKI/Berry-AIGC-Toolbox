import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import test from "node:test";
import ts from "typescript";
import { ref } from "vue";
import { setSensitiveMasking, isSensitiveMasked } from "../src/utils/gallery-privacy.ts";

function setup() {
  const script = fs.readFileSync('src/App.vue','utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile('app.ts',script,ts.ScriptTarget.Latest,true);
  const writes=[];
  const state=vm.createContext({setSensitiveMasking,blurNsfw:ref(true),revealedNsfw:ref(new Set(['/sensitive.png'])),
    document:{activeElement:null},isEditableTarget:target=>!!target?.editable,hasActiveDialog:()=>false,
    viewMode:ref('grid'),setStorageItem:(...args)=>writes.push(args),showCardBadges:ref(true),
    invoke:()=>assert.fail('session masking must not invoke IPC'),saveAppConfig:()=>assert.fail('session masking must not save config'),
  });
  for(const name of ['onToggleGlobalMasking','handleWindowKeyDown','setViewMode','onSettingsSaved']) {
    const fn=ast.statements.find(node=>ts.isFunctionDeclaration(node)&&node.name?.text===name);
    vm.runInContext(ts.transpile(fn.getText(ast),{target:ts.ScriptTarget.ESNext}),state);
  }
  return {state,writes};
}

test("production session toggle clears prior reveals on enabling and survives view/navigation changes without saving", () => {
  const {state,writes}=setup();
  const sensitive={path:'/sensitive.png',is_nsfw:true};
  const safe={path:'/safe.png',is_nsfw:false};
  state.onToggleGlobalMasking();
  assert.equal(state.blurNsfw.value,false);
  assert.equal(isSensitiveMasked(sensitive,state.blurNsfw.value,state.revealedNsfw.value),false);
  for(const mode of ['table','masonry','grid']) {
    state.setViewMode(mode);
    assert.equal(state.blurNsfw.value,false);
  }
  state.onToggleGlobalMasking();
  assert.equal(state.revealedNsfw.value.size,0);
  assert.equal(isSensitiveMasked(sensitive,state.blurNsfw.value,state.revealedNsfw.value),true);
  assert.equal(isSensitiveMasked(safe,state.blurNsfw.value,state.revealedNsfw.value),false);
  assert.equal(sensitive.is_nsfw,true);
  assert.ok(writes.every(([key])=>key==='default_view'));
  state.onToggleGlobalMasking();
  state.onSettingsSaved({blurNsfw:true,showCardBadges:true,defaultView:'table'});
  assert.equal(state.blurNsfw.value,true);
  assert.equal(state.revealedNsfw.value.size,0);
});

test("production Shift+M respects editable focus, dialogs, modifiers and handled events", () => {
  const {state}=setup();
  const event=()=>({key:'M',shiftKey:true,ctrlKey:false,metaKey:false,altKey:false,defaultPrevented:false,target:null,
    preventDefault(){this.defaultPrevented=true;}});
  const enabled=event(); state.handleWindowKeyDown(enabled);
  assert.equal(state.blurNsfw.value,false); assert.equal(enabled.defaultPrevented,true);
  for(const block of ['target','focus','dialog','handled','ctrl','meta','alt']) {
    const e=event();
    state.document.activeElement=null; state.hasActiveDialog=()=>false;
    if(block==='target') e.target={editable:true};
    if(block==='focus') state.document.activeElement={editable:true};
    if(block==='dialog') state.hasActiveDialog=()=>true;
    if(block==='handled') e.defaultPrevented=true;
    if(['ctrl','meta','alt'].includes(block)) e[block+'Key']=true;
    state.handleWindowKeyDown(e);
    assert.equal(state.blurNsfw.value,false,block);
  }
});
