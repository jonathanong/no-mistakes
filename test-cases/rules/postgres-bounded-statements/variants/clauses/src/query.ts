import { query, sql } from "@example/db";
query(sql`UPDATE ${flag ? sql`
  accounts SET email = 'x' -- findings: table:accounts
` : sql`accounts SET email = 'x' WHERE id = $1`}`);
query(sql`DELETE FROM ${flag ? sql`
  accounts -- findings: table:accounts
` : sql`accounts WHERE id = $1`}`);
query(sql`SELECT id FROM (${flag ? sql`
  SELECT id FROM accounts -- findings: table:accounts
` : sql`SELECT id FROM accounts LIMIT 1`}) q`);
