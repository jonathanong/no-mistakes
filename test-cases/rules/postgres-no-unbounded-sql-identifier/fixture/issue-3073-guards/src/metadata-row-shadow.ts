import { query } from "@example/db";
const SCOPED_ROWS = [{ key: "user", table: "users" }];
function scopedRow(key: string) {
  const selected = SCOPED_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function shadowed(scopedRow: (key: string) => { table: string }) {
  const selected = scopedRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}
