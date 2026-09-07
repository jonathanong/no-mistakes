import { query } from "@data-stores/psql";

const sql = "SELECT 1" - "x";

export function load() {
  return query(sql);
}
