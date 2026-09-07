import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
for (let i = 0; i < 2; i++) sql.append(" WHERE id = 1");

export function load() {
  return query(sql);
}
