import { query } from "@example/db";

const sql = "SELECT id FROM topics".append(" WHERE " + "active");

export function load() {
  return query(sql);
}
