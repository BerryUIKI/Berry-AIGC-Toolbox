export const TABLE_COLUMN_IDS = ["preview", "name", "container", "size", "modified", "platform", "prompt", "dimensions", "model"] as const;
export type TableColumnId = typeof TABLE_COLUMN_IDS[number];
export interface TableColumnPreference { id: TableColumnId; visible: boolean; width: number }

const defaults: TableColumnPreference[] = TABLE_COLUMN_IDS.map((id, index) => ({
  id, visible: ["preview", "name", "prompt", "model"].includes(id),
  width: [44, 100, 70, 80, 150, 90, 160, 100, 100][index],
}));

export function normalizeTableColumns(value: unknown): TableColumnPreference[] {
  const input = Array.isArray(value) ? value : [];
  return defaults.map(fallback => {
    const candidate = input.find(item => item && item.id === fallback.id);
    const minimum = ["name", "prompt", "model"].includes(fallback.id) ? 80 : 44;
    return { id: fallback.id,
      visible: fallback.id === "name" || (typeof candidate?.visible === "boolean" ? candidate.visible : fallback.visible),
      width: typeof candidate?.width === "number" && Number.isFinite(candidate.width)
        ? Math.round(Math.max(minimum, Math.min(640, candidate.width))) : fallback.width,
    };
  });
}
