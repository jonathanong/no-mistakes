import { query, sql } from "@example/db";
/* deadlock-safe */ query(flag ? sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE`
  : sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE`);
/* deadlock-safe */
query(flag ? sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE`
  : sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE`);
