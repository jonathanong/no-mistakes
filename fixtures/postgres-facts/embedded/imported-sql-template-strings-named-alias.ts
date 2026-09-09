import { helper as sql } from "sql-template-strings";
import { query } from "@data-stores/psql";

export function load(id: number) {
  return query(sql`SELECT * FROM topics WHERE id = ${id}`);
}
