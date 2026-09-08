import { query } from "@data-stores/psql";

export function lockJobsAndUsers(ids: string[]) {
  return query(`SELECT * FROM jobs AS j JOIN users AS u ON u.id = j.user_id WHERE u.id = ANY($1) ORDER BY j.id FOR UPDATE`, [ids]);
}
