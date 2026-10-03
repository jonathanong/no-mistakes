import { query } from "@example/db";

export function lockRows(ids: string[]) {
  /* ordered-locks: callers serialize ids */
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
