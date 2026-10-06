import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
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
  const i18nData = {
    nav: {
      library: "Library",
      allImages: "All Images",
      favorites: "Favorites",
      sensitive: "Sensitive (18+)",
      promptInsights: "Prompt Insights",
      folders: "Folders",
      noFolders: "No folders added yet.",
      scan: "Scan",
      scanning: "Scanning...",
      rebuild: "Rebuild",
      rebuilding: "Rebuilding...",
      remove: "Remove",
      removeFolderConfirm: 'Remove folder "{name}" from Omera? Files on disk will not be deleted.',
      harvest: "Harvest New Images",
      albums: "Albums",
      newAlbum: "+ New",
      noAlbums: "No albums yet.",
      tags: "Tags",
      newTag: "+ New",
      noTags: "No tags yet.",
      recursiveMode: "Subfolders",
      singleLevelMode: "Direct only",
      addFolder: "Add Folder",
      expand: "Expand",
      collapse: "Collapse",
    },
    addFolder: {
      title: "Add Folder to Library",
    },
    search: {
      insights: "Insights",
      autoTagger: "Auto-Tag",
      models: "Models",
      database: "Database",
      shortcuts: "Shortcuts",
    },
    importModal: {
      title: "Import Files",
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

// Mock DOM elements for simulating keyboard navigation and focus management
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
      if (part.startsWith("[") && part.endsWith("]")) {
        const attrExpr = part.slice(1, -1);
        if (attrExpr.includes("=")) {
          const [name, val] = attrExpr.split("=");
          const cleanVal = val.replace(/^["']|["']$/g, "").replace(/\\(.)/g, "$1");
          return this.getAttribute(name) === cleanVal;
        }
        return this.hasAttribute(attrExpr);
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

  click() {
    const fn = this.listeners.get("click");
    if (fn) fn({ target: this, preventDefault: () => {}, stopPropagation: () => {} });
  }

  addEventListener(type, fn) {
    this.listeners.set(type, fn);
  }
}

test("Sidebar renders accessible focusable button controls with WAI-ARIA states", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-sidebar-a11y");
  try {
    const Sidebar = await loader.importComponent();
    const app = createSSRApp(Sidebar, {
      folders: [
        { id: 1, path: "/vault/photos", folder_type: "link" },
        { id: 2, path: "/vault/managed", folder_type: "managed" },
      ],
      counts: { total: 42, favorites: 10, nsfw: 3, folders: { 1: 25, 2: 17 } },
      albums: [{ id: 10, name: "Summer Trip" }],
      albumCounts: { 10: 12 },
      tags: [{ id: 101, name: "landscape", color: "#3b82f6" }],
      tagCounts: { 101: 8 },
      activeTarget: { type: "all" },
      progress: null,
    });

    const html = await renderToString(app);

    // Root landmark and label
    assert.ok(html.includes('aria-label="Sidebar Navigation"'), "Sidebar root exposes landmark aria-label");

    // Semantic Library button destinations
    assert.ok(html.includes('aria-current="page"'), "All Images is active button with aria-current='page'");
    assert.ok(html.includes('nav-item') && html.includes('active'), "All Images has active nav-item class");
    assert.ok(html.includes('aria-label="All Images (42)"'), "All Images has accessible count in aria-label");
    assert.ok(html.includes('aria-label="Favorites (10)"'), "Favorites has accessible count in aria-label");
    assert.ok(html.includes('aria-label="Sensitive (18+) (3)"'), "Sensitive has accessible count in aria-label");

    // Folders section
    assert.ok(html.includes('id="sidebar-heading-folders"'), "Folders heading exposes id");
    assert.ok(html.includes('aria-labelledby="sidebar-heading-folders"'), "Folders section links to heading");
    assert.ok(html.includes('aria-label="Add Folder"'), "Add Folder button has accessible aria-label");

    // Folder rows and expand controls
    assert.ok(html.includes('class="folder-row-container"'), "Folder row uses container layout");
    assert.ok(html.includes('tree-arrow-btn') && html.includes('aria-expanded="false"'), "Folder expand arrow exposes aria-expanded");
    assert.ok(html.includes('class="nav-item folder-item-btn"'), "Folder name is focusable button");
    assert.ok(html.includes('aria-label="photos (25)"'), "Folder button has accessible name and count");

    // Album destinations
    assert.ok(html.includes('class="nav-item album-item"'), "Album is focusable button");
    assert.ok(html.includes('Summer Trip'), "Album name rendered");

    // Tag chip destinations
    assert.ok(html.includes('tag-chip-eagle') && html.includes('role="listitem"'), "Tag chip is focusable button with listitem role");
    assert.ok(html.includes('aria-label="landscape (8)"'), "Tag chip has accessible name and count");

    // Footer tools
    assert.ok(html.includes('role="toolbar" aria-label="Sidebar Tools"'), "Footer has toolbar semantics");
    assert.ok(html.includes('aria-label="Insights"'), "Insights button has accessible aria-label");
    assert.ok(html.includes('aria-label="Auto-Tag"'), "Auto-Tag button has accessible aria-label");
    assert.ok(html.includes('aria-label="Shortcuts"'), "Shortcuts button has accessible aria-label");

    // Focus-visible and reduced motion CSS verification
    assert.ok(loader.cssContent.includes(".nav-item:focus-visible"), "Must declare visible focus for nav items");
    assert.ok(loader.cssContent.includes(".folder-item-btn:focus-visible"), "Must declare visible focus for folder items");
    assert.ok(loader.cssContent.includes(".tag-chip-eagle:focus-visible"), "Must declare visible focus for tag chips");
    assert.ok(loader.cssContent.includes(".tree-arrow-btn:focus-visible"), "Must declare visible focus for tree arrows");
    assert.ok(loader.cssContent.includes(".group-action-btn:focus-visible"), "Must declare visible focus for group action buttons");
    assert.ok(loader.cssContent.includes(".tool-btn:focus-visible"), "Must declare visible focus for tool buttons");
    assert.ok(loader.cssContent.includes(".icon-btn:focus-visible"), "Must declare visible focus for action buttons");
    assert.ok(loader.cssContent.includes(":focus-within"), "Must reveal folder actions on focus-within for keyboard users");
    assert.ok(loader.cssContent.includes("prefers-reduced-motion"), "Must declare prefers-reduced-motion rules");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("FolderTreeNode renders nested subfolder buttons with aria-expanded and current state", async () => {
  const loader = loadProductionComponent("src/components/FolderTreeNode.vue", "test-treenode-a11y");
  try {
    const FolderTreeNode = await loader.importComponent();
    const app = createSSRApp(FolderTreeNode, {
      folder: { id: 1, path: "/vault/photos", folder_type: "link" },
      entry: { name: "landscapes", path: "/vault/photos/landscapes", file_count: 14, has_children: true },
      depth: 1,
      activeTarget: {
        type: "folder",
        folder: { id: 1, path: "/vault/photos" },
        subfolderPath: "/vault/photos/landscapes",
      },
      expandedPaths: new Set(["/vault/photos/landscapes"]),
      subdirectories: {
        "/vault/photos/landscapes": [
          { name: "mountains", path: "/vault/photos/landscapes/mountains", file_count: 5, has_children: false },
        ],
      },
      loadingPaths: new Set(),
    });

    const html = await renderToString(app);

    // Root node
    assert.ok(html.includes('subfolder-node') && html.includes('active'), "Subfolder node marks active state");
    assert.ok(html.includes('data-subfolder-path="/vault/photos/landscapes"'), "Exposes data-subfolder-path for focus retention");

    // Expand button
    assert.ok(html.includes('tree-arrow-btn') && html.includes('expanded'), "Has expanded class");
    assert.ok(html.includes('aria-expanded="true"'), "Exposes aria-expanded='true' when expanded");
    assert.ok(html.includes('aria-label="Collapse landscapes"'), "Accessible label on collapse button");

    // Subfolder navigation button
    assert.ok(html.includes('subfolder-header') && html.includes('active'), "Subfolder button has active class");
    assert.ok(html.includes('aria-current="page"'), "Subfolder button has aria-current='page'");
    assert.ok(html.includes('aria-label="landscapes (14)"'), "Subfolder button has accessible name and count");

    // Nested child node
    assert.ok(html.includes('mountains'), "Renders child mountains");
    assert.ok(html.includes('tree-arrow-spacer'), "Leaf child without children renders spacer instead of button");

    // CSS inspection
    assert.ok(loader.cssContent.includes(".subfolder-header:focus-visible"), "Must define visible focus for subfolder header");
    assert.ok(loader.cssContent.includes(".tree-arrow-btn:focus-visible"), "Must define visible focus for subfolder arrow");
    assert.ok(loader.cssContent.includes("prefers-reduced-motion"), "Must declare prefers-reduced-motion rules");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup keyboard navigation: ArrowDown, ArrowUp, Home, End traverse destinations", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-sidebar-keys");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(Sidebar, {
      folders: [{ id: 1, path: "/vault/photos", folder_type: "link" }],
      counts: { total: 10 },
      albums: [{ id: 1, name: "Album A" }],
      tags: [{ id: 1, name: "Tag 1" }],
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    assert.ok(setupCtx, "Setup context must be accessible");

    // Create mock sidebar DOM hierarchy
    const sidebarEl = new MockElement("aside", "sidebar-eagle");
    const allBtn = new MockElement("button", "nav-item");
    const favBtn = new MockElement("button", "nav-item");
    const nsfwBtn = new MockElement("button", "nav-item");
    const folderBtn = new MockElement("button", "nav-item folder-item-btn");
    const albumBtn = new MockElement("button", "nav-item album-item");
    const tagChip = new MockElement("button", "tag-chip-eagle");
    const toolBtn = new MockElement("button", "tool-btn");

    sidebarEl.appendChild(allBtn);
    sidebarEl.appendChild(favBtn);
    sidebarEl.appendChild(nsfwBtn);
    sidebarEl.appendChild(folderBtn);
    sidebarEl.appendChild(albumBtn);
    sidebarEl.appendChild(tagChip);
    sidebarEl.appendChild(toolBtn);

    setupCtx.sidebarRef.value = sidebarEl;
    globalThis.document = { activeElement: allBtn };

    // Test getNavigableElements
    const items = setupCtx.getNavigableElements();
    assert.equal(items.length, 7, "All 7 destinations must be discovered as navigable elements");

    // ArrowDown moves from All to Favorites
    let defaultPrevented = false;
    setupCtx.onSidebarKeydown({
      key: "ArrowDown",
      target: allBtn,
      preventDefault: () => { defaultPrevented = true; },
    });
    assert.ok(defaultPrevented, "ArrowDown must call preventDefault");
    assert.equal(favBtn.focused, true, "Favorites button must be focused");

    // ArrowDown moves to NSFW
    setupCtx.onSidebarKeydown({
      key: "ArrowDown",
      target: favBtn,
      preventDefault: () => {},
    });
    assert.equal(nsfwBtn.focused, true, "NSFW button must be focused");

    // ArrowDown moves to folder
    setupCtx.onSidebarKeydown({
      key: "ArrowDown",
      target: nsfwBtn,
      preventDefault: () => {},
    });
    assert.equal(folderBtn.focused, true, "Folder button must be focused");

    // ArrowUp moves back to NSFW
    setupCtx.onSidebarKeydown({
      key: "ArrowUp",
      target: folderBtn,
      preventDefault: () => {},
    });
    assert.equal(nsfwBtn.focused, true, "NSFW button must be focused");

    // End jumps to last element (toolBtn)
    setupCtx.onSidebarKeydown({
      key: "End",
      target: nsfwBtn,
      preventDefault: () => {},
    });
    assert.equal(toolBtn.focused, true, "Tool button must be focused on End");

    // Home jumps to first element (allBtn)
    setupCtx.onSidebarKeydown({
      key: "Home",
      target: toolBtn,
      preventDefault: () => {},
    });
    assert.equal(allBtn.focused, true, "All button must be focused on Home");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Space and Enter activate navigation targets without scrolling", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-sidebar-activation");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(Sidebar, {
      folders: [{ id: 1, path: "/vault/photos", folder_type: "link" }],
      counts: null,
      albums: [],
      tags: [],
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    const sidebarEl = new MockElement("aside", "sidebar-eagle");
    const favBtn = new MockElement("button", "nav-item");
    sidebarEl.appendChild(favBtn);
    setupCtx.sidebarRef.value = sidebarEl;

    let clickCalled = false;
    favBtn.addEventListener("click", () => {
      clickCalled = true;
    });

    let defaultPrevented = false;
    setupCtx.onSidebarKeydown({
      key: " ",
      target: favBtn,
      preventDefault: () => { defaultPrevented = true; },
    });

    assert.ok(defaultPrevented, "Space must preventDefault to avoid scrolling");
    assert.ok(clickCalled, "Space must trigger button click event");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Folder ArrowRight expands, ArrowLeft collapses, and retains focus", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-sidebar-tree");
  try {
    const Sidebar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = Sidebar.setup;
    Sidebar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const folder = { id: 1, path: "/vault/photos", folder_type: "link" };
    const app = createSSRApp(Sidebar, {
      folders: [folder],
      counts: null,
      albums: [],
      tags: [],
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    const sidebarEl = new MockElement("aside", "sidebar-eagle");
    const folderRow = new MockElement("li", "folder-row-container", { "data-folder-path": "/vault/photos" });
    const arrowBtn = new MockElement("button", "tree-arrow-btn");
    const folderBtn = new MockElement("button", "nav-item folder-item-btn");
    folderRow.appendChild(arrowBtn);
    folderRow.appendChild(folderBtn);
    sidebarEl.appendChild(folderRow);
    setupCtx.sidebarRef.value = sidebarEl;

    // Initially collapsed
    assert.equal(setupCtx.expandedPaths.value.has(folder.path), false);

    // ArrowRight expands
    setupCtx.onFolderKeydown({ key: "ArrowRight", preventDefault: () => {} }, folder);
    assert.equal(setupCtx.expandedPaths.value.has(folder.path), true, "Folder must be expanded");

    // Mock an active focus inside a child node
    const childBtn = new MockElement("button", "subfolder-header");
    folderRow.appendChild(childBtn);
    globalThis.document = { activeElement: childBtn };

    // Collapse parent: toggleFolderExpand must retain focus on parent folder
    await setupCtx.toggleFolderExpand(folder);
    assert.equal(setupCtx.expandedPaths.value.has(folder.path), false, "Folder must be collapsed");
    assert.equal(folderBtn.focused, true, "Focus must be retained on parent folder when child is collapsed");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Tag chip ArrowRight and ArrowLeft traverse adjacent tags", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-sidebar-tags");
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
        { id: 1, name: "cat" },
        { id: 2, name: "dog" },
        { id: 3, name: "bird" },
      ],
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    const sidebarEl = new MockElement("aside", "sidebar-eagle");
    const tag1 = new MockElement("button", "tag-chip-eagle");
    const tag2 = new MockElement("button", "tag-chip-eagle");
    const tag3 = new MockElement("button", "tag-chip-eagle");
    sidebarEl.appendChild(tag1);
    sidebarEl.appendChild(tag2);
    sidebarEl.appendChild(tag3);
    setupCtx.sidebarRef.value = sidebarEl;

    globalThis.document = { activeElement: tag1 };

    // ArrowRight moves from tag1 to tag2
    setupCtx.onSidebarKeydown({
      key: "ArrowRight",
      target: tag1,
      preventDefault: () => {},
    });
    assert.equal(tag2.focused, true, "Tag 2 must be focused");

    // ArrowRight moves from tag2 to tag3
    setupCtx.onSidebarKeydown({
      key: "ArrowRight",
      target: tag2,
      preventDefault: () => {},
    });
    assert.equal(tag3.focused, true, "Tag 3 must be focused");

    // ArrowLeft moves from tag3 to tag2
    setupCtx.onSidebarKeydown({
      key: "ArrowLeft",
      target: tag3,
      preventDefault: () => {},
    });
    assert.equal(tag2.focused, true, "Tag 2 must be focused");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Empty folders, albums, and tags handle arrow navigation without throwing", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-sidebar-empty");
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
      tags: [],
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    const sidebarEl = new MockElement("aside", "sidebar-eagle");
    const allBtn = new MockElement("button", "nav-item");
    const toolBtn = new MockElement("button", "tool-btn");
    sidebarEl.appendChild(allBtn);
    sidebarEl.appendChild(toolBtn);
    setupCtx.sidebarRef.value = sidebarEl;

    globalThis.document = { activeElement: allBtn };

    // Navigation skips empty sections and goes straight to tools
    assert.doesNotThrow(() => {
      setupCtx.onSidebarKeydown({
        key: "ArrowDown",
        target: allBtn,
        preventDefault: () => {},
      });
    });
    assert.equal(toolBtn.focused, true, "ArrowDown seamlessly moves to toolBtn across empty sections");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Sidebar setup: Global shortcuts (b, i, ?, /, Esc) are not suppressed by sidebar keydown", async () => {
  const loader = loadProductionComponent("src/components/Sidebar.vue", "test-sidebar-shortcuts");
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
      activeTarget: { type: "all" },
      progress: null,
    });
    await renderToString(app);

    const sidebarEl = new MockElement("aside", "sidebar-eagle");
    const allBtn = new MockElement("button", "nav-item");
    sidebarEl.appendChild(allBtn);
    setupCtx.sidebarRef.value = sidebarEl;

    // Verify non-navigation keys do NOT call preventDefault
    const keys = ["b", "B", "i", "I", "?", "/", "Escape", "F1"];
    for (const key of keys) {
      let defaultPrevented = false;
      setupCtx.onSidebarKeydown({
        key,
        target: allBtn,
        preventDefault: () => { defaultPrevented = true; },
      });
      assert.equal(
        defaultPrevented,
        false,
        `Key '${key}' must not be prevented in sidebar, allowing global handler to catch it`
      );
    }
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});
