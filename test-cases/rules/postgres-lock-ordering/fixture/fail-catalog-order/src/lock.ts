import { query } from "@data-stores/psql";

export function lockRows(ids: string[]) {
  return query(`SELECT * FROM jobs WHERE id = ANY($1) ORDER BY priority FOR UPDATE`, [ids]);
}
