<script setup lang="ts">
import { useNotification } from "../utils/notification";

const { toasts, dismissToast } = useNotification();
</script>

<template>
  <div v-if="toasts.length > 0" class="toast-container" role="region" aria-live="polite" aria-label="Notifications">
    <transition-group name="toast-fade" tag="div" class="toast-list">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast-card', `toast-${toast.type}`]"
        role="alert"
      >
        <div class="toast-icon">
          <span v-if="toast.type === 'error'">⚠️</span>
          <span v-else-if="toast.type === 'warning'">⚡</span>
          <span v-else-if="toast.type === 'success'">✓</span>
          <span v-else>ℹ️</span>
        </div>
        <div class="toast-body">
          <h4 v-if="toast.title" class="toast-title">{{ toast.title }}</h4>
          <p class="toast-message">{{ toast.message }}</p>
        </div>
        <button
          type="button"
          class="toast-dismiss-btn"
          aria-label="Dismiss notification"
          @click="dismissToast(toast.id)"
        >
          ✕
        </button>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 48px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  pointer-events: none;
  max-width: 420px;
  width: calc(100vw - 40px);
}

.toast-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.toast-card {
  pointer-events: auto;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  border-radius: 8px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--border-color);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-error {
  border-left: 4px solid var(--color-danger, #ef4444);
  background: var(--color-bg-secondary);
}

.toast-warning {
  border-left: 4px solid var(--color-warning, #f59e0b);
}

.toast-success {
  border-left: 4px solid var(--color-success, #10b981);
}

.toast-info {
  border-left: 4px solid var(--color-primary, #a855f7);
}

.toast-icon {
  font-size: 1rem;
  line-height: 1.2;
  flex-shrink: 0;
}

.toast-body {
  flex: 1;
  min-width: 0;
}

.toast-title {
  margin: 0 0 2px 0;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.toast-message {
  margin: 0;
  font-size: 0.78rem;
  line-height: 1.35;
  color: var(--color-text-secondary);
  word-break: break-word;
}

.toast-dismiss-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--color-text-muted);
  font-size: 0.75rem;
  padding: 2px 4px;
  border-radius: 4px;
  line-height: 1;
  flex-shrink: 0;
  transition: color 0.15s ease, background 0.15s ease;
}

.toast-dismiss-btn:hover {
  color: var(--color-text-primary);
  background: var(--color-bg-hover);
}

.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: all 0.25s ease;
}

.toast-fade-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
