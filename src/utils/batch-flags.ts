/** Apply only confirmed groups; always reconcile after possible partial IPC failure. */
export async function applyBooleanFlagMutation(
  values: Map<number, boolean>,
  write: (ids: number[], value: boolean) => Promise<void>,
  apply: (ids: number[], value: boolean) => void,
  refresh: () => Promise<void>,
): Promise<void> {
  try {
    for (const value of [true, false]) {
      const ids = [...values].filter(([, flag]) => flag === value).map(([id]) => id);
      if (!ids.length) continue;
      await write(ids, value);
      apply(ids, value);
    }
  } finally {
    await refresh();
  }
}
