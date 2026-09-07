import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
holder.sql.append(" WHERE id = 1");
getSql().append(" AND true");

export function load() {
  return query(sql);
}
