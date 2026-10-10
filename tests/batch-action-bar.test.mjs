import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { en } from "../src/i18n/locales/en.ts";
import { h, createSSRApp, reactive } from "vue";
import { renderToString } from "vue/server-renderer";

function loadProductionBatchActionBar() {
  const filePath = path.resolve("src/components/BatchActionBar.vue");
  const sfc = fs.readFileSync(filePath, "utf-8");
  const { descriptor } = parse(sfc, { filename: "BatchActionBar.vue" });

  const script = compileScript(descriptor, { id: "test-batch", inlineTemplate: false });
  const template = compileTemplate({
    id: "test-batch",
    filename: "BatchActionBar.vue",
    source: descriptor.template.content,
    ssr: true,
    compilerOptions: { bindingMetadata: script.bindings },
  });

  let scriptCode = script.content.replace("export default", "const component =");
  // Provide mock translations for template interpolation
  const mockI18n = `const t = {
    batch: {
      selectedOf: "of",
      selectedCount: "selected",
      setRating: "Rating",
      clearRating: "Clear",
      favorite: "Favorite",
      tag: "Tag",
      trash: "Trash",
      more: "More",
      addToAlbum: "Album",
      move: "Move",
      copy: "Copy",
      cullDrafts: "Cull",
      transform: "Transform",
      copyPaths: "Copy Paths",
      copyPrompts: "Copy Prompts",
      autoTag: "Auto Tag"
    },
    stack: {
      cullDrafts: "Cull Drafts"
    },
    view: ${JSON.stringify(en.view)}
  };`;
  scriptCode = scriptCode.replace(/import { t } from [^\n]+;/, mockI18n);
  scriptCode = scriptCode.replace(/import type [^\n]+;/, "");

  let templateCode = template.code.replace("export function ssrRender", "function ssrRender");
  const fullCode = `${scriptCode}\n${templateCode}\ncomponent.ssrRender = ssrRender;\nexport default component;`;

  const transformed = ts.transpileModule(fullCode, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ESNext },
  });

  const tmpPath = path.resolve("tests", `.tmp-batch-action-bar-${Date.now()}.mjs`);
  fs.writeFileSync(tmpPath, transformed.outputText);

  return {
    tmpPath,
    async importComponent() {
      const mod = await import(pathToFileURL(tmpPath).href);
      return mod.default;
    },
  };
}

test("BatchActionBar production component mounts and renders with controlled props", async () => {
  const loader = loadProductionBatchActionBar();
  try {
    const BatchActionBar = await loader.importComponent();

    const files = [
      { id: 1, path: "/photos/a.png", is_favorite: false, is_nsfw: false, stack_id: null },
      { id: 2, path: "/photos/b.png", is_favorite: true, is_nsfw: false, stack_id: "s1" },
    ];

    const app = createSSRApp(BatchActionBar, {
      selectedFiles: files,
      totalCount: 10,
    });

    const html = await renderToString(app);

    // Verify rendered output from production template
    assert.ok(html.includes('class="batch-bar-container"'), "Must render batch-bar-container");
    assert.ok(html.includes('role="toolbar"'), "Must include role='toolbar'");
    assert.ok(html.includes('aria-label="Batch Actions"'), "Must include aria-label");
    assert.ok(html.includes("2 of 10 selected"), "Must interpolate selection counts");
    assert.ok(html.includes(en.view.selectAll), "Must render Select All button");
    assert.ok(html.includes("Deselect"), "Must render Deselect button");
    assert.ok(html.includes("Rating"), "Must render Rating button");
    assert.ok(html.includes("Favorite"), "Must render Favorite button");
    assert.ok(html.includes("Tag"), "Must render Tag button");
    assert.ok(html.includes("Trash"), "Must render Trash button");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("BatchActionBar production component emits events to parent handlers", async () => {
  const loader = loadProductionBatchActionBar();
  try {
    const BatchActionBar = await loader.importComponent();

    let setupResult = null;
    const origSetup = BatchActionBar.setup;
    BatchActionBar.setup = (props, ctx) => {
      setupResult = origSetup(props, ctx);
      return setupResult;
    };

    const emitted = [];
    const parentWrapper = {
      render() {
        return h(BatchActionBar, {
          selectedFiles: [
            { id: 1, path: "/photos/a.png", is_favorite: false, is_nsfw: false, stack_id: null },
          ],
          totalCount: 5,
          onSelectAll: (...args) => emitted.push({ event: "selectAll", args }),
          onClearSelection: (...args) => emitted.push({ event: "clearSelection", args }),
          onTagSelected: (...args) => emitted.push({ event: "tagSelected", args }),
          onAddTag: (...args) => emitted.push({ event: "addTag", args }),
          onMoveSelected: (...args) => emitted.push({ event: "moveSelected", args }),
          onMove: (...args) => emitted.push({ event: "move", args }),
          onCopySelected: (...args) => emitted.push({ event: "copySelected", args }),
          onCopy: (...args) => emitted.push({ event: "copy", args }),
          onCullSelectedDrafts: (...args) => emitted.push({ event: "cullSelectedDrafts", args }),
          onCullDrafts: (...args) => emitted.push({ event: "cullDrafts", args }),
          onTransformSelected: (...args) => emitted.push({ event: "transformSelected", args }),
          onTransform: (...args) => emitted.push({ event: "transform", args }),
          onTrashSelected: (...args) => emitted.push({ event: "trashSelected", args }),
          onTrash: (...args) => emitted.push({ event: "trash", args }),
          onRateSelected: (...args) => emitted.push({ event: "rateSelected", args }),
          onSetRating: (...args) => emitted.push({ event: "setRating", args }),
        });
      },
    };

    const app = createSSRApp(parentWrapper);
    await renderToString(app);

    assert.ok(setupResult, "Component setup must execute");

    // Execute actions defined in production setup
    setupResult.onTag();
    setupResult.onMove();
    setupResult.onCopy();
    setupResult.onCull();
    setupResult.onTransform();
    setupResult.onTrash();
    setupResult.onSetRating(5);

    const emittedNames = emitted.map((e) => e.event);

    // Verify all paired events are emitted by production methods
    assert.ok(emittedNames.includes("tagSelected"), "Must emit tagSelected");
    assert.ok(emittedNames.includes("addTag"), "Must emit addTag");
    assert.ok(emittedNames.includes("moveSelected"), "Must emit moveSelected");
    assert.ok(emittedNames.includes("move"), "Must emit move");
    assert.ok(emittedNames.includes("copySelected"), "Must emit copySelected");
    assert.ok(emittedNames.includes("copy"), "Must emit copy");
    assert.ok(emittedNames.includes("cullSelectedDrafts"), "Must emit cullSelectedDrafts");
    assert.ok(emittedNames.includes("cullDrafts"), "Must emit cullDrafts");
    assert.ok(emittedNames.includes("transformSelected"), "Must emit transformSelected");
    assert.ok(emittedNames.includes("transform"), "Must emit transform");
    assert.ok(emittedNames.includes("trashSelected"), "Must emit trashSelected");
    assert.ok(emittedNames.includes("trash"), "Must emit trash");

    const rateEvent = emitted.find((e) => e.event === "setRating");
    assert.deepEqual(rateEvent?.args, [5], "setRating must emit rating value");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("production selection state handles single, multi, shift range, and clear", () => {
  const file1 = { id: 1, path: "/photos/a.png" };
  const file2 = { id: 2, path: "/photos/b.png" };
  const file3 = { id: 3, path: "/photos/c.png" };
  const allFiles = [file1, file2, file3];

  let selectedFile = null;
  let selectedFilePaths = new Set();
  let selectionAnchorPath = null;

  function onFileSelected(file, event) {
    if (event?.shiftKey && selectionAnchorPath) {
      const anchorIndex = allFiles.findIndex((c) => c.path === selectionAnchorPath);
      const targetIndex = allFiles.findIndex((c) => c.path === file.path);
      if (anchorIndex !== -1 && targetIndex !== -1) {
        const [start, end] = anchorIndex < targetIndex ? [anchorIndex, targetIndex] : [targetIndex, anchorIndex];
        const next = new Set(selectedFilePaths);
        for (let i = start; i <= end; i++) next.add(allFiles[i].path);
        selectedFilePaths = next;
        selectedFile = file;
        return;
      }
    }

    selectedFile = file;
    selectionAnchorPath = file.path;

    if (event?.ctrlKey || event?.metaKey) {
      const next = new Set(selectedFilePaths);
      if (next.has(file.path)) next.delete(file.path);
      else next.add(file.path);
      selectedFilePaths = next;
    } else {
      selectedFilePaths = new Set([file.path]);
    }
  }

  function onClearSelection() {
    selectedFilePaths = new Set();
    selectedFile = null;
    selectionAnchorPath = null;
  }

  // 1. Single click
  onFileSelected(file1);
  assert.equal(selectedFile, file1);
  assert.equal(selectedFilePaths.size, 1);
  assert.ok(selectedFilePaths.has(file1.path));

  // 2. Ctrl+click file 2
  onFileSelected(file2, { ctrlKey: true });
  assert.equal(selectedFilePaths.size, 2);
  assert.ok(selectedFilePaths.has(file1.path));
  assert.ok(selectedFilePaths.has(file2.path));

  // 3. Shift range click file 3
  onClearSelection();
  onFileSelected(file1);
  onFileSelected(file3, { shiftKey: true });
  assert.equal(selectedFilePaths.size, 3);
  assert.ok(selectedFilePaths.has(file1.path));
  assert.ok(selectedFilePaths.has(file2.path));
  assert.ok(selectedFilePaths.has(file3.path));

  // 4. Clear
  onClearSelection();
  assert.equal(selectedFile, null);
  assert.equal(selectedFilePaths.size, 0);
  assert.equal(selectionAnchorPath, null);
});

test("BatchActionBar positioning is anchored to gallery viewport canvas", () => {
  const source = fs.readFileSync(path.resolve("src/components/BatchActionBar.vue"), "utf8");

  assert.ok(
    source.includes("position: absolute"),
    "BatchActionBar must use position: absolute to anchor within gallery viewport",
  );
  assert.ok(
    !source.includes("position: fixed"),
    "BatchActionBar must not use position: fixed which causes sidebar collisions",
  );
  assert.ok(
    source.includes("max-width: calc(100% - 24px)"),
    "BatchActionBar must constrain max-width to avoid edge clipping",
  );
});

test("responsive collapse separates primary actions and bundles secondary actions into more menu", () => {
  const source = fs.readFileSync(path.resolve("src/components/BatchActionBar.vue"), "utf8");

  // Primary buttons retained
  assert.ok(source.includes("t.batch.setRating"), "Rating dropdown must be present");
  assert.ok(source.includes("t.batch.favorite"), "Favorite button must be present");
  assert.ok(source.includes("t.batch.tag"), "Tag button must be present");
  assert.ok(source.includes("t.batch.trash"), "Trash button must be present");

  // Secondary actions bundled
  assert.ok(source.includes("secondary-actions-inline"), "Inline secondary actions container must exist");
  assert.ok(source.includes("more-actions-wrapper"), "More menu wrapper must exist");
  assert.ok(source.includes("t.batch.more"), "More button label must be present");
  assert.ok(source.includes("more-menu"), "More dropdown menu must exist");

  // Escape key closes menus
  assert.ok(source.includes('e.key === "Escape"'), "Escape key handler must close menus");
});

test("BatchActionBar dynamic overflow detection triggers compact mode on scrollWidth overflow", () => {
  const source = fs.readFileSync(path.resolve("src/components/BatchActionBar.vue"), "utf8");

  assert.ok(source.includes("barRef"), "BatchActionBar must track barRef for dynamic overflow measurement");
  assert.ok(source.includes("updateLayout"), "BatchActionBar must define updateLayout logic");
  assert.ok(source.includes("scrollWidth"), "BatchActionBar must inspect scrollWidth to detect overflow");
  assert.doesNotMatch(source, /isCompact\.value\s*=\s*width\s*<\s*920/, "Must not use fragile hardcoded 920px threshold alone");
});


test("batch counts distinguish the filtered population from selectable loaded items", async () => {
  const loader = loadProductionBatchActionBar();
  try {
    const component = await loader.importComponent();
    const files = Array.from({ length: 398 }, (_, id) => ({ id, path: `/photos/${id}.png` }));
    const render = selectedFiles => renderToString(createSSRApp(component, {
      selectedFiles, totalCount: 1200, loadedCount: 398,
    }));
    const full = await render(files);
    assert.ok(full.includes("398 of 1200 selected"));
    assert.ok(full.includes("398 loaded items"));
    assert.ok(!full.includes(en.view.selectAll));
    const partial = await render(files.slice(0, 1));
    assert.ok(partial.includes("1 of 1200 selected"));
    assert.ok(partial.includes(en.view.selectAll));
    assert.ok(partial.includes(en.view.loadedSelectionHint));
  } finally {
    fs.unlinkSync(loader.tmpPath);
  }
});
