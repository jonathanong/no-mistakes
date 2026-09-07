import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
flag ? sql.append(" WHERE id = 1") : null;

export function load() {
  return query(sql);
}
