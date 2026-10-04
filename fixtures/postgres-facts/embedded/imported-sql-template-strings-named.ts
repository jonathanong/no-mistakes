import { sql, unused } from "sql-template-strings";
import { query } from "@example/db";

export function load(id: number) {
  return query(sql`SELECT * FROM topics WHERE id = ${id}`, unused);
}
