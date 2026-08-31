<script setup lang="ts">
import { computed } from "vue";
import type { AppInfo, ScanProgress } from "../types";

const props = defineProps<{
  totalCount: number;
  filteredCount: number;
  selectedCount: number;
  info: AppInfo | null;
  progress: ScanProgress | null;
  thumbProgress?: { current: number; total: number; active: boolean } | null;
  hasFilter?: boolean;
}>();

const progressPercent = computed(() => {
  if (!props.progress || props.progress.found === 0) return 0;
  return Math.min(100, Math.round((props.progress.scanned / props.progress.found) * 100));
});

const thumbPercent = computed(() => {
  if (!props.thumbProgress || props.thumbProgress.total === 0) return 0;
  return Math.min(100, Math.round((props.thumbProgress.current / props.thumbProgress.total) * 100));
});
</script>

<template>
  <footer class="statusbar">
    <!-- Left: Item Counts & Selection -->
    <div class="status-left">
      <span class="status-item">
        <span class="dot"></span>
        <span v-if="hasFilter">
          筛选匹配: <strong>{{ filteredCount }}</strong> / {{ totalCount }} 项
        </span>
        <span v-else>
          已索引: <strong>{{ totalCount }}</strong> 项
        </span>
      </span>

      <span v-if="selectedCount > 0" class="status-item selection-stat">
        已选中 <strong>{{ selectedCount }}</strong> 项
      </span>
    </div>

    <!-- Center: App / DB Meta -->
    <div class="status-center">
      <span v-if="info" class="db-indicator" :title="info.database_path">
        SQLite v{{ info.schema_version }} · {{ info.database_path.split(/[\\/]/).pop() }}
      </span>
    </div>

    <!-- Right: Background Scan / Thumbnail Progress -->
    <div class="status-right">
      <!-- Thumbnail generation progress -->
      <div v-if="thumbProgress?.active && thumbProgress.total > 0 && thumbProgress.current < thumbProgress.total" class="scan-status">
        <span class="scan-label" style="color: #67e8f9;">
          ⚡ 正在生成缩略图: {{ thumbProgress.current }} / {{ thumbProgress.total }} ({{ thumbPercent }}%)
        </span>
        <div class="mini-progress-track">
          <div class="mini-progress-fill cyan" :style="{ width: `${thumbPercent}%` }"></div>
        </div>
      </div>

      <!-- Scan Progress -->
      <div v-else-if="progress && progress.found > 0 && progress.scanned < progress.found" class="scan-status">
        <span class="scan-label">
          正在后台扫描: {{ progress.scanned }} / {{ progress.found }} ({{ progressPercent }}%)
        </span>
        <div class="mini-progress-track">
          <div class="mini-progress-fill" :style="{ width: `${progressPercent}%` }"></div>
        </div>
      </div>
      <span v-else class="ready-badge">就绪</span>
    </div>
  </footer>
</template>

<style scoped>
.statusbar {
  height: 26px;
  min-height: 26px;
  background: #111114;
  border-top: 1px solid rgba(255, 255, 255, 0.07);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  font-size: 0.72rem;
  color: #64748b;
  user-select: none;
  z-index: 90;
}

.status-left,
.status-center,
.status-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 5px;
  color: #94a3b8;
}

.status-item strong {
  color: #f1f5f9;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #22c55e;
}

.selection-stat {
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.25);
  color: #93c5fd;
  padding: 1px 6px;
  border-radius: 4px;
}

.selection-stat strong {
  color: #bfdbfe;
}

.db-indicator {
  color: #64748b;
  max-width: 320px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.scan-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.scan-label {
  color: #38bdf8;
  font-weight: 500;
}

.mini-progress-track {
  width: 80px;
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.mini-progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #38bdf8, #818cf8);
  transition: width 0.2s ease;
}

.mini-progress-fill.cyan {
  background: linear-gradient(90deg, #12b5cb, #fab82b);
}

.ready-badge {
  color: #64748b;
}
</style>
