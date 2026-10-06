import assert from "node:assert/strict";
import test from "node:test";
import { ref } from "vue";
import {
  isSensitiveMasked,
  isSensitiveRevealed,
  revealSensitive,
  remaskSensitive,
  toggleSensitiveReveal,
  clearSensitiveReveals,
  useGalleryPrivacy,
} from "../src/utils/gallery-privacy.ts";

test("isSensitiveMasked: correctly evaluates masking policy without mutating file", () => {
  const sfwFile = { id: 1, path: "/library/sfw.jpg", is_nsfw: false, container: "jpg" };
  const unflaggedFile = { id: 2, path: "/library/unknown.jpg", container: "jpg" };
  const sensitiveFile = { id: 3, path: "/library/sensitive.png", is_nsfw: true, container: "png" };

  // SFW and unflagged files are never masked
  assert.equal(isSensitiveMasked(sfwFile, true), false);
  assert.equal(isSensitiveMasked(unflaggedFile, true), false);
  assert.equal(isSensitiveMasked(null, true), false);
  assert.equal(isSensitiveMasked(undefined, true), false);

  // When blur preference is disabled, sensitive file is not masked
  assert.equal(isSensitiveMasked(sensitiveFile, false), false);

  // When blur preference is enabled, sensitive file is masked by default
  const emptyRevealed = new Set();
  assert.equal(isSensitiveMasked(sensitiveFile, true, emptyRevealed), true);

  // When explicitly revealed, sensitive file is not masked
  const revealedSet = new Set([sensitiveFile.path]);
  assert.equal(isSensitiveMasked(sensitiveFile, true, revealedSet), false);

  // Classification flag is untouched
  assert.equal(sensitiveFile.is_nsfw, true);
  assert.equal(sfwFile.is_nsfw, false);
});

test("revealSensitive, remaskSensitive, and toggleSensitiveReveal update reactive set without mutating file objects", () => {
  const scopedRef = ref(new Set());
  const pathA = "/gallery/imgA.png";
  const pathB = "/gallery/imgB.webp";

  assert.equal(isSensitiveRevealed(pathA, scopedRef.value), false);
  assert.equal(scopedRef.value.size, 0);

  // Reveal pathA
  revealSensitive(pathA, scopedRef);
  assert.equal(isSensitiveRevealed(pathA, scopedRef.value), true);
  assert.equal(scopedRef.value.size, 1);

  // Repeated reveal is idempotent
  const prevSet = scopedRef.value;
  revealSensitive(pathA, scopedRef);
  assert.equal(scopedRef.value, prevSet, "Idempotent reveal should not recreate set reference");

  // Reveal pathB
  revealSensitive(pathB, scopedRef);
  assert.equal(isSensitiveRevealed(pathB, scopedRef.value), true);
  assert.equal(scopedRef.value.size, 2);

  // Remask pathA
  remaskSensitive(pathA, scopedRef);
  assert.equal(isSensitiveRevealed(pathA, scopedRef.value), false);
  assert.equal(isSensitiveRevealed(pathB, scopedRef.value), true);
  assert.equal(scopedRef.value.size, 1);

  // Toggle pathA: reveals it again
  const toggledOn = toggleSensitiveReveal(pathA, scopedRef);
  assert.equal(toggledOn, true);
  assert.equal(isSensitiveRevealed(pathA, scopedRef.value), true);
  assert.equal(scopedRef.value.size, 2);

  // Toggle pathA: remasks it
  const toggledOff = toggleSensitiveReveal(pathA, scopedRef);
  assert.equal(toggledOff, false);
  assert.equal(isSensitiveRevealed(pathA, scopedRef.value), false);
  assert.equal(scopedRef.value.size, 1);

  // Clear all reveals
  clearSensitiveReveals(scopedRef);
  assert.equal(scopedRef.value.size, 0);
  assert.equal(isSensitiveRevealed(pathB, scopedRef.value), false);
});

test("useGalleryPrivacy composable encapsulates scoped privacy lifecycle", () => {
  const customRef = ref(new Set());
  const privacy = useGalleryPrivacy(customRef);

  const file = {
    id: 10,
    path: "/vault/sample.jpg",
    is_nsfw: true,
    container: "jpg",
  };

  assert.equal(privacy.isMasked(file, true), true);
  assert.equal(privacy.isRevealed(file.path), false);

  privacy.reveal(file.path);
  assert.equal(privacy.isMasked(file, true), false);
  assert.equal(privacy.isRevealed(file.path), true);

  privacy.remask(file.path);
  assert.equal(privacy.isMasked(file, true), true);
  assert.equal(privacy.isRevealed(file.path), false);

  privacy.toggleReveal(file.path);
  assert.equal(privacy.isMasked(file, true), false);

  privacy.clearReveals();
  assert.equal(privacy.isMasked(file, true), true);
});
