import sql from "sql-template-strings";
import { query } from "@example/db";

// Each FROM sits on a later line than the template start. The finding must
// anchor at the relation's own line, not at the call or template start.
export function topics(id: string) {
  return query<{ id: string }>(sql`
    SELECT id
      FROM topics
     WHERE id = ${id}
  `);
}

export function orders(id: string) {
  return query<{ id: string }>(sql`
    SELECT id
      FROM orders
     WHERE id = ${id}
  `);
}
