import { query } from "@data-stores/psql";

const sql = "SELECT 1";
other.append(" FROM items");

export function load() {
  return query(sql);
}
