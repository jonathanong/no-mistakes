import { query } from "@example/db";

const TABLE_LIST = ["users", "posts"] as const;

export function byIndex(index: number) {
  return query(`SELECT id FROM ${TABLE_LIST[index]}`);
}
