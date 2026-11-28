import { query } from "@example/db";
import sql from "sql-template-strings";

type MixedTarget = "entries" | "entries_summary";
type ViewOnlyTarget = "entries_summary" | "audit_view";
const TABLE_CHOICES = ["records", "audit_view"] as const;
type IndexedTarget = (typeof TABLE_CHOICES)[number];

export function loadMixedUntagged(table: MixedTarget, id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function loadMixedTagged(table: MixedTarget, id: string) {
  const q = sql`SELECT * FROM `;
  q.append(table);
  q.append(sql` WHERE id = ${id}`);
  return query(q);
}

export function loadViewsUntagged(table: ViewOnlyTarget, id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function loadViewsTagged(table: ViewOnlyTarget, id: string) {
  const q = sql`SELECT * FROM `;
  q.append(table);
  q.append(sql` WHERE id = ${id}`);
  return query(q);
}

export function loadIndexedUnion(table: IndexedTarget, id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function loadDirectIndexedAndAlias(
  table: (typeof TABLE_CHOICES)[number],
  id: string,
) {
  const relation = table;
  return query(`SELECT * FROM ${relation} WHERE id = $1`, [id]);
}
