import { query } from "@data-stores/psql";

/* deadlock-safe: this comment is more than 200 characters before the call site so it must not suppress the multi-row lock */
const padding =
  "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

export function lockRows(ids: string[]) {
  void padding;
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
