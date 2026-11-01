import { query } from "@example/db";

const ALLOWED = new Set(["users", "posts"]);

export function block(table: string) {
  if (!ALLOWED.has(table)) {
    throw new Error("table");
  }
  return query(`SELECT id FROM ${table}`);
}

export function statement(table: string) {
  if (!ALLOWED.has(table)) throw new Error("table");
  return query(`SELECT id FROM ${table}`);
}
