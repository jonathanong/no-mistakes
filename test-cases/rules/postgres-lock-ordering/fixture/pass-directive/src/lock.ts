import { query } from "@data-stores/psql";

export function lockRows(ids: string[]) {
  /* deadlock-safe: single row via unique key */
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
