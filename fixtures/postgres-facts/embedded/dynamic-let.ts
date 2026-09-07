import { query } from "@data-stores/psql";

let sql = "SELECT id FROM topics";
sql = "SELECT 1";

export function load() {
  return query(sql);
}
