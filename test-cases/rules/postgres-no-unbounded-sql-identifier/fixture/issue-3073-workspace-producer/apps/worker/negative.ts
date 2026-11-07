import { read } from "db";
import { getRelationTableNameOrThrow } from "./wrong.ts";

export function unchecked(value: unknown) {
  const table = getRelationTableNameOrThrow(value);
  return read(`SELECT object_id FROM ${table}`);
}
