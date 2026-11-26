import { query } from "@example/db";

export function lockRows(ids: string[]) {
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}

export function lockRowsInline(ids: string[]) {
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`); // no-mistakes-disable-line postgres-lock-ordering: fixture exception
}
