import { query } from "@data-stores/psql";

let sql = "SELECT id FROM " + "topics";

export function load() {
  return query(sql);
}
