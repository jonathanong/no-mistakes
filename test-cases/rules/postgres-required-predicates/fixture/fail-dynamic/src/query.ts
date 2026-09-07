import { query } from "@data-stores/psql";

export function load(table: string) {
  return query(`SELECT id FROM ${table} WHERE id = $1`);
}
