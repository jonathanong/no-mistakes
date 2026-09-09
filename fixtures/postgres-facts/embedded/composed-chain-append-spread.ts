import { query } from "@data-stores/psql";

const parts: string[] = [];
const sql = "SELECT id FROM topics".append(...parts);

export function load() {
  return query(sql);
}
