import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import vm from "node:vm";
import { pathToFileURL } from "node:url";
import ts from "typescript";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import { en } from "../src/i18n/locales/en.ts";
import { zhCN } from "../src/i18n/locales/zh-CN.ts";
import { zhTW } from "../src/i18n/locales/zh-TW.ts";
import { ja } from "../src/i18n/locales/ja.ts";
import { de } from "../src/i18n/locales/de.ts";
import { fr } from "../src/i18n/locales/fr.ts";
import { es } from "../src/i18n/locales/es.ts";

const locales = [en, zhCN, zhTW, ja, de, fr, es];
const escapeHtml = text => text.replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');

async function component(name, locale, run) {
  const { descriptor } = parse(fs.readFileSync(`src/components/${name}.vue`, "utf8"));
  const script = compileScript(descriptor, { id: name });
  const template = compileTemplate({ id: name, filename: name, source: descriptor.template.content,
    ssr: true, compilerOptions: { bindingMetadata: script.bindings } });
  let code = script.content.replace('export default', 'const component =')
    .replace(/import { t } from [^;]+;/, `const locale = ${JSON.stringify(locale)}; const t = new Proxy(locale, {get:(data,key)=>key === 'value' ? data : data[key]});`)
    .replace(/import { invoke } from [^;]+;/, 'const invoke = () => new Promise(() => {});')
    .replace(/import { open as openDialog } from [^;]+;/, 'const openDialog = async () => null;')
    .replace(/import { useNotification } from [^;]+;/, 'const useNotification = () => ({});')
    .replace(/import FolderTreeNode from [^;]+;/, 'const FolderTreeNode = {ssrRender(){}};')
    .replace(/import type [^;]+;/g, '');
  code = 'const window = {addEventListener(){},removeEventListener(){}};\n' + code;
  code += '\n' + template.code.replace('export function ssrRender', 'function ssrRender') + '\ncomponent.ssrRender = ssrRender; export default component;';
  const temporary = path.resolve('tests', `.tmp-review-${name}-${Math.random()}.mjs`);
  fs.writeFileSync(temporary, ts.transpileModule(code, {compilerOptions:{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ESNext}}).outputText);
  try { await run((await import(pathToFileURL(temporary).href)).default); }
  finally { fs.unlinkSync(temporary); }
}

test("production statistics render loading, empty and populated hints in every locale", async () => {
  for (const locale of locales) {
    await component('PromptStatsModal', locale, async modal => {
      let fixture = { loading: true, stats: null };
      const setup = modal.setup;
      modal.setup = (props, context) => {
        const state = setup(props, context);
        state.loading.value = fixture.loading;
        state.stats.value = fixture.stats;
        return state;
      };
      const render = () => {
        const app = createSSRApp(modal, {open:true});
        app.directive('dialog', {getSSRProps:()=>({})});
        return renderToString(app);
      };
      let html = await render();
      for (const key of ['loading', 'refreshing', 'close', 'tip', 'done']) {
        assert.ok(html.includes(escapeHtml(locale.promptStatsModal[key])), key);
      }
      fixture = { loading:false, stats:{total_analyzed:1,top_positive_words:[],top_negative_words:[],top_models:[],top_samplers:[]} };
      html = await render();
      assert.ok(html.includes(escapeHtml(locale.promptStatsModal.empty)));
      fixture.stats.top_positive_words = [{keyword:'fixture',count:1}];
      html = await render();
      assert.ok(html.includes(escapeHtml(locale.promptStatsModal.searchKeyword)));
      assert.ok(html.includes(escapeHtml(locale.promptStatsModal.filterLibrary)));
    });
  }
});

test("production Sidebar landmarks use each selected locale", async () => {
  for (const locale of locales) {
    await component('Sidebar', locale, async sidebar => {
      const html = await renderToString(createSSRApp(sidebar, {folders:[],tags:[],albums:[],activeTarget:{type:'all'},progress:null}));
      assert.ok(html.includes(`aria-label="${escapeHtml(locale.nav.navigationLabel)}"`));
      assert.ok(html.includes(`aria-label="${escapeHtml(locale.nav.toolsLabel)}"`));
    });
  }
});

test("production history handlers translate actions and current-locale success/failure notifications", async () => {
  const source = fs.readFileSync('src/App.vue','utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile('app.ts', source, ts.ScriptTarget.Latest, true);
  for (const locale of locales) {
    const actions = [], messages = [];
    const context = vm.createContext({t:{value:locale},selectedFilesList:{value:[{id:1,path:'/fixture.png'}]},
      actionHistory:{execute:async action=>actions.push(action),undo:async()=>actions[0].name,redo:async()=>actions[0].name},
      notification:{showSuccess:message=>messages.push(message),showError:message=>messages.push(message)},error:{value:''},
    });
    for (const name of ['onBatchRate','onBatchToggleFavorite','onBatchToggleNsfw','onUndo','onRedo']) {
      const fn = ast.statements.find(node=>ts.isFunctionDeclaration(node)&&node.name?.text===name);
      vm.runInContext(ts.transpile(fn.getText(ast),{target:ts.ScriptTarget.ESNext}),context);
    }
    await context.onBatchRate(3);
    await context.onBatchToggleFavorite(true);
    await context.onBatchToggleFavorite(false);
    await context.onBatchToggleNsfw(true);
    await context.onBatchToggleNsfw(false);
    assert.deepEqual(actions.map(a=>a.name), ['rating','favorite','unfavorite','sensitive','safe'].map(key=>locale.history[key].replace('{rating}','3').replace('{count}','1')));
    await context.onUndo(); await context.onRedo();
    assert.deepEqual(messages, ['undid','redid'].map(key=>locale.history[key].replace('{action}',actions[0].name)));
    context.actionHistory.undo = context.actionHistory.redo = async()=>{throw 'fixture error';};
    await context.onUndo(); await context.onRedo();
    assert.deepEqual(messages.slice(2), ['undoFailed','redoFailed'].map(key=>locale.history[key].replace('{error}','fixture error')));
  }
});
