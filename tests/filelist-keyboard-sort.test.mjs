import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

test("FileList template implements sortable interactive headers and grid ARIA semantics", () => {
  const filePath = path.resolve("src/components/FileList.vue");
  const content = fs.readFileSync(filePath, "utf-8");

  // Grid ARIA semantics
  assert.ok(content.includes('role="grid"'), "FileList table must have role='grid'");
  assert.ok(content.includes('role="row"'), "FileList row must have role='row'");
  assert.ok(content.includes('role="columnheader"'), "Headers must have role='columnheader'");
  assert.ok(content.includes(':aria-sort='), "Sortable headers must declare aria-sort attribute");
  assert.ok(content.includes(':aria-rowindex='), "Virtual rows must declare aria-rowindex attribute");
  assert.ok(content.includes(':aria-selected='), "Rows must declare aria-selected attribute");

  // Sortable header class and click listeners
  assert.ok(content.includes('class="th-sortable"'), "Headers must include th-sortable class");
  assert.ok(content.includes("onHeaderSort('path')"), "Name header must handle onHeaderSort");
  assert.ok(content.includes("onHeaderSort('size_bytes')"), "Size header must handle onHeaderSort");
  assert.ok(content.includes("onHeaderSort('modified_at')"), "Modified header must handle onHeaderSort");

  // Keyboard navigation
  assert.ok(content.includes('window.addEventListener("keydown", handleKeyDown)'), "FileList must attach keydown listener");
  assert.ok(content.includes('window.removeEventListener("keydown", handleKeyDown)'), "FileList must remove keydown listener");
  assert.ok(content.includes('ArrowDown') && content.includes('ArrowUp'), "handleKeyDown must handle ArrowDown and ArrowUp navigation");
});
