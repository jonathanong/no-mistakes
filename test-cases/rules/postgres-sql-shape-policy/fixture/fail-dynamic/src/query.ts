import { query } from "@example/db";

export function load(table: string) {
  return query(`SELECT 1 FROM ${table}`);
}
