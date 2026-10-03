import { query } from "@example/db";

const sql = "SELECT id FROM " + table;

export function load() {
  return query(sql);
}
