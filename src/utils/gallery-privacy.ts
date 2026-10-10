import { ref, type Ref } from "vue";
import type { ImageFile } from "../types";

/**
 * Shared in-memory set of file paths that the user has intentionally revealed during this session.
 * Preserved across view mode changes (Grid, Waterfall, Table).
 */
const sharedRevealedPaths = ref<Set<string>>(new Set());

/**
 * Check whether a file should be visually masked/concealed based on:
 * - blurNsfw preference (defaults to true)
 * - file.is_nsfw flag
 * - whether the user has temporarily revealed this file path
 */
export function isSensitiveMasked(
  file: ImageFile | null | undefined,
  blurNsfw = true,
  revealedSet: Set<string> = sharedRevealedPaths.value,
): boolean {
  if (!file || !file.is_nsfw) return false;
  if (!blurNsfw) return false;
  return !revealedSet.has(file.path);
}

/**
 * Check whether a specific file path is currently in the revealed set.
 */
export function isSensitiveRevealed(
  path: string,
  revealedSet: Set<string> = sharedRevealedPaths.value,
): boolean {
  return revealedSet.has(path);
}

/**
 * Reveal a sensitive file path. Creates a new Set reference to ensure reactivity.
 */
export function revealSensitive(
  path: string,
  targetRef: Ref<Set<string>> = sharedRevealedPaths,
): void {
  if (!targetRef.value.has(path)) {
    const next = new Set(targetRef.value);
    next.add(path);
    targetRef.value = next;
  }
}

/**
 * Re-mask a previously revealed file path.
 */
export function remaskSensitive(
  path: string,
  targetRef: Ref<Set<string>> = sharedRevealedPaths,
): void {
  if (targetRef.value.has(path)) {
    const next = new Set(targetRef.value);
    next.delete(path);
    targetRef.value = next;
  }
}

/**
 * Toggle reveal/re-mask state for a file path.
 * Returns true if newly revealed, false if newly masked.
 */
export function toggleSensitiveReveal(
  path: string,
  targetRef: Ref<Set<string>> = sharedRevealedPaths,
): boolean {
  const isCurrentlyRevealed = targetRef.value.has(path);
  if (isCurrentlyRevealed) {
    remaskSensitive(path, targetRef);
    return false;
  } else {
    revealSensitive(path, targetRef);
    return true;
  }
}

/**
 * Clear all temporary reveals (e.g. when global masking is reset).
 */
export function clearSensitiveReveals(
  targetRef: Ref<Set<string>> = sharedRevealedPaths,
): void {
  if (targetRef.value.size > 0) {
    targetRef.value = new Set();
  }
}

/** Apply session masking without changing classification or saved preferences. */
export function setSensitiveMasking(
  masking: Ref<boolean>,
  enabled: boolean,
  revealed: Ref<Set<string>> = sharedRevealedPaths,
): void {
  if (enabled) clearSensitiveReveals(revealed);
  masking.value = enabled;
}

/**
 * Composable returning accessors and helpers for gallery privacy state.
 */
export function useGalleryPrivacy(customRef?: Ref<Set<string>>) {
  const activeRef = customRef ?? sharedRevealedPaths;
  return {
    revealedPaths: activeRef,
    isMasked: (file: ImageFile | null | undefined, blurNsfw = true) =>
      isSensitiveMasked(file, blurNsfw, activeRef.value),
    isRevealed: (path: string) => isSensitiveRevealed(path, activeRef.value),
    reveal: (path: string) => revealSensitive(path, activeRef),
    remask: (path: string) => remaskSensitive(path, activeRef),
    toggleReveal: (path: string) => toggleSensitiveReveal(path, activeRef),
    clearReveals: () => clearSensitiveReveals(activeRef),
  };
}
