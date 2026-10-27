import { query } from "@example/db";

export function lockOne() {
  return query(`SELECT id FROM items WHERE id = 1 FOR UPDATE`);
}
