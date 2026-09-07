import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
for (const key in parts) sql.append(" WHERE id = 1");

export function load() {
  return query(sql);
}
