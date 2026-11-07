import { read } from "db";
import { getRelationTableNameOrThrow } from "./barrel.ts";
const TABLE = "items";

export function valueOnly(input: unknown) {
  return read(`SELECT id FROM ${TABLE} WHERE id = ${getRelationTableNameOrThrow(input)}`);
}

export function valueAliasOnly(input: unknown) {
  const value = getRelationTableNameOrThrow(input);
  return read(`SELECT id FROM ${TABLE} WHERE id = ${value}`);
}
