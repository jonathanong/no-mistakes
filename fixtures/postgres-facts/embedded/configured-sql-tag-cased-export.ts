import { sql as SQL } from "sql-template-strings";
import { query } from "@example/db";

export function load(id: string) {
  return query(SQL`SELECT id FROM documents WHERE id = ${id}`);
}
