import { query } from "@example/db";
import { TABLES, type Table } from "./catalog";

export function partition(table: Table, yyyy: number, mm: number) {
  return query(`SELECT id FROM ${table}__p_${yyyy}_${mm}`);
}

export async function eachPartition(yyyy: number, mm: number) {
  for (const table of TABLES) {
    await query(`SELECT id FROM ${table}__p_${yyyy}_${mm}`);
  }
}
