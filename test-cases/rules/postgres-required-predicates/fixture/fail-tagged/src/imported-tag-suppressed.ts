import { query, sql } from "@example/db";

// The directive sits on the FROM line. It suppresses only after trustedSqlTags
// makes that line the finding anchor; without the option the call stays
// unanalyzable and this comment does not apply.
export function topics(id: string) {
  return query(sql`
    SELECT id
      -- no-mistakes-disable-next-line postgres-required-predicates
      FROM topics
     WHERE id = ${id}
  `);
}
