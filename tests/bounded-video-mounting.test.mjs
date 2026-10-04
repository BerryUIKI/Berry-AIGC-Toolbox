import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

test("VirtualGrid restricts native video element strictly to single active hover/scrub item", () => {
  const fileContent = fs.readFileSync(path.resolve("src/components/VirtualGrid.vue"), "utf-8");

  // Verify no unconditional or unhovered <video> poster element exists in the template
  assert.ok(
    !fileContent.includes('class="thumbnail-img thumbnail-video video-poster"'),
    "VirtualGrid must not mount video-poster video elements for every unhovered visible video",
  );

  // Verify lightweight video placeholder is rendered when video is not actively hovered
  assert.ok(
    fileContent.includes('class="thumbnail-video-placeholder"'),
    "VirtualGrid must render a lightweight placeholder div for unhovered video cards",
  );

  // Verify native video element is strictly bounded to hover/scrub interaction
  assert.ok(
    fileContent.includes('v-if="isVideoContainer(file.container) && hoveredVideoPath === file.path"'),
    "VirtualGrid must mount <video> exclusively when hoveredVideoPath matches file.path",
  );

  // Verify CSS styles exist for .thumbnail-video-placeholder
  assert.ok(
    fileContent.includes(".thumbnail-video-placeholder"),
    "VirtualGrid must include .thumbnail-video-placeholder CSS styling",
  );
});
