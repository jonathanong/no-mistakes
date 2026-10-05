import sql from "sql-template-strings";
import { query } from "@example/db";

// An in-SQL directive above the FROM line suppresses the finding anchored there.
export function topics(id: string) {
  return query<{ id: string }>(sql`
    SELECT id
      -- no-mistakes-disable-next-line postgres-required-predicates
      FROM topics
     WHERE id = ${id}
  `);
}

export function orders(id: string) {
  return query<{ id: string }>(sql`
    SELECT id
      -- no-mistakes-disable-next-line postgres-required-predicates
      FROM orders
     WHERE id = ${id}
  `);
}
