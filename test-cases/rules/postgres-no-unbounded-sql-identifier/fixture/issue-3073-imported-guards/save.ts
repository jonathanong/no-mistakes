import { write } from "db";
import { ALLOWED_TABLES } from "./tables.ts";
type Allowed = (typeof ALLOWED_TABLES)[number];
export async function lockRows(tableName: Allowed, ids: string[]) {
  if (!ALLOWED_TABLES.includes(tableName)) {
    throw new Error(`Invalid table name: ${tableName}`);
  }
  await write(
    `/* lockRows */ SELECT id FROM ${tableName} WHERE id = ANY($1::uuid[]) ORDER BY id FOR UPDATE`,
    [ids],
  );
}

export async function stringInput(tableName: string, ids: string[]) {
  if (!ALLOWED_TABLES.includes(tableName)) throw new Error(`Invalid table: ${tableName}`);
  await write(`SELECT id FROM ${tableName} WHERE id = ANY($1::uuid[])`, [ids]);
}
