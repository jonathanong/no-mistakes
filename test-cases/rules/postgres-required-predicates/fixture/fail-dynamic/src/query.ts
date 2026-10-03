import { query } from "@example/db";

export function load(table: string) {
  return query(`SELECT id FROM ${table} WHERE id = $1`);
}
