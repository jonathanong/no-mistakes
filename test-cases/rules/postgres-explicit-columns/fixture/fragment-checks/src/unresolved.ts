import { query } from "@example/db";
import sql, { SQLStatement } from "sql-template-strings";

const RETURNING_ALL = "RETURNING *";

export function appendReturningOnParam(q: SQLStatement) {
  q.append(sql` RETURNING *`);
}

export function appendUnknownRelationOnParam(q: SQLStatement) {
  q.append(" SELECT upserted.* FROM upserted");
}

export function loadOpenTableTagged(table: string, id: string) {
  return query(sql`SELECT * FROM ${table} WHERE id = ${id}`);
}

export function loadOpenTableUntagged(table: string, id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function concatReturningOntoOpaqueHead(head: string) {
  return query(head + " " + RETURNING_ALL);
}
