import sql from 'sql-template-strings';
export const selectedFields = sql`w.id, w.label`;
export const fixedJoin = sql` JOIN paint_colors c ON c.id = w.color_id`;
export function makeSelection(prefix: ReturnType<typeof sql>, extra: ReturnType<typeof sql>) {
  return prefix.append(selectedFields).append(extra).append(sql` FROM painted_widgets w`);
}
export function dynamicFrom(table: string) {
  return sql`FROM ${table}`;
}
