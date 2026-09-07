import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics";
sql.append(...parts);

export function load() {
  return query(sql);
}
