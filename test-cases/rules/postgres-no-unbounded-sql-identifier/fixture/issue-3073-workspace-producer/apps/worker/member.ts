import { read } from "db";
import { getRelationTableNameOrThrow } from "./barrel.ts";

export function unchecked() {
  const row = getRelationTableNameOrThrow({ subject: "topic", predicate: "parent", object: "topic" });
  return read(`SELECT object_id FROM ${row.table_name}`);
}

export function directUnchecked(value: unknown) {
  return read(`SELECT object_id FROM ${getRelationTableNameOrThrow(value).table_name}`);
}
