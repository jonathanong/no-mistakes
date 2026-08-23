import { query } from "@data-stores/psql";

export function list() {
  // no-mistakes-disable-next-line postgres-require-query-annotation: seed helper
  return query(`SELECT id FROM posts ORDER BY id DESC`);
}
