<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, shallowRef } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { currentLocaleSetting, setLocale, SUPPORTED_LOCALES, t, type LocaleSetting } from "../i18n";

withDefaults(defineProps<{
  canOrganizeCurrent?: boolean;
  canOrganizeAll?: boolean;
  organizing?: boolean;
}>(), {
  canOrganizeCurrent: false,
  canOrganizeAll: false,
  organizing: false,
});

const emit = defineEmits<{
  addFolder: [];
  scanActive: [];
  rescanAll: [];
  openDbManager: [];
  openLegacyMigration: [];
  openSettings: [];
  selectAll: [];
  clearSelection: [];
  batchAlbum: [];
  batchTag: [];
  batchAutoTag: [];
  batchTrash: [];
  batchMove: [];
  batchCopy: [];
  batchExport: [];
  batchRate: [rating: number | null];
  setViewMode: [mode: "grid" | "table"];
  toggleSidebar: [];
  toggleInspector: [];
  openLightbox: [];
  zoomIn: [];
  zoomOut: [];
  resetZoom: [];
  openAutoTagger: [];
  openPromptStats: [];
  openModelManager: [];
  openClipManager: [];
  openLoraManager: [];
  organizeLibrary: [scope: "current" | "all"];
  openShortcutsHelp: [];
  openHelpGuide: [];
  openUpdater: [];
  openAbout: [];
}>();

const MENUS = ["file", "edit", "view", "tools", "help"] as const;
type MenuId = (typeof MENUS)[number];

const activeMenu = ref<MenuId | null>(null);
const focusedMenuTrigger = ref<MenuId>("file");
const isLanguageSubmenuOpen = ref(false);
const isOrganizeSubmenuOpen = ref(false);

const menuBarRef = shallowRef<HTMLElement | null>(null);
const triggerRefs = ref<Record<MenuId, HTMLButtonElement | null>>({
  file: null,
  edit: null,
  view: null,
  tools: null,
  help: null,
});
const dropdownRef = shallowRef<HTMLElement | null>(null);
const organizeSubmenuRef = shallowRef<HTMLElement | null>(null);
const languageSubmenuRef = shallowRef<HTMLElement | null>(null);
const organizeTriggerRef = shallowRef<HTMLButtonElement | null>(null);
const languageTriggerRef = shallowRef<HTMLButtonElement | null>(null);

function getDropdownItems(container: HTMLElement | null): HTMLButtonElement[] {
  if (!container) return [];
  return Array.from(
    container.querySelectorAll<HTMLButtonElement>(".dropdown-item:not(:disabled)"),
  ).filter((el) => el.closest('[role="menu"]') === container);
}

function focusFirstItem(container: HTMLElement | null) {
  const items = getDropdownItems(container);
  if (items.length > 0) {
    items[0]?.focus();
  }
}

function focusLastItem(container: HTMLElement | null) {
  const items = getDropdownItems(container);
  if (items.length > 0) {
    items[items.length - 1]?.focus();
  }
}

function focusTrigger(menuId: MenuId) {
  focusedMenuTrigger.value = menuId;
  const trigger = triggerRefs.value[menuId];
  if (trigger && trigger.isConnected) {
    trigger.focus();
  }
}

async function openMenu(menuId: MenuId, focusTarget: "first" | "last" | "none" = "first") {
  activeMenu.value = menuId;
  focusedMenuTrigger.value = menuId;
  isLanguageSubmenuOpen.value = false;
  isOrganizeSubmenuOpen.value = false;
  if (focusTarget !== "none") {
    await nextTick();
    if (focusTarget === "first") {
      focusFirstItem(dropdownRef.value);
    } else if (focusTarget === "last") {
      focusLastItem(dropdownRef.value);
    }
  }
}

function closeAll(restoreFocus = false) {
  const previousMenu = activeMenu.value;
  activeMenu.value = null;
  isLanguageSubmenuOpen.value = false;
  isOrganizeSubmenuOpen.value = false;
  if (restoreFocus && previousMenu) {
    focusTrigger(previousMenu);
  }
}

function toggleMenu(menu: MenuId) {
  if (activeMenu.value === menu) {
    closeAll(true);
  } else {
    openMenu(menu, "none");
  }
}

function onMenuHover(menu: MenuId) {
  if (activeMenu.value !== null) {
    activeMenu.value = menu;
    focusedMenuTrigger.value = menu;
    isLanguageSubmenuOpen.value = false;
    isOrganizeSubmenuOpen.value = false;
  }
}

async function openOrganizeSubmenu(focusFirst = true) {
  isOrganizeSubmenuOpen.value = true;
  if (focusFirst) {
    await nextTick();
    focusFirstItem(organizeSubmenuRef.value);
  }
}

function closeOrganizeSubmenu(returnFocus = true) {
  isOrganizeSubmenuOpen.value = false;
  if (returnFocus && organizeTriggerRef.value && organizeTriggerRef.value.isConnected) {
    organizeTriggerRef.value.focus();
  }
}

async function openLanguageSubmenu(focusFirst = true) {
  isLanguageSubmenuOpen.value = true;
  if (focusFirst) {
    await nextTick();
    const items = getDropdownItems(languageSubmenuRef.value);
    const selected = items.find((item) => item.classList.contains("selected"));
    (selected ?? items[0])?.focus();
  }
}

function closeLanguageSubmenu(returnFocus = true) {
  isLanguageSubmenuOpen.value = false;
  if (returnFocus && languageTriggerRef.value && languageTriggerRef.value.isConnected) {
    languageTriggerRef.value.focus();
  }
}

function handleAction(action: () => void) {
  const previousMenu = activeMenu.value;
  closeAll(false);
  action();
  if (previousMenu) {
    const trigger = triggerRefs.value[previousMenu];
    if (trigger && trigger.isConnected) {
      trigger.focus();
    }
  }
}

function onSelectLocale(setting: LocaleSetting) {
  const previousMenu = activeMenu.value;
  setLocale(setting);
  closeAll(false);
  if (previousMenu) {
    const trigger = triggerRefs.value[previousMenu];
    if (trigger && trigger.isConnected) {
      trigger.focus();
    }
  }
}

async function quitApp() {
  closeAll(false);
  try {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  } catch (err) {
    console.error("Quit app failed:", err);
  }
}

function onTriggerKeydown(event: KeyboardEvent, menuId: MenuId) {
  const currentIndex = MENUS.indexOf(menuId);
  switch (event.key) {
    case "ArrowRight": {
      event.preventDefault();
      const nextIndex = (currentIndex + 1) % MENUS.length;
      const nextMenu = MENUS[nextIndex];
      if (activeMenu.value !== null) {
        openMenu(nextMenu, "first");
      } else {
        focusTrigger(nextMenu);
      }
      break;
    }
    case "ArrowLeft": {
      event.preventDefault();
      const prevIndex = (currentIndex - 1 + MENUS.length) % MENUS.length;
      const prevMenu = MENUS[prevIndex];
      if (activeMenu.value !== null) {
        openMenu(prevMenu, "first");
      } else {
        focusTrigger(prevMenu);
      }
      break;
    }
    case "ArrowDown": {
      event.preventDefault();
      openMenu(menuId, "first");
      break;
    }
    case "ArrowUp": {
      event.preventDefault();
      openMenu(menuId, "last");
      break;
    }
    case "Home": {
      event.preventDefault();
      focusTrigger(MENUS[0]);
      break;
    }
    case "End": {
      event.preventDefault();
      focusTrigger(MENUS[MENUS.length - 1]);
      break;
    }
    case "Enter":
    case " ": {
      event.preventDefault();
      if (activeMenu.value === menuId) {
        closeAll(true);
      } else {
        openMenu(menuId, "first");
      }
      break;
    }
    case "Escape": {
      if (activeMenu.value !== null) {
        event.preventDefault();
        closeAll(true);
      }
      break;
    }
    case "Tab": {
      if (activeMenu.value !== null) {
        closeAll(false);
      }
      break;
    }
  }
}

function onDropdownKeydown(event: KeyboardEvent) {
  const currentMenu = activeMenu.value;
  if (!currentMenu) return;

  const target = event.target as HTMLElement;
  const isInsideOrganizeSubmenu = !!organizeSubmenuRef.value?.contains(target);
  const isInsideLanguageSubmenu = !!languageSubmenuRef.value?.contains(target);

  if (isInsideOrganizeSubmenu) {
    handleSubmenuKeydown(event, "organize");
    return;
  }
  if (isInsideLanguageSubmenu) {
    handleSubmenuKeydown(event, "language");
    return;
  }

  const items = getDropdownItems(dropdownRef.value);
  const currentIndex = items.indexOf(target as HTMLButtonElement);

  switch (event.key) {
    case "ArrowDown": {
      event.preventDefault();
      if (items.length === 0) return;
      const nextIndex = currentIndex >= 0 ? (currentIndex + 1) % items.length : 0;
      items[nextIndex]?.focus();
      break;
    }
    case "ArrowUp": {
      event.preventDefault();
      if (items.length === 0) return;
      const prevIndex = currentIndex >= 0 ? (currentIndex - 1 + items.length) % items.length : items.length - 1;
      items[prevIndex]?.focus();
      break;
    }
    case "ArrowRight": {
      event.preventDefault();
      if (target === organizeTriggerRef.value) {
        openOrganizeSubmenu(true);
      } else if (target === languageTriggerRef.value) {
        openLanguageSubmenu(true);
      } else {
        const menuIdx = MENUS.indexOf(currentMenu);
        const nextMenu = MENUS[(menuIdx + 1) % MENUS.length];
        openMenu(nextMenu, "first");
      }
      break;
    }
    case "ArrowLeft": {
      event.preventDefault();
      const menuIdx = MENUS.indexOf(currentMenu);
      const prevMenu = MENUS[(menuIdx - 1 + MENUS.length) % MENUS.length];
      openMenu(prevMenu, "first");
      break;
    }
    case "Home": {
      event.preventDefault();
      items[0]?.focus();
      break;
    }
    case "End": {
      event.preventDefault();
      items[items.length - 1]?.focus();
      break;
    }
    case "Escape": {
      event.preventDefault();
      event.stopPropagation();
      closeAll(true);
      break;
    }
    case "Tab": {
      closeAll(false);
      break;
    }
    case "Enter":
    case " ": {
      if (target === organizeTriggerRef.value) {
        event.preventDefault();
        openOrganizeSubmenu(true);
      } else if (target === languageTriggerRef.value) {
        event.preventDefault();
        openLanguageSubmenu(true);
      }
      break;
    }
  }
}

function handleSubmenuKeydown(event: KeyboardEvent, which: "organize" | "language") {
  const container = which === "organize" ? organizeSubmenuRef.value : languageSubmenuRef.value;
  const items = getDropdownItems(container);
  const target = event.target as HTMLButtonElement;
  const currentIndex = items.indexOf(target);

  switch (event.key) {
    case "ArrowDown": {
      event.preventDefault();
      if (items.length === 0) return;
      const nextIndex = currentIndex >= 0 ? (currentIndex + 1) % items.length : 0;
      items[nextIndex]?.focus();
      break;
    }
    case "ArrowUp": {
      event.preventDefault();
      if (items.length === 0) return;
      const prevIndex = currentIndex >= 0 ? (currentIndex - 1 + items.length) % items.length : items.length - 1;
      items[prevIndex]?.focus();
      break;
    }
    case "ArrowLeft":
    case "Escape": {
      event.preventDefault();
      event.stopPropagation();
      if (which === "organize") {
        closeOrganizeSubmenu(true);
      } else {
        closeLanguageSubmenu(true);
      }
      break;
    }
    case "Tab": {
      closeAll(false);
      break;
    }
  }
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest(".menu-bar-container")) {
    closeAll(false);
  }
}

function onWindowKeydown(e: KeyboardEvent) {
  if (activeMenu.value !== null && e.key === "Escape") {
    e.preventDefault();
    closeAll(true);
  }
}

onMounted(() => {
  window.addEventListener("click", onClickOutside);
  window.addEventListener("keydown", onWindowKeydown);
});

onUnmounted(() => {
  window.removeEventListener("click", onClickOutside);
  window.removeEventListener("keydown", onWindowKeydown);
});
</script>

<template>
  <nav
    ref="menuBarRef"
    class="menu-bar-container"
    role="menubar"
    :aria-label="t.menu.applicationMenu || 'Application Menu'"
    data-tauri-drag-region="false"
  >
    <!-- File Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'file' }">
      <button
        id="menu-trigger-file"
        :ref="(el) => { triggerRefs.file = el as HTMLButtonElement | null; }"
        type="button"
        class="menu-trigger"
        role="menuitem"
        aria-haspopup="true"
        :aria-expanded="activeMenu === 'file'"
        aria-controls="dropdown-menu-file"
        :tabindex="focusedMenuTrigger === 'file' ? 0 : -1"
        @click="toggleMenu('file')"
        @mouseenter="onMenuHover('file')"
        @keydown="onTriggerKeydown($event, 'file')"
      >
        {{ t.menu.file }}
      </button>
      <div
        v-if="activeMenu === 'file'"
        id="dropdown-menu-file"
        ref="dropdownRef"
        class="dropdown-menu"
        role="menu"
        :aria-label="t.menu.file"
        aria-labelledby="menu-trigger-file"
        tabindex="-1"
        @keydown="onDropdownKeydown"
      >
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Control+O"
          @click="handleAction(() => emit('addFolder'))"
        >
          <span class="item-icon" aria-hidden="true">📁</span>
          <span class="item-title">{{ t.menu.addFolder }}</span>
          <span class="item-key" aria-hidden="true">Ctrl+O</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('scanActive'))"
        >
          <span class="item-icon" aria-hidden="true">🔄</span>
          <span class="item-title">{{ t.menu.scanActive }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('rescanAll'))"
        >
          <span class="item-icon" aria-hidden="true">⚡</span>
          <span class="item-title">{{ t.menu.rescanAll }}</span>
        </button>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openDbManager'))"
        >
          <span class="item-icon" aria-hidden="true">🗄️</span>
          <span class="item-title">{{ t.menu.dbManager }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openLegacyMigration'))"
        >
          <span class="item-icon" aria-hidden="true">📦</span>
          <span class="item-title">{{ t.legacyMigration.title }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Control+,"
          @click="handleAction(() => emit('openSettings'))"
        >
          <span class="item-icon" aria-hidden="true">⚙️</span>
          <span class="item-title">{{ t.menu.preferences }}</span>
          <span class="item-key" aria-hidden="true">Ctrl+,</span>
        </button>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item danger"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Alt+F4"
          @click="quitApp"
        >
          <span class="item-icon" aria-hidden="true">❌</span>
          <span class="item-title">{{ t.menu.exit }}</span>
          <span class="item-key" aria-hidden="true">Alt+F4</span>
        </button>
      </div>
    </div>

    <!-- Edit Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'edit' }">
      <button
        id="menu-trigger-edit"
        :ref="(el) => { triggerRefs.edit = el as HTMLButtonElement | null; }"
        type="button"
        class="menu-trigger"
        role="menuitem"
        aria-haspopup="true"
        :aria-expanded="activeMenu === 'edit'"
        aria-controls="dropdown-menu-edit"
        :tabindex="focusedMenuTrigger === 'edit' ? 0 : -1"
        @click="toggleMenu('edit')"
        @mouseenter="onMenuHover('edit')"
        @keydown="onTriggerKeydown($event, 'edit')"
      >
        {{ t.menu.edit }}
      </button>
      <div
        v-if="activeMenu === 'edit'"
        id="dropdown-menu-edit"
        ref="dropdownRef"
        class="dropdown-menu"
        role="menu"
        :aria-label="t.menu.edit"
        aria-labelledby="menu-trigger-edit"
        tabindex="-1"
        @keydown="onDropdownKeydown"
      >
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Control+A"
          @click="handleAction(() => emit('selectAll'))"
        >
          <span class="item-icon" aria-hidden="true">✓</span>
          <span class="item-title">{{ t.menu.selectAll }}</span>
          <span class="item-key" aria-hidden="true">Ctrl+A</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Escape"
          @click="handleAction(() => emit('clearSelection'))"
        >
          <span class="item-icon" aria-hidden="true">✕</span>
          <span class="item-title">{{ t.menu.clearSelection }}</span>
          <span class="item-key" aria-hidden="true">Esc</span>
        </button>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('batchTag'))"
        >
          <span class="item-icon" aria-hidden="true">🏷️</span>
          <span class="item-title">{{ t.menu.batchTag }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('batchAutoTag'))"
        >
          <span class="item-icon" aria-hidden="true">🤖</span>
          <span class="item-title">{{ t.menu.batchAutoTag }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('batchAlbum'))"
        >
          <span class="item-icon" aria-hidden="true">📚</span>
          <span class="item-title">{{ t.menu.batchAlbum }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('batchMove'))"
        >
          <span class="item-icon" aria-hidden="true">↗</span>
          <span class="item-title">{{ t.menu.batchMove }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('batchCopy'))"
        >
          <span class="item-icon" aria-hidden="true">📋</span>
          <span class="item-title">{{ t.menu.batchCopy }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Control+E"
          @click="handleAction(() => emit('batchExport'))"
        >
          <span class="item-icon" aria-hidden="true">📤</span>
          <span class="item-title">{{ t.batch.export }}</span>
          <span class="item-key" aria-hidden="true">Ctrl+E</span>
        </button>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item danger"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Delete"
          @click="handleAction(() => emit('batchTrash'))"
        >
          <span class="item-icon" aria-hidden="true">🗑️</span>
          <span class="item-title">{{ t.menu.batchTrash }}</span>
          <span class="item-key" aria-hidden="true">Del</span>
        </button>
      </div>
    </div>

    <!-- View Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'view' }">
      <button
        id="menu-trigger-view"
        :ref="(el) => { triggerRefs.view = el as HTMLButtonElement | null; }"
        type="button"
        class="menu-trigger"
        role="menuitem"
        aria-haspopup="true"
        :aria-expanded="activeMenu === 'view'"
        aria-controls="dropdown-menu-view"
        :tabindex="focusedMenuTrigger === 'view' ? 0 : -1"
        @click="toggleMenu('view')"
        @mouseenter="onMenuHover('view')"
        @keydown="onTriggerKeydown($event, 'view')"
      >
        {{ t.menu.view }}
      </button>
      <div
        v-if="activeMenu === 'view'"
        id="dropdown-menu-view"
        ref="dropdownRef"
        class="dropdown-menu"
        role="menu"
        :aria-label="t.menu.view"
        aria-labelledby="menu-trigger-view"
        tabindex="-1"
        @keydown="onDropdownKeydown"
      >
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('setViewMode', 'grid'))"
        >
          <span class="item-icon" aria-hidden="true">⊞</span>
          <span class="item-title">{{ t.menu.grid }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('setViewMode', 'table'))"
        >
          <span class="item-icon" aria-hidden="true">☰</span>
          <span class="item-title">{{ t.menu.table }}</span>
        </button>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="B"
          @click="handleAction(() => emit('toggleSidebar'))"
        >
          <span class="item-icon" aria-hidden="true">📁</span>
          <span class="item-title">{{ t.menu.toggleSidebar }}</span>
          <span class="item-key" aria-hidden="true">B</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="I"
          @click="handleAction(() => emit('toggleInspector'))"
        >
          <span class="item-icon" aria-hidden="true">👁️</span>
          <span class="item-title">{{ t.menu.toggleInspector }}</span>
          <span class="item-key" aria-hidden="true">I</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="Space"
          @click="handleAction(() => emit('openLightbox'))"
        >
          <span class="item-icon" aria-hidden="true">🔍</span>
          <span class="item-title">{{ t.menu.lightbox }}</span>
          <span class="item-key" aria-hidden="true">Space</span>
        </button>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('zoomIn'))"
        >
          <span class="item-icon" aria-hidden="true">➕</span>
          <span class="item-title">{{ t.menu.zoomIn }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('zoomOut'))"
        >
          <span class="item-icon" aria-hidden="true">➖</span>
          <span class="item-title">{{ t.menu.zoomOut }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('resetZoom'))"
        >
          <span class="item-icon" aria-hidden="true">↺</span>
          <span class="item-title">{{ t.menu.resetZoom }}</span>
        </button>
      </div>
    </div>

    <!-- Tools Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'tools' }">
      <button
        id="menu-trigger-tools"
        :ref="(el) => { triggerRefs.tools = el as HTMLButtonElement | null; }"
        type="button"
        class="menu-trigger"
        role="menuitem"
        aria-haspopup="true"
        :aria-expanded="activeMenu === 'tools'"
        aria-controls="dropdown-menu-tools"
        :tabindex="focusedMenuTrigger === 'tools' ? 0 : -1"
        @click="toggleMenu('tools')"
        @mouseenter="onMenuHover('tools')"
        @keydown="onTriggerKeydown($event, 'tools')"
      >
        {{ t.menu.tools }}
      </button>
      <div
        v-if="activeMenu === 'tools'"
        id="dropdown-menu-tools"
        ref="dropdownRef"
        class="dropdown-menu"
        role="menu"
        :aria-label="t.menu.tools"
        aria-labelledby="menu-trigger-tools"
        tabindex="-1"
        @keydown="onDropdownKeydown"
      >
        <div
          class="dropdown-submenu-wrapper"
          @mouseenter="isOrganizeSubmenuOpen = true"
          @mouseleave="isOrganizeSubmenuOpen = false"
        >
          <button
            id="menu-item-organize"
            ref="organizeTriggerRef"
            type="button"
            class="dropdown-item has-submenu"
            role="menuitem"
            aria-haspopup="true"
            :aria-expanded="isOrganizeSubmenuOpen"
            aria-controls="dropdown-submenu-organize"
            tabindex="-1"
            @click="isOrganizeSubmenuOpen ? closeOrganizeSubmenu(true) : openOrganizeSubmenu(true)"
          >
            <span class="item-icon" aria-hidden="true">🗂️</span>
            <span class="item-title">{{ t.menu.organizeByPrompt }}</span>
            <span class="submenu-arrow" aria-hidden="true">▶</span>
          </button>
          <div
            v-if="isOrganizeSubmenuOpen"
            id="dropdown-submenu-organize"
            ref="organizeSubmenuRef"
            class="dropdown-submenu organize-submenu"
            role="menu"
            :aria-label="t.menu.organizeByPrompt"
            aria-labelledby="menu-item-organize"
            tabindex="-1"
          >
            <button
              type="button"
              class="dropdown-item"
              role="menuitem"
              tabindex="-1"
              :disabled="!canOrganizeCurrent || organizing"
              :aria-disabled="!canOrganizeCurrent || organizing"
              @click="handleAction(() => emit('organizeLibrary', 'current'))"
            >
              <span class="item-icon" aria-hidden="true">📂</span>
              <span class="item-title">{{ t.menu.organizeCurrentFolder }}</span>
            </button>
            <button
              type="button"
              class="dropdown-item"
              role="menuitem"
              tabindex="-1"
              :disabled="!canOrganizeAll || organizing"
              :aria-disabled="!canOrganizeAll || organizing"
              @click="handleAction(() => emit('organizeLibrary', 'all'))"
            >
              <span class="item-icon" aria-hidden="true">🗃️</span>
              <span class="item-title">{{ t.menu.organizeAllFolders }}</span>
            </button>
          </div>
        </div>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openPromptStats'))"
        >
          <span class="item-icon" aria-hidden="true">📊</span>
          <span class="item-title">{{ t.menu.promptStats }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openAutoTagger'))"
        >
          <span class="item-icon" aria-hidden="true">🤖</span>
          <span class="item-title">{{ t.menu.autoTagger }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openModelManager'))"
        >
          <span class="item-icon" aria-hidden="true">🧠</span>
          <span class="item-title">{{ t.menu.modelManager }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openClipManager'))"
        >
          <span class="item-icon" aria-hidden="true">🔎</span>
          <span class="item-title">{{ t.clipModal.title }}</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openLoraManager'))"
        >
          <span class="item-icon" aria-hidden="true">🎨</span>
          <span class="item-title">{{ t.loraModal.title }}</span>
        </button>
      </div>
    </div>

    <!-- Help Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'help' }">
      <button
        id="menu-trigger-help"
        :ref="(el) => { triggerRefs.help = el as HTMLButtonElement | null; }"
        type="button"
        class="menu-trigger"
        role="menuitem"
        aria-haspopup="true"
        :aria-expanded="activeMenu === 'help'"
        aria-controls="dropdown-menu-help"
        :tabindex="focusedMenuTrigger === 'help' ? 0 : -1"
        @click="toggleMenu('help')"
        @mouseenter="onMenuHover('help')"
        @keydown="onTriggerKeydown($event, 'help')"
      >
        {{ t.menu.help }}
      </button>
      <div
        v-if="activeMenu === 'help'"
        id="dropdown-menu-help"
        ref="dropdownRef"
        class="dropdown-menu"
        role="menu"
        :aria-label="t.menu.help"
        aria-labelledby="menu-trigger-help"
        tabindex="-1"
        @keydown="onDropdownKeydown"
      >
        <!-- Language Submenu -->
        <div
          class="dropdown-submenu-wrapper"
          @mouseenter="isLanguageSubmenuOpen = true"
          @mouseleave="isLanguageSubmenuOpen = false"
        >
          <button
            id="menu-item-language"
            ref="languageTriggerRef"
            type="button"
            class="dropdown-item has-submenu"
            role="menuitem"
            aria-haspopup="true"
            :aria-expanded="isLanguageSubmenuOpen"
            aria-controls="dropdown-submenu-language"
            tabindex="-1"
            @click="isLanguageSubmenuOpen ? closeLanguageSubmenu(true) : openLanguageSubmenu(true)"
          >
            <span class="item-icon" aria-hidden="true">🌐</span>
            <span class="item-title">{{ t.menu.language }}</span>
            <span class="submenu-arrow" aria-hidden="true">▶</span>
          </button>
          <div
            v-if="isLanguageSubmenuOpen"
            id="dropdown-submenu-language"
            ref="languageSubmenuRef"
            class="dropdown-submenu"
            role="menu"
            :aria-label="t.menu.language"
            aria-labelledby="menu-item-language"
            tabindex="-1"
          >
            <button
              v-for="loc in SUPPORTED_LOCALES"
              :key="loc.key"
              type="button"
              class="dropdown-item"
              :class="{ selected: currentLocaleSetting === loc.key }"
              role="menuitemradio"
              :aria-checked="currentLocaleSetting === loc.key"
              tabindex="-1"
              @click="onSelectLocale(loc.key)"
            >
              <span class="check-icon" aria-hidden="true">{{ currentLocaleSetting === loc.key ? '✓' : '' }}</span>
              <span class="item-title">{{ loc.label }}</span>
            </button>
          </div>
        </div>

        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="F1"
          @click="handleAction(() => emit('openHelpGuide'))"
        >
          <span class="item-icon" aria-hidden="true">📖</span>
          <span class="item-title">{{ t.menu.helpGuide || 'Feature Guide & Documentation' }}</span>
          <span class="item-key" aria-hidden="true">F1</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          aria-keyshortcuts="?"
          @click="handleAction(() => emit('openShortcutsHelp'))"
        >
          <span class="item-icon" aria-hidden="true">⌨️</span>
          <span class="item-title">{{ t.menu.shortcuts }}</span>
          <span class="item-key" aria-hidden="true">?</span>
        </button>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openUpdater'))"
        >
          <span class="item-icon" aria-hidden="true">🚀</span>
          <span class="item-title">{{ t.menu.checkUpdates }}</span>
        </button>
        <div class="menu-divider" role="separator" aria-orientation="horizontal"></div>
        <button
          type="button"
          class="dropdown-item"
          role="menuitem"
          tabindex="-1"
          @click="handleAction(() => emit('openAbout'))"
        >
          <span class="item-icon" aria-hidden="true">
            <img src="../assets/logo.png" alt="" width="14" height="14" style="display:block; object-fit:contain;" />
          </span>
          <span class="item-title">{{ t.menu.about }}</span>
        </button>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.menu-bar-container {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 100%;
  user-select: none;
  font-size: 0.76rem;
  z-index: 1000;
  position: relative;
}

.menu-item {
  position: relative;
  height: 100%;
  display: flex;
  align-items: center;
}

.menu-trigger {
  background: transparent;
  border: none;
  color: var(--color-text-secondary);
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.74rem;
  font-family: inherit;
  transition: all 0.12s ease;
  white-space: nowrap;
  outline: none;
}

.menu-trigger:hover,
.menu-item.open .menu-trigger {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.menu-trigger:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: 1px;
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.25);
  min-width: 190px;
  padding: 4px 0;
  z-index: 1001;
  display: flex;
  flex-direction: column;
  backdrop-filter: blur(8px);
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: transparent;
  border: none;
  color: var(--color-text-secondary);
  font-size: 0.74rem;
  font-family: inherit;
  cursor: pointer;
  text-align: left;
  transition: background-color 0.1s, color 0.1s;
  width: 100%;
  outline: none;
}

.dropdown-item:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.dropdown-item:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: -2px;
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.dropdown-item:disabled {
  cursor: default;
  opacity: 0.45;
  background: transparent;
  color: var(--color-text-muted);
}

.dropdown-item.danger:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

.dropdown-item.danger:focus-visible {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
  outline: 2px solid rgba(239, 68, 68, 0.8);
}

.item-icon {
  width: 16px;
  font-size: 0.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.item-title {
  flex: 1;
  white-space: nowrap;
}

.item-key {
  font-size: 0.68rem;
  color: var(--color-text-muted);
  margin-left: 12px;
  flex-shrink: 0;
}

.menu-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}

.dropdown-submenu-wrapper {
  position: relative;
  width: 100%;
}

.dropdown-item.has-submenu {
  display: flex;
  justify-content: space-between;
}

.submenu-arrow {
  font-size: 0.6rem;
  color: var(--color-text-muted);
}

.dropdown-submenu {
  position: absolute;
  top: 0;
  left: 100%;
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.25);
  min-width: 175px;
  padding: 4px 0;
  z-index: 1002;
  display: flex;
  flex-direction: column;
}

.organize-submenu {
  min-width: 230px;
}

.check-icon {
  width: 14px;
  font-size: 0.72rem;
  color: var(--badge-cyan-text, #155e75);
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dropdown-item.selected {
  color: var(--badge-cyan-text, #155e75);
  font-weight: 600;
}

@media (prefers-reduced-motion: reduce) {
  .menu-trigger,
  .dropdown-item {
    transition: none;
  }
}
</style>
