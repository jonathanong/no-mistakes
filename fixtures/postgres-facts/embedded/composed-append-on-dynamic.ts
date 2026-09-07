import { query } from "@data-stores/psql";

const sql = `SELECT id FROM topics WHERE id = ${id}`;
sql.append(" AND true");

export function load() {
  return query(sql);
}
