import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import {
  clearSensitiveReveals,
  isSensitiveMasked,
  isSensitiveRevealed,
  remaskSensitive,
  revealSensitive,
  toggleSensitiveReveal,
  useGalleryPrivacy,
} from "../src/utils/gallery-privacy.ts";

class MockElement {
  constructor(tagName = "DIV") {
    this.tagName = tagName;
  }
  closest() {
    return null;
  }
}

if (!globalThis.HTMLElement) {
  globalThis.HTMLElement = MockElement;
}
if (!globalThis.window) {
  globalThis.window = {
    addEventListener: () => {},
    removeEventListener: () => {},
    __TAURI_INTERNALS__: {
      convertFileSrc: (p) => `asset://${p}`,
    },
    location: { origin: "http://localhost" },
  };
}
if (!globalThis.document) {
  globalThis.document = {
    activeElement: null,
  };
}

function loadProductionComponent(relativePath, componentId) {
  const filePath = path.resolve(relativePath);
  const sfc = fs.readFileSync(filePath, "utf-8");
  const { descriptor } = parse(sfc, { filename: path.basename(filePath) });

  const script = compileScript(descriptor, { id: componentId, inlineTemplate: false });
  const template = compileTemplate({
    id: componentId,
    filename: path.basename(filePath),
    source: descriptor.template.content,
    ssr: true,
    compilerOptions: { bindingMetadata: script.bindings },
  });

  let scriptCode = script.content.replace("export default", "const component =");

  const mockI18n = `
  const i18nData = {
    view: { loading: "Loading...", selectAll: "Select All" },
    review: { noMatches: "No items match your filter", retry: "Reset Filters", gallery: "Gallery Table" },
    sort: { name: "Name", size: "Size", modified: "Modified" },
    preview: {
      preview: "Preview",
      container: "Format",
      platform: "Platform",
      prompt: "Prompt",
      dimensions: "Dimensions",
      modelName: "Model",
      retryThumbnail: "Retry loading thumbnail",
      clickToReveal: "Click to reveal sensitive content",
      remaskContent: "Conceal sensitive content",
      remaskShortcut: "Reveal / Conceal Sensitive Content (R)",
    },
  };
  const t = new Proxy(i18nData, {
    get(target, prop) {
      if (prop === "value") return target;
      return target[prop];
    },
  });
  `;

  scriptCode = scriptCode.replace(/import { t } from [^;]+;/, mockI18n);
  scriptCode = scriptCode.replace(/(['"])\.\.\/utils\//g, "$1../src/utils/");
  scriptCode = scriptCode.replace(/import type [^\n]+;/g, "");

  let templateCode = template.code.replace("export function ssrRender", "function ssrRender");
  const fullCode = `${scriptCode}\n${templateCode}\ncomponent.ssrRender = ssrRender;\nexport default component;`;

  const transformed = ts.transpileModule(fullCode, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ESNext },
  });

  const tmpPath = path.resolve(
    "tests",
    `.tmp-trans-${componentId}-${Date.now()}-${Math.random().toString(36).slice(2)}.mjs`
  );
  fs.writeFileSync(tmpPath, transformed.outputText);

  return {
    tmpPath,
    async importComponent() {
      return (await import(`file:///${tmpPath.replace(/\\/g, "/")}`)).default;
    },
  };
}

test("Gallery privacy transitions: Mode changes (Table -> Grid -> Waterfall) never reveal concealed items", async () => {
  clearSensitiveReveals();
  const fileListLoader = loadProductionComponent("src/components/FileList.vue", "test-trans-filelist");
  const gridLoader = loadProductionComponent("src/components/VirtualGrid.vue", "test-trans-grid");

  try {
    const FileList = await fileListLoader.importComponent();
    const VirtualGrid = await gridLoader.importComponent();

    const sensitiveFile = { id: 101, path: "/lib/flagged.png", is_nsfw: true, container: "png" };
    const cleanFile = { id: 102, path: "/lib/clean.png", is_nsfw: false, container: "png" };
    const files = [sensitiveFile, cleanFile];

    const privacy = useGalleryPrivacy();

    // 1. Render in Table Mode
    const tableApp = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: true,
      revealedNsfw: privacy.revealedPaths.value,
    });
    const tableHtml = await renderToString(tableApp);
    assert.ok(tableHtml.includes("nsfw-blurred"), "Table mode: sensitive file must be blurred");
    assert.ok(tableHtml.includes("table-nsfw-overlay"), "Table mode: sensitive file renders overlay button");

    // 2. Switch to Grid Mode WITHOUT revealing -> must NOT reveal
    const gridApp = createSSRApp(VirtualGrid, {
      files,
      layout: "grid",
      blurNsfw: true,
      revealedNsfw: privacy.revealedPaths.value,
    });
    const gridHtml = await renderToString(gridApp);
    assert.ok(gridHtml.includes("nsfw-blurred"), "Grid mode: sensitive file remains blurred after transition");
    assert.ok(gridHtml.includes('class="nsfw-overlay"'), "Grid mode: sensitive file renders overlay after transition");
    assert.ok(!gridHtml.includes("card-remask-btn"), "Grid mode: concealed item does not render remask button");

    // 3. Switch to Waterfall Mode WITHOUT revealing -> must NOT reveal
    const waterfallApp = createSSRApp(VirtualGrid, {
      files,
      layout: "masonry",
      blurNsfw: true,
      revealedNsfw: privacy.revealedPaths.value,
    });
    const waterfallHtml = await renderToString(waterfallApp);
    assert.ok(waterfallHtml.includes("nsfw-blurred"), "Waterfall mode: sensitive file remains blurred");
    assert.ok(waterfallHtml.includes('class="nsfw-overlay"'), "Waterfall mode: sensitive file renders overlay");

    // Classification flags must remain unchanged
    assert.equal(sensitiveFile.is_nsfw, true);
    assert.equal(cleanFile.is_nsfw, false);
  } finally {
    if (fs.existsSync(fileListLoader.tmpPath)) fs.unlinkSync(fileListLoader.tmpPath);
    if (fs.existsSync(gridLoader.tmpPath)) fs.unlinkSync(gridLoader.tmpPath);
    clearSensitiveReveals();
  }
});

test("Gallery privacy transitions: Intentional reveal persists across mode switches and can be re-masked", async () => {
  clearSensitiveReveals();
  const fileListLoader = loadProductionComponent("src/components/FileList.vue", "test-reveal-filelist");
  const gridLoader = loadProductionComponent("src/components/VirtualGrid.vue", "test-reveal-grid");

  try {
    const FileList = await fileListLoader.importComponent();
    const VirtualGrid = await gridLoader.importComponent();

    const sensitiveFile = { id: 201, path: "/lib/sensitive201.jpg", is_nsfw: true, container: "jpg" };
    const files = [sensitiveFile];
    const privacy = useGalleryPrivacy();

    // 1. Initially concealed in Table
    let tableApp = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: true,
      revealedNsfw: privacy.revealedPaths.value,
    });
    let tableHtml = await renderToString(tableApp);
    assert.ok(tableHtml.includes("nsfw-blurred"));

    // 2. User intentionally reveals sensitive item (e.g. clicked reveal in Table or pressed R)
    privacy.reveal(sensitiveFile.path);
    assert.equal(privacy.isRevealed(sensitiveFile.path), true);

    // Render Table after reveal: blur removed, remask button visible
    tableApp = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: true,
      revealedNsfw: privacy.revealedPaths.value,
    });
    tableHtml = await renderToString(tableApp);
    assert.ok(!tableHtml.includes("nsfw-blurred"), "Table: blur removed after intentional reveal");
    assert.ok(tableHtml.includes("table-nsfw-remask-btn"), "Table: remask button present after reveal");

    // 3. User switches to Grid view: Item MUST remain revealed in Grid view
    let gridApp = createSSRApp(VirtualGrid, {
      files,
      layout: "grid",
      blurNsfw: true,
      revealedNsfw: privacy.revealedPaths.value,
    });
    let gridHtml = await renderToString(gridApp);
    assert.ok(!gridHtml.includes("nsfw-blurred"), "Grid: blur removed for intentionally revealed item");
    assert.ok(gridHtml.includes("card-remask-btn"), "Grid: remask button present on revealed card");

    // 4. User clicks re-mask in Grid view
    privacy.remask(sensitiveFile.path);
    assert.equal(privacy.isRevealed(sensitiveFile.path), false);

    // 5. User switches back to Table view: Item MUST be concealed again
    tableApp = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: true,
      revealedNsfw: privacy.revealedPaths.value,
    });
    tableHtml = await renderToString(tableApp);
    assert.ok(tableHtml.includes("nsfw-blurred"), "Table: item is concealed again after remasking");
    assert.ok(tableHtml.includes("table-nsfw-overlay"), "Table: reveal overlay restored after remasking");
    assert.ok(!tableHtml.includes("table-nsfw-remask-btn"), "Table: remask button removed after concealment");

    // Classification flag is untouched throughout
    assert.equal(sensitiveFile.is_nsfw, true);
  } finally {
    if (fs.existsSync(fileListLoader.tmpPath)) fs.unlinkSync(fileListLoader.tmpPath);
    if (fs.existsSync(gridLoader.tmpPath)) fs.unlinkSync(gridLoader.tmpPath);
    clearSensitiveReveals();
  }
});

test("Gallery privacy settings: Toggling blurNsfw preference updates masking across views", async () => {
  clearSensitiveReveals();
  const fileListLoader = loadProductionComponent("src/components/FileList.vue", "test-setting-filelist");
  const gridLoader = loadProductionComponent("src/components/VirtualGrid.vue", "test-setting-grid");

  try {
    const FileList = await fileListLoader.importComponent();
    const VirtualGrid = await gridLoader.importComponent();

    const sensitiveFile = { id: 301, path: "/lib/target.png", is_nsfw: true, container: "png" };
    const files = [sensitiveFile];

    // Disabled blur preference (blurNsfw: false)
    const tableDisabled = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: false,
      revealedNsfw: new Set(),
    });
    const tableDisabledHtml = await renderToString(tableDisabled);
    assert.ok(!tableDisabledHtml.includes("nsfw-blurred"), "Table: no blur when blurNsfw=false");
    assert.ok(!tableDisabledHtml.includes("table-nsfw-overlay"), "Table: no overlay when blurNsfw=false");

    const gridDisabled = createSSRApp(VirtualGrid, {
      files,
      blurNsfw: false,
      revealedNsfw: new Set(),
    });
    const gridDisabledHtml = await renderToString(gridDisabled);
    assert.ok(!gridDisabledHtml.includes("nsfw-blurred"), "Grid: no blur when blurNsfw=false");
    assert.ok(!gridDisabledHtml.includes("nsfw-overlay"), "Grid: no overlay when blurNsfw=false");
  } finally {
    if (fs.existsSync(fileListLoader.tmpPath)) fs.unlinkSync(fileListLoader.tmpPath);
    if (fs.existsSync(gridLoader.tmpPath)) fs.unlinkSync(gridLoader.tmpPath);
    clearSensitiveReveals();
  }
});
