import { query } from "@example/db";
import { sql } from "@example/dbx";

// A sibling package that only shares the module prefix is not a subpath.
export function topics(id: string) {
  return query(sql`
    SELECT id
      FROM topics
     WHERE id = ${id}
  `);
}
