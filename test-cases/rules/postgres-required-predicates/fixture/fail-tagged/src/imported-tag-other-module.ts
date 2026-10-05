import { query } from "@example/db";
import { sql } from "@other/db";

// Same export name, different module: trustedSqlTags for @example/db does not
// apply, so the call stays unanalyzable on this line.
export function topics(id: string) {
  return query(sql`
    SELECT id
      FROM topics
     WHERE id = ${id}
  `);
}
