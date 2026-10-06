import test from 'node:test';
import assert from 'node:assert/strict';
import { ActionHistory } from '../../../src/utils/history.ts';

test('failed undo must retain its command for recovery or retry', async () => {
  const history = new ActionHistory();
  await history.execute({ name: 'set rating', execute: async () => {}, undo: async () => { throw new Error('simulated IPC rejection'); } });
  await assert.rejects(history.undo());
  assert.equal(history.canUndo(), true, 'Failed undo removed the only recoverable command');
});

test('failed redo must retain its command for recovery or retry', async () => {
  const history = new ActionHistory();
  await history.execute({ name: 'set rating', execute: async () => {}, undo: async () => {}, redo: async () => { throw new Error('simulated IPC rejection'); } });
  await history.undo();
  await assert.rejects(history.redo());
  assert.equal(history.canRedo(), true, 'Failed redo removed the only recoverable command');
});
