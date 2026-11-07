import { query } from "@example/db";
import { TABLES, type Table } from "./catalog";

export function aliased(table: Table) {
  const t = table;
  return query(`SELECT id FROM ${t}`);
}

export async function loopAlias() {
  for (const table of TABLES) {
    const t = table;
    await query(`SELECT id FROM ${t}`);
  }
}
