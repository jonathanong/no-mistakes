import { query } from "@data-stores/psql";

export function load(table: string) {
  return query(`INSERT INTO ${table} (id) VALUES (1)`);
}
