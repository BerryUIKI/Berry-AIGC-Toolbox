import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";

test("FileOperationModal: guards against moving from or writing into linked folders", async () => {
  const content = await fs.readFile(
    new URL("../src/components/FileOperationModal.vue", import.meta.url),
    "utf8",
  );

  // 1. Excludes or disallows selecting linked folders as destinations
  assert.match(
    content,
    /folder\.folder_type === ['"]link['"]/,
    "Must detect linked folders in folder list",
  );
  assert.match(
    content,
    /:disabled="folder\.folder_type === ['"]link['"]"/,
    "Radio inputs for linked folders must be disabled",
  );
  assert.match(
    content,
    /folder-badge-link/,
    "Linked folders must display a read-only badge indicator",
  );

  // 2. Guards against moving files originating from linked folders
  assert.match(
    content,
    /hasLinkedSourceFiles/,
    "Must detect if any source file originates from a linked folder",
  );
  assert.match(
    content,
    /isMoveBlocked/,
    "Moving linked files must be explicitly blocked",
  );
  assert.match(
    content,
    /linkedReadOnlyWarning/,
    "Must display warning banner when moving linked folder files",
  );
});

test("BatchTransformModal: preserves read-only contract for external linked folders", async () => {
  const content = await fs.readFile(
    new URL("../src/components/BatchTransformModal.vue", import.meta.url),
    "utf8",
  );

  // Gating managed vs linked files
  assert.match(content, /const managedFiles = computed/);
  assert.match(content, /const linkedFiles = computed/);
  assert.match(content, /externalLinkNotice/);
});

test("Backend scan and transform contracts enforce read-only linked folder boundaries", async () => {
  const fileOpsContent = await fs.readFile(
    new URL("../crates/omera-scan/src/file_operations.rs", import.meta.url),
    "utf8",
  );
  const transformContent = await fs.readFile(
    new URL("../crates/omera-scan/src/transform.rs", import.meta.url),
    "utf8",
  );

  // 1. file_operations: Move from linked folder is blocked
  assert.match(
    fileOpsContent,
    /Cannot move .* source is in an external linked folder/,
    "file_operations must block Move from external linked folders",
  );

  // 2. file_operations: Move or copy into linked folder is blocked
  assert.match(
    fileOpsContent,
    /Destination folder cannot be an external linked folder/,
    "file_operations must block writes into external linked folders",
  );

  // 3. transform: batch transcode skips linked folders
  assert.match(
    transformContent,
    /external_folder_read_only/,
    "transform_library_batch must skip linked folders with external_folder_read_only",
  );
});
