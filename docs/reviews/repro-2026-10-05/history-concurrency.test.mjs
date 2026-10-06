import test from 'node:test';
import assert from 'node:assert/strict';
import { ActionHistory } from '../../../src/utils/history.ts';

test('overlapping undo requests must not run commands concurrently', async () => {
  const history = new ActionHistory();
  let active = 0;
  let peak = 0;
  let release;
  const barrier = new Promise(resolve => { release = resolve; });
  const command = name => ({
    name, execute: async () => {},
    undo: async () => { active += 1; peak = Math.max(peak, active); await barrier; active -= 1; },
  });
  await history.execute(command('A'));
  await history.execute(command('B'));
  const first = history.undo();
  const second = history.undo();
  const observed = peak;
  release();
  await Promise.allSettled([first, second]);
  assert.equal(observed, 1, 'History ran two undo commands concurrently');
});

