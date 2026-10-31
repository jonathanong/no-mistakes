import { query } from "@example/db";

export function load(
  name: string,
  cursor: string,
  limit: number,
  pageSize: number,
  batchSize: number,
) {
  return query(`SELECT id FROM ${name} WHERE id > ${cursor} LIMIT ${limit}`, [
    pageSize,
    batchSize,
  ]);
}
