import { query } from "@data-stores/psql";

export function load() {
  return query(`SELECT id FROM orders WHERE created_at > $1`);
}
