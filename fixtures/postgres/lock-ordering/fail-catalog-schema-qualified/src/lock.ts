import { query } from "@example/db";

export function lockArchivedItems(ids: string[]) {
  return query(`SELECT * FROM archive.items WHERE archive_id = ANY($1) ORDER BY id FOR UPDATE`, [ids]);
}
