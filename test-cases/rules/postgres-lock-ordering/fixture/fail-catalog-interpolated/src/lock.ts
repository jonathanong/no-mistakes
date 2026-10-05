import { query } from "@example/db";

const TABLES = ["orders", "invoices"] as const;
type Table = (typeof TABLES)[number];

export function lockRows(table: Table, ids: string[]) {
  return query(`SELECT id FROM ${table} WHERE id = ANY($1::uuid[]) ORDER BY id FOR UPDATE`, [ids]);
}
