import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  SELECT id FROM accounts WHERE id IN ($1, $2) ORDER BY name FOR UPDATE -- findings: lock-ordering
` : sql`SELECT id FROM accounts WHERE id IN ($1, $2) ORDER BY id FOR UPDATE`}`);
query(sql`${flag ? sql`
  SELECT id FROM ${tableName} WHERE id IN ($1, $2) ORDER BY id FOR UPDATE -- findings: unresolved-relation
` : sql`SELECT id FROM accounts WHERE id IN ($1, $2) ORDER BY id FOR UPDATE`}`);
query(sql`${flag ? sql`
  SELECT id FROM accounts WHERE id = $1 AND name IN ($2, $3) FOR UPDATE
` : sql`SELECT id FROM accounts WHERE id IN ($1, $2) ORDER BY id FOR UPDATE`}`);
query(sql`${flag ? sql`
  /* deadlock-safe */ SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE
` : sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE SKIP LOCKED`}`);
