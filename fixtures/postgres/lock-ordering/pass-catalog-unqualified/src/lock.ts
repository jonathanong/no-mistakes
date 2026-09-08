import { query } from "@data-stores/psql";

export function lockUsers(ids: string[]) {
  return query(`SELECT * FROM users WHERE id = ANY($1) ORDER BY id FOR UPDATE`, [ids]);
}
