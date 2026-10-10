import { query } from "@example/db";

export function page(limit: number) {
  // A `$1` bind keeps the SQL static; an untagged `${...}` LIMIT would be unanalyzable.
  return query(`SELECT id FROM posts ORDER BY id DESC LIMIT $1`, [limit + 1]);
}
