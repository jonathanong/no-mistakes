import { query } from "@example/db";

export function lockRows(ids: string[]) {
  return query(`SELECT * FROM jobs WHERE id = ANY($1) ORDER BY jobs.id FOR UPDATE`, [ids]);
}
