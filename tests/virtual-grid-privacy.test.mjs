import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";

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

function loadProductionVirtualGrid(componentId) {
  const filePath = path.resolve("src/components/VirtualGrid.vue");
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
    view: { loading: "Loading...", selectAll: "Select All", deselect: "Deselect" },
    review: { noMatches: "No items match your filter", retry: "Reset Filters", gallery: "Gallery" },
    preview: {
      preview: "Preview",
      findSimilar: "Find Similar",
      retryThumbnail: "Retry loading thumbnail",
      clickToReveal: "Click to reveal sensitive content",
      remaskContent: "Conceal sensitive content",
      remaskShortcut: "Reveal / Conceal Sensitive Content (R)",
    },
    common: { nsfw: "NSFW" },
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
    `.tmp-virtualgrid-${componentId}-${Date.now()}-${Math.random().toString(36).slice(2)}.mjs`
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

test("VirtualGrid renders masked cards and accessible reveal overlay or re-mask controls", async () => {
  const loader = loadProductionVirtualGrid("test-grid-masking");
  try {
    const VirtualGrid = await loader.importComponent();

    const files = [
      { id: 1, path: "/lib/sfw.jpg", is_nsfw: false, container: "jpg" },
      { id: 2, path: "/lib/sensitive.png", is_nsfw: true, container: "png" },
    ];

    // Case 1: Masked by default
    const app = createSSRApp(VirtualGrid, {
      files,
      blurNsfw: true,
      revealedNsfw: new Set(),
    });
    const html = await renderToString(app);

    assert.ok(html.includes("nsfw-blurred"), "Sensitive card has nsfw-blurred class");
    assert.ok(html.includes('class="nsfw-overlay"'), "Renders nsfw-overlay button");
    assert.ok(html.includes('title="Click to reveal sensitive content"'), "Overlay has localized title");
    assert.ok(html.includes('aria-label="Click to reveal sensitive content"'), "Overlay has accessible aria-label");
    assert.ok(html.includes("🔞"), "Overlay shows warning icon");

    // Case 2: Explicitly revealed
    const appRevealed = createSSRApp(VirtualGrid, {
      files,
      blurNsfw: true,
      revealedNsfw: new Set(["/lib/sensitive.png"]),
    });
    const htmlRevealed = await renderToString(appRevealed);

    assert.ok(!htmlRevealed.includes("nsfw-overlay"), "Revealed card does not show overlay");
    assert.ok(htmlRevealed.includes('class="card-remask-btn"'), "Revealed card renders re-mask button");
    assert.ok(htmlRevealed.includes('title="Conceal sensitive content"'), "Re-mask button has localized title");
    assert.ok(htmlRevealed.includes('aria-label="Conceal sensitive content"'), "Re-mask button has accessible aria-label");
    assert.ok(htmlRevealed.includes("🔒"), "Re-mask button shows lock icon");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("VirtualGrid setup: isMasked, isRevealed, toggleNsfwReveal and keyboard r/R operation", async () => {
  const loader = loadProductionVirtualGrid("test-grid-setup");
  try {
    const VirtualGrid = await loader.importComponent();
    let setupCtx = null;
    let emittedToggle = null;
    const origSetup = VirtualGrid.setup;
    VirtualGrid.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const nsfwFile = { id: 20, path: "/lib/grid_nsfw.png", is_nsfw: true, container: "png" };
    const sfwFile = { id: 21, path: "/lib/grid_sfw.png", is_nsfw: false, container: "png" };
    const files = [nsfwFile, sfwFile];

    const app = createSSRApp(VirtualGrid, {
      files,
      selectedFile: nsfwFile,
      blurNsfw: true,
      revealedNsfw: new Set(),
      onToggleReveal: (path) => { emittedToggle = path; },
    });
    await renderToString(app);

    // Initial state
    assert.equal(setupCtx.isMasked(nsfwFile), true);
    assert.equal(setupCtx.isMasked(sfwFile), false);
    assert.equal(setupCtx.isRevealed(nsfwFile.path), false);

    // Keyboard shortcut 'r' on selected card
    setupCtx.selectedIndex.value = 0;
    let prevented = false;
    setupCtx.handleKeyDown({
      key: "r",
      ctrlKey: false,
      metaKey: false,
      altKey: false,
      preventDefault: () => { prevented = true; },
    });
    assert.ok(prevented, "Key 'r' on sensitive card must preventDefault");
    assert.equal(emittedToggle, nsfwFile.path, "Emitted toggleReveal for sensitive card");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("VirtualGrid styles declare card-remask-btn, nsfw-overlay button styling, and reduced-motion", () => {
  const loader = loadProductionVirtualGrid("test-grid-css");
  try {
    const css = loader.cssContent;
    assert.ok(css.includes(".card-remask-btn"), "CSS declares .card-remask-btn");
    assert.ok(css.includes(".card-remask-icon"), "CSS declares .card-remask-icon");
    assert.ok(css.includes(".nsfw-overlay:focus-visible"), "CSS declares focus-visible for nsfw-overlay");
    assert.ok(
      css.includes("prefers-reduced-motion: reduce") &&
      css.includes(".card-remask-btn") &&
      css.includes(".nsfw-overlay"),
      "Reduced motion media query disables transitions for card-remask-btn and nsfw-overlay"
    );
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});
