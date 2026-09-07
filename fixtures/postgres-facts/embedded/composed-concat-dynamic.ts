import { query } from "@data-stores/psql";

const sql = "SELECT id FROM " + table;

export function load() {
  return query(sql);
}
