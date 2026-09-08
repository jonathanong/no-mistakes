import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics".append(" WHERE " + "active");

export function load() {
  return query(sql);
}
