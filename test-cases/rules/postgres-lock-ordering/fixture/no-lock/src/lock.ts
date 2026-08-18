import { query } from "@data-stores/psql";

export function listRows(ids: string[]) {
  return query(`SELECT * FROM t WHERE id = ANY($1)`);
}
