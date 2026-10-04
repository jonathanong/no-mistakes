import { query } from "@example/db";

const sql = "SELECT id FROM topics";
holder.sql.append(" WHERE id = 1");
getSql().append(" AND true");

export function load() {
  return query(sql);
}
