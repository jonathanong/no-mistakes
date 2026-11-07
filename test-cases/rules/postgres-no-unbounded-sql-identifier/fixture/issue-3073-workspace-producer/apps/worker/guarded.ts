import { read } from "db";
import { ALLOWED_TABLES } from "./tables.ts";
type Allowed = (typeof ALLOWED_TABLES)[number];

export function guardedAlias(table: Allowed) {
  if (!ALLOWED_TABLES.includes(table)) throw new Error("Unknown table");
  const alias = table;
  const forwarded = alias;
  return read(`SELECT * FROM ${forwarded}`);
}

export function capturedValue(table: Allowed) {
  if (!ALLOWED_TABLES.includes(table)) throw new Error("Unknown table");
  const alias = table;
  table = process.env.UNCHECKED_TABLE as Allowed;
  return read(`SELECT * FROM ${alias}`);
}

export function numericSuffix(suffix: number) {
  const alias = suffix;
  return read(`SELECT * FROM table_${alias}`);
}
