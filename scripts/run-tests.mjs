import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const testsDir = path.resolve(process.cwd(), "tests");
if (!fs.existsSync(testsDir)) {
  console.error(`Tests directory does not exist: ${testsDir}`);
  process.exit(1);
}

// Discover all test files in tests/
const entries = fs.readdirSync(testsDir);
const testFiles = entries
  .filter((f) => f.endsWith(".test.mjs") || (f.endsWith(".mjs") && f.includes("bench")))
  .sort();

if (testFiles.length === 0) {
  console.error("No test files discovered in tests/ directory.");
  process.exit(1);
}

console.log(`Discovered ${testFiles.length} regression test files in tests/:`);
for (const file of testFiles) {
  console.log(`  - tests/${file}`);
}

const isWindows = process.platform === "win32";
const bin = path.resolve(
  process.cwd(),
  "node_modules",
  ".bin",
  isWindows ? "tsx.cmd" : "tsx"
);

const args = ["--test", ...testFiles.map((f) => `tests/${f}`)];

const result = spawnSync(bin, args, {
  stdio: "inherit",
  shell: true,
});

if (result.error) {
  console.error("Failed to execute test runner:", result.error);
  process.exit(1);
}

process.exit(result.status ?? 0);
