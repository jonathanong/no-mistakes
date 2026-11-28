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
  return query(sql`SELECT * FROM ${table} WHERE id = ${id}`);
}

export function loadViewsUntagged(table: ViewOnlyTarget, id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function loadViewsTagged(table: ViewOnlyTarget, id: string) {
  return query(sql`SELECT * FROM ${table} WHERE id = ${id}`);
}

export function loadIndexedUnion(table: IndexedTarget, id: string) {
  return query(sql`SELECT * FROM ${table} WHERE id = ${id}`);
}
