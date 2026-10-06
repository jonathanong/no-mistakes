import { query, sql } from "@example/db";

// Without trustedSqlTags, `sql` from the database module is an arbitrary
// function, so this SQL is not statically recoverable. The finding stays on
// the call line. trustedSqlTags opts this named import in; then the finding
// anchors at FROM and an in-SQL directive can suppress it.
export function topics(id: string) {
  return query(sql`
    SELECT id
      FROM topics
     WHERE id = ${id}
  `);
}
