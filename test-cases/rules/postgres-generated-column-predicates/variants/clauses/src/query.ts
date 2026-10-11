import { query, sql } from "@example/db";
query(sql`SELECT a.id FROM orders a JOIN orders b ON ${flag ? sql`
  a.created_at = b.created_at -- findings: created_at,created_at
` : sql`a.id = b.id`}`);
query(sql`SELECT id FROM orders ORDER BY ${flag ? sql`
  created_at -- findings: created_at
` : sql`id`}`);
query(sql`UPDATE orders SET id = $2 WHERE ${flag ? sql`
  created_at > $1 -- findings: created_at
` : sql`id > $1`}`);
query(sql`DELETE FROM orders WHERE ${flag ? sql`
  created_at > $1 -- findings: created_at
` : sql`id > $1`}`);
