import { write } from "@example/db";
import sql from "sql-template-strings";

export async function shadowed(sql: (parts: TemplateStringsArray, value: string) => string, table: string) {
  return write(sql`SELECT id FROM ${table}`);
}

export async function shadowedLocal(table: string) {
  const sql = String.raw;
  return write(sql`SELECT id FROM ${table}`);
}

export const unused = sql;
