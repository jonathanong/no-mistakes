import { query } from "@example/db";
import { getConfig, getRelationTable } from "./schema";

export function list(kind: string) {
  return query(`SELECT id FROM ${getRelationTable(kind)} WHERE id = $1`);
}

export function correlated(kind: string) {
  const config = getConfig(kind);
  return query(`DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}
