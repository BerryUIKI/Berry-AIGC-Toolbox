<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type { Album, Folder, LibraryCounts, NavTarget, ScanProgress, ScanStats, SubdirectoryEntry, Tag } from "../types";
import FolderTreeNode from "./FolderTreeNode.vue";
import { useNotification } from "../utils/notification";

const props = defineProps<{
  folders: Folder[];
  counts?: LibraryCounts | null;
  albums?: Album[];
  albumCounts?: Record<number, number>;
  tags?: Tag[];
  tagCounts?: Record<number, number>;
  activeTarget: NavTarget;
  progress: ScanProgress | null;
  collapsed?: boolean;
}>();

const emit = defineEmits<{
  folderAdded: [folder: Folder];
  removed: [folderId: number];
  scanned: [folderId: number];
  selectNav: [target: NavTarget];
  openAlbumModal: [];
  openTagModal: [];
  openPromptStats: [];
  openAutoTagger: [];
  openModelManager: [];
  openDbManager: [];
  openShortcutsHelp: [];
  openAddFolderModal: [];
  openImportModal: [payload: { filePaths: string[]; folderId?: number | null; albumId?: number | null }];
  moveFilesToFolder: [payload: { filePaths: string[]; folderId: number }];
  addFilesToAlbum: [payload: { fileIds: number[]; albumId: number }];
  importExternalFilesToAlbum: [payload: { filePaths: string[]; albumId: number }];
  tagFiles: [payload: { fileIds: number[]; tagId: number }];
  toggleCollapse: [];
}>();

const addingFolder = ref(false);
const running = ref<{ id: number; action: "scan" | "rebuild" | "harvest" } | null>(null);
const error = ref("");
const notification = useNotification();
const sidebarRef = ref<HTMLElement | null>(null);

const tagQuery = ref("");
const tagSearchInputRef = ref<HTMLInputElement | null>(null);
const MAX_VISIBLE_TAGS = 200;

const normalizedTagQuery = computed(() => {
  return tagQuery.value.trim().normalize("NFC").toLocaleLowerCase();
});

const filteredTags = computed(() => {
  const allTags = props.tags || [];
  const q = normalizedTagQuery.value;
  if (!q) {
    if (allTags.length <= MAX_VISIBLE_TAGS) {
      return allTags;
    }
    const sliced = allTags.slice(0, MAX_VISIBLE_TAGS);
    if (props.activeTarget.type === "tag") {
      const activeId = props.activeTarget.tag.id;
      if (!sliced.some((t) => t.id === activeId)) {
        const activeTag = allTags.find((t) => t.id === activeId);
        if (activeTag) {
          sliced.push(activeTag);
        }
      }
    }
    return sliced;
  }

  const matches: Tag[] = [];
  for (let i = 0; i < allTags.length; i++) {
    const tag = allTags[i];
    if (tag.name.normalize("NFC").toLocaleLowerCase().includes(q)) {
      matches.push(tag);
      if (matches.length >= MAX_VISIBLE_TAGS) {
        break;
      }
    }
  }
  return matches;
});

function clearTagFilter() {
  tagQuery.value = "";
  if (tagSearchInputRef.value) {
    tagSearchInputRef.value.focus();
  }
}

function onTagSearchKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (!sidebarRef.value) return;
    const firstChip = sidebarRef.value.querySelector<HTMLElement>(".tag-chip-eagle");
    if (firstChip) {
      firstChip.focus();
      firstChip.scrollIntoView({ block: "nearest" });
    } else {
      const toolBtn = sidebarRef.value.querySelector<HTMLElement>(".tool-btn");
      if (toolBtn) {
        toolBtn.focus();
        toolBtn.scrollIntoView({ block: "nearest" });
      }
    }
  } else if (e.key === "Enter") {
    if (filteredTags.value.length > 0) {
      e.preventDefault();
      const firstTag = filteredTags.value[0];
      emit("selectNav", { type: "tag", tag: firstTag });
      if (sidebarRef.value) {
        const firstChip = sidebarRef.value.querySelector<HTMLElement>(".tag-chip-eagle");
        firstChip?.focus();
      }
    }
  } else if (e.key === "Escape") {
    if (tagQuery.value) {
      e.preventDefault();
      clearTagFilter();
    }
  }
}

const subdirectories = ref<Record<string, SubdirectoryEntry[]>>({});
const expandedPaths = ref<Set<string>>(new Set());
const loadingPaths = ref<Set<string>>(new Set());

function displayPath(path: string): string {
  return path.replace(/^\\\\\?\\/, "");
}

function getFolderName(path: string): string {
  const clean = displayPath(path);
  const parts = clean.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] || clean;
}

function isBusy(id: number): boolean {
  return running.value?.id === id;
}

function pickFolder() {
  emit("openAddFolderModal");
}

async function harvest(folder: Folder, e: MouseEvent) {
  e.stopPropagation();
  error.value = "";
  running.value = { id: folder.id, action: "harvest" };
  try {
    await invoke<number>("harvest_pipeline_folder", { folderId: folder.id });
    emit("scanned", folder.id);
  } catch (e) {
    error.value = String(e);
    notification.showError(String(e), "Harvest failed");
  } finally {
    running.value = null;
  }
}

async function scan(folder: Folder, action: "scan" | "rebuild" = "scan") {
  error.value = "";
  running.value = { id: folder.id, action };
  try {
    await invoke<ScanStats>(
      action === "rebuild" ? "rebuild_metadata" : "scan_folder",
      { folderId: folder.id },
    );
    if (expandedPaths.value.has(folder.path)) {
      void loadSubdirectories(folder, folder.path);
    }
    emit("scanned", folder.id);
  } catch (e) {
    error.value = String(e);
    notification.showError(String(e), "Folder scan failed");
  } finally {
    running.value = null;
  }
}

async function removeFolder(folder: Folder, e: MouseEvent | KeyboardEvent) {
  e.stopPropagation();
  const folderName = getFolderName(folder.path);
  const template = t.value.nav.removeFolderConfirm || 'Remove folder "{name}" from Omera? Files on disk will not be deleted.';
  const confirmMsg = template.replace("{name}", folderName);
  if (!window.confirm(confirmMsg)) {
    return;
  }
  error.value = "";
  try {
    let fallbackTarget: HTMLElement | null = null;
    if (typeof document !== "undefined" && sidebarRef.value) {
      const currentFolderRow = sidebarRef.value.querySelector(`[data-folder-id="${folder.id}"]`);
      if (currentFolderRow && currentFolderRow.contains(document.activeElement)) {
        const allFolderRows = Array.from(sidebarRef.value.querySelectorAll(".folder-row-container"));
        const idx = allFolderRows.indexOf(currentFolderRow as any);
        if (idx > 0) {
          fallbackTarget = allFolderRows[idx - 1].querySelector(".folder-item-btn, .tree-arrow-btn");
        } else if (idx < allFolderRows.length - 1) {
          fallbackTarget = allFolderRows[idx + 1].querySelector(".folder-item-btn, .tree-arrow-btn");
        } else {
          fallbackTarget = sidebarRef.value.querySelector(".group-action-btn");
        }
      }
    }

    await invoke("remove_folder", { folderId: folder.id });
    expandedPaths.value.delete(folder.path);
    delete subdirectories.value[folder.path];
    emit("removed", folder.id);

    if (fallbackTarget && fallbackTarget.isConnected) {
      fallbackTarget.focus();
    }
  } catch (e) {
    error.value = String(e);
    notification.showError(String(e), "Failed to remove folder");
  }
}

async function loadSubdirectories(folder: Folder, path: string) {
  loadingPaths.value.add(path);
  loadingPaths.value = new Set(loadingPaths.value);
  try {
    const entries = await invoke<SubdirectoryEntry[]>("list_subdirectories", {
      folderId: folder.id,
      dirPath: path,
    });
    subdirectories.value[path] = entries;
  } catch (err) {
    console.error("Failed to load subdirectories for", path, err);
  } finally {
    loadingPaths.value.delete(path);
    loadingPaths.value = new Set(loadingPaths.value);
  }
}

function escapeCss(str: string): string {
  if (typeof CSS !== "undefined" && typeof CSS.escape === "function") {
    return CSS.escape(str);
  }
  return str.replace(/([!"#$%&'()*+,.\/:;<=>?@[\\\]^`{|}~])/g, "\\$1");
}

async function toggleFolderExpand(folder: Folder) {
  const rootPath = folder.path;
  if (expandedPaths.value.has(rootPath)) {
    if (typeof document !== "undefined" && document.activeElement && sidebarRef.value) {
      const activeEl = document.activeElement as HTMLElement;
      const folderRow = sidebarRef.value.querySelector(`[data-folder-path="${escapeCss(rootPath)}"]`);
      if (folderRow && folderRow.contains(activeEl)) {
        const btn =
          folderRow.querySelector<HTMLElement>(".folder-item-btn") ||
          folderRow.querySelector<HTMLElement>(".tree-arrow-btn");
        btn?.focus();
      }
    }
    expandedPaths.value.delete(rootPath);
    expandedPaths.value = new Set(expandedPaths.value);
    return;
  }
  expandedPaths.value.add(rootPath);
  expandedPaths.value = new Set(expandedPaths.value);

  if (!subdirectories.value[rootPath]) {
    await loadSubdirectories(folder, rootPath);
  }
}

async function toggleSubfolderExpand(folder: Folder, path: string) {
  if (expandedPaths.value.has(path)) {
    if (typeof document !== "undefined" && document.activeElement && sidebarRef.value) {
      const activeEl = document.activeElement as HTMLElement;
      const subfolderEl = sidebarRef.value.querySelector(`[data-subfolder-path="${escapeCss(path)}"]`);
      if (subfolderEl && subfolderEl.contains(activeEl)) {
        const btn =
          subfolderEl.querySelector<HTMLElement>(".subfolder-header") ||
          subfolderEl.querySelector<HTMLElement>(".tree-arrow-btn");
        btn?.focus();
      }
    }
    expandedPaths.value.delete(path);
    expandedPaths.value = new Set(expandedPaths.value);
    return;
  }
  expandedPaths.value.add(path);
  expandedPaths.value = new Set(expandedPaths.value);

  if (!subdirectories.value[path]) {
    await loadSubdirectories(folder, path);
  }
}

function selectSubfolder(folder: Folder, subfolderPath: string) {
  emit("selectNav", {
    type: "folder",
    folder,
    subfolderPath,
    recursive: false,
  });
}

function getNavigableElements(): HTMLElement[] {
  if (!sidebarRef.value) return [];
  const selector = [
    ".nav-item:not(.tree-arrow-btn):not(.icon-btn)",
    ".subfolder-header",
    ".tag-chip-eagle",
    ".tool-btn",
  ].join(", ");
  const elements = Array.from(sidebarRef.value.querySelectorAll<HTMLElement>(selector));
  return elements.filter((el) => {
    return !el.hasAttribute("disabled") && el.offsetParent !== null;
  });
}

function focusNextNavItem(current: HTMLElement, offset: number) {
  const items = getNavigableElements();
  if (!items.length) return;

  let currentIndex = items.indexOf(current);
  if (currentIndex === -1) {
    const rowNav = current.closest("li, .tag-chip-eagle")?.querySelector<HTMLElement>(
      ".nav-item, .subfolder-header, .tag-chip-eagle, .tool-btn",
    );
    if (rowNav) {
      currentIndex = items.indexOf(rowNav);
    }
  }

  let nextIndex: number;
  if (currentIndex === -1) {
    nextIndex = offset > 0 ? 0 : items.length - 1;
  } else {
    nextIndex = currentIndex + offset;
    if (nextIndex < 0) nextIndex = 0;
    else if (nextIndex >= items.length) nextIndex = items.length - 1;
  }

  const nextItem = items[nextIndex];
  if (nextItem) {
    nextItem.focus();
    nextItem.scrollIntoView({ block: "nearest", inline: "nearest" });
  }
}

function focusFirstNavItem() {
  const items = getNavigableElements();
  if (items.length > 0) {
    items[0].focus();
    items[0].scrollIntoView({ block: "nearest", inline: "nearest" });
  }
}

function focusLastNavItem() {
  const items = getNavigableElements();
  if (items.length > 0) {
    items[items.length - 1].focus();
    items[items.length - 1].scrollIntoView({ block: "nearest", inline: "nearest" });
  }
}

function focusNextTagChip(current: HTMLElement, offset: number) {
  if (!sidebarRef.value) return;
  const chips = Array.from(sidebarRef.value.querySelectorAll<HTMLElement>(".tag-chip-eagle"));
  const idx = chips.indexOf(current);
  if (idx !== -1) {
    const nextIdx = Math.max(0, Math.min(chips.length - 1, idx + offset));
    chips[nextIdx]?.focus();
    chips[nextIdx]?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }
}

function onFolderKeydown(e: KeyboardEvent, folder: Folder) {
  if (e.key === "ArrowRight") {
    if (!expandedPaths.value.has(folder.path)) {
      e.preventDefault();
      void toggleFolderExpand(folder);
    } else {
      const currentLi = (e.target as HTMLElement).closest("li");
      const firstChildBtn = currentLi?.querySelector<HTMLElement>(
        ".subfolder-tree-list .subfolder-header",
      );
      if (firstChildBtn) {
        e.preventDefault();
        firstChildBtn.focus();
        firstChildBtn.scrollIntoView({ block: "nearest" });
      }
    }
  } else if (e.key === "ArrowLeft") {
    if (expandedPaths.value.has(folder.path)) {
      e.preventDefault();
      void toggleFolderExpand(folder);
    }
  } else if (e.key === " " || e.key === "Spacebar") {
    e.preventDefault();
    emit("selectNav", { type: "folder", folder, subfolderPath: undefined, recursive: true });
  }
}

function onSidebarKeydown(e: KeyboardEvent) {
  const target = e.target as HTMLElement | null;
  if (!target || !sidebarRef.value?.contains(target)) return;
  if (target.matches("input, textarea, select")) return;

  if (e.key === "ArrowDown") {
    e.preventDefault();
    focusNextNavItem(target, 1);
  } else if (e.key === "ArrowUp") {
    if (target.matches(".tag-chip-eagle") && tagSearchInputRef.value && sidebarRef.value) {
      const chips = Array.from(sidebarRef.value.querySelectorAll<HTMLElement>(".tag-chip-eagle"));
      if (chips.indexOf(target) === 0) {
        e.preventDefault();
        tagSearchInputRef.value.focus();
        tagSearchInputRef.value.scrollIntoView({ block: "nearest" });
        return;
      }
    }
    e.preventDefault();
    focusNextNavItem(target, -1);
  } else if (
    e.key === "Home" &&
    target.matches(".nav-item, .subfolder-header, .tag-chip-eagle, .tool-btn")
  ) {
    e.preventDefault();
    focusFirstNavItem();
  } else if (
    e.key === "End" &&
    target.matches(".nav-item, .subfolder-header, .tag-chip-eagle, .tool-btn")
  ) {
    e.preventDefault();
    focusLastNavItem();
  } else if (target.matches(".tag-chip-eagle")) {
    if (e.key === "ArrowRight") {
      e.preventDefault();
      focusNextTagChip(target, 1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      focusNextTagChip(target, -1);
    }
  } else if (e.key === " " || e.key === "Spacebar") {
    if (
      target.matches(
        ".nav-item, .subfolder-header, .tag-chip-eagle, .tree-arrow-btn, .group-action-btn, .icon-btn, .tool-btn",
      )
    ) {
      e.preventDefault();
      target.click();
    }
  }
}

function isTargetActive(target: NavTarget): boolean {
  if (props.activeTarget.type !== target.type) return false;
  if (target.type === "folder" && props.activeTarget.type === "folder") {
    if (props.activeTarget.folder.id !== target.folder.id) return false;
    const activeSub = props.activeTarget.subfolderPath || "";
    const targetSub = target.subfolderPath || "";
    return activeSub === targetSub;
  }
  if (target.type === "album" && props.activeTarget.type === "album") {
    return props.activeTarget.album.id === target.album.id;
  }
  if (target.type === "tag" && props.activeTarget.type === "tag") {
    return props.activeTarget.tag.id === target.tag.id;
  }
  return true;
}

function onDropOnFolder(e: DragEvent, folder: Folder) {
  e.preventDefault();
  const data = e.dataTransfer?.getData("application/json");
  if (data) {
    try {
      const payload = JSON.parse(data);
      if (payload.file_paths && payload.file_paths.length > 0) {
        emit("moveFilesToFolder", {
          filePaths: payload.file_paths,
          folderId: folder.id,
        });
        return;
      }
    } catch (err) {
      console.error("Drop on folder parse error:", err);
    }
  }

  // Handle external OS file drops onto managed folder
  if (folder.folder_type === "managed") {
    const droppedFiles = e.dataTransfer?.files;
    if (droppedFiles && droppedFiles.length > 0) {
      const filePaths: string[] = [];
      for (let i = 0; i < droppedFiles.length; i++) {
        const f = droppedFiles[i] as any;
        if (f.path) {
          filePaths.push(f.path);
        }
      }
      if (filePaths.length > 0) {
        emit("openImportModal", {
          filePaths,
          folderId: folder.id,
        });
        return;
      }
    }
  }
}

async function handleImportToManaged(folder: Folder, e: MouseEvent) {
  e.stopPropagation();
  try {
    const selected = await openDialog({
      multiple: true,
      directory: false,
      title: t.value.importModal.title,
      filters: [
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "webp", "avif", "bmp", "gif", "tiff", "tga"],
        },
      ],
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) {
        emit("openImportModal", { filePaths: paths, folderId: folder.id });
      }
    }
  } catch (err) {
    console.error("Open file dialog error:", err);
  }
}

function onDropOnAlbum(e: DragEvent, album: Album) {
  e.preventDefault();
  const data = e.dataTransfer?.getData("application/json");
  if (data) {
    try {
      const payload = JSON.parse(data);
      if (payload.file_ids && payload.file_ids.length > 0) {
        emit("addFilesToAlbum", {
          fileIds: payload.file_ids,
          albumId: album.id,
        });
        return;
      }
    } catch (err) {
      console.error("Drop on album parse error:", err);
    }
  }

  // Handle external OS file drops
  const droppedFiles = e.dataTransfer?.files;
  if (droppedFiles && droppedFiles.length > 0) {
    const filePaths: string[] = [];
    for (let i = 0; i < droppedFiles.length; i++) {
      const f = droppedFiles[i] as any;
      if (f.path) {
        filePaths.push(f.path);
      }
    }
    if (filePaths.length > 0) {
      emit("importExternalFilesToAlbum", {
        filePaths,
        albumId: album.id,
      });
      return;
    }
  }

  // Handle URI / plain text drop fallback
  const text = e.dataTransfer?.getData("text/plain") || e.dataTransfer?.getData("text/uri-list");
  if (text) {
    const lines = text.split(/[\r\n]+/).map((s) => s.trim()).filter(Boolean);
    const filePaths: string[] = [];
    for (const line of lines) {
      if (line.startsWith("file://")) {
        try {
          const url = new URL(line);
          let pathname = decodeURIComponent(url.pathname);
          if (/^\/[a-zA-Z]:/.test(pathname)) {
            pathname = pathname.substring(1);
          }
          filePaths.push(pathname);
        } catch {}
      } else if (/^[a-zA-Z]:[\\/]/.test(line) || line.startsWith("/")) {
        filePaths.push(line);
      }
    }
    if (filePaths.length > 0) {
      emit("importExternalFilesToAlbum", {
        filePaths,
        albumId: album.id,
      });
    }
  }
}

function onDropOnTag(e: DragEvent, tag: Tag) {
  e.preventDefault();
  const data = e.dataTransfer?.getData("application/json");
  if (!data) return;
  try {
    const payload = JSON.parse(data);
    if (payload.file_ids && payload.file_ids.length > 0) {
      emit("tagFiles", {
        fileIds: payload.file_ids,
        tagId: tag.id,
      });
    }
  } catch (err) {
    console.error("Drop on tag parse error:", err);
  }
}
</script>

<template>
  <aside
    ref="sidebarRef"
    class="sidebar-eagle"
    :class="{ collapsed }"
    aria-label="Sidebar Navigation"
    @keydown="onSidebarKeydown"
  >
    <div class="sidebar-scrollable">
      <!-- Section: Library -->
      <section class="nav-group" aria-labelledby="sidebar-heading-library">
        <div class="group-header">
          <span id="sidebar-heading-library" class="group-title">{{ t.nav.library }}</span>
        </div>
        <ul class="nav-list" role="list">
          <li role="none">
            <button
              type="button"
              class="nav-item"
              :class="{ active: isTargetActive({ type: 'all' }) }"
              :aria-current="isTargetActive({ type: 'all' }) ? 'page' : undefined"
              :aria-label="t.nav.allImages + (counts ? ` (${counts.total})` : '')"
              @click="emit('selectNav', { type: 'all' })"
            >
              <span class="item-icon" aria-hidden="true">
                <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
                  <path d="M2 3a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3zm1 0v10h10V3H3zm2 7.5l2-2.5 1.5 2 2.5-3.5 3 4H4l1-1.5z" />
                </svg>
              </span>
              <span class="item-label">{{ t.nav.allImages }}</span>
              <span v-if="counts" class="item-badge" aria-hidden="true">{{ counts.total }}</span>
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              class="nav-item"
              :class="{ active: isTargetActive({ type: 'favorites' }) }"
              :aria-current="isTargetActive({ type: 'favorites' }) ? 'page' : undefined"
              :aria-label="t.nav.favorites + (counts?.favorites !== undefined ? ` (${counts.favorites})` : '')"
              @click="emit('selectNav', { type: 'favorites' })"
            >
              <span class="item-icon star-icon" aria-hidden="true">★</span>
              <span class="item-label">{{ t.nav.favorites }}</span>
              <span v-if="counts?.favorites !== undefined" class="item-badge" aria-hidden="true">{{ counts.favorites }}</span>
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              class="nav-item"
              :class="{ active: isTargetActive({ type: 'nsfw' }) }"
              :aria-current="isTargetActive({ type: 'nsfw' }) ? 'page' : undefined"
              :aria-label="t.nav.sensitive + (counts?.nsfw !== undefined ? ` (${counts.nsfw})` : '')"
              @click="emit('selectNav', { type: 'nsfw' })"
            >
              <span class="item-icon nsfw-icon" aria-hidden="true">🔞</span>
              <span class="item-label">{{ t.nav.sensitive }}</span>
              <span v-if="counts?.nsfw !== undefined" class="item-badge" aria-hidden="true">{{ counts.nsfw }}</span>
            </button>
          </li>
        </ul>
      </section>

      <!-- Section: Folders -->
      <section class="nav-group" aria-labelledby="sidebar-heading-folders">
        <div class="group-header">
          <span id="sidebar-heading-folders" class="group-title">{{ t.nav.folders }}</span>
          <button
            type="button"
            class="group-action-btn"
            :disabled="addingFolder"
            :title="t.nav.addFolder || t.addFolder?.title || 'Add Folder'"
            :aria-label="t.nav.addFolder || t.addFolder?.title || 'Add Folder'"
            @click="pickFolder"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor" aria-hidden="true">
              <path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/>
            </svg>
          </button>
        </div>
        <ul class="nav-list" role="list">
          <template v-for="folder in folders" :key="folder.id">
            <li
              class="folder-row-container"
              :class="{ active: isTargetActive({ type: 'folder', folder }) }"
              :data-folder-id="folder.id"
              :data-folder-path="folder.path"
              role="none"
              @dragover.prevent
              @drop="onDropOnFolder($event, folder)"
            >
              <button
                type="button"
                class="tree-arrow-btn"
                :class="{ expanded: expandedPaths.has(folder.path) }"
                :aria-expanded="expandedPaths.has(folder.path)"
                :aria-label="(expandedPaths.has(folder.path) ? (t.nav?.collapse || 'Collapse') : (t.nav?.expand || 'Expand')) + ' ' + getFolderName(folder.path)"
                :title="expandedPaths.has(folder.path) ? (t.nav?.collapse || 'Collapse') : (t.nav?.expand || 'Expand')"
                @click.stop="toggleFolderExpand(folder)"
              >
                <span v-if="loadingPaths.has(folder.path)" class="tree-loading-dot">…</span>
                <span v-else>{{ expandedPaths.has(folder.path) ? '▼' : '▶' }}</span>
              </button>

              <button
                type="button"
                class="nav-item folder-item-btn"
                :class="{ active: isTargetActive({ type: 'folder', folder }) }"
                :aria-current="isTargetActive({ type: 'folder', folder }) ? 'page' : undefined"
                :aria-label="getFolderName(folder.path) + (counts?.folders?.[folder.id] !== undefined ? ` (${counts.folders[folder.id]})` : '')"
                :title="displayPath(folder.path)"
                @click="emit('selectNav', { type: 'folder', folder, subfolderPath: undefined, recursive: true })"
                @keydown="onFolderKeydown($event, folder)"
              >
                <span class="item-icon" aria-hidden="true">
                  {{ folder.folder_type === 'pipeline' ? '⚡' : folder.folder_type === 'managed' ? '📦' : (expandedPaths.has(folder.path) ? '📂' : '📁') }}
                </span>
                <span class="item-label truncate">{{ getFolderName(folder.path) }}</span>
                <span v-if="counts?.folders" class="item-badge" aria-hidden="true">{{ counts.folders[folder.id] ?? 0 }}</span>
              </button>

              <div class="folder-actions" @click.stop>
                <button
                  v-if="folder.folder_type === 'managed'"
                  type="button"
                  class="icon-btn import-btn"
                  :disabled="isBusy(folder.id)"
                  :title="t.importModal?.title || 'Import Files'"
                  :aria-label="t.importModal?.title || 'Import Files'"
                  @click="handleImportToManaged(folder, $event)"
                >
                  📥
                </button>
                <button
                  v-if="folder.folder_type === 'pipeline'"
                  type="button"
                  class="icon-btn harvest-btn"
                  :disabled="isBusy(folder.id)"
                  :title="t.nav?.harvest || 'Harvest New Images'"
                  :aria-label="t.nav?.harvest || 'Harvest New Images'"
                  @click="harvest(folder, $event)"
                >
                  {{ isBusy(folder.id) ? '⏳' : '⚡' }}
                </button>
                <button
                  type="button"
                  class="icon-btn"
                  :disabled="isBusy(folder.id)"
                  :title="t.nav.scan"
                  :aria-label="t.nav.scan"
                  @click="scan(folder, 'scan')"
                >
                  {{ isBusy(folder.id) ? '⏳' : '🔄' }}
                </button>
                <button
                  type="button"
                  class="icon-btn remove-btn"
                  :title="t.nav.remove"
                  :aria-label="t.nav.remove"
                  @click="removeFolder(folder, $event)"
                >
                  ✕
                </button>
              </div>
            </li>

            <!-- Recursive Subdirectory Tree -->
            <ul
              v-if="expandedPaths.has(folder.path) && subdirectories[folder.path]?.length"
              class="subfolder-tree-list root-subfolder-list"
              role="list"
            >
              <FolderTreeNode
                v-for="child in subdirectories[folder.path]"
                :key="child.path"
                :folder="folder"
                :entry="child"
                :depth="1"
                :active-target="activeTarget"
                :expanded-paths="expandedPaths"
                :subdirectories="subdirectories"
                :loading-paths="loadingPaths"
                @toggle-expand="toggleSubfolderExpand"
                @select-subfolder="selectSubfolder"
              />
            </ul>
          </template>
          <li v-if="folders.length === 0" class="empty-hint">
            {{ t.nav.noFolders }}
          </li>
        </ul>
      </section>

      <!-- Section: Albums -->
      <section class="nav-group" aria-labelledby="sidebar-heading-albums">
        <div class="group-header">
          <span id="sidebar-heading-albums" class="group-title">{{ t.nav.albums }}</span>
          <button
            type="button"
            class="group-action-btn"
            :title="t.nav.newAlbum"
            :aria-label="t.nav.newAlbum"
            @click="emit('openAlbumModal')"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor" aria-hidden="true">
              <path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/>
            </svg>
          </button>
        </div>
        <ul class="nav-list" role="list">
          <li
            v-for="album in albums || []"
            :key="album.id"
            class="album-item-container"
            role="none"
            @dragover.prevent
            @drop="onDropOnAlbum($event, album)"
          >
            <button
              type="button"
              class="nav-item album-item"
              :class="{ active: isTargetActive({ type: 'album', album }) }"
              :aria-current="isTargetActive({ type: 'album', album }) ? 'page' : undefined"
              :aria-label="album.name + (albumCounts?.[album.id] !== undefined ? ` (${albumCounts[album.id]})` : '')"
              @click="emit('selectNav', { type: 'album', album })"
            >
              <span class="item-icon" aria-hidden="true">🗂️</span>
              <span class="item-label truncate">{{ album.name }}</span>
              <span class="item-badge" aria-hidden="true">{{ albumCounts?.[album.id] ?? 0 }}</span>
            </button>
          </li>
          <li v-if="!albums || albums.length === 0" class="empty-hint">
            {{ t.nav.noAlbums }}
          </li>
        </ul>
      </section>

      <!-- Section: Tags -->
      <section class="nav-group" aria-labelledby="sidebar-heading-tags">
        <div class="group-header">
          <span id="sidebar-heading-tags" class="group-title">{{ t.nav.tags }}</span>
          <button
            type="button"
            class="group-action-btn"
            :title="t.nav.newTag"
            :aria-label="t.nav.newTag"
            @click="emit('openTagModal')"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor" aria-hidden="true">
              <path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/>
            </svg>
          </button>
        </div>
        <!-- Tag Search / Filter Control -->
        <div v-if="(tags && tags.length > 0) || tagQuery" class="tag-search-box">
          <div class="tag-search-input-wrapper">
            <span class="tag-search-icon" aria-hidden="true">
              <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor">
                <path d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0z"/>
              </svg>
            </span>
            <input
              ref="tagSearchInputRef"
              v-model="tagQuery"
              type="text"
              class="tag-search-input"
              :placeholder="t.nav.filterTags || 'Filter tags…'"
              :aria-label="t.nav.filterTags || 'Filter tags…'"
              autocomplete="off"
              spellcheck="false"
              @keydown="onTagSearchKeydown"
            />
            <button
              v-if="tagQuery"
              type="button"
              class="tag-search-clear-btn"
              :title="t.nav.clearTagFilter || 'Clear tag filter'"
              :aria-label="t.nav.clearTagFilter || 'Clear tag filter'"
              @click="clearTagFilter"
            >
              ✕
            </button>
          </div>
        </div>

        <div class="tags-container" role="list" :aria-label="t.nav.tags">
          <button
            v-for="tag in filteredTags"
            :key="tag.id"
            type="button"
            class="tag-chip-eagle"
            role="listitem"
            :class="{ active: isTargetActive({ type: 'tag', tag }) }"
            :aria-current="isTargetActive({ type: 'tag', tag }) ? 'page' : undefined"
            :aria-label="tag.name + (tagCounts?.[tag.id] ? ` (${tagCounts[tag.id]})` : '')"
            @click="emit('selectNav', { type: 'tag', tag })"
            @dragover.prevent
            @drop="onDropOnTag($event, tag)"
          >
            <span
              class="tag-dot"
              aria-hidden="true"
              :style="{ backgroundColor: tag.color || '#8b5cf6' }"
            ></span>
            <span class="tag-name">{{ tag.name }}</span>
            <span v-if="tagCounts?.[tag.id]" class="tag-count" aria-hidden="true">{{ tagCounts[tag.id] }}</span>
          </button>
          <p v-if="tags === undefined" class="empty-hint">
            {{ t.nav.loadingTags || 'Loading tags…' }}
          </p>
          <p v-else-if="tags.length === 0" class="empty-hint">
            {{ t.nav.noTags }}
          </p>
          <p v-else-if="filteredTags.length === 0" class="empty-hint no-match-hint">
            {{ t.nav.noMatchingTags || 'No matching tags.' }}
          </p>
        </div>
      </section>
    </div>

    <!-- Sidebar Bottom Action Bar (Eagle Style) -->
    <footer class="sidebar-footer">
      <div class="footer-tools" role="toolbar" aria-label="Sidebar Tools">
        <button
          type="button"
          class="tool-btn"
          :title="t.search.insights"
          :aria-label="t.search.insights"
          @click="emit('openPromptStats')"
        >
          <span class="tool-icon" aria-hidden="true">📊</span>
          <span class="tool-text">{{ t.search.insights }}</span>
        </button>
        <button
          type="button"
          class="tool-btn"
          :title="t.search.autoTagger"
          :aria-label="t.search.autoTagger"
          @click="emit('openAutoTagger')"
        >
          <span class="tool-icon" aria-hidden="true">🤖</span>
          <span class="tool-text">{{ t.search.autoTagger }}</span>
        </button>
        <button
          type="button"
          class="tool-btn"
          :title="t.search.models"
          :aria-label="t.search.models"
          @click="emit('openModelManager')"
        >
          <span class="tool-icon" aria-hidden="true">🧠</span>
          <span class="tool-text">{{ t.search.models }}</span>
        </button>
        <button
          type="button"
          class="tool-btn"
          :title="t.search.database"
          :aria-label="t.search.database"
          @click="emit('openDbManager')"
        >
          <span class="tool-icon" aria-hidden="true">🗄️</span>
          <span class="tool-text">{{ t.search.database }}</span>
        </button>
        <button
          type="button"
          class="tool-btn"
          :title="t.search.shortcuts"
          :aria-label="t.search.shortcuts"
          @click="emit('openShortcutsHelp')"
        >
          <span class="tool-icon" aria-hidden="true">⌨️</span>
          <span class="tool-text">{{ t.search.shortcuts }}</span>
        </button>
      </div>
    </footer>
  </aside>
</template>

<style scoped>
.sidebar-eagle {
  width: 220px;
  min-width: 220px;
  max-width: 220px;
  flex-shrink: 0;
  height: 100%;
  background: var(--color-bg-primary);
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  user-select: none;
  overflow: hidden;
}

.sidebar-scrollable {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.sidebar-scrollable::-webkit-scrollbar {
  width: 4px;
}
.sidebar-scrollable::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
}

.group-title {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #64748b;
}

.group-action-btn {
  background: transparent;
  border: none;
  color: #64748b;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.12s;
}

.group-action-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.folder-row-container {
  display: flex;
  align-items: center;
  gap: 2px;
  width: 100%;
  border-radius: 6px;
  position: relative;
  box-sizing: border-box;
}

.folder-item-btn {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 6px;
  font-size: 0.82rem;
  color: #94a3b8;
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
  background: transparent;
  border: none;
  text-align: left;
  font-family: inherit;
  box-sizing: border-box;
}

.folder-item-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.folder-item-btn.active,
.folder-row-container.active > .folder-item-btn {
  background: var(--color-bg-active);
  color: var(--color-text-primary);
  font-weight: 600;
}

.folder-item-btn:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: -1px;
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.album-item-container {
  display: flex;
  width: 100%;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 6px;
  font-size: 0.82rem;
  color: #94a3b8;
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
  position: relative;
  background: transparent;
  border: none;
  width: 100%;
  text-align: left;
  font-family: inherit;
  box-sizing: border-box;
}

.nav-item:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.nav-item.active {
  background: var(--color-bg-active);
  color: var(--color-text-primary);
  font-weight: 600;
}

.nav-item:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: -1px;
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  color: #94a3b8;
  font-size: 0.85rem;
}

.nav-item.active .item-icon,
.folder-item-btn.active .item-icon {
  color: #a855f7;
}

.item-icon.star-icon {
  color: #fbbf24;
}

.item-label {
  flex: 1;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-badge {
  font-size: 0.7rem;
  padding: 0 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  color: #64748b;
  font-weight: 500;
}

.nav-item.active .item-badge,
.folder-item-btn.active .item-badge {
  background: rgba(168, 85, 247, 0.2);
  color: #e9d5ff;
}

.folder-actions {
  display: none;
  align-items: center;
  gap: 2px;
  position: absolute;
  right: 4px;
  background: var(--color-bg-primary);
  border-radius: 4px;
  padding-left: 4px;
}

.folder-row-container:hover .folder-actions,
.folder-row-container:focus-within .folder-actions {
  display: flex;
}

.folder-row-container:hover .folder-item-btn .item-badge,
.folder-row-container:focus-within .folder-item-btn .item-badge {
  display: none;
}

.folder-row-container:hover .folder-actions {
  background: var(--color-bg-hover);
}

.folder-row-container.active .folder-actions {
  background: var(--color-bg-active);
}

.tree-arrow-btn {
  background: transparent;
  border: none;
  color: #64748b;
  font-size: 0.65rem;
  width: 16px;
  height: 16px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.12s ease;
  flex-shrink: 0;
}

.tree-arrow-btn:hover {
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.08);
}

.tree-arrow-btn:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: 1px;
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.08);
}

.group-action-btn:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: 1px;
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.tree-loading-dot {
  font-size: 0.65rem;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.subfolder-tree-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.icon-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 2px;
  border-radius: 3px;
  font-size: 0.72rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.icon-btn:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: 1px;
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.tag-search-box {
  padding: 2px 6px 4px;
  width: 100%;
  box-sizing: border-box;
}

.tag-search-input-wrapper {
  display: flex;
  align-items: center;
  gap: 5px;
  width: 100%;
  box-sizing: border-box;
  background: var(--color-bg-secondary, rgba(255, 255, 255, 0.04));
  border: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
  border-radius: 4px;
  padding: 3px 6px;
  transition: border-color 0.12s, box-shadow 0.12s;
}

.tag-search-input-wrapper:focus-within {
  border-color: var(--color-accent, #6366f1);
  box-shadow: 0 0 0 1px var(--color-accent, #6366f1);
}

.tag-search-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #64748b;
  flex-shrink: 0;
}

.tag-search-input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  color: var(--color-text-primary, #f1f5f9);
  font-size: 0.74rem;
  font-family: inherit;
  padding: 0;
  box-sizing: border-box;
}

.tag-search-input::placeholder {
  color: #64748b;
  font-size: 0.72rem;
}

.tag-search-clear-btn {
  background: transparent;
  border: none;
  color: #64748b;
  font-size: 0.68rem;
  width: 16px;
  height: 16px;
  border-radius: 3px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  transition: all 0.12s;
}

.tag-search-clear-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.tag-search-clear-btn:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: 1px;
  color: #f8fafc;
}

.no-match-hint {
  color: #94a3b8;
}

.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  padding: 4px 6px;
}

.tag-chip-eagle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--color-bg-hover);
  font-size: 0.74rem;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.12s;
  border: 1px solid transparent;
  font-family: inherit;
}

.tag-chip-eagle:hover {
  background: var(--color-bg-active);
  color: var(--color-text-primary);
}

.tag-chip-eagle.active {
  background: rgba(139, 92, 246, 0.2);
  color: #c4b5fd;
  font-weight: 600;
}

.tag-chip-eagle:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: 1px;
  background: var(--color-bg-active);
  color: var(--color-text-primary);
}

.tag-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.tag-count {
  font-size: 0.68rem;
  opacity: 0.7;
  margin-left: 2px;
}

.empty-hint {
  font-size: 0.74rem;
  color: #475569;
  padding: 4px 8px;
  margin: 0;
  font-style: italic;
}

.sidebar-footer {
  padding: 8px 8px 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  background: var(--color-bg-app);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.footer-tools {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
}

.tool-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 6px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 5px;
  color: #94a3b8;
  font-size: 0.72rem;
  cursor: pointer;
  transition: all 0.12s;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.07);
  color: #f1f5f9;
}

.tool-btn:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: 1px;
  background: rgba(255, 255, 255, 0.07);
  color: #f1f5f9;
}

.tool-icon {
  font-size: 0.8rem;
}

.tool-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.footer-bottom-row {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

@media (prefers-reduced-motion: reduce) {
  .tree-loading-dot {
    animation: none;
    opacity: 0.6;
  }
  .nav-item,
  .folder-item-btn,
  .tag-chip-eagle,
  .tool-btn,
  .icon-btn,
  .group-action-btn,
  .tree-arrow-btn,
  .tag-search-input-wrapper,
  .tag-search-clear-btn {
    transition: none !important;
  }
}
</style>
