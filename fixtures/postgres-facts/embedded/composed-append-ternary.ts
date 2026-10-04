import { query } from "@example/db";

const sql = "SELECT id FROM topics";
flag ? sql.append(" WHERE id = 1") : null;

export function load() {
  return query(sql);
}
