import { query } from "@example/db";

const ALLOWED = new Set(["users", "posts"]);

export function list(table: string, ok: boolean) {
  if (ok) {
    if (!ALLOWED.has(table)) throw new Error("table");
    query(`SELECT id FROM ${table}`);
  }
  return query(`SELECT id FROM ${table}`);
}
