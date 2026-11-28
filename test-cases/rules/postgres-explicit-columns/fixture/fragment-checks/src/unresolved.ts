import { query } from "@example/db";
import sql, { SQLStatement } from "sql-template-strings";

const RETURNING_ALL = "RETURNING *";
const WIDENED_TABLES = ["entries", "entries_summary"];
type WidenedTarget = (typeof WIDENED_TABLES)[number];

export function appendReturningOnParam(q: SQLStatement) {
  q.append(sql` RETURNING *`);
}

export function appendUnknownRelationOnParam(q: SQLStatement) {
  q.append(" SELECT upserted.* FROM upserted");
}

export function loadOpenTableTagged(table: string, id: string) {
  const q = sql`SELECT * FROM `;
  q.append(table);
  q.append(sql` WHERE id = ${id}`);
  return query(q);
}

export function loadOpenTableUntagged(table: string, id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function loadWidenedArrayTable(table: WidenedTarget, id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function concatReturningOntoOpaqueHead(head: string) {
  return query(head + " " + RETURNING_ALL);
}
