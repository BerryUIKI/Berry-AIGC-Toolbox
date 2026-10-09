import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { formatPlatformName } from "../src/utils/image.ts";

test("Rust technical metadata fixture retains geometry without a generator badge", () => {
  const metadata = JSON.parse(readFileSync(new URL("./fixtures/derivative-dimensions.json", import.meta.url), "utf8"));
  assert.deepEqual([metadata.width, metadata.height], [32, 16]);
  assert.equal(formatPlatformName(metadata.format), "");
  assert.equal(metadata.prompt, null);
});
