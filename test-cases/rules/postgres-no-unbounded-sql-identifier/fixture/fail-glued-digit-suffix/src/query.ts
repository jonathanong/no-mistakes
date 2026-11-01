import { query } from "@example/db";

export function shard(schema: string, id: string) {
  return query(`SELECT 1 FROM ${schema}2 WHERE id = ${id}`);
}
