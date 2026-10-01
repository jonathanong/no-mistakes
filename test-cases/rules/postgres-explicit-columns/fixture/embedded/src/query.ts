import { query } from "@data-stores/psql";

export function load(id: string) {
  return query(`SELECT * FROM orders WHERE id = $1`);
}
