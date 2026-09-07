import { query } from "@data-stores/psql";

export function load(table: string) {
  return query(`SELECT 1 FROM ${table}`);
}
