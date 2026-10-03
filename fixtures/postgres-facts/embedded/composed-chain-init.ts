import { query } from "@example/db";

const sql = "SELECT id FROM topics".append(" WHERE id = 1").append(" AND active");

export function load() {
  return query(sql);
}
