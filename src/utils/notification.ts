import { ref } from "vue";

export interface ToastNotification {
  id: string;
  type: "error" | "warning" | "info" | "success";
  title?: string;
  message: string;
  timestamp: number;
}

const toasts = ref<ToastNotification[]>([]);
let nextToastId = 1;

export function useNotification() {
  function showToast(
    message: string,
    type: "error" | "warning" | "info" | "success" = "error",
    title?: string,
    durationMs = 5000,
  ) {
    if (!message) return;
    const id = `toast-${nextToastId++}`;
    const item: ToastNotification = {
      id,
      type,
      title,
      message,
      timestamp: Date.now(),
    };
    toasts.value.push(item);

    if (durationMs > 0) {
      setTimeout(() => {
        dismissToast(id);
      }, durationMs);
    }
  }

  function showError(message: string, title?: string, durationMs = 6000) {
    showToast(message, "error", title, durationMs);
  }

  function showSuccess(message: string, title?: string, durationMs = 3500) {
    showToast(message, "success", title, durationMs);
  }

  function dismissToast(id: string) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  function clearAllToasts() {
    toasts.value = [];
  }

  return {
    toasts,
    showToast,
    showError,
    showSuccess,
    dismissToast,
    clearAllToasts,
  };
}
