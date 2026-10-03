import { query } from "@example/db";

export function lockRows(ids: string[]) {
  return query(`-- deadlock-safe: unique key
SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
