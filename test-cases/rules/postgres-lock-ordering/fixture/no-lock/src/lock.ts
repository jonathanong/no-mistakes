import { query } from "@example/db";

export function listRows(ids: string[]) {
  return query(`SELECT * FROM t WHERE id = ANY($1)`);
}
