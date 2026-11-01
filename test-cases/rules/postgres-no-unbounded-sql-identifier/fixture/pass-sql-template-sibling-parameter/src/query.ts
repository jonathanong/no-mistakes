import { write } from "@example/db";
import sql from "sql-template-strings";

export function unrelated(sql: string) {
  return sql.length;
}

export async function tagged(table: string) {
  return write(sql`SELECT id FROM ${table}`);
}
