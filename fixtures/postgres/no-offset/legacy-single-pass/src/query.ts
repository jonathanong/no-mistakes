import { query } from "@data-stores/psql";

export function firstPage(limit: number) {
  return query(`SELECT id FROM posts ORDER BY id DESC LIMIT ${limit + 1}`);
}
