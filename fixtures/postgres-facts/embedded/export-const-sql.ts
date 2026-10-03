import { query } from "@example/db";

export const sql = "SELECT id FROM topics";

export function load() {
  return query(sql);
}
