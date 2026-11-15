import { query } from "@example/db";
import { getRelationTable } from "./schema";

export function list(kind: string) {
  return query(`SELECT id FROM ${getRelationTable(kind)} WHERE id = $1`);
}
