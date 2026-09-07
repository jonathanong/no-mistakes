import { query } from "@data-stores/psql";

const sql = `SELECT id FROM topics WHERE parent_id IS NOT NULL`;

export function load() {
  return query(sql);
}
