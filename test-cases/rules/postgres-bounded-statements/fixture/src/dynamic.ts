import { query } from "@example/db";

export function dynamic(table: string) {
  return query(`SELECT id FROM ${table}`);
}
