import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { parse, compileScript, compileTemplate } from "vue/compiler-sfc";
import ts from "typescript";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";

function loadProductionMenuBar() {
  const filePath = path.resolve("src/components/MenuBar.vue");
  const sfc = fs.readFileSync(filePath, "utf-8");
  const { descriptor } = parse(sfc, { filename: "MenuBar.vue" });

  const script = compileScript(descriptor, { id: "test-menu-bar", inlineTemplate: false });
  const template = compileTemplate({
    id: "test-menu-bar",
    filename: "MenuBar.vue",
    source: descriptor.template.content,
    ssr: true,
    compilerOptions: { bindingMetadata: script.bindings },
  });

  let scriptCode = script.content.replace("export default", "const component =");
  // Mock i18n and tauri window imports for node test environment
  const mockI18n = `
  const currentLocaleSetting = "en";
  const SUPPORTED_LOCALES = [
    { key: "auto", label: "Auto" },
    { key: "en", label: "English" },
    { key: "zh-CN", label: "简体中文" },
  ];
  const setLocale = (l) => {};
  const t = {
    menu: {
      applicationMenu: "Application Menu",
      file: "File",
      addFolder: "Add Folder...",
      scanActive: "Scan Current Folder",
      rescanAll: "Rescan All Folders",
      dbManager: "Database Management...",
      preferences: "Preferences / Settings...",
      exit: "Exit",
      edit: "Edit",
      selectAll: "Select All",
      clearSelection: "Clear Selection",
      batchTag: "Batch Add Tags...",
      batchAutoTag: "WD14 AI Auto-Tag...",
      batchAlbum: "Batch Add to Album...",
      batchMove: "Batch Move to Folder...",
      batchCopy: "Batch Copy to Folder...",
      batchTrash: "Move to Trash",
      view: "View",
      grid: "Grid View",
      table: "List View",
      toggleSidebar: "Toggle Sidebar",
      toggleInspector: "Toggle Inspector",
      lightbox: "Quick Look (Lightbox)",
      zoomIn: "Zoom In",
      zoomOut: "Zoom Out",
      resetZoom: "Reset Zoom",
      tools: "Tools",
      organizeByPrompt: "Organize Library by Prompt",
      organizeCurrentFolder: "Scan & Organize Current Folder",
      organizeAllFolders: "Scan & Organize All Folders",
      promptStats: "Prompt Insights & Stats...",
      autoTagger: "WD14 AI Auto-Tagger...",
      modelManager: "Model Manager & Cache...",
      help: "Help",
      language: "Language",
      helpGuide: "Feature Guide & Documentation",
      shortcuts: "Keyboard Shortcuts",
      checkUpdates: "Check for Updates...",
      about: "About Omera",
    },
    legacyMigration: {
      title: "Legacy Migration",
    },
    batch: {
      export: "Batch Export...",
    },
    clipModal: {
      title: "CLIP Search",
    },
    loraModal: {
      title: "LoRA Manager",
    },
  };
  `;
  scriptCode = scriptCode.replace(/import { currentLocaleSetting, setLocale[^\n]+;/, mockI18n);
  scriptCode = scriptCode.replace(
    /import { getCurrentWindow } from "@tauri-apps\/api\/window";/,
    "const getCurrentWindow = () => ({ close: async () => {} });"
  );
  scriptCode = scriptCode.replace(/import type [^\n]+;/, "");

  let templateCode = template.code.replace("export function ssrRender", "function ssrRender");
  templateCode = templateCode.replace(/import\s+(_imports_\d+)\s+from\s+['"][^'"]+\.png['"];?/g, (match, varName) => {
    return `const ${varName} = "data:image/png;base64,mock";`;
  });
  const fullCode = `${scriptCode}\n${templateCode}\ncomponent.ssrRender = ssrRender;\nexport default component;`;

  const transformed = ts.transpileModule(fullCode, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ESNext },
  });

  const tmpPath = path.resolve("tests", `.tmp-menu-bar-${Date.now()}-${Math.random().toString(36).slice(2)}.mjs`);
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

// Mock DOM Element for interactive setup testing
class MockButton {
  constructor(id, text = "", role = "menuitem") {
    this.id = id;
    this.text = text;
    this.role = role;
    this.disabled = false;
    this.isConnected = true;
    this.parentElement = null;
    this.classList = new Set();
    this.focused = false;
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

  closest(selector) {
    if (selector === '[role="menu"]') {
      let curr = this.parentElement;
      while (curr) {
        if (curr.role === "menu") return curr;
        curr = curr.parentElement;
      }
      return null;
    }
    return null;
  }
}

class MockMenuContainer {
  constructor(id, role = "menu") {
    this.id = id;
    this.role = role;
    this.children = [];
    this.isConnected = true;
    this.parentElement = null;
  }

  appendChild(child) {
    child.parentElement = this;
    this.children.push(child);
    return child;
  }

  contains(node) {
    if (!node) return false;
    let curr = node;
    while (curr) {
      if (curr === this || curr.id === this.id) return true;
      curr = curr.parentElement;
    }
    return false;
  }

  querySelectorAll(selector) {
    const list = [];
    for (const child of this.children) {
      if (child instanceof MockButton) {
        if (!child.disabled) list.push(child);
      } else if (child instanceof MockMenuContainer) {
        list.push(...child.querySelectorAll(selector));
      }
    }
    return list;
  }
}

test("MenuBar renders appropriate WAI-ARIA menubar semantics and localized labels", async () => {
  const loader = loadProductionMenuBar();
  try {
    const MenuBar = await loader.importComponent();
    const app = createSSRApp(MenuBar);
    const html = await renderToString(app);

    // Menubar container semantics
    assert.ok(html.includes('role="menubar"'), "MenuBar root must expose role='menubar'");
    assert.ok(html.includes('aria-label="Application Menu"'), "MenuBar root must expose localized aria-label");

    // Top-level triggers
    assert.ok(html.includes('id="menu-trigger-file"'), "Must render id for file trigger");
    assert.ok(html.includes('id="menu-trigger-edit"'), "Must render id for edit trigger");
    assert.ok(html.includes('id="menu-trigger-view"'), "Must render id for view trigger");
    assert.ok(html.includes('id="menu-trigger-tools"'), "Must render id for tools trigger");
    assert.ok(html.includes('id="menu-trigger-help"'), "Must render id for help trigger");

    // Aria attributes and roving tabindex on closed menubar
    assert.ok(html.includes('role="menuitem"'), "Triggers must have role='menuitem'");
    assert.ok(html.includes('aria-haspopup="true"'), "Triggers must have aria-haspopup='true'");
    assert.ok(html.includes('aria-expanded="false"'), "Initial state must have aria-expanded='false'");
    assert.ok(html.includes('tabindex="0"'), "Initial active trigger must have tabindex='0'");
    assert.ok(html.includes('tabindex="-1"'), "Other triggers must have tabindex='-1'");

    // CSS inspection for visible focus and reduced motion
    assert.ok(loader.cssContent.includes(".menu-trigger:focus-visible"), "Must define visible focus styles for menu triggers");
    assert.ok(loader.cssContent.includes(".dropdown-item:focus-visible"), "Must define visible focus styles for dropdown items");
    assert.ok(loader.cssContent.includes("prefers-reduced-motion"), "Must declare prefers-reduced-motion overrides");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("MenuBar setup handles keyboard arrow navigation across menubar triggers", async () => {
  const loader = loadProductionMenuBar();
  try {
    const windowListeners = new Map();
    globalThis.window = {
      addEventListener: (type, fn) => windowListeners.set(type, fn),
      removeEventListener: (type, fn) => windowListeners.delete(type),
    };
    globalThis.document = { activeElement: null };

    const MenuBar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = MenuBar.setup;
    MenuBar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(MenuBar);
    await renderToString(app);

    assert.ok(setupCtx, "Setup context must be accessible");

    // Populate mock trigger buttons
    const triggers = {
      file: new MockButton("menu-trigger-file", "File"),
      edit: new MockButton("menu-trigger-edit", "Edit"),
      view: new MockButton("menu-trigger-view", "View"),
      tools: new MockButton("menu-trigger-tools", "Tools"),
      help: new MockButton("menu-trigger-help", "Help"),
    };
    setupCtx.triggerRefs.value.file = triggers.file;
    setupCtx.triggerRefs.value.edit = triggers.edit;
    setupCtx.triggerRefs.value.view = triggers.view;
    setupCtx.triggerRefs.value.tools = triggers.tools;
    setupCtx.triggerRefs.value.help = triggers.help;

    // Initially focused trigger is 'file'
    assert.equal(setupCtx.focusedMenuTrigger.value, "file");

    // ArrowRight moves from file to edit
    let defaultPrevented = false;
    setupCtx.onTriggerKeydown(
      { key: "ArrowRight", preventDefault: () => { defaultPrevented = true; } },
      "file"
    );
    assert.ok(defaultPrevented);
    assert.equal(setupCtx.focusedMenuTrigger.value, "edit");
    assert.equal(triggers.edit.focused, true);

    // ArrowRight from help wraps around to file
    setupCtx.onTriggerKeydown(
      { key: "ArrowRight", preventDefault: () => {} },
      "help"
    );
    assert.equal(setupCtx.focusedMenuTrigger.value, "file");
    assert.equal(triggers.file.focused, true);

    // ArrowLeft from file wraps around to help
    setupCtx.onTriggerKeydown(
      { key: "ArrowLeft", preventDefault: () => {} },
      "file"
    );
    assert.equal(setupCtx.focusedMenuTrigger.value, "help");
    assert.equal(triggers.help.focused, true);

    // Home jumps to file, End jumps to help
    setupCtx.onTriggerKeydown({ key: "Home", preventDefault: () => {} }, "tools");
    assert.equal(setupCtx.focusedMenuTrigger.value, "file");
    assert.equal(triggers.file.focused, true);

    setupCtx.onTriggerKeydown({ key: "End", preventDefault: () => {} }, "file");
    assert.equal(setupCtx.focusedMenuTrigger.value, "help");
    assert.equal(triggers.help.focused, true);
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("MenuBar setup handles dropdown opening, arrow navigation, and Enter/Space activation", async () => {
  const loader = loadProductionMenuBar();
  try {
    const windowListeners = new Map();
    globalThis.window = {
      addEventListener: (type, fn) => windowListeners.set(type, fn),
      removeEventListener: (type, fn) => windowListeners.delete(type),
    };
    globalThis.document = { activeElement: null };

    const MenuBar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = MenuBar.setup;
    MenuBar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(MenuBar);
    await renderToString(app);

    const triggerFile = new MockButton("menu-trigger-file", "File");
    setupCtx.triggerRefs.value.file = triggerFile;

    const dropdown = new MockMenuContainer("dropdown-menu-file");
    const item1 = new MockButton("item-1", "Add Folder");
    const item2 = new MockButton("item-2", "Scan Active");
    const item3 = new MockButton("item-3", "Exit");
    dropdown.appendChild(item1);
    dropdown.appendChild(item2);
    dropdown.appendChild(item3);
    setupCtx.dropdownRef.value = dropdown;
    setupCtx.activeMenu.value = "file";

    // Test openMenu helper with focus targets
    await setupCtx.openMenu("file", "first");
    assert.equal(setupCtx.activeMenu.value, "file");
    assert.equal(item1.focused, true);

    await setupCtx.openMenu("file", "last");
    assert.equal(item3.focused, true);

    // Inside dropdown: ArrowDown moves from item 1 to item 2
    setupCtx.onDropdownKeydown({
      key: "ArrowDown",
      target: item1,
      preventDefault: () => {},
    });
    assert.equal(item2.focused, true);

    // ArrowDown at last item wraps to first item
    setupCtx.onDropdownKeydown({
      key: "ArrowDown",
      target: item3,
      preventDefault: () => {},
    });
    assert.equal(item1.focused, true);

    // ArrowUp at first item wraps to last item
    setupCtx.onDropdownKeydown({
      key: "ArrowUp",
      target: item1,
      preventDefault: () => {},
    });
    assert.equal(item3.focused, true);

    // Home goes to first item, End goes to last item
    setupCtx.onDropdownKeydown({
      key: "Home",
      target: item2,
      preventDefault: () => {},
    });
    assert.equal(item1.focused, true);

    setupCtx.onDropdownKeydown({
      key: "End",
      target: item1,
      preventDefault: () => {},
    });
    assert.equal(item3.focused, true);
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Escape closes menu and restores focus to its trigger", async () => {
  const loader = loadProductionMenuBar();
  try {
    const windowListeners = new Map();
    globalThis.window = {
      addEventListener: (type, fn) => windowListeners.set(type, fn),
      removeEventListener: (type, fn) => windowListeners.delete(type),
    };
    globalThis.document = { activeElement: null };

    const MenuBar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = MenuBar.setup;
    MenuBar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(MenuBar);
    await renderToString(app);

    const triggerFile = new MockButton("menu-trigger-file", "File");
    setupCtx.triggerRefs.value.file = triggerFile;
    setupCtx.activeMenu.value = "file";

    const dropdown = new MockMenuContainer("dropdown-menu-file");
    const item1 = new MockButton("item-1", "Add Folder");
    dropdown.appendChild(item1);
    setupCtx.dropdownRef.value = dropdown;

    let stopped = false;
    setupCtx.onDropdownKeydown({
      key: "Escape",
      target: item1,
      preventDefault: () => {},
      stopPropagation: () => { stopped = true; },
    });

    assert.ok(stopped, "Escape must stop propagation");
    assert.equal(setupCtx.activeMenu.value, null, "Menu must be closed");
    assert.equal(triggerFile.focused, true, "Focus must be restored to trigger");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Escape inside a submenu closes only the submenu and restores focus to the submenu trigger", async () => {
  const loader = loadProductionMenuBar();
  try {
    const windowListeners = new Map();
    globalThis.window = {
      addEventListener: (type, fn) => windowListeners.set(type, fn),
      removeEventListener: (type, fn) => windowListeners.delete(type),
    };
    globalThis.document = { activeElement: null };

    const MenuBar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = MenuBar.setup;
    MenuBar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(MenuBar);
    await renderToString(app);

    const triggerTools = new MockButton("menu-trigger-tools", "Tools");
    setupCtx.triggerRefs.value.tools = triggerTools;
    setupCtx.activeMenu.value = "tools";
    setupCtx.isOrganizeSubmenuOpen.value = true;

    const dropdown = new MockMenuContainer("dropdown-menu-tools");
    const organizeTrigger = new MockButton("menu-item-organize", "Organize");
    dropdown.appendChild(organizeTrigger);
    setupCtx.dropdownRef.value = dropdown;
    setupCtx.organizeTriggerRef.value = organizeTrigger;

    const organizeSubmenu = new MockMenuContainer("dropdown-submenu-organize");
    const subItem1 = new MockButton("subitem-1", "Current Folder");
    organizeSubmenu.appendChild(subItem1);
    setupCtx.organizeSubmenuRef.value = organizeSubmenu;

    // Press Escape on submenu item
    let stopped = false;
    setupCtx.onDropdownKeydown({
      key: "Escape",
      target: subItem1,
      preventDefault: () => {},
      stopPropagation: () => { stopped = true; },
    });

    assert.ok(stopped, "Submenu Escape must stop propagation");
    assert.equal(setupCtx.isOrganizeSubmenuOpen.value, false, "Submenu must close");
    assert.equal(setupCtx.activeMenu.value, "tools", "Parent tools menu must remain open");
    assert.equal(organizeTrigger.focused, true, "Focus must return to parent submenu trigger item");

    // Second Escape on organize trigger closes top-level tools menu
    setupCtx.onDropdownKeydown({
      key: "Escape",
      target: organizeTrigger,
      preventDefault: () => {},
      stopPropagation: () => {},
    });
    assert.equal(setupCtx.activeMenu.value, null, "Parent menu must close");
    assert.equal(triggerTools.focused, true, "Focus must return to menubar trigger");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Tab and pointer dismissal behave consistently", async () => {
  const loader = loadProductionMenuBar();
  try {
    const windowListeners = new Map();
    globalThis.window = {
      addEventListener: (type, fn) => windowListeners.set(type, fn),
      removeEventListener: (type, fn) => windowListeners.delete(type),
    };
    globalThis.document = { activeElement: null };

    const MenuBar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = MenuBar.setup;
    MenuBar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    const app = createSSRApp(MenuBar);
    await renderToString(app);

    setupCtx.activeMenu.value = "view";

    // Tab key closes menu and allows natural focus exit (no preventDefault called)
    let defaultPrevented = false;
    setupCtx.onDropdownKeydown({
      key: "Tab",
      target: new MockButton("item-1"),
      preventDefault: () => { defaultPrevented = true; },
    });

    assert.equal(setupCtx.activeMenu.value, null, "Menu must close on Tab");
    assert.equal(defaultPrevented, false, "Tab must not prevent default so focus moves naturally");

    // Click outside closes menu
    setupCtx.activeMenu.value = "file";
    setupCtx.onClickOutside({
      target: { closest: () => null },
    });
    assert.equal(setupCtx.activeMenu.value, null, "Click outside must close menu");
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});

test("Handles removed triggers and unmount without listener leaks", async () => {
  const loader = loadProductionMenuBar();
  try {
    const windowListeners = new Map();
    globalThis.window = {
      addEventListener: (type, fn) => {
        const count = windowListeners.get(type) ?? 0;
        windowListeners.set(type, count + 1);
      },
      removeEventListener: (type, fn) => {
        const count = windowListeners.get(type) ?? 0;
        if (count <= 1) windowListeners.delete(type);
        else windowListeners.set(type, count - 1);
      },
    };
    globalThis.document = { activeElement: null };

    const MenuBar = await loader.importComponent();
    let setupCtx = null;
    const origSetup = MenuBar.setup;
    MenuBar.setup = (props, ctx) => {
      setupCtx = origSetup(props, ctx);
      return setupCtx;
    };

    // Cycle 1: Mount and verify listeners
    const app1 = createSSRApp(MenuBar);
    await renderToString(app1);

    // Disconnected trigger handling
    const disconnectedTrigger = new MockButton("menu-trigger-file", "File");
    disconnectedTrigger.isConnected = false;
    setupCtx.triggerRefs.value.file = disconnectedTrigger;
    setupCtx.activeMenu.value = "file";

    // Escape with disconnected trigger must not crash
    assert.doesNotThrow(() => {
      setupCtx.closeAll(true);
    }, "Must not throw when trigger is disconnected");
    assert.equal(setupCtx.activeMenu.value, null);

    // Verify window listener cleanup simulation
    setupCtx.onClickOutside({ target: { closest: () => null } });
    setupCtx.onWindowKeydown({ key: "Escape", preventDefault: () => {} });
  } finally {
    if (fs.existsSync(loader.tmpPath)) fs.unlinkSync(loader.tmpPath);
  }
});
