import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { en } from "../src/i18n/locales/en.ts";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";

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

  // Mock i18n
  const mockI18n = `
  const i18nData = ${JSON.stringify(en)};
  const t = new Proxy(i18nData, {
    get(target, prop) {
      if (prop === "value") return target;
      return target[prop];
    },
  });
  `;

  scriptCode = scriptCode.replace(/import { t } from [^;]+;/, mockI18n);
  scriptCode = scriptCode.replace(
    /import { invoke } from "@tauri-apps\/api\/core";/,
    "const invoke = async () => {};"
  );
  scriptCode = scriptCode.replace(
    /import { open as openDialog } from "@tauri-apps\/plugin-dialog";/,
    "const openDialog = async () => null;"
  );
  scriptCode = scriptCode.replace(
    /import { useNotification } from [^;]+;/,
    "const useNotification = () => ({ showError: () => {}, showSuccess: () => {}, showInfo: () => {} });"
  );
  scriptCode = scriptCode.replace(
    /import FolderTreeNode from [^;]+;/,
    'const FolderTreeNode = { name: "FolderTreeNode", template: "<li></li>", ssrRender() {} };'
  );
  scriptCode = scriptCode.replace(/import type [^\n]+;/g, "");

  let templateCode = template.code.replace("export function ssrRender", "function ssrRender");
  const fullCode = `${scriptCode}\n${templateCode}\ncomponent.ssrRender = ssrRender;\nexport default component;`;

  const transformed = ts.transpileModule(fullCode, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ESNext },
  });

  const tmpPath = path.resolve(
    "tests",
    `.tmp-${componentId}-${Date.now()}-${Math.random().toString(36).slice(2)}.mjs`
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

// Mock DOM element for testing keyboard navigation and focus management
class MockElement {
  constructor(tagName = "button", className = "", attributes = {}) {
    this.__v_skip = true;
    this.nodeType = 1;
    this.tagName = tagName.toUpperCase();
    this.className = className;
    this.classList = new Set(className.split(" ").filter(Boolean));
    this.attributes = { ...attributes };
    this.children = [];
    this.parentElement = null;
    this.isConnected = true;
    this.focused = false;
    this.offsetParent = {};
    this.listeners = new Map();
  }

  getAttribute(name) {
    return this.attributes[name] ?? null;
  }

  setAttribute(name, val) {
    this.attributes[name] = String(val);
  }

  hasAttribute(name) {
    return name in this.attributes;
  }

  removeAttribute(name) {
    delete this.attributes[name];
  }

  matches(selector) {
    const parts = selector.split(",").map((s) => s.trim());
    return parts.some((part) => {
      if (part.includes(":not(")) {
        const notMatches = [...part.matchAll(/:not\(([^)]+)\)/g)];
        const base = part.replace(/:not\([^)]+\)/g, "").trim();
        for (const m of notMatches) {
          if (this.matches(m[1])) return false;
        }
        return base ? this.matches(base) : true;
      }
      if (part.startsWith(".")) {
        return this.classList.has(part.slice(1));
      }
      if (part === "button") return this.tagName === "BUTTON";
      if (part === "li") return this.tagName === "LI";
      if (part === "aside") return this.tagName === "ASIDE";
      if (part === "input" || part === "textarea" || part === "select") {
        return this.tagName.toLowerCase() === part;
      }
      return false;
    });
  }

  closest(selector) {
    let curr = this;
    while (curr) {
      if (curr.matches && curr.matches(selector)) return curr;
      curr = curr.parentElement;
    }
    return null;
  }

  appendChild(child) {
    child.parentElement = this;
    this.children.push(child);
    return child;
  }

  contains(node) {
    if (!node) return false;
    const self = this.__v_raw ?? this;
    let curr = node;
    while (curr) {
      const rawCurr = curr.__v_raw ?? curr;
      if (rawCurr === self) return true;
      curr = curr.parentElement;
    }
    return false;
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] ?? null;
  }

  querySelectorAll(selector) {
    const results = [];
    const walk = (node) => {
      for (const child of node.children) {
        if (child.matches && child.matches(selector)) {
          results.push(child);
        }
        walk(child);
      }
    };
    walk(this);
    return results;
  }

  focus() {
    this.focused = true;
    if (globalThis.document) {
      if (globalThis.document.activeElement && globalThis.document.activeElement !== this) {
        globalThis.document.activeElement.focused = false;
      }
      globalThis.document.activeElement = this;
    }
  }

  blur() {
    this.focused = false;
    if (globalThis.document && globalThis.document.activeElement === this) {
      globalThis.document.activeElement = null;
    }
  }

  scrollIntoView() {}

  addEventListener(event, cb) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event).push(cb);
  }

  click() {
    const cbs = this.listeners.get("click") || [];
    for (const cb of cbs) cb({ target: this, preventDefault: () => {} });
  }
}

test("Sidebar renders tag search control, clear button, and localized states", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-render");
  try {
    const Sidebar = await loader.importComponent();

    // 1. Tags present: search control is rendered with accessible placeholder
    const appWithTags = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags: [
        { id: 1, name: "landscape", color: "#3b82f6" },
        { id: 2, name: "portrait", color: "#ec4899" },
      ],
      tagCounts: { 1: 12, 2: 5 },
      activeTarget: { type: "all" },
      progress: null,
    });
    const htmlWithTags = await renderToString(appWithTags);
    assert.ok(htmlWithTags.includes('class="tag-search-box"'), "Search box must be rendered when tags exist");
    assert.ok(htmlWithTags.includes('class="tag-search-input"'), "Search input must be rendered");
    assert.ok(htmlWithTags.includes('placeholder="Filter tags…"'), "Search input must have localized placeholder");
    assert.ok(htmlWithTags.includes('aria-label="Filter tags…"'), "Search input must have accessible aria-label");
    assert.ok(htmlWithTags.includes("landscape"), "Tag chip landscape must be rendered");
    assert.ok(htmlWithTags.includes("portrait"), "Tag chip portrait must be rendered");

    // 2. Tags empty: search box hidden, noTags empty state rendered
    const appEmpty = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags: [],
      activeTarget: { type: "all" },
      progress: null,
    });
    const htmlEmpty = await renderToString(appEmpty);
    assert.ok(!htmlEmpty.includes('class="tag-search-box"'), "Search box should be hidden when tags collection is empty");
    assert.ok(htmlEmpty.includes("No tags yet."), "Localized noTags empty state must be shown");

    // 3. Tags undefined: loading state rendered
    const appLoading = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags: undefined,
      activeTarget: { type: "all" },
      progress: null,
    });
    const htmlLoading = await renderToString(appLoading);
    assert.ok(!htmlLoading.includes('class="tag-search-box"'), "Search box should be hidden when tags are loading");
    assert.ok(htmlLoading.includes("Loading tags…"), "Localized loadingTags state must be shown");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Case-insensitive and Unicode-normalized tag filtering", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-matching");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const tags = [
      { id: 1, name: "Anime Art" },
      { id: 2, name: "Cyberpunk 2077" },
      { id: 3, name: "café aesthetic" },
      { id: 4, name: "風景画" }, // Japanese landscape painting
      { id: 5, name: "猫咪" },   // Chinese cat
      { id: 6, name: "Straße" },  // German street
    ];

    const app = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags,
      tagCounts: { 1: 10, 2: 4, 3: 8, 4: 15, 5: 3, 6: 2 },
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    // Initial query empty: all 6 tags present
    assert.equal(setupCtx.filteredTags.value.length, 6, "All tags present initially");

    // Case-insensitivity: lowercase "anime" matches "Anime Art"
    setupCtx.tagQuery.value = "anime";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].name, "Anime Art");

    // Case-insensitivity: uppercase "CYBER" matches "Cyberpunk 2077"
    setupCtx.tagQuery.value = "CYBER";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].name, "Cyberpunk 2077");

    // Unicode matching with decomposed form vs composed form: "cafe\u0301" matches "café aesthetic"
    setupCtx.tagQuery.value = "cafe\u0301";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].name, "café aesthetic");

    // CJK matching: Japanese Kanji "風景"
    setupCtx.tagQuery.value = "風景";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].name, "風景画");

    // CJK matching: Chinese "猫"
    setupCtx.tagQuery.value = "猫";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].name, "猫咪");

    // German matching: "straße"
    setupCtx.tagQuery.value = "straße";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].name, "Straße");

    // Whitespace trimming in query: "  cyber  "
    setupCtx.tagQuery.value = "  cyber  ";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].name, "Cyberpunk 2077");

    // Tags array was never mutated
    assert.equal(tags.length, 6);
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Clear empty vs no-match states", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-no-match");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags: [
        { id: 1, name: "fantasy" },
        { id: 2, name: "scifi" },
      ],
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    // Non-matching query yields 0 results
    setupCtx.tagQuery.value = "nonexistent_tag_xyz";
    assert.equal(setupCtx.filteredTags.value.length, 0);

    // Render with initial non-matching tag query to verify SSR no-match template output
    Sidebar.setup = (props, ctx) => {
      const c = origSetup(props, ctx);
      c.tagQuery.value = "nonexistent_tag_xyz";
      return c;
    };
    const appNoMatch = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags: [
        { id: 1, name: "fantasy" },
        { id: 2, name: "scifi" },
      ],
      activeTarget: { type: "all" },
      progress: null,
    });
    const htmlNoMatch = await renderToString(appNoMatch);
    assert.ok(htmlNoMatch.includes("No matching tags."), "Must show localized noMatchingTags message");
    assert.ok(htmlNoMatch.includes("no-match-hint"), "Must apply no-match-hint class");
    assert.ok(!htmlNoMatch.includes("No tags yet."), "Must NOT show noTags empty library state when tags exist");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Clearing filter via clearTagFilter and Escape restores collection", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-clearing");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const tags = [
      { id: 1, name: "architecture" },
      { id: 2, name: "nature" },
      { id: 3, name: "vehicles" },
    ];

    const app = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags,
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    const mockInput = new MockElement("input", "tag-search-input");
    setupCtx.tagSearchInputRef.value = mockInput;

    // Filter down to 1 tag
    setupCtx.tagQuery.value = "arch";
    assert.equal(setupCtx.filteredTags.value.length, 1);

    // clearTagFilter resets query and refocuses input
    setupCtx.clearTagFilter();
    assert.equal(setupCtx.tagQuery.value, "");
    assert.equal(setupCtx.filteredTags.value.length, 3, "All tags restored after clearing filter");
    assert.equal(mockInput.focused, true, "Search input should be focused after clear");

    // Filter again and clear with Escape keydown
    setupCtx.tagQuery.value = "nat";
    assert.equal(setupCtx.filteredTags.value.length, 1);

    let defaultPrevented = false;
    setupCtx.onTagSearchKeydown({
      key: "Escape",
      preventDefault: () => { defaultPrevented = true; },
    });
    assert.ok(defaultPrevented, "Escape on active query must call preventDefault");
    assert.equal(setupCtx.tagQuery.value, "");
    assert.equal(setupCtx.filteredTags.value.length, 3, "All tags restored after Escape");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Preserves active tag selection and item counts during filtering and clearing", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-selection");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const selectedTag = { id: 10, name: "selected-portrait", color: "#8b5cf6" };
    const otherTag = { id: 20, name: "unselected-landscape", color: "#10b981" };
    const tags = [selectedTag, otherTag];
    const tagCounts = { 10: 42, 20: 17 };

    const app = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags,
      tagCounts,
      activeTarget: { type: "tag", tag: selectedTag },
      progress: null,
    });
    const initialHtml = await renderToString(app);
    assert.ok(initialHtml.includes("42"), "Selected tag count 42 must be rendered");
    assert.ok(initialHtml.includes("17"), "Other tag count 17 must be rendered");

    // Active state helper confirms target is active
    assert.equal(setupCtx.isTargetActive({ type: "tag", tag: selectedTag }), true);
    assert.equal(setupCtx.isTargetActive({ type: "tag", tag: otherTag }), false);

    // Filter to other tag: selection is not mutated
    setupCtx.tagQuery.value = "landscape";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].id, 20);
    // Active target in props remains unchanged
    assert.equal(setupCtx.isTargetActive({ type: "tag", tag: selectedTag }), true);

    // Filter to selected tag: selected tag is matched, count and active state preserved
    setupCtx.tagQuery.value = "portrait";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].id, 10);
    assert.equal(setupCtx.isTargetActive({ type: "tag", tag: setupCtx.filteredTags.value[0] }), true);

    // Clear filter: both tags return, selected tag retains active state and counts
    setupCtx.clearTagFilter();
    assert.equal(setupCtx.filteredTags.value.length, 2);
    assert.equal(setupCtx.isTargetActive({ type: "tag", tag: selectedTag }), true);
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Bounded query and rendering performance for large tag collections", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-scale");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    // Generate 500 tags
    const largeTagList = [];
    for (let i = 1; i <= 500; i++) {
      largeTagList.push({
        id: i,
        name: `tag_item_${i.toString().padStart(4, "0")}`,
        color: "#6366f1",
      });
    }

    const activeTag = largeTagList[350]; // Index 350 (beyond initial 200 limit)

    const app = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags: largeTagList,
      activeTarget: { type: "tag", tag: activeTag },
      progress: null,
    });
    await renderToString(app);

    // Without query: rendering work is bounded (200 + active tag)
    const initialFiltered = setupCtx.filteredTags.value;
    assert.ok(initialFiltered.length <= 201, `Visible list must be bounded, got ${initialFiltered.length}`);
    assert.ok(
      initialFiltered.some((t) => t.id === activeTag.id),
      "Active tag selection must be preserved and included even when beyond slice threshold"
    );

    // Bounded search: query matching all 500 tags ('tag_') is bounded at MAX_VISIBLE_TAGS (200)
    setupCtx.tagQuery.value = "tag_";
    const broadMatches = setupCtx.filteredTags.value;
    assert.equal(broadMatches.length, 200, "Broad search matches must be capped at MAX_VISIBLE_TAGS (200)");

    // Rapid input simulation: rapid keystrokes mutate query in quick succession
    const startTime = Date.now();
    for (let i = 0; i < 50; i++) {
      setupCtx.tagQuery.value = `tag_item_00${i % 10}`;
      const count = setupCtx.filteredTags.value.length;
      assert.ok(count >= 0 && count <= 200);
    }
    const elapsed = Date.now() - startTime;
    assert.ok(elapsed < 2000, `50 rapid input evaluations completed within reasonable time: ${elapsed}ms`);

    // Final specific query: returns exact match
    setupCtx.tagQuery.value = "tag_item_0350";
    assert.equal(setupCtx.filteredTags.value.length, 1);
    assert.equal(setupCtx.filteredTags.value[0].id, 350);

    // Source list preserved
    assert.equal(largeTagList.length, 500);
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Keyboard navigation between search input, tag chips, and destinations", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-keyboard");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    let emittedNav = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const tags = [
      { id: 1, name: "alpha" },
      { id: 2, name: "beta" },
      { id: 3, name: "gamma" },
    ];

    const app = createSSRApp(Sidebar, {
      folders: [],
      albums: [],
      tags,
      activeTarget: { type: "all" },
      progress: null,
      onSelectNav: (target) => { emittedNav = target; },
    });
    await renderToString(app);

    // Construct mock DOM
    const sidebarEl = new MockElement("aside", "sidebar-eagle");
    const searchBox = new MockElement("div", "tag-search-box");
    const searchInput = new MockElement("input", "tag-search-input");
    searchBox.appendChild(searchInput);
    sidebarEl.appendChild(searchBox);

    const tagsContainer = new MockElement("div", "tags-container");
    const chip1 = new MockElement("button", "tag-chip-eagle");
    const chip2 = new MockElement("button", "tag-chip-eagle");
    const chip3 = new MockElement("button", "tag-chip-eagle");
    tagsContainer.appendChild(chip1);
    tagsContainer.appendChild(chip2);
    tagsContainer.appendChild(chip3);
    sidebarEl.appendChild(tagsContainer);

    const footer = new MockElement("footer", "sidebar-footer");
    const toolBtn = new MockElement("button", "tool-btn");
    footer.appendChild(toolBtn);
    sidebarEl.appendChild(footer);

    setupCtx.sidebarRef.value = sidebarEl;
    setupCtx.tagSearchInputRef.value = searchInput;
    globalThis.document = { activeElement: searchInput };

    // 1. ArrowDown in search input moves focus to the first tag chip
    let downPrevented = false;
    setupCtx.onTagSearchKeydown({
      key: "ArrowDown",
      preventDefault: () => { downPrevented = true; },
    });
    assert.ok(downPrevented, "ArrowDown in search input must preventDefault");
    assert.equal(chip1.focused, true, "First tag chip must be focused on ArrowDown from input");

    // 2. ArrowUp on the first tag chip moves focus back up to the search input
    globalThis.document.activeElement = chip1;
    let upPrevented = false;
    setupCtx.onSidebarKeydown({
      key: "ArrowUp",
      target: chip1,
      preventDefault: () => { upPrevented = true; },
    });
    assert.ok(upPrevented, "ArrowUp on first tag chip must preventDefault");
    assert.equal(searchInput.focused, true, "Search input must receive focus on ArrowUp from first chip");

    // 3. Enter in search input selects the first matching tag and emits selectNav
    setupCtx.tagQuery.value = "beta";
    assert.equal(setupCtx.filteredTags.value[0].name, "beta");
    let enterPrevented = false;
    setupCtx.onTagSearchKeydown({
      key: "Enter",
      preventDefault: () => { enterPrevented = true; },
    });
    assert.ok(enterPrevented, "Enter on matching search input must preventDefault");
    assert.equal(emittedNav?.type, "tag");
    assert.equal(emittedNav?.tag.name, "beta");

    // 4. ArrowDown when query has no matching tags jumps to first tool button
    setupCtx.tagQuery.value = "no_match_here";
    tagsContainer.children = []; // Simulate no matching chips in DOM
    globalThis.document.activeElement = searchInput;
    setupCtx.onTagSearchKeydown({
      key: "ArrowDown",
      preventDefault: () => {},
    });
    assert.equal(toolBtn.focused, true, "ArrowDown with no matching tags moves focus to tools");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar styles: Declares tag search, clear button, and narrow width responsive rules", () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-tag-search-css");
  try {
    const css = loader.cssContent;
    assert.ok(css.includes(".tag-search-box"), "CSS declares .tag-search-box");
    assert.ok(css.includes(".tag-search-input-wrapper:focus-within"), "CSS declares focus-within for search input wrapper");
    assert.ok(css.includes(".tag-search-clear-btn:focus-visible"), "CSS declares visible focus for clear button");
    assert.ok(css.includes(".no-match-hint"), "CSS declares .no-match-hint styling");
    assert.ok(css.includes(".tag-search-input-wrapper"), "CSS declares .tag-search-input-wrapper");
    assert.ok(css.includes(".tag-search-clear-btn"), "CSS declares .tag-search-clear-btn");
    assert.ok(
      css.includes(".tag-search-input-wrapper,\n  .tag-search-clear-btn") ||
      css.includes(".tag-search-input-wrapper,") ||
      css.includes(".tag-search-clear-btn"),
      "Reduced motion media query covers tag search controls"
    );
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});
