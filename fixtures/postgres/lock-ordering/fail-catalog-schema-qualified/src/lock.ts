import { query } from "@data-stores/psql";

export function lockArchivedItems(ids: string[]) {
  return query(`SELECT * FROM archive.items WHERE archive_id = ANY($1) ORDER BY id FOR UPDATE`, [ids]);
}
