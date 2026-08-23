import { query } from "@data-stores/psql";

export function list() {
  return query(`SELECT id FROM posts ORDER BY id DESC`);
}
