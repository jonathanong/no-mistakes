import { query } from "@data-stores/psql";

export function lockUsers(ids: string[]) {
  return query(`SELECT * FROM jobs JOIN users ON true WHERE users.id = ANY($1) ORDER BY jobs.id FOR UPDATE OF users`, [ids]);
}
