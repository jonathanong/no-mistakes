import { query, sql } from "@example/db";
query(sql`${flag ? sql`-- parse failure stays a statement diagnostic
  SELECT id FROM WHERE id IN ($1, $2) FOR UPDATE
` : sql`SELECT id FROM accounts LIMIT 1`}`);
