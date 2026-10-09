import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { randomUUID } from "node:crypto";
import { pathToFileURL } from "node:url";
import test from "node:test";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createSSRApp, h } from "vue";
import { renderToString } from "vue/server-renderer";

async function renderModal(name, handler, props) {
  const filename = `${name}.vue`;
  const source = fs.readFileSync(path.resolve("src/components", filename), "utf8");
  const { descriptor } = parse(source.replace("</script>",
    `\ndefineExpose({ format, error, ${handler} });\n</script>`), { filename });
  const script = compileScript(descriptor, { id: name });
  const template = compileTemplate({ source: descriptor.template.content, filename, id: name,
    ssr: true, ssrCssVars: [], compilerOptions: { bindingMetadata: script.bindings } });
  assert.deepEqual(template.errors, []);
  let code = script.content
    .replace('import { invoke } from "@tauri-apps/api/core";', "const invoke = async () => { throw new Error('unexpected IPC'); };")
    .replace('import { listen, type UnlistenFn } from "@tauri-apps/api/event";', "type UnlistenFn = () => void; const listen = async () => () => {};")
    .replace('from "../i18n"', 'from "../src/i18n/index.ts"')
    .replace('export default', 'const component =');
  code += `\n${template.code.replace('export function ssrRender', 'function ssrRender')}\ncomponent.ssrRender = ssrRender;\nexport default component;`;
  const file = path.resolve("tests", `.tmp-avif-${randomUUID()}.mjs`);
  fs.writeFileSync(file, ts.transpileModule(code, { compilerOptions: {
    module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ESNext,
  } }).outputText);
  try {
    const component = (await import(pathToFileURL(file).href)).default;
    let controls;
    const setup = component.setup;
    component.setup = (props, context) => setup(props, { ...context,
      expose(value) { controls = value; context.expose(value); },
    });
    const app = createSSRApp({ render: () => h(component, props) });
    app.directive("dialog", { getSSRProps: () => ({}) });
    const html = await renderToString(app);
    return { html, controls };
  } finally {
    fs.unlinkSync(file);
  }
}

const folders = [{ id: 1, folder_type: "managed", path: "/vault" }];

for (const [name, handler, props] of [
  ["ImportTransformModal", "handleStartImport", { show: true, filePaths: ["/source.png"], folders, albums: [] }],
  ["BatchTransformModal", "handleStartTransform", { show: true, files: [{ id: 1, folder_id: 1, path: "/source.png" }], folders }],
]) {
  test(`${name} disables AVIF choices and rejects stale AVIF state without IPC`, async () => {
    const { html, controls } = await renderModal(name, handler, props);
    assert.match(html, /<option[^>]*value="avif"[^>]*disabled/);
    assert.match(html, /<option[^>]*value="archive_avif"[^>]*disabled/);
    assert.match(html, /AVIF is export-only/);
    controls.format.value = "avif";
    await controls[handler]();
    assert.match(controls.error.value, /AVIF is export-only/);
  });
}
