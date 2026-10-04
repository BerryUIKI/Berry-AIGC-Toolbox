import assert from "node:assert/strict";
import test from "node:test";
import { BatchAutoTagController } from "../src/utils/batch-tagger.ts";

test("BatchAutoTagController: starts in idle state and can process batch", async () => {
  let invokedCommand = "";
  let invokedArgs = null;
  let listenerRegistered = false;
  let eventCallback = null;

  const mockInvoke = async (cmd, args) => {
    invokedCommand = cmd;
    invokedArgs = args;
    if (cmd === "batch_auto_tag_files") {
      // Simulate backend progress event
      if (eventCallback) {
        await eventCallback({
          payload: {
            current: 1,
            total: 2,
            percent: 50,
            current_file: "test.png",
            processed_files: 1,
            failed_files: 0,
            tags_added: 5,
            is_complete: false,
            is_canceled: false,
          },
        });
      }
      return { processed_files: 2, tags_added: 10 };
    }
    return null;
  };

  const mockListen = async (event, handler) => {
    if (event === "tagger-batch-progress") {
      listenerRegistered = true;
      eventCallback = handler;
    }
    return () => {
      listenerRegistered = false;
    };
  };

  const controller = new BatchAutoTagController(mockInvoke, mockListen);
  assert.equal(controller.status, "idle");
  assert.equal(controller.isRunning, false);

  const progressUpdates = [];
  const result = await controller.start({
    fileIds: [1, 2],
    config: {
      general_threshold: 0.35,
      character_threshold: 0.85,
      include_rating: false,
      max_tags: 50,
      write_to_prompt: false,
      append_prompt: false,
    },
    onProgress: (p) => {
      progressUpdates.push(p);
    },
  });

  assert.equal(invokedCommand, "batch_auto_tag_files");
  assert.deepEqual(invokedArgs.fileIds, [1, 2]);
  assert.equal(result.processed_files, 2);
  assert.equal(result.tags_added, 10);
  assert.equal(controller.status, "completed");
  assert.equal(controller.isRunning, false);
  assert.equal(progressUpdates.length, 1);
  assert.equal(progressUpdates[0].current_file, "test.png");
  assert.equal(listenerRegistered, false, "Listener should be cleaned up upon completion");
});

test("BatchAutoTagController: cancellation triggers cancel_batch_auto_tag command and updates status", async () => {
  let canceledCalled = false;

  const mockInvoke = async (cmd) => {
    if (cmd === "cancel_batch_auto_tag") {
      canceledCalled = true;
      return null;
    }
    if (cmd === "batch_auto_tag_files") {
      // Simulate waiting for cancellation
      await new Promise((r) => setTimeout(r, 20));
      return { processed_files: 1, tags_added: 3 };
    }
    return null;
  };

  const mockListen = async () => () => {};

  const controller = new BatchAutoTagController(mockInvoke, mockListen);
  const startPromise = controller.start({
    fileIds: [1, 2, 3],
    config: {
      general_threshold: 0.35,
      character_threshold: 0.85,
      include_rating: false,
      max_tags: 50,
      write_to_prompt: false,
      append_prompt: false,
    },
  });

  assert.equal(controller.isRunning, true);
  await controller.cancel();
  assert.equal(canceledCalled, true);

  await startPromise;
  assert.equal(controller.status, "canceled");
  assert.equal(controller.isRunning, false);
});
