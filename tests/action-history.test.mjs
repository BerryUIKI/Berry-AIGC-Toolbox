import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { ActionHistory } from "../src/utils/history.ts";

test("ActionHistory executes, un-does and re-does commands sequentially", async () => {
  const history = new ActionHistory(10);
  let state = 0;

  const makeIncrementCommand = (delta) => ({
    name: `Increment by ${delta}`,
    execute: async () => {
      state += delta;
    },
    undo: async () => {
      state -= delta;
    },
  });

  await history.execute(makeIncrementCommand(5));
  assert.equal(state, 5);
  assert.equal(history.canUndo(), true);
  assert.equal(history.canRedo(), false);

  await history.execute(makeIncrementCommand(10));
  assert.equal(state, 15);

  const undid = await history.undo();
  assert.equal(undid, "Increment by 10");
  assert.equal(state, 5);
  assert.equal(history.canRedo(), true);

  const redid = await history.redo();
  assert.equal(redid, "Increment by 10");
  assert.equal(state, 15);
  assert.equal(history.canRedo(), false);
});

test("App.vue implements Ctrl+Z and Ctrl+Y handlers and reversible batch operations", () => {
  const appContent = fs.readFileSync(path.resolve("src/App.vue"), "utf-8");
  assert.ok(appContent.includes("onUndo"), "App.vue must declare onUndo function");
  assert.ok(appContent.includes("onRedo"), "App.vue must declare onRedo function");
  assert.ok(appContent.includes("actionHistory.execute"), "App.vue must record operations in actionHistory");
  assert.ok(appContent.includes("actionHistory.undo"), "App.vue must trigger actionHistory.undo on Ctrl+Z");
  assert.ok(appContent.includes("actionHistory.redo"), "App.vue must trigger actionHistory.redo on Ctrl+Y / Shift+Z");
});
