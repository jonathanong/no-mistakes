import { query } from "@example/db";

const ALLOWED_LIST = ["users", "posts"] as const;

export function list(table: string) {
  if (!ALLOWED_LIST.includes(table)) {
    throw new Error("table");
  }
  return query(`SELECT id FROM ${table}`);
}
