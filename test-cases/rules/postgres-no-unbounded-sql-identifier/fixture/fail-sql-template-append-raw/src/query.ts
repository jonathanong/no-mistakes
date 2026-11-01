import { write } from "@example/db";
import sql from "sql-template-strings";

export async function appendRaw(table: string, id: string) {
  return write(sql`SELECT id FROM ${id}`.append(table));
}
