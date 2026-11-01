import { query } from "@example/db";

export function list(table: string) {
  if (!/^[a-z_]+$/.test(table)) {
    throw new Error("table");
  }
  return query(`SELECT id FROM ${table}`);
}
