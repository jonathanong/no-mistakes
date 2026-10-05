import { query, sql } from "@example/db";

// `sql` imported from the database module is an arbitrary function, not the
// trusted tag, so this SQL is not statically recoverable. With no recovered
// relation there is no FROM line to anchor to: the finding stays on the call
// line, like any other unrecoverable embedded SQL.
export function topics(id: string) {
  return query(sql`
    SELECT id
      FROM topics
     WHERE id = ${id}
  `);
}
