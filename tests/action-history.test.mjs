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

test("failed undo and redo remain retryable without duplication", async () => {
  const history = new ActionHistory();
  let rejectUndo = true;
  let rejectRedo = true;
  await history.execute({ name: "A", execute: async () => {},
    undo: async () => { if (rejectUndo) throw new Error("undo failed"); },
    redo: async () => { if (rejectRedo) throw new Error("redo failed"); },
  });
  await assert.rejects(history.undo(), /undo failed/);
  assert.equal(history.canUndo(), true);
  assert.equal(history.canRedo(), false);
  rejectUndo = false;
  assert.equal(await history.undo(), "A");
  await assert.rejects(history.redo(), /redo failed/);
  assert.equal(history.canRedo(), true);
  assert.equal(history.canUndo(), false);
  rejectRedo = false;
  assert.equal(await history.redo(), "A");
  assert.equal(await history.undo(), "A");
  assert.equal(await history.undo(), null);
});

for (const operation of ["execute", "undo", "redo"]) {
  test(`pending ${operation} excludes every overlapping transition and clear`, async () => {
    const history = new ActionHistory();
    let release;
    const barrier = new Promise(resolve => { release = resolve; });
    let pause = false;
    let calls = 0;
    const callback = async () => { calls++; if (pause) await barrier; };
    const command = { name: "A", execute: callback, undo: callback, redo: callback };
    if (operation !== "execute") await history.execute(command);
    if (operation === "redo") await history.undo();
    pause = true;
    const pending = operation === "execute" ? history.execute(command) : history[operation]();
    const before = calls;
    try {
      assert.equal(history.canUndo(), false);
      assert.equal(history.canRedo(), false);
      await assert.rejects(history.execute(command), /already pending/);
      await assert.rejects(history.undo(), /already pending/);
      await assert.rejects(history.redo(), /already pending/);
      assert.throws(() => history.clear(), /pending transition/);
      assert.equal(calls, before);
    } finally { release(); await pending; }
    history.clear();
    assert.equal(history.canUndo(), false);
    assert.equal(history.canRedo(), false);
  });
}

test("failed execute keeps the existing redo chain and releases the guard", async () => {
  const history = new ActionHistory();
  await history.execute({ name: "A", execute: async () => {}, undo: async () => {} });
  await history.undo();
  await assert.rejects(history.execute({ name: "B", execute: async () => { throw new Error("failed"); }, undo: async () => {} }));
  assert.equal(await history.redo(), "A");
  assert.equal(await history.undo(), "A");
});
