import { query } from "@example/db";

export function list(table: string, extra: string) {
  const ALLOWED = new Set(["users", extra]);
  if (!ALLOWED.has(table)) throw new Error("table");
  return query(`SELECT id FROM ${table}`);
}
