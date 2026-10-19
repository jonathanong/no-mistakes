import { sql as tag } from "sql-template-strings";
import { query } from "@example/db";

export function load(id: string) {
  return query(tag`SELECT id FROM documents WHERE id = ${id}`);
}
