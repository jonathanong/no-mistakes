import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  SELECT id FROM accounts OFFSET 0 -- findings: offset
` : sql`SELECT id FROM accounts LIMIT 1`}`);
query(sql`${flag ? sql`
  SELECT id FROM accounts OFFSET $1 -- findings: offset#2
` : sql`SELECT id FROM accounts LIMIT $1`}`);
