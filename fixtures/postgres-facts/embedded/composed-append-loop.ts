import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
for (const part of parts) sql.append(" WHERE id = 1");

export function load() {
  return query(sql);
}
