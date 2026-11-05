import { sql } from "@example/db";

export function rootsSql() {
  return sql`FROM accounts a CROSS JOIN orders o`;
}
