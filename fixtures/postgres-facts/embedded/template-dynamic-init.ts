import { query } from "@data-stores/psql";

const sql = `SELECT id FROM topics WHERE id = ${id}`;

export function load() {
  return query(sql);
}
