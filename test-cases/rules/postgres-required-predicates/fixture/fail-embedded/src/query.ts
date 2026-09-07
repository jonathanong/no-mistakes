import { query } from "@data-stores/psql";

export function load(id: string) {
  return query(`SELECT id FROM topics WHERE id = $1`);
}
