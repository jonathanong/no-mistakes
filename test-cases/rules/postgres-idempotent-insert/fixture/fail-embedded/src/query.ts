import { query } from "@example/db";

export function insertItem() {
  return query(`INSERT INTO items (id) VALUES (1)`);
}
