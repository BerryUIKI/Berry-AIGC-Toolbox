<script setup lang="ts">
import { computed } from "vue";
import { t } from "../i18n";
import type { Folder, NavTarget, SubdirectoryEntry } from "../types";

defineOptions({
  name: "FolderTreeNode",
});

const props = defineProps<{
  folder: Folder;
  entry: SubdirectoryEntry;
  depth: number;
  activeTarget: NavTarget;
  expandedPaths: Set<string>;
  subdirectories: Record<string, SubdirectoryEntry[]>;
  loadingPaths: Set<string>;
}>();

const emit = defineEmits<{
  (e: "toggleExpand", folder: Folder, path: string): void;
  (e: "selectSubfolder", folder: Folder, path: string): void;
}>();

const isTargetActive = computed(() => {
  return (
    props.activeTarget.type === "folder" &&
    props.activeTarget.folder.id === props.folder.id &&
    props.activeTarget.subfolderPath === props.entry.path
  );
});

const isExpanded = computed(() => props.expandedPaths.has(props.entry.path));
const isLoading = computed(() => props.loadingPaths.has(props.entry.path));
const children = computed(() => props.subdirectories[props.entry.path] ?? []);

function onSubfolderKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowRight") {
    if (props.entry.has_children && !isExpanded.value) {
      e.preventDefault();
      emit("toggleExpand", props.folder, props.entry.path);
    } else if (isExpanded.value) {
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
    if (isExpanded.value) {
      e.preventDefault();
      emit("toggleExpand", props.folder, props.entry.path);
    } else {
      const currentLi = (e.target as HTMLElement).closest("li");
      const parentLi = currentLi?.parentElement?.closest("li");
      const parentBtn = parentLi?.querySelector<HTMLElement>(
        ":scope > .subfolder-row > .subfolder-header, :scope > .folder-item-btn",
      );
      if (parentBtn) {
        e.preventDefault();
        parentBtn.focus();
        parentBtn.scrollIntoView({ block: "nearest" });
      }
    }
  } else if (e.key === " " || e.key === "Spacebar") {
    e.preventDefault();
    emit("selectSubfolder", props.folder, props.entry.path);
  }
}
</script>

<template>
  <li
    class="subfolder-node"
    :class="{ active: isTargetActive }"
    :data-subfolder-path="entry.path"
    role="none"
  >
    <div
      class="subfolder-row"
      :style="{ paddingLeft: `${18 + depth * 14}px` }"
    >
      <button
        v-if="entry.has_children"
        type="button"
        class="tree-arrow-btn"
        :class="{ expanded: isExpanded }"
        :aria-expanded="isExpanded"
        :aria-label="(isExpanded ? (t.nav?.collapse || 'Collapse') : (t.nav?.expand || 'Expand')) + ' ' + entry.name"
        :title="isExpanded ? (t.nav?.collapse || 'Collapse') : (t.nav?.expand || 'Expand')"
        @click.stop="emit('toggleExpand', folder, entry.path)"
      >
        <span v-if="isLoading" class="tree-loading-dot">…</span>
        <span v-else>{{ isExpanded ? '▼' : '▶' }}</span>
      </button>
      <span v-else class="tree-arrow-spacer" aria-hidden="true"></span>

      <button
        type="button"
        class="subfolder-header"
        :class="{ active: isTargetActive }"
        :aria-current="isTargetActive ? 'page' : undefined"
        :aria-label="entry.name + (entry.file_count > 0 ? ` (${entry.file_count})` : '')"
        :title="entry.path"
        @click="emit('selectSubfolder', folder, entry.path)"
        @keydown="onSubfolderKeydown"
      >
        <span class="item-icon" aria-hidden="true">
          {{ isExpanded ? '📂' : '📁' }}
        </span>

        <span class="item-label truncate">{{ entry.name }}</span>

        <span v-if="entry.file_count > 0" class="item-badge" aria-hidden="true">{{ entry.file_count }}</span>
      </button>
    </div>

    <!-- Recursive Subfolder Children -->
    <ul v-if="isExpanded && children.length > 0" class="subfolder-tree-list" role="list">
      <FolderTreeNode
        v-for="child in children"
        :key="child.path"
        :folder="folder"
        :entry="child"
        :depth="depth + 1"
        :active-target="activeTarget"
        :expanded-paths="expandedPaths"
        :subdirectories="subdirectories"
        :loading-paths="loadingPaths"
        @toggle-expand="(f, p) => emit('toggleExpand', f, p)"
        @select-subfolder="(f, p) => emit('selectSubfolder', f, p)"
      />
    </ul>
  </li>
</template>

<style scoped>
.subfolder-node {
  display: flex;
  flex-direction: column;
  width: 100%;
}

.subfolder-row {
  display: flex;
  align-items: center;
  gap: 2px;
  width: 100%;
  box-sizing: border-box;
}

.subfolder-header {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  padding-top: 4px;
  padding-bottom: 4px;
  padding-right: 8px;
  border-radius: 6px;
  font-size: 0.8rem;
  color: #94a3b8;
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
  box-sizing: border-box;
  background: transparent;
  border: none;
  text-align: left;
  font-family: inherit;
}

.subfolder-header:hover {
  background: var(--color-bg-hover, rgba(255, 255, 255, 0.05));
  color: var(--color-text-primary, #f8fafc);
}

.subfolder-header.active,
.subfolder-node.active > .subfolder-row > .subfolder-header {
  background: var(--color-bg-active, rgba(139, 92, 246, 0.2));
  color: var(--color-text-primary, #f8fafc);
  font-weight: 600;
}

.subfolder-header:focus-visible {
  outline: 2px solid var(--color-accent, #6366f1);
  outline-offset: -1px;
  background: var(--color-bg-hover, rgba(255, 255, 255, 0.05));
  color: var(--color-text-primary, #f8fafc);
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

.tree-arrow-spacer {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.tree-loading-dot {
  font-size: 0.65rem;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

@media (prefers-reduced-motion: reduce) {
  .tree-loading-dot {
    animation: none;
    opacity: 0.6;
  }
  .subfolder-header,
  .tree-arrow-btn {
    transition: none !important;
  }
}

.item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.9rem;
  flex-shrink: 0;
}

.item-label {
  flex: 1;
  min-width: 0;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-badge {
  font-size: 0.68rem;
  padding: 1px 6px;
  border-radius: 9999px;
  background: rgba(255, 255, 255, 0.08);
  color: #94a3b8;
  font-weight: 500;
  flex-shrink: 0;
}

.subfolder-tree-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
</style>
