import { query } from "@example/db";

const sql = "SELECT id FROM topics";
sql.append(" WHERE id = 1");

export function load() {
  return query(sql);
}
