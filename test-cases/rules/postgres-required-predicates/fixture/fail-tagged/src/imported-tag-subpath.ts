import { query } from "@example/db";
import { sql } from "@example/db/sql";

// A subpath of the configured module is the same named tag.
export function topics(id: string) {
  return query(sql`
    SELECT id
      FROM topics
     WHERE id = ${id}
  `);
}
