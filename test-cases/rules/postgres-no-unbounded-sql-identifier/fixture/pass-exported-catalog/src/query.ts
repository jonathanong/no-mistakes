import { query } from "@example/db";
import { PARTITIONED, TABLES, TABLE_NAMES } from "./catalog";

export async function eachTable() {
  for (const table of TABLES) {
    await query(`SELECT count(*) FROM ${table}`);
  }
}

export function indexed(index: number) {
  return query(`SELECT id FROM ${TABLES[index]}`);
}

export function named() {
  return query(`SELECT id FROM ${TABLE_NAMES.user}`);
}

export async function eachPartitioned() {
  for (const entry of PARTITIONED) {
    await query(`SELECT count(*) FROM ${entry.table}`);
  }
}
