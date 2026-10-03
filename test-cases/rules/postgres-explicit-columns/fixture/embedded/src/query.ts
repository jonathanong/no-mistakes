import { query } from "@example/db";

export function load(id: string) {
  return query(`SELECT * FROM orders WHERE id = $1`);
}
