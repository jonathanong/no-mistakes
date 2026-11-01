import { query } from "@example/db";

export function archived(id: string) {
  return query(`SELECT 1 FROM audit_sql_placeholder_archive WHERE id = ${id}`);
}
