declare function getRows(): Array<{ mark: boolean }>;
export function existingReceiver(rows: Array<{ mark: boolean }>) {
  rows = getRows();
  const value = rows[0];
  if (value) value.mark = true;
  const out: unknown[] = [];
  rows.forEach(value => { out.push(value); });
  return out;
}
declare function getOpaque(): unknown;
export function existingShared(value: { mark: boolean }) {
  const rows = [value, getOpaque()];
  rows[0].mark = true;
}
