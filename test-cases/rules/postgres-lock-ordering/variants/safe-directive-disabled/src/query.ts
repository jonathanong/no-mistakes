import { query, sql } from "@example/db";
query(flag ? sql`/* deadlock-safe */ SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE`
  : sql`SELECT id FROM accounts WHERE id IN ($1, $2) ORDER BY id FOR UPDATE`);
