import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { useNotification } from "../src/utils/notification.ts";

test("useNotification adds and dismisses toasts cleanly", () => {
  const { toasts, showError, showSuccess, dismissToast, clearAllToasts } = useNotification();
  clearAllToasts();
  assert.equal(toasts.value.length, 0);

  showError("Network timeout", "Connection Error", 0);
  assert.equal(toasts.value.length, 1);
  assert.equal(toasts.value[0].type, "error");
  assert.equal(toasts.value[0].title, "Connection Error");
  assert.equal(toasts.value[0].message, "Network timeout");

  showSuccess("Files updated", undefined, 0);
  assert.equal(toasts.value.length, 2);
  assert.equal(toasts.value[1].type, "success");

  const firstId = toasts.value[0].id;
  dismissToast(firstId);
  assert.equal(toasts.value.length, 1);
  assert.equal(toasts.value[0].type, "success");

  clearAllToasts();
  assert.equal(toasts.value.length, 0);
});

test("App.vue mounts ToastContainer and watches error state", () => {
  const appContent = fs.readFileSync(path.resolve("src/App.vue"), "utf-8");
  assert.ok(appContent.includes("<ToastContainer"), "App.vue must mount ToastContainer");
  assert.ok(appContent.includes("useNotification()"), "App.vue must instantiate useNotification");
  assert.ok(appContent.includes("notification.showError(newErr)"), "App.vue must show error toasts when error ref updates");
});
