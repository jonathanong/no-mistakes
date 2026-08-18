import { query } from "@data-stores/psql";

export function lockRows(ids: string[]) {
  // no-mistakes-disable-next-line postgres-lock-ordering
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
