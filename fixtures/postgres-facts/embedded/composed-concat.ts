import { query } from "@data-stores/psql";

const sql = "SELECT id FROM " + "topics";

export function load() {
  return query(sql);
}
