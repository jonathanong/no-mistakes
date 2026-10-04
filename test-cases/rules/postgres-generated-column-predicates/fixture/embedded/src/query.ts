import { query } from "@example/db";

export function load() {
  return query(`SELECT id FROM orders WHERE created_at > $1`);
}
