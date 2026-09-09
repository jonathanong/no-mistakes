import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics".append(` WHERE id = 1`);

export function load() {
  return query(sql);
}
