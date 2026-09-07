import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
sql.append();

export function load() {
  return query(sql);
}
