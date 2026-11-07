import { write } from "db";
import { ALLOWED_TABLES } from "./tables.ts";
type Allowed = (typeof ALLOWED_TABLES)[number];
export function unchecked(tableName: Allowed) {
  return write(`SELECT * FROM ${tableName}`);
}
export function shadowed(tableName: string, ALLOWED_TABLES: string[]) {
  if (!ALLOWED_TABLES.includes(tableName)) throw new Error("bad");
  return write(`SELECT * FROM ${tableName}`);
}
export function outside(tableName: string, enabled: boolean) {
  if (enabled) {
    if (!ALLOWED_TABLES.includes(tableName)) throw new Error("bad");
  }
  return write(`SELECT * FROM ${tableName}`);
}
