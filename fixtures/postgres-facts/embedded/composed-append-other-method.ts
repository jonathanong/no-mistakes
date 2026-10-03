import { query } from "@example/db";

const sql = "SELECT id FROM topics";
sql.push(" WHERE id = 1");

export function load() {
  return query(sql);
}
