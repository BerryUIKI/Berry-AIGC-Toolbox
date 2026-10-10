import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import { en } from "../src/i18n/locales/en.ts";

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
      convertFileSrc: (path) => `asset://${path}`,
    },
    location: { origin: "http://localhost" },
  };
}
if (!globalThis.document) {
  globalThis.document = {
    activeElement: null,
  };
}

function loadProductionFileList(componentId, viewMessages = {}) {
  const filePath = path.resolve("src/components/FileList.vue");
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

  // Mock i18n
  const mockI18n = `
  const i18nData = {
    stack: ${JSON.stringify(en.stack)},
    nav: ${JSON.stringify(en.nav)},
    view: ${JSON.stringify({ ...en.view, ...viewMessages })},
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
    `.tmp-filelist-${componentId}-${Date.now()}-${Math.random().toString(36).slice(2)}.mjs`
  );
  fs.writeFileSync(tmpPath, transformed.outputText);

  return {
    tmpPath,
    cssContent: descriptor.styles.map((s) => s.content).join("\n"),
    async importComponent() {
      const mod = await import(pathToFileURL(tmpPath).href);
      return mod.default;
    },
  };
}

test("FileList renders masked thumbnails and accessible reveal overlay for sensitive items", async () => {
  const loader = loadProductionFileList("test-render-masked");
  try {
    const FileList = await loader.importComponent();

    const files = [
      { id: 1, path: "/lib/sfw.jpg", is_nsfw: false, container: "jpg", size_bytes: 1024, modified_at: 100 },
      { id: 2, path: "/lib/sensitive.png", is_nsfw: true, container: "png", size_bytes: 2048, modified_at: 200 },
      { id: 3, path: "/lib/video.mp4", is_nsfw: true, container: "mp4", size_bytes: 4096, modified_at: 300 },
    ];

    // Case 1: Default blur preference (blurNsfw: true), no revealed items
    const app = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: true,
      revealedNsfw: new Set(),
    });
    const html = await renderToString(app);

    // Verify .table-thumb-wrapper exists
    assert.ok(html.includes('class="table-thumb-wrapper"'), "Preview cell wraps thumbnails in table-thumb-wrapper");

    // Verify sensitive items receive nsfw-blurred class
    assert.ok(html.includes("nsfw-blurred"), "Sensitive rows must have nsfw-blurred class applied");

    // Verify reveal button is rendered with accessible localized title and aria-label
    assert.ok(html.includes('class="table-nsfw-overlay"'), "Must render table-nsfw-overlay button for masked rows");
    assert.ok(html.includes('title="Click to reveal sensitive content"'), "Reveal button must have localized title");
    assert.ok(html.includes('aria-label="Click to reveal sensitive content"'), "Reveal button must have accessible aria-label");
    assert.ok(html.includes("🔞"), "Reveal button shows sensitive warning badge icon");

    // Case 2: blurNsfw disabled (blurNsfw: false)
    const appDisabled = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: false,
      revealedNsfw: new Set(),
    });
    const htmlDisabled = await renderToString(appDisabled);
    assert.ok(!htmlDisabled.includes("table-nsfw-overlay"), "No reveal overlay when blurNsfw is disabled");
    assert.ok(!htmlDisabled.includes("nsfw-blurred"), "No blur applied when blurNsfw is disabled");

    // Case 3: One item revealed in revealedNsfw set
    const revealedSet = new Set(["/lib/sensitive.png"]);
    const appRevealed = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: true,
      revealedNsfw: revealedSet,
    });
    const htmlRevealed = await renderToString(appRevealed);

    // Revealed item renders re-mask button with localized title and icon
    assert.ok(htmlRevealed.includes('class="table-nsfw-remask-btn"'), "Revealed sensitive row must render re-mask button");
    assert.ok(htmlRevealed.includes('title="Conceal sensitive content"'), "Re-mask button must have localized title");
    assert.ok(htmlRevealed.includes('aria-label="Conceal sensitive content"'), "Re-mask button must have accessible aria-label");
    assert.ok(htmlRevealed.includes("🔒"), "Re-mask button shows lock icon");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("FileList setup: isMasked, isRevealed, and toggleNsfwReveal manage presentation state", async () => {
  const loader = loadProductionFileList("test-privacy-logic");
  try {
    const FileList = await loader.importComponent();
    let setupCtx = null;
    let emittedToggle = null;
    const origSetup = FileList.setup;
    FileList.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const sfwFile = { id: 1, path: "/lib/clean.png", is_nsfw: false, container: "png" };
    const nsfwFile1 = { id: 2, path: "/lib/flagged1.png", is_nsfw: true, container: "png" };
    const nsfwFile2 = { id: 3, path: "/lib/flagged2.png", is_nsfw: true, container: "png" };
    const files = [sfwFile, nsfwFile1, nsfwFile2];

    const revealedNsfw = new Set(["/lib/flagged1.png"]);

    const app = createSSRApp(FileList, {
      files,
      loading: false,
      blurNsfw: true,
      revealedNsfw,
      onToggleReveal: (path) => { emittedToggle = path; },
    });
    await renderToString(app);

    // Logic checks
    assert.equal(setupCtx.isMasked(sfwFile), false, "SFW file is never masked");
    assert.equal(setupCtx.isMasked(nsfwFile1), false, "Explicitly revealed sensitive file is not masked");
    assert.equal(setupCtx.isMasked(nsfwFile2), true, "Concealed sensitive file is masked");

    assert.equal(setupCtx.isRevealed(nsfwFile1.path), true);
    assert.equal(setupCtx.isRevealed(nsfwFile2.path), false);

    // Trigger toggleNsfwReveal
    setupCtx.toggleNsfwReveal(nsfwFile2.path);
    assert.equal(emittedToggle, nsfwFile2.path, "toggleNsfwReveal must emit toggleReveal with path");

    // Original file classification flag was not mutated
    assert.equal(nsfwFile1.is_nsfw, true);
    assert.equal(nsfwFile2.is_nsfw, true);
    assert.equal(sfwFile.is_nsfw, false);
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("FileList setup: Keyboard r/R toggles sensitive reveal on selected file without breaking selection", async () => {
  const loader = loadProductionFileList("test-keyboard-r");
  try {
    const FileList = await loader.importComponent();
    let setupCtx = null;
    let emittedToggle = null;
    let selectedFile = null;
    const origSetup = FileList.setup;
    FileList.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const nsfwFile = { id: 10, path: "/lib/sensitive10.png", is_nsfw: true, container: "png" };
    const sfwFile = { id: 11, path: "/lib/clean11.png", is_nsfw: false, container: "png" };
    const files = [nsfwFile, sfwFile];

    const app = createSSRApp(FileList, {
      files,
      loading: false,
      selectedFile: nsfwFile,
      blurNsfw: true,
      onToggleReveal: (path) => { emittedToggle = path; },
      onSelect: (file) => { selectedFile = file; },
    });
    await renderToString(app);

    // 1. Pressing 'r' on selected NSFW file triggers reveal toggle
    let rPrevented = false;
    setupCtx.handleKeyDown({
      key: "r",
      ctrlKey: false,
      metaKey: false,
      altKey: false,
      preventDefault: () => { rPrevented = true; },
    });
    assert.ok(rPrevented, "handleKeyDown with 'r' on sensitive file must call preventDefault");
    assert.equal(emittedToggle, nsfwFile.path, "Emitted toggleReveal for sensitive file");

    // Selection was not cleared or mutated
    assert.equal(selectedFile, null, "Selection was not overwritten or cleared by 'r' toggle");

    // 2. Pressing 'R' (capital) on sensitive file also works
    emittedToggle = null;
    let capRPrevented = false;
    setupCtx.handleKeyDown({
      key: "R",
      ctrlKey: false,
      metaKey: false,
      altKey: false,
      preventDefault: () => { capRPrevented = true; },
    });
    assert.ok(capRPrevented, "handleKeyDown with 'R' must call preventDefault");
    assert.equal(emittedToggle, nsfwFile.path);

    // 3. ArrowDown navigates to next file
    let downPrevented = false;
    setupCtx.handleKeyDown({
      key: "ArrowDown",
      preventDefault: () => { downPrevented = true; },
    });
    assert.ok(downPrevented);
    assert.equal(selectedFile?.id, sfwFile.id, "ArrowDown selected next file");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("FileList styles declare table-thumb-wrapper, nsfw-blurred, overlays, and reduced-motion", () => {
  const loader = loadProductionFileList("test-css-rules");
  try {
    const css = loader.cssContent;
    assert.ok(css.includes(".table-thumb-wrapper"), "CSS declares .table-thumb-wrapper");
    assert.ok(css.includes(".thumb.nsfw-blurred"), "CSS declares .thumb.nsfw-blurred");
    assert.ok(css.includes(".table-nsfw-overlay"), "CSS declares .table-nsfw-overlay");
    assert.ok(css.includes(".table-nsfw-badge"), "CSS declares .table-nsfw-badge");
    assert.ok(css.includes(".table-nsfw-remask-btn"), "CSS declares .table-nsfw-remask-btn");
    assert.ok(css.includes(".table-remask-icon"), "CSS declares .table-remask-icon");
    assert.ok(
      css.includes("prefers-reduced-motion: reduce") &&
      css.includes(".thumb.nsfw-blurred"),
      "Reduced motion media query disables transitions for nsfw-blurred and overlays"
    );
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});


test("FileList names selected and unselected files correctly in every locale", async () => {
  for (const [locale, symbol] of Object.entries({ en: "en", "zh-CN": "zhCN", "zh-TW": "zhTW", ja: "ja", de: "de", fr: "fr", es: "es" })) {
    const messages = (await import(`../src/i18n/locales/${locale}.ts`))[symbol];
    const loader = loadProductionFileList(`selection-${locale}`, messages.view);
    try {
      const component = await loader.importComponent();
      const files = [
        { id: 1, path: "/lib/first.png", container: "png", size_bytes: 1, modified_at: 1 },
        { id: 2, path: "/lib/second.png", container: "png", size_bytes: 1, modified_at: 1 },
      ];
      const html = await renderToString(createSSRApp(component, { files, selectedFilePaths: new Set([files[1].path]) }));
      assert.ok(html.includes(`aria-label="${messages.view.selectFile.replace("{name}", "first.png")}"`), locale);
      assert.ok(html.includes(`aria-label="${messages.view.deselectFile.replace("{name}", "second.png")}"`), locale);
      assert.match(html, /type="checkbox" checked aria-label=/);
    } finally { fs.unlinkSync(loader.tmpPath); }
  }
});


test("Table renders one accessible stack control and emits expansion without changing selection", async () => {
  const loader = loadProductionFileList("table-stacks");
  try {
    const FileList = await loader.importComponent();
    const members = [1, 2, 3].map(id => ({id,path:`/stack/${id}.png`,stack_id:"s",stack_order:id-1,container:"png",size_bytes:1,modified_at:1}));
    let state;
    const original = FileList.setup;
    FileList.setup = (props, context) => { state = original(props, context); return state; };
    const events = [];
    const props = {files:members,loading:false,stackMap:{s:{count:3,heroId:1}},expandedStacks:new Set(["s"]),
      onToggleStackExpand:id=>events.push(id)};
    let html = await renderToString(createSSRApp(FileList,props));
    assert.equal((html.match(/class="table-stack-btn"/g)??[]).length,1);
    assert.ok(html.includes('aria-expanded="true"'));
    assert.ok(html.includes('Collapse: 1.png · 3 images in stack'));
    state.emit('toggleStackExpand','s');
    assert.deepEqual(events,['s']);
    props.files = members.slice(0,1); props.expandedStacks = new Set();
    html = await renderToString(createSSRApp(FileList,props));
    assert.ok(html.includes('aria-expanded="false"'));
    assert.ok(html.includes('Expand: 1.png · 3 images in stack'));
    assert.equal((html.match(/class="data-row/g)??[]).length,1);
  } finally { fs.unlinkSync(loader.tmpPath); }
});
