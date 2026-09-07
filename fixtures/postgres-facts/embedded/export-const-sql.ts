import { query } from "@data-stores/psql";

export const sql = "SELECT id FROM topics";

export function load() {
  return query(sql);
}
