import { query, sql } from "@example/db";

export function direct() {
  return query(sql`SELECT a.id, o.id
    FROM accounts a
    CROSS JOIN orders o`);
}
