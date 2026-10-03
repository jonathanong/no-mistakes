import { query } from "@example/db";

export function load(id: string) {
  return query(`SELECT id FROM topics WHERE id = $1`);
}
