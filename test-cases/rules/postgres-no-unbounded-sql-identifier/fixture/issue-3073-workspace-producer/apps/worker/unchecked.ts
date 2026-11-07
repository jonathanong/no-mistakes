import { read } from "db";
import { ALLOWED_TABLES } from "./tables.ts";
type Allowed = (typeof ALLOWED_TABLES)[number];
export function unchecked(table: Allowed) {
  return read(`SELECT * FROM ${table}`);
}

export function uncheckedAlias(table: Allowed) {
  const alias = table;
  return read(`SELECT * FROM ${alias}`);
}

export function capturedBeforeGuard(table: Allowed) {
  const alias = table;
  if (!ALLOWED_TABLES.includes(table)) throw new Error("Unknown table");
  return read(`SELECT * FROM ${alias}`);
}

export function mutableAlias(table: Allowed) {
  if (!ALLOWED_TABLES.includes(table)) throw new Error("Unknown table");
  let alias = table;
  alias = process.env.UNCHECKED_TABLE as Allowed;
  return read(`SELECT * FROM ${alias}`);
}
