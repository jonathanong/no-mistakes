export function collect(count: number, rows: Array<{ slot: number; key: string; kind: string }>) {
  const out = Array.from({ length: count }, () => ({}) as Record<string, Record<string, boolean>>);
  for (const row of rows) {
    const slot = out[row.slot];
    if (!slot) continue;
    slot[row.key] ||= {};
    slot[row.key]![row.kind] = true;
  }
  return out;
}
