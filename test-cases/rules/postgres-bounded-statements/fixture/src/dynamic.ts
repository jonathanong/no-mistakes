import { query } from "@data-stores/psql";

export function dynamic(table: string) {
  return query(`SELECT id FROM ${table}`);
}
