import { query, sql } from "@example/db";
query(sql`UPDATE accounts SET email = $1 RETURNING ${flag ? sql`
  * -- findings: accounts
` : sql`id`}`);
query(sql`SELECT ${flag ? sql`
  a.* -- findings: accounts
` : sql`a.id`} FROM accounts a`);
query(sql`SELECT ${flag ? sql`
  row_to_json(a) -- a whole-row identifier has no star token
` : sql`a.id`} FROM accounts a`);
query(sql`SELECT ${flag ? sql`
  row_to_json(a.*) -- findings: accounts
` : sql`a.id`} FROM accounts a`);
