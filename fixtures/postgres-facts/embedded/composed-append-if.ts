import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
if (flag) sql.append(" WHERE id = 1");

export function load() {
  return query(sql);
}
