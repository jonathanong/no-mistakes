import { write } from "@example/db";
import sql from "sql-template-strings";

export async function declared(table: string) {
  function sql(parts: TemplateStringsArray, value: string) {
    return String.raw(parts, value);
  }
  return write(sql`SELECT id FROM ${table}`);
}

export async function looped(table: string) {
  for (const sql of [String.raw]) {
    await write(sql`SELECT id FROM ${table}`);
  }
}

export async function caught(table: string) {
  try {
    await Promise.resolve();
  } catch (sql: any) {
    await write(sql`SELECT id FROM ${table}`);
  }
}

export const unused = sql;
