import { query } from "@data-stores/psql";

export function lockRows(ids: string[]) {
  return query(`-- deadlock-safe: unique key
SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
