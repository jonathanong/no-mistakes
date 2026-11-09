export function serviceBLOCK(items: number[], options: {limit: number; enabled: boolean}) {
  const selected = items.map((value, index) => ({
    ...helper(value),
    ordinal: index,
    details: { ...(options.enabled ? { active: true } : { active: false }), limit: options.limit },
    readers: [() => value + options.limit, () => index * value],
  })).filter(entry => entry.active && entry.ordinal < options.limit);
  const grouped = selected.reduce((state, entry) => ({
    ...state,
    [String(entry.ordinal)]: { ...entry, values: entry.readers.map(reader => reader()) },
  }), {});
  return { ...grouped, count: selected.length, enabled: options.enabled };
}
export const callbackBLOCK = (value: number) => serviceBLOCK([value], { limit: 5, enabled: true });
