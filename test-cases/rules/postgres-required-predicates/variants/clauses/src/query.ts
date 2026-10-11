import { query, sql } from "@example/db";
query(sql`UPDATE ${flag ? sql`
  accounts SET email = $1 WHERE id = $2 -- findings: accounts
` : sql`accounts SET email = $1 WHERE accounts.active = TRUE`}`);
query(sql`DELETE FROM ${flag ? sql`
  accounts WHERE id = $1 -- findings: accounts
` : sql`accounts WHERE accounts.active = TRUE`}`);
query(sql`SELECT a.id FROM accounts a JOIN ${flag ? sql`
  accounts b ON b.id = a.id -- findings: accounts
` : sql`other b ON b.id = a.id`} WHERE a.active = TRUE`);
