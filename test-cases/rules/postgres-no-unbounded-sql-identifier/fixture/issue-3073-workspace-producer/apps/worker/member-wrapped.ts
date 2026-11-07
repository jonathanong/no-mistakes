import { read } from "db";
import { getRelationTableNameOrThrow } from "./barrel.ts";

export async function awaitedMember(value: unknown) {
  return read(`SELECT * FROM ${(await getRelationTableNameOrThrow(value)).table_name}`);
}

export function conditionalMember(value: unknown, flag: boolean) {
  return read(`SELECT * FROM ${(flag ? getRelationTableNameOrThrow(value) : getRelationTableNameOrThrow(value)).table_name}`);
}

export function nullishMember(value: unknown) {
  return read(`SELECT * FROM ${(getRelationTableNameOrThrow(value) ?? "items").table_name}`);
}

export async function awaitedAliasMember(value: unknown) {
  const table = getRelationTableNameOrThrow(value);
  return read(`SELECT * FROM ${(await table).table_name}`);
}
